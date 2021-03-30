//! Defines the routes specific to project operations.

use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::task::Poll;

use actix_multipart::Multipart;
use actix_web::{dev::HttpResponseBuilder, http::StatusCode, web, HttpResponse};
use futures::stream::poll_fn;
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
use utils::compress::{compress_bytes, decompress_data};
use utils::ColumnType;

use crate::{
    auth,
    error::{ServerError, ServerResponse, ServerResult},
    routes::{check_user_owns_project, payloads, response_from_json},
    State,
};

static JOB_TOPIC: &str = "jobs";
static ANALYTICS_TOPIC: &str = "analytics";

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

    let project = Project::new(&name, &description, claims.id.clone());

    let document = to_document(&project)?;
    let id = projects.insert_one(document, None).await?.inserted_id;

    response_from_json(doc! {"project_id": id})
}

/// Adds data to a given project.
///
/// This will first check whether the project already has data associated with it, deleting it if
/// this is the case. It will then clean the incoming data and analyse it, splitting it into
/// training and prediction. After this, the data will be compressed and inserted into the
/// database, with the status being updated to [`Status::Ready`].
pub async fn add_data(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
    mut dataset: Multipart,
) -> ServerResponse {
    let datasets = state.database.collection("datasets");
    let dataset_details = state.database.collection("dataset_details");
    let projects = state.database.collection("projects");

    // let data = clean(doc.get_str("content")?);
    let data = "";
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
    let mut dataset_buffer: Vec<u8> = Vec::new();
    let mut predict_buffer: Vec<u8> = Vec::new();
    let mut general_buffer: Vec<u8> = Vec::new();

    // Stream the data into it
    while let Some(Ok(chunk)) = field.next().await {
        if initial {
            let data_head = std::str::from_utf8(&chunk).unwrap();
            let data_head = data_head.split("\n").take(6).collect::<Vec<_>>().join("\n");
            log::info!("First 5 lines: {:?}", &data_head);
            let header = data_head.split("\n").take(1).collect::<Vec<_>>()[0];
            let header = format!("{}\n", &header);
            dataset_buffer.extend_from_slice(&header.as_bytes());
            predict_buffer.extend_from_slice(&header.as_bytes());

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

        let data_string = std::str::from_utf8(&gen_buf_chunk)
            .unwrap()
            .split("\n")
            .skip(1);
        // Split data into rows
        for row in data_string {
            // Determine if it is a dataset row or predict row
            // Add to the correct buffer
            let cols = row.split(",").collect::<Vec<_>>();
            if cols.len() == col_num {
                if row.split(',').last().unwrap().is_empty() {
                    let complete_row = format!("{}\n", row);
                    predict_buffer.extend_from_slice(complete_row.as_bytes());
                } else {
                    let complete_row = format!("{}\n", row);
                    dataset_buffer.extend_from_slice(complete_row.as_bytes());
                }
            } else {
                // If any incomplete row, set as general buffer
                general_buffer = row.as_bytes().iter().cloned().collect();
            }

            // Check the size of the buffers
            // If a buffer is too big, flush it into mongo as a chunk
            if predict_buffer.len() >= 100_000 {
                log::debug!(
                    "Uploading a predict chunk of size: {}",
                    predict_buffer.len()
                );
                // Flush buffer into mongodb
                // Compress data before upload
                let compressed_predict = compress_bytes(&predict_buffer)?;
                let bytes_predict = actix_web::web::Bytes::from(compressed_predict);
                predict
                    .upload_chunk(&state.database, &bytes_predict)
                    .await?;
                predict_buffer.clear();
                log::info!("Flushed Predict Buffer");
            }
            if dataset_buffer.len() >= 100_000 {
                // Flush buffer into mongodb
                log::debug!(
                    "Uploading a dataset chunk of size: {}",
                    dataset_buffer.len()
                );
                // Compress data before upload
                let compressed = compress_bytes(&dataset_buffer)?;
                let bytes_data = actix_web::web::Bytes::from(compressed);
                dataset.upload_chunk(&state.database, &bytes_data).await?;
                dataset_buffer.clear();
                log::info!("Flushed Dataset Buffer");
            }
        }
    }

    // If anything let in buffers, upload as final chunks
    let compressed = compress_bytes(&dataset_buffer)?;
    let bytes_data = actix_web::web::Bytes::from(compressed);
    dataset.upload_chunk(&state.database, &bytes_data).await?;

    let compressed_predict = compress_bytes(&predict_buffer)?;
    let bytes_predict = actix_web::web::Bytes::from(compressed_predict);
    predict
        .upload_chunk(&state.database, &bytes_predict)
        .await?;

    // Finalise the file and upload its data
    log::debug!("Uploading the dataset and file itself");
    dataset.finalise(&state.database).await?;
    predict.finalise(&state.database).await?;

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

/// Gets the dataset associated with a given project.
///
/// Queries the database for the dataset related with a given identifier and decompresses the
/// training data. The data is then converted back to a [`str`] and cleaned before
/// being sent to the frontend for display.
pub async fn get_dataset(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
) -> ServerResponse {
    let datasets = state.database.collection("datasets");
    let projects = state.database.collection("projects");
    let files = state.database.collection("files");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects)
        .await
        .unwrap();
    let filter = doc! { "project_id": &object_id };

    // Find the dataset in the database
    let document = datasets
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let dataset: Dataset = from_document(document)?;

    let filter = doc! { "_id": &dataset.dataset.unwrap() };

    // Find the file in the database
    let document = files
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let file: gridfs::File = from_document(document)?;

    let mut cursor = file.download_chunks(&state.database).await?;
    let byte_stream = poll_fn(
        move |_| -> Poll<Option<Result<actix_web::web::Bytes, actix_web::error::Error>>> {
            // While Cursor not empty
            match futures::executor::block_on(cursor.next()) {
                Some(next) => {
                    let chunk: gridfs::Chunk = from_document(next.unwrap()).unwrap();
                    let chunk_bytes = chunk.data.bytes;
                    let decomp_train = decompress_data(&chunk_bytes).unwrap();
                    Poll::Ready(Some(Ok(actix_web::web::Bytes::from(decomp_train))))
                }
                None => Poll::Ready(None),
            }
        },
    );

    Ok(HttpResponseBuilder::new(StatusCode::OK).streaming(byte_stream))
}

/// Gets the predict data associated with a given project.
///
/// Queries the database for the dataset related with a given identifier and decompresses the
/// prediction data. The data is then converted back to a [`str`] and cleaned before
/// being sent to the frontend for display.
pub async fn get_predict(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
) -> ServerResponse {
    let datasets = state.database.collection("datasets");
    let projects = state.database.collection("projects");
    let files = state.database.collection("files");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects)
        .await
        .unwrap();
    let filter = doc! { "project_id": &object_id };

    // Find the dataset in the database
    let document = datasets
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let dataset: Dataset = from_document(document)?;

    let filter = doc! { "_id": &dataset.predict.unwrap() };

    // Find the file in the database
    let document = files
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let file: gridfs::File = from_document(document)?;

    let mut cursor = file.download_chunks(&state.database).await?;
    let byte_stream = poll_fn(
        move |_| -> Poll<Option<Result<actix_web::web::Bytes, actix_web::error::Error>>> {
            // While Cursor not empty
            match futures::executor::block_on(cursor.next()) {
                Some(next) => {
                    let chunk: gridfs::Chunk = from_document(next.unwrap()).unwrap();
                    let chunk_bytes = chunk.data.bytes;
                    let decomp_train = decompress_data(&chunk_bytes).unwrap();
                    Poll::Ready(Some(Ok(actix_web::web::Bytes::from(decomp_train))))
                }
                None => Poll::Ready(None),
            }
        },
    );

    produce_analytics_message(&object_id).await;

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

    let column_types = dataset_detail
        .column_types
        .values()
        .map(|x| match x.column_type {
            ColumnType::Categorical(..) => String::from("Categorical"),
            ColumnType::Numerical(..) => String::from("Numerical"),
        })
        .collect();

    // Send a request to the interface layer
    let config = JobConfiguration {
        dataset_id: dataset.id.clone(),
        timeout: payload.timeout as i32,
        cluster_size: payload.cluster_size as i32,
        column_types,
        prediction_column: payload.prediction_column.clone(),
        prediction_type: payload.prediction_type,
    };
    let job = Job::new(config);

    log::debug!("Created a new job: {:?}", job);

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
        move |_| -> Poll<Option<Result<actix_web::web::Bytes, actix_web::error::Error>>> {
            // While Cursor not empty
            match futures::executor::block_on(cursor.next()) {
                Some(next) => {
                    let chunk: gridfs::Chunk = from_document(next.unwrap()).unwrap();
                    let chunk_bytes = chunk.data.bytes;
                    let decomp_train = decompress_data(&chunk_bytes).unwrap();
                    Poll::Ready(Some(Ok(actix_web::web::Bytes::from(decomp_train))))
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
