//! Defines the routes specific to project operations.

use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::task::Poll;

use actix_multipart::Multipart;
use actix_web::{
    dev::HttpResponseBuilder,
    http::StatusCode,
    web::{self, Bytes},
    HttpResponse,
};
use futures::stream::poll_fn;
use math::round;
use mongodb::{
    bson::{de::from_document, doc, document::Document, oid::ObjectId, ser::to_document},
    Collection,
};
use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaResult;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;
use tokio_stream::StreamExt;

use models::dataset_details::DatasetDetails;
use models::datasets::Dataset;
use models::gridfs;
use models::jobs::{Job, JobConfiguration};
use models::predictions::Prediction;
use models::projects::{Project, Status};
use models::users::User;
use utils::compress::{compress_data, decompress_data};
use utils::finance::{job_cost, pay};
use utils::ColumnType;

use crate::{
    auth,
    error::{ServerError, ServerResponse, ServerResult},
    routes::{check_user_owns_project, payloads, response_from_json},
    State,
};

static JOB_TOPIC: &str = "jobs";
static ANALYTICS_TOPIC: &str = "analytics";
static CHUNK_SIZE: usize = 10_000;

/// Enum to decide type of dataset to return
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DatasetType {
    /// Training dataset type
    Train,
    /// Predict dataset type
    Predict,
}

/// Finds a project in the database given an identifier.
///
/// Given a project identifier, finds the project in the database and returns it as a JSON object.
/// If the project does not exist, returns a 404 response code.
pub async fn get_project(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
) -> ServerResponse {
    let projects = state.database.collection("projects");
    let details = state.database.collection("dataset_details");
    let analysis = state.database.collection("dataset_analysis");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    let filter = doc! { "_id": &object_id };
    let doc = projects
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // get that project from the projects collection
    let filter = doc! { "project_id": &object_id };
    let details_doc = details.find_one(filter.clone(), None).await?;

    // get that project from the projects collection
    let analysis_doc = analysis.find_one(filter, None).await?;

    // Begin by adding the project as we know we will respond with that
    let mut response = doc! { "project": doc };

    if let Some(details_doc) = details_doc {
        // Insert the details as well
        response.insert("details", details_doc);
    }

    if let Some(analysis_doc) = analysis_doc {
        // Insert the details as well
        response.insert("analysis", analysis_doc);
    }

    response_from_json(response)
}

/// Patches a given project with the provided data.
///
/// Patch documents specify the fields to change and are a key-value pairing of the fields found in
/// the project itself. This can be used to change the name or description of the project, for
/// example.
pub async fn patch_project(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
    payload: web::Json<payloads::PatchProjectOptions>,
) -> ServerResponse {
    let projects = state.database.collection("projects");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    let filter = doc! { "_id": &object_id };
    let update_doc = doc! { "$set": &payload.changes };
    projects.update_one(filter, update_doc, None).await?;

    Ok(HttpResponse::Ok().finish())
}

/// Deletes a given project.
///
/// Checks whether the project exists, before deleting it from the database.
pub async fn delete_project(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
) -> ServerResponse {
    let projects = state.database.collection("projects");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    let filter = doc! { "_id": &object_id };
    let document = projects.find_one(filter, None).await?;

    if let Some(document) = document {
        let project: Project = from_document(document)?;
        project.delete(&state.database).await?;
    }

    Ok(HttpResponse::Ok().finish())
}

/// Finds all the projects related to a given user.
///
/// Given a user identifier, finds all the projects in the database that the user owns. If the user
/// doesn't exist or an invalid identifier is given, returns a 404 response.
pub async fn get_user_projects(claims: auth::Claims, state: web::Data<State>) -> ServerResponse {
    let projects = state.database.collection("projects");

    let filter = doc! { "user_id": &claims.id };
    let cursor = projects.find(filter, None).await?;
    let documents: Vec<Document> = cursor.collect::<Result<_, _>>().await?;

    response_from_json(documents)
}

