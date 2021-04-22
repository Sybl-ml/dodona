//! Defines the routes specific to project operations.

use std::task::Poll;

use actix_multipart::Multipart;
use actix_web::{
    dev::HttpResponseBuilder,
    http::StatusCode,
    web::{self, Bytes},
    HttpResponse,
};
use futures::stream::poll_fn;
use itertools::Itertools;
use mongodb::{
    bson::{de::from_document, doc, document::Document, oid::ObjectId, ser::to_document},
    options, Collection,
};
use tokio_stream::StreamExt;

use messages::kafka_message::produce_message;
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

static CHUNK_SIZE: usize = 10_000;

/// Enum to decide type of dataset to return
#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum DatasetType {
    /// Training dataset type
    Train,
    /// Predict dataset type. Represents just predict data
    Predict,
    /// Prediction dataset type. Represents both predict and predictions data
    Prediction,
}

/// Struct to capture query string information
#[derive(Deserialize, Debug)]
pub struct QueryInfo {
    /// Sort order for data
    pub sort: Option<String>,
    /// What page is being viewed
    pub page: usize,
    /// How many items per page
    pub per_page: usize,
}

/// Struct to capture query string information
#[derive(Deserialize, Debug)]
pub struct DataCollection {
    /// Min row being collected
    pub min_row: usize,
    /// Max row being collected
    pub max_row: usize,
    /// Lower chunk id
    pub lower_chunk: usize,
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

    let project_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    let filter = doc! { "_id": &project_id };
    let doc = projects
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // get that project from the projects collection
    let (analysis_doc, details_doc) = get_all_project_info(&project_id, &state.database).await?;
    let mut response = doc! {"project": doc};

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
    let mut cursor = projects.find(filter, None).await?;

    let mut projects = Vec::new();

    while let Some(Ok(proj)) = cursor.next().await {
        let (analysis_doc, details_doc) =
            get_all_project_info(proj.get_object_id("_id").unwrap(), &state.database).await?;

        let mut response = doc! {"project": proj};

        if let Some(details_doc) = details_doc {
            // Insert the details as well
            response.insert("details", details_doc);
        }

        if let Some(analysis_doc) = analysis_doc {
            // Insert the details as well
            response.insert("analysis", analysis_doc);
        }

        projects.push(response);
    }

    response_from_json(projects)
}