/// Creates a new project for a given user.
pub async fn new(
    claims: auth::Claims,
    state: web::Data<State>,
    payload: web::Json<payloads::NewProjectOptions>,
) -> ServerResponse {
    let projects = state.database.collection("projects");

    let name = crypto::clean(&payload.name);
    let description = crypto::clean(&payload.description);

    let project = Project::new(&name, &description, payload.tags.clone(), claims.id.clone());

    let document = to_document(&project)?;
    let id = projects.insert_one(document, None).await?.inserted_id;

    response_from_json(doc! {"project_id": id})
}

/// Adds data to a given project.
///
/// This will first check whether the project already has data associated with it, deleting it if
/// this is the case. It will then clean the incoming data and analyse it, hoever the data is already
/// split into train and predict. After this, the data will be compressed and inserted into the
/// database, with the status being updated to [`Status::Ready`].
pub async fn upload_train_and_predict(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
    mut dataset: Multipart,
) -> ServerResponse {
    let datasets = state.database.collection("datasets");
    let dataset_details = state.database.collection("dataset_details");
    let projects = state.database.collection("projects");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    // Create dataset object
    let mut dataset_doc = Dataset::new(object_id.clone(), None, None);

    let mut initial: bool = true;

    let mut col_num = 1;
    let mut buffer: Vec<String> = Vec::new();
    let mut general_buffer: Vec<u8> = Vec::new();
    let mut data_size: i32 = 0;

    // Read the train from the upload
    while let Some(Ok(mut field)) = dataset.next().await {
        let content_disposition = field
            .content_disposition()
            .ok_or(ServerError::UnprocessableEntity)?;

        let filename = content_disposition
            .get_filename()
            .ok_or(ServerError::UnprocessableEntity)?;

        log::info!("Filename: {}", filename);

        let name = content_disposition
            .get_name()
            .ok_or(ServerError::UnprocessableEntity)?;

        let mut dataset = gridfs::File::new(String::from(filename));

        while let Some(Ok(chunk)) = field.next().await {
            if name == "train" {
                if initial {
                    let data_head = std::str::from_utf8(&chunk).unwrap();
                    let data_head = data_head.split("\n").take(6).collect::<Vec<_>>().join("\n");
                    log::info!("First 5 lines: {:?}", &data_head);

                    initial = false;

                    let analysis = utils::analysis::analyse(&data_head);

                    let column_types = analysis.types;
                    col_num = column_types.len();

                    let details = DatasetDetails::new(
                        String::from(filename),
                        object_id.clone(),
                        data_head,
                        column_types,
                    );

                    let document = to_document(&details)?;
                    dataset_details.insert_one(document, None).await?;
                }
            }
            // Fill file
            general_buffer.extend_from_slice(&chunk);

            let gen_buf_chunk = general_buffer.clone();

            let data_string = std::str::from_utf8(&gen_buf_chunk).unwrap().split("\n");
            // Split data into rows
            for row in data_string {
                let cols = row.split(",").collect::<Vec<_>>();
                // Check for half row to put in general buffer
                if cols.len() != col_num {
                    // If any incomplete row, set as general buffer
                    general_buffer = row.as_bytes().iter().cloned().collect();
                } else {
                    buffer.push(String::from(row));
                    data_size += 1;
                }

                if buffer.len() == CHUNK_SIZE {
                    log::debug!("Uploading a {} chunk of size: {}", &name, buffer.len());
                    // Flush buffer into mongodb
                    // Compress data before upload
                    let chunk = buffer.join("\n");
                    let compressed = compress_data(&chunk)?;
                    let bytes = Bytes::from(compressed);
                    dataset.upload_chunk(&state.database, &bytes).await?;
                    buffer.clear();
                }
            }
        }

        // Add file to db and set OIDs in Dataset
        let chunk = buffer.join("\n");
        let compressed = compress_data(&chunk)?;
        let bytes = Bytes::from(compressed);

        dataset.upload_chunk(&state.database, &bytes).await?;
        dataset.finalise(&state.database).await?;

        if name == "train" {
            dataset_doc.dataset = Some(dataset.id);
            // Update dataset details with train size
            dataset_details
                .update_one(
                    doc! { "project_id": &object_id},
                    doc! {"$set": {"predict_size": round::ceil(data_size as f64, -2) as i32}},
                    None,
                )
                .await?;
        } else {
            dataset_doc.predict = Some(dataset.id);
            // Update dataset details with predict size
            dataset_details
                .update_one(
                    doc! { "project_id": &object_id},
                    doc! {"$set": {"predict_size": round::ceil(data_size as f64, -2) as i32}},
                    None,
                )
                .await?;
        }
        data_size = 0;
        log::info!("Finalised the {} set of data", name);
    }

    let existing_data = datasets
        .find_one(doc! { "project_id": &object_id }, None)
        .await?;

    // If the project has data, delete the existing information
    if let Some(existing_data) = existing_data {
        let data: Dataset = from_document(existing_data)?;
        log::debug!("Deleting existing project data with id={}", data.id);
        data.delete(&state.database).await?;
    }

    // Update the project status
    projects
        .update_one(
            doc! { "_id": &object_id},
            doc! {"$set": {"status": Status::Ready}},
            None,
        )
        .await?;

    let document = to_document(&dataset_doc)?;
    let id = datasets.insert_one(document, None).await?.inserted_id;

    response_from_json(doc! {"dataset_id": id})
}

/// Adds data to a given project.
///
/// This will first check whether the project already has data associated with it, deleting it if
/// this is the case. It will then clean the incoming data and analyse it, splitting it into
/// training and prediction. After this, the data will be compressed and inserted into the
/// database, with the status being updated to [`Status::Ready`].
pub async fn upload_and_split(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
    mut dataset: Multipart,
) -> ServerResponse {
    let datasets = state.database.collection("datasets");
    let dataset_details = state.database.collection("dataset_details");
    let projects = state.database.collection("projects");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    // Read the dataset from the upload
    let mut field = dataset
        .try_next()
        .await?
        .ok_or(ServerError::UnprocessableEntity)?;

    let content_disposition = field
        .content_disposition()
        .ok_or(ServerError::UnprocessableEntity)?;

    let filename = content_disposition
        .get_filename()
        .ok_or(ServerError::UnprocessableEntity)?;

    // Create a new instance of our GridFS files
    log::info!("Creating a new file with name: {}", filename);
    let mut dataset = gridfs::File::new(String::from(filename));
    let mut predict = gridfs::File::new(String::from(filename));

    let mut initial: bool = true;

    let mut col_num = 1;
    let mut dataset_buffer: Vec<String> = Vec::new();
    let mut predict_buffer: Vec<String> = Vec::new();
    let mut general_buffer: Vec<u8> = Vec::new();

    let mut train_size: i32 = 0;
    let mut predict_size: i32 = 0;

    // Stream the data into it
    while let Some(Ok(chunk)) = field.next().await {
        if initial {
            let data_head = std::str::from_utf8(&chunk).unwrap();
            let data_head = data_head.split("\n").take(6).collect::<Vec<_>>().join("\n");
            log::info!("First 5 lines: {:?}", &data_head);
            let header = data_head.split("\n").next().unwrap();
            predict_buffer.push(String::from(header));

            initial = false;

            let analysis = utils::analysis::analyse(&data_head);

            let column_types = analysis.types;
            col_num = column_types.len();

            let details = DatasetDetails::new(
                String::from(filename),
                object_id.clone(),
                data_head,
                column_types,
            );

            let document = to_document(&details)?;
            dataset_details.insert_one(document, None).await?;
        }

        // Concat with general buffer (if it has data)
        general_buffer.extend_from_slice(&chunk);

        let gen_buf_chunk = general_buffer.clone();

        let data_string = std::str::from_utf8(&gen_buf_chunk).unwrap().split("\n");
        // Split data into rows
        for row in data_string {
            // Determine if it is a dataset row or predict row
            // Add to the correct buffer
            let cols = row.split(",").collect::<Vec<_>>();
            if cols.len() == col_num {
                if row.split(',').last().unwrap().is_empty() {
                    predict_buffer.push(String::from(row));
                    predict_size += 1;
                } else {
                    dataset_buffer.push(String::from(row));
                    train_size += 1;
                }
            } else {
                // If any incomplete row, set as general buffer
                general_buffer = row.as_bytes().iter().cloned().collect();
            }

            // Check the size of the buffers
            // If a buffer is too big, flush it into mongo as a chunk
            if predict_buffer.len() == CHUNK_SIZE {
                log::debug!(
                    "Uploading a predict chunk of size: {}",
                    predict_buffer.len()
                );
                // Flush buffer into mongodb
                // Compress data before upload
                let predict_chunk = &predict_buffer.join("\n");
                let compressed_predict = compress_data(predict_chunk)?;
                let bytes_predict = Bytes::from(compressed_predict);
                predict
                    .upload_chunk(&state.database, &bytes_predict)
                    .await?;
                predict_buffer.clear();
                log::info!("Flushed Predict Buffer");
            }
            if dataset_buffer.len() == CHUNK_SIZE {
                // Flush buffer into mongodb
                log::debug!(
                    "Uploading a dataset chunk of size: {}",
                    dataset_buffer.len()
                );
                // Compress data before upload
                let dataset_chunk = &dataset_buffer.join("\n");
                let compressed = compress_data(dataset_chunk)?;
                let bytes_data = Bytes::from(compressed);
                dataset.upload_chunk(&state.database, &bytes_data).await?;
                dataset_buffer.clear();
                log::info!("Flushed Dataset Buffer");
            }
        }
    }

    // If anything let in buffers, upload as final chunks
    let dataset_chunk = &dataset_buffer.join("\n");
    let compressed = compress_data(dataset_chunk)?;
    let bytes_data = Bytes::from(compressed);
    dataset.upload_chunk(&state.database, &bytes_data).await?;

    let predict_chunk = &predict_buffer.join("\n");
    let compressed_predict = compress_data(predict_chunk)?;
    let bytes_predict = Bytes::from(compressed_predict);
    predict
        .upload_chunk(&state.database, &bytes_predict)
        .await?;

    // Finalise the file and upload its data
    log::debug!("Uploading the dataset and file itself");
    dataset.finalise(&state.database).await?;
    predict.finalise(&state.database).await?;

    // Update dataset details with train and predict sizes
    dataset_details
    .update_one(
        doc! { "project_id": &object_id},
        doc! {"$set": {"train_size": round::ceil(train_size as f64, -2) as i32, "predict_size": round::ceil(predict_size as f64, -2) as i32}},
        None,
    )
    .await?;

    // Check whether the project has data already
    let existing_data = datasets
        .find_one(doc! { "project_id": &object_id }, None)
        .await?;

    // If the project has data, delete the existing information
    if let Some(existing_data) = existing_data {
        let data: Dataset = from_document(existing_data)?;
        log::debug!("Deleting existing project data with id={}", data.id);
        data.delete(&state.database).await?;
    }

    let dataset_doc = Dataset::new(object_id.clone(), Some(dataset.id), Some(predict.id));
    let document = to_document(&dataset_doc)?;
    let id = datasets.insert_one(document, None).await?.inserted_id;

    // Update the project status
    projects
        .update_one(
            doc! { "_id": &object_id},
            doc! {"$set": {"status": Status::Ready}},
            None,
        )
        .await?;

    response_from_json(doc! {"dataset_id": id})
}