async fn get_all_project_info(
    project_id: &ObjectId,
    database: &mongodb::Database,
) -> ServerResult<(Option<Document>, Option<Document>)> {
    let details = database.collection("dataset_details");
    let analysis = database.collection("dataset_analysis");

    // get that project from the projects collection
    let filter = doc! { "project_id": &project_id };
    let details_doc = details.find_one(filter.clone(), None).await?;

    // get that analysis from the analysis collection
    let analysis_doc = analysis.find_one(filter, None).await?;

    Ok((analysis_doc, details_doc))
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

    // Track the train and predict identifiers
    let mut train_id = None;
    let mut predict_id = None;

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

        let name = content_disposition
            .get_name()
            .ok_or(ServerError::UnprocessableEntity)?;

        let mut dataset = gridfs::File::new(String::from(filename));

        while let Some(Ok(chunk)) = field.next().await {
            if name == "train" {
                if initial {
                    let data_head = std::str::from_utf8(&chunk).unwrap();
                    let data_head = data_head.lines().take(6).collect::<Vec<_>>().join("\n");

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

            let data_string = std::str::from_utf8(&gen_buf_chunk).unwrap().lines();
            // Split data into rows
            for row in data_string {
                let cols = row.split(',').count();

                // Check for half row to put in general buffer
                if cols != col_num {
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
            train_id = Some(dataset.id);
            // Update dataset details with train size
            dataset_details
                .update_one(
                    doc! { "project_id": &object_id},
                    doc! {"$set": {"train_size": data_size}},
                    None,
                )
                .await?;
        } else {
            predict_id = Some(dataset.id);
            // Update dataset details with predict size
            dataset_details
                .update_one(
                    doc! { "project_id": &object_id},
                    doc! {"$set": {"predict_size": data_size }},
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
    let option = options::FindOneAndUpdateOptions::builder()
        .return_document(options::ReturnDocument::After)
        .build();
    let project_doc = projects
        .find_one_and_update(
            doc! { "_id": &object_id},
            doc! {"$set": {"status": Status::Ready}},
            option,
        )
        .await?
        .unwrap();

    let dataset_doc = Dataset::new(
        object_id.clone(),
        train_id.ok_or(ServerError::UnprocessableEntity)?,
        predict_id.ok_or(ServerError::UnprocessableEntity)?,
    );

    let document = to_document(&dataset_doc)?;
    let _id = datasets.insert_one(document, None).await?.inserted_id;

    // Inform the analysis server of the new job
    let analytics_job = serde_json::to_string(&object_id).unwrap();
    let topic = "analytics";
    produce_message(&analytics_job, &analytics_job, &topic).await;

    let (analysis_doc, details_doc) = get_all_project_info(&object_id, &state.database).await?;
    let mut response = doc! {"project": project_doc};

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
            let data_head = data_head.lines().take(6).collect::<Vec<_>>().join("\n");
            log::info!("First 5 lines: {:?}", &data_head);
            let header = data_head.lines().next().unwrap();
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

        let data_string = std::str::from_utf8(&gen_buf_chunk).unwrap().lines();
        // Split data into rows
        for row in data_string {
            // Determine if it is a dataset row or predict row
            // Add to the correct buffer
            let cols = row.split(',');

            // Tests to see if there is a predition column in row
            if cols.clone().count() == col_num {
                if cols.filter(|x| x.trim().is_empty()).count() == 1 {
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
            doc! {"$set": {"train_size": train_size, "predict_size": predict_size }},
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

    let dataset_doc = Dataset::new(object_id.clone(), dataset.id, predict.id);
    let document = to_document(&dataset_doc)?;
    let _id = datasets.insert_one(document, None).await?.inserted_id;

    // Update the project status
    let option = options::FindOneAndUpdateOptions::builder()
        .return_document(options::ReturnDocument::After)
        .build();
    let project_doc = projects
        .find_one_and_update(
            doc! { "_id": &object_id},
            doc! {"$set": {"status": Status::Ready}},
            option,
        )
        .await?
        .unwrap();

    // Inform the analysis server of the new job
    let analytics_job = serde_json::to_string(&object_id).unwrap();
    let topic = "analytics";
    produce_message(&analytics_job, &analytics_job, &topic).await;

    let (analysis_doc, details_doc) = get_all_project_info(&object_id, &state.database).await?;
    let mut response = doc! {"project": project_doc};

    if let Some(details_doc) = details_doc {
        // Insert the details as well
        response.insert("details", details_doc);
    }
    if let Some(analysis_doc) = analysis_doc {
        // Insert the details as well
        response.insert("analysis", analysis_doc);
    }
    // Communicate with Analytics Server
    let analytics_job = serde_json::to_string(&object_id).unwrap();
    let topic = "analytics";
    produce_message(&analytics_job, &analytics_job, &topic).await;

    response_from_json(response)
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
    let predictions = state.database.collection("predictions");
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
        DatasetType::Train => doc! { "_id": &dataset.dataset },
        DatasetType::Predict => doc! { "_id": &dataset.predict },
        // Need to find a way to stream combined version
        DatasetType::Prediction => doc! { "_id": &dataset.predict },
    };

    // Find the file in the database
    let document = files
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let file: gridfs::File = from_document(document)?;

    let mut cursor = file.download_chunks(&state.database).await?;

    match dataset_type {
        DatasetType::Prediction => {
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
            let predictions_file: gridfs::File = from_document(document)?;

            let mut predictions_cursor = predictions_file.download_chunks(&state.database).await?;

            let mut initial: bool = true;

            let byte_stream = poll_fn(
                move |_| -> Poll<Option<Result<Bytes, actix_web::error::Error>>> {
                    // While Cursor not empty
                    match futures::executor::block_on(zip(&mut predictions_cursor, &mut cursor)) {
                        (Some(prediction_chunk), Some(predict_chunk)) => {
                            let chunk: gridfs::Chunk =
                                from_document(predict_chunk.unwrap()).unwrap();
                            let chunk_bytes = chunk.data.bytes;
                            let decomp_predict = decompress_data(&chunk_bytes).unwrap();
                            // Convert both to utf 8
                            let utf_predict = String::from_utf8(decomp_predict).unwrap();
                            let predict_rows = utf_predict.lines();

                            let chunk: gridfs::Chunk =
                                from_document(prediction_chunk.unwrap()).unwrap();
                            let chunk_bytes = chunk.data.bytes;
                            let decomp_prediction = decompress_data(&chunk_bytes).unwrap();
                            // Convert both to utf 8
                            let utf_prediction = String::from_utf8(decomp_prediction).unwrap();
                            let prediction_rows = utf_prediction.lines();

                            let mut chunk_vec: Vec<String> = Vec::new();

                            // Split based on rows
                            for (predicted_row, predict_row) in prediction_rows.zip(predict_rows) {
                                // Work out if header
                                if initial {
                                    // Header
                                    initial = false;
                                    chunk_vec.push(String::from(predict_row));
                                    continue;
                                }
                                //      Split predict based on commas
                                let row = predict_row
                                    .split(',')
                                    .map(|v| {
                                        v.trim().is_empty().then(|| predicted_row).unwrap_or(v)
                                    })
                                    .join(",");
                                // push to whole chunk vector
                                chunk_vec.push(row);
                            }
                            let joined_chunk: String = chunk_vec.join("\n");
                            // Join together whole chunk
                            // convert to bytes
                            // return chunk
                            Poll::Ready(Some(Ok(Bytes::from(joined_chunk.into_bytes()))))
                        }
                        _ => Poll::Ready(None),
                    }
                },
            );

            Ok(HttpResponseBuilder::new(StatusCode::OK).streaming(byte_stream))
        }
        _ => {
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
    }
}

/// Function to zip 2 cursors so they iterate together
async fn zip(
    left: &mut mongodb::Cursor,
    right: &mut mongodb::Cursor,
) -> (
    Option<mongodb::error::Result<Document>>,
    Option<mongodb::error::Result<Document>>,
) {
    tokio::join!(left.next(), right.next())
}

fn chunk_to_bytes(chunk: Document) -> ServerResult<Vec<u8>> {
    let chunk: gridfs::Chunk = from_document(chunk)?;
    let chunk_bytes = chunk.data.bytes;
    Ok(decompress_data(&chunk_bytes)?)
}

/// function to get single pagination page and returns
/// the correct JSON reply.
async fn data_collection(
    filter: Document,
    chunk_vec: Vec<i32>,
    database: &mongodb::Database,
    info: DataCollection,
    dataset_detail: DatasetDetails,
    dataset_type: &DatasetType,
) -> ServerResponse {
    let files = database.collection("files");
    let mut header: String = String::new();

    let document = files
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let file: gridfs::File = from_document(document)?;

    let mut cursor = file.download_specific_chunks(&database, &chunk_vec).await?;

    // Here we have our chunks X
    // Get the header from the first chunk X
    // If the lower chunk == 0, add it to the byte buffer, else continue X
    // Go through rest of chunks and workout if they are needed, add to byte buffer X

    let mut byte_buffer: Vec<u8> = Vec::new();
    let mut chunk_vec_iter = chunk_vec.iter();

    while let Some(chunk) = cursor.next().await {
        // Get the number of the chunk
        let chunk_num = chunk_vec_iter.next().unwrap();
        let decomp_data: Vec<u8> = chunk_to_bytes(chunk?)?;
        if *chunk_num == 0 {
            // Get header
            let header_idx = decomp_data
                .iter()
                .enumerate()
                .filter_map(|(i, b)| (*b == b'\n').then(|| i))
                .next()
                .expect("Failed to find header");
            let header_bytes = &decomp_data[..header_idx];
            header = String::from_utf8(Vec::from(header_bytes)).unwrap();
            if info.lower_chunk == 0 {
                byte_buffer.extend_from_slice(&decomp_data[header_idx..]);
            }
        } else {
            byte_buffer.extend_from_slice(&decomp_data);
        }
    }

    // Workout index of min_row and max_row in byte buffer

    let page_size = info.max_row - info.min_row - 1;
    let buffer_min = info.min_row - (info.lower_chunk * CHUNK_SIZE);

    // Get byte slice up to that row (take fewer rows if less rows there)
    let start: usize = byte_buffer
        .iter()
        .enumerate()
        .filter_map(|(i, b)| (*b == b'\n').then(|| i))
        .nth(buffer_min)
        .unwrap_or_default();

    let slice = &byte_buffer[start + 1..];

    let end: usize = slice
        .iter()
        .enumerate()
        .filter_map(|(i, b)| (*b == b'\n').then(|| i))
        .nth(page_size)
        .unwrap_or_else(|| slice.len());

    let rows = std::str::from_utf8(&slice[..end])?;

    let total = match dataset_type {
        DatasetType::Train => dataset_detail.train_size,
        _ => dataset_detail.predict_size,
    };
    // Structure for VueTables2
    response_from_json(doc! {
        "data": rows,
        "fields": header,
        // Work this out
        "total": total
    })
}

/// Route for returning pagination data
///
/// Passing a full dataset to the frontend can be heavy duty and take up a
/// lot of data. Using pagination, sections of a dataset can be given to the
/// frontend when they are needed so that data usage can be reduced.
pub async fn pagination(
    claims: auth::Claims,
    state: web::Data<State>,
    extractor: web::Path<(String, DatasetType)>,
    query: web::Query<QueryInfo>,
) -> ServerResponse {
    let datasets = state.database.collection("datasets");
    let dataset_details = state.database.collection("dataset_details");
    let projects = state.database.collection("projects");
    let files = state.database.collection("files");
    let predictions = state.database.collection("predictions");

    let project_id = &extractor.0;
    let dataset_type = &extractor.1;
    let amount = query.per_page;
    let page_num = query.page;

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;
    let filter = doc! { "project_id": &object_id };

    // Find the dataset in the database
    let document = datasets
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let dataset: Dataset = from_document(document)?;

    let filter = doc! { "project_id": &object_id };

    // Find the dataset details in the database
    let document = dataset_details
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let dataset_detail: DatasetDetails = from_document(document)?;

    // Calculate the chunk that is needed
    let min_row = (page_num - 1) * amount;
    let max_row = page_num * amount;

    let (chunk_vec, lower_chunk) = utils::calculate_chunk_indices(min_row, max_row, CHUNK_SIZE);

    let filter = match dataset_type {
        DatasetType::Train => doc! { "_id": &dataset.dataset },
        DatasetType::Predict | DatasetType::Prediction => doc! { "_id": &dataset.predict },
    };

    // Decide filter based off enum
    match dataset_type {
        DatasetType::Train | DatasetType::Predict => {
            // If Train
            //      Get correct chunks
            //      Extract data page that is needed
            //      Return it to frontend
            let info = DataCollection {
                min_row,
                max_row,
                lower_chunk: lower_chunk as usize,
            };

            // Find the file in the database
            data_collection(
                filter,
                chunk_vec,
                &state.database,
                info,
                dataset_detail,
                dataset_type,
            )
            .await
        }
        DatasetType::Prediction => {
            // If Prediction
            //      Get correct predict chunk
            //      Get correct predictions chunk
            //      Extract data page from both that is needed
            //      Combine the two pages into 1
            //      Return to frontend

            let mut initial = true;
            let mut page: Vec<String> = vec![];
            let mut header: String = String::new();
            let mut row_count: usize = 0;

            if lower_chunk != 0 {
                // calculate the base row of Chunk1
                row_count = (((lower_chunk + 1) * CHUNK_SIZE as i32) - 1) as usize;
            }

            let document = files
                .find_one(filter, None)
                .await?
                .ok_or(ServerError::NotFound)?;

            // Parse the dataset itself
            let predict_file: gridfs::File = from_document(document)?;

            let mut predict_cursor = predict_file
                .download_specific_chunks(&state.database, &chunk_vec)
                .await?;

            // Get the predictions
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
            let predictions_file: gridfs::File = from_document(document)?;

            let mut predictions_cursor = predictions_file
                .download_specific_chunks(&state.database, &chunk_vec)
                .await?;

            while let Some(chunk) = predict_cursor.next().await {
                // Predict Chunk
                let chunk: gridfs::Chunk = from_document(chunk?)?;
                let chunk_bytes = chunk.data.bytes;
                let decomp_data = decompress_data(&chunk_bytes).unwrap();
                let chunk_string = String::from_utf8(decomp_data)?;

                // Predictions Chunk
                let pred_chunk_string = match predictions_cursor.next().await {
                    Some(chunk) => {
                        let chunk: gridfs::Chunk = from_document(chunk?)?;
                        let chunk_bytes = chunk.data.bytes;
                        let decomp_data = decompress_data(&chunk_bytes).unwrap();
                        Some(String::from_utf8(decomp_data)?)
                    }
                    None => None,
                }
                .ok_or(ServerError::NotFound)?;

                for (predict_row, predicted_row) in
                    chunk_string.lines().zip(pred_chunk_string.lines())
                {
                    // If header, add to page
                    if initial {
                        header = String::from(predict_row);
                        initial = false;
                        if lower_chunk != 0 {
                            continue;
                        }
                    }
                    // If within bounds, create predictions row and add to page
                    if row_count > min_row && row_count <= max_row {
                        let row = predict_row
                            .split(',')
                            .map(|v| v.trim().is_empty().then(|| predicted_row).unwrap_or(v))
                            .join(",");

                        // Add to page
                        page.push(row);
                    } else if row_count > max_row {
                        // End of page search
                        break;
                    }

                    row_count += 1;
                }
                // End of page search
                if row_count > max_row {
                    break;
                }
            }

            // Structure for VueTables2
            response_from_json(doc! {
                "data": page.join("\n"),
                "fields": header,
                // Work this out
                "total": dataset_detail.predict_size
            })
        }
    }
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

    // Ensure a dataset exists for this project
    let filter = doc! { "project_id": &object_id };
    datasets
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

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

    let feature_dim = column_types.len() as i8;

    let cost = job_cost(
        payload.cluster_size as i32,
        feature_dim as i32,
        dataset_detail.train_size + dataset_detail.predict_size,
    );
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

    // Send a request to the interface layer
    let config = JobConfiguration {
        project_id: object_id.clone(),
        node_computation_time: payload.node_computation_time as i32,
        cluster_size: payload.cluster_size as i32,
        column_types,
        feature_dim,
        train_size: ((dataset_detail.train_size + 99) / 100) * 100,
        predict_size: ((dataset_detail.predict_size + 99) / 100) * 100,
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

    let job_message = serde_json::to_string(&job).unwrap();
    let job_key = &job.id.to_string();
    let topic = "jobs";

    produce_message(&job_message, job_key, &topic).await;

    // Delete previous predictions for project
    let filter = doc! {"project_id": &object_id};

    let prediction_removed = predictions.find_one(filter, None).await?;

    if let Some(prediction_removed) = prediction_removed {
        let prediction: Prediction = from_document(prediction_removed)?;
        prediction.delete(&state.database).await?;
    }

    // Mark the project as processing
    let filter = doc! { "_id": &object_id};
    let update =
        doc! { "$set": doc!{ "status": Status::Processing { model_success: 0, model_err: 0 } } };
    projects.update_one(filter, update, None).await?;

    response_from_json(job)
}

/// Queries the currently running job for a given project, if one exists.
pub async fn currently_running_job(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
) -> ServerResponse {
    let projects = state.database.collection("projects");
    let jobs = state.database.collection("jobs");
    let project_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    // Query the jobs for this project, sorting by date
    let filter = doc! { "config.project_id": &project_id };
    let sort = doc! { "date_created": -1 };
    let options = options::FindOneOptions::builder().sort(sort).build();

    let last_job = jobs
        .find_one(filter, options)
        .await?
        .ok_or(ServerError::NotFound)?;

    let mut document = Document::new();

    document.insert("job", last_job);

    response_from_json(document)
}

/// Queries the currently running job for a given project, if one exists. Gets the job statistics
pub async fn get_job_statistics(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
) -> ServerResponse {
    let projects = state.database.collection("projects");
    let jobs = state.database.collection("jobs");
    let job_statistics = state.database.collection("job_statistics");
    let project_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    // Query the jobs for this project, sorting by date
    let filter = doc! { "config.project_id": &project_id };
    let sort = doc! { "date_created": -1 };
    let options = options::FindOneOptions::builder().sort(sort).build();

    let job = jobs
        .find_one(filter, options)
        .await?
        .ok_or(ServerError::NotFound)?;

    let job: Job = from_document(job)?;

    let filter = doc! {"job_id": &job.id};

    let job_statistic = job_statistics
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    response_from_json(job_statistic)
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