/// Gets the dataset details associated with a given project.
pub async fn overview(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
) -> ServerResponse {
    let dataset_details = state.database.collection("dataset_details");
    let projects = state.database.collection("projects");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    let filter = doc! { "project_id": &object_id };
    let document = dataset_details
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    response_from_json(document)
}

/// Gets the train/predict dataset associated with a given project.
///
/// Queries the database for the train/predict dataset related with a given identifier and
/// decompresses the requested data. The data is then converted back to a [`str`] and cleaned before
/// being sent to the frontend for display.
pub async fn get_dataset(
    claims: auth::Claims,
    state: web::Data<State>,
    extractor: web::Path<(String, DatasetType)>,
) -> ServerResponse {
    let datasets = state.database.collection("datasets");
    let projects = state.database.collection("projects");
    let files = state.database.collection("files");

    let project_id = &extractor.0;
    let dataset_type = &extractor.1;

    log::info!("Dataset Type: {:?}", dataset_type);
    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;
    let filter = doc! { "project_id": &object_id };

    // Find the dataset in the database
    let document = datasets
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let dataset: Dataset = from_document(document)?;

    // Decide filter based off enum
    let filter = match dataset_type {
        DatasetType::Train => doc! { "_id": &dataset.dataset.unwrap() },
        DatasetType::Predict => doc! { "_id": &dataset.predict.unwrap() },
    };

    // Find the file in the database
    let document = files
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let file: gridfs::File = from_document(document)?;

    let mut cursor = file.download_chunks(&state.database).await?;
    let byte_stream = poll_fn(
        move |_| -> Poll<Option<Result<Bytes, actix_web::error::Error>>> {
            // While Cursor not empty
            match futures::executor::block_on(cursor.next()) {
                Some(next) => {
                    let chunk: gridfs::Chunk = from_document(next.unwrap()).unwrap();
                    let chunk_bytes = chunk.data.bytes;
                    let decomp_train = decompress_data(&chunk_bytes).unwrap();
                    Poll::Ready(Some(Ok(Bytes::from(decomp_train))))
                }
                None => Poll::Ready(None),
            }
        },
    );

    Ok(HttpResponseBuilder::new(StatusCode::OK).streaming(byte_stream))
}

/// Removes the data associated with a given project identifier.
///
/// Searches for the project in the database to check whether it has a dataset. If a dataset is
/// found, it will be deleted from the database and the project status will be returned to
/// [`Status::Unfinished`].
pub async fn remove_data(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
) -> ServerResponse {
    let datasets = state.database.collection("datasets");
    let projects = state.database.collection("projects");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;
    let filter = doc! { "project_id": &object_id };

    // Remove the dataset associated with the project id in dataset
    let dataset_removed = datasets.find_one(filter, None).await?;

    if let Some(dataset_removed) = dataset_removed {
        let dataset: Dataset = from_document(dataset_removed)?;
        dataset.delete(&state.database).await?;
    }

    let filter = doc! { "_id": &object_id };
    let update = doc! { "$set": { "status": Status::Unfinished } };
    projects.update_one(filter, update, None).await?;

    response_from_json(doc! {"success": true})
}

/// Begins the processing of data associated with a project.
///
/// Checks that the project exists, before sending the identifier of its dataset to the interface
/// layer, which will then forward it to the DCL for processing. Updates the project state to
/// [`Status::Processing`].
pub async fn begin_processing(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
    payload: web::Json<payloads::ProcessingOptions>,
) -> ServerResponse {
    let projects = state.database.collection("projects");
    let datasets = state.database.collection("datasets");
    let users = state.database.collection("users");
    let dataset_details = state.database.collection("dataset_details");
    let predictions = state.database.collection("predictions");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    // Find the dataset in the database
    let filter = doc! { "project_id": &object_id };
    let document = datasets
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let dataset: Dataset = from_document(document)?;

    let filter = doc! { "project_id": &object_id };
    let document = dataset_details
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset detail itself
    let dataset_detail: DatasetDetails = from_document(document)?;

    let column_types: Vec<String> = dataset_detail
        .column_types
        .values()
        .map(|x| match x.column_type {
            ColumnType::Categorical(..) => String::from("Categorical"),
            ColumnType::Numerical(..) => String::from("Numerical"),
        })
        .collect();

    let cost = job_cost(payload.cluster_size);
    let query = doc! { "_id": &claims.id };
    let document = users
        .find_one(query, None)
        .await?
        .ok_or(ServerError::NotFound)?;
    let user: User = from_document(document)?;

    if user.credits < cost {
        log::warn!(
            "User {} has insufficient credits ({}) to run this job (requires {} credits)",
            &claims.id,
            &user.credits,
            &cost
        );
        return Err(ServerError::PaymentRequired);
    }

    let feature_dim = column_types.len() as i8;

    // Send a request to the interface layer
    let config = JobConfiguration {
        dataset_id: dataset.id.clone(),
        timeout: payload.timeout as i32,
        cluster_size: payload.cluster_size as i32,
        column_types,
        feature_dim,
        train_size: dataset_detail.train_size,
        predict_size: dataset_detail.predict_size,
        prediction_column: payload.prediction_column.clone(),
        prediction_type: payload.prediction_type,
        cost,
    };
    let job = Job::new(config);

    log::debug!("Created a new job: {:?}", job);

    pay(state.database.clone(), &claims.id, -cost).await?;
    log::debug!("Charged user {} {} credits", &claims.id, cost);

    // Insert to MongoDB first, so the interface can immediately mark as processed if needed
    insert_to_queue(&job, state.database.collection("jobs")).await?;

    if produce_message(&job).await.is_err() {
        log::warn!("Failed to forward job_id={} to Kafka", job.id);
    }

    // Delete previous predictions for project
    let filter = doc! {"project_id": &object_id};

    let prediction_removed = predictions.find_one(filter, None).await?;

    if let Some(prediction_removed) = prediction_removed {
        let prediction: Prediction = from_document(prediction_removed)?;
        prediction.delete(&state.database).await?;
    }

    // Mark the project as processing
    let filter = doc! { "_id": &object_id};
    let update = doc! { "$set": doc!{ "status": Status::Processing } };
    projects.update_one(filter, update, None).await?;

    response_from_json(doc! {"success": true})
}

/// Gets the predicted data for a project.
///
/// Queries the database for all [`Prediction`] instances for a given project identifier, before
/// decompressing each and returning them.
pub async fn get_predictions(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
) -> ServerResponse {
    // Get the projects and the predictions collections
    let projects = state.database.collection("projects");
    let predictions = state.database.collection("predictions");
    let files = state.database.collection("files");

    // Get the project identifier and check it exists
    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    let filter = doc! { "project_id": &object_id };

    // Find the dataset in the database
    let document = predictions
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the prediction itself
    let prediction: Prediction = from_document(document)?;

    let filter = doc! { "_id": &prediction.predictions };

    // Find the file in the database
    let document = files
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let file: gridfs::File = from_document(document)?;

    let mut cursor = file.download_chunks(&state.database).await?;
    let byte_stream = poll_fn(
        move |_| -> Poll<Option<Result<Bytes, actix_web::error::Error>>> {
            // While Cursor not empty
            match futures::executor::block_on(cursor.next()) {
                Some(next) => {
                    let chunk: gridfs::Chunk = from_document(next.unwrap()).unwrap();
                    let chunk_bytes = chunk.data.bytes;
                    let decomp_train = decompress_data(&chunk_bytes).unwrap();
                    Poll::Ready(Some(Ok(Bytes::from(decomp_train))))
                }
                None => Poll::Ready(None),
            }
        },
    );

    produce_analytics_message(&object_id).await;

    Ok(HttpResponseBuilder::new(StatusCode::OK).streaming(byte_stream))
}

async fn produce_message(job: &Job) -> KafkaResult<()> {
    // Get the environment variable for the kafka broker
    // if not set use 9092
    let var = env::var("BROKER_PORT").unwrap_or_else(|_| "9092".to_string());
    let port = u16::from_str(&var).expect("BROKER_PORT must be a u16");

    // Build the address to send to
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).to_string();
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &addr)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    let job_message = serde_json::to_string(&job.config).unwrap();

    let delivery_status = producer
        .send(
            FutureRecord::to(JOB_TOPIC)
                .payload(&job_message)
                .key(&job.id.to_string()),
            Timeout::Never,
        )
        .await;

    log::debug!("Message sent result: {:?}", delivery_status);

    Ok(())
}

async fn produce_analytics_message(project_id: &ObjectId) {
    let var = env::var("BROKER_PORT").unwrap_or_else(|_| "9092".to_string());
    let port = u16::from_str(&var).expect("BROKER_PORT must be a u16");

    // Build the address to send to
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).to_string();
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &addr)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    let analytics_job = serde_json::to_string(&project_id).unwrap();

    let delivery_status = producer
        .send(
            FutureRecord::to(ANALYTICS_TOPIC)
                .payload(&analytics_job)
                .key(&analytics_job),
            Timeout::Never,
        )
        .await;

    log::debug!("Message sent result: {:?}", delivery_status);
}

/// Inserts a [`JobConfiguration`] into MongoDB.
async fn insert_to_queue(job: &Job, collection: Collection) -> ServerResult<()> {
    let document = to_document(&job)?;

    if collection.insert_one(document, None).await.is_ok() {
        log::info!("Inserted job_id={} to the MongoDB queue", job.id);
    } else {
        log::error!("Failed to insert job_id={} to the MongoDB queue", job.id);
    }

    Ok(())
}
