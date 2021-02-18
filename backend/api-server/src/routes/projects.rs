//! Defines the routes specific to project operations.

use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;

use actix_web::{web, HttpResponse};
use mongodb::{
    bson::{de::from_document, doc, document::Document, ser::to_document},
    Collection,
};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;

use messages::WriteLengthPrefix;
use models::dataset_details::DatasetDetails;
use models::datasets::Dataset;
use models::jobs::{Job, JobConfiguration};
use models::predictions::Prediction;
use models::projects::{Project, Status};
use utils::compress::{compress_vec, decompress_data};
use utils::ColumnType;

use crate::{
    auth,
    error::{ServerError, ServerResponse, ServerResult},
    routes::{check_user_owns_project, payloads, response_from_json},
    State,
};

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

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    let filter = doc! { "_id": &object_id };
    let doc = projects
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // get that project from the projects collection
    let filter = doc! { "project_id": &object_id };
    let details_doc = details.find_one(filter, None).await?;

    // Begin by adding the project as we know we will respond with that
    let mut response = doc! { "project": doc };

    if let Some(details_doc) = details_doc {
        // Insert the details as well
        response.insert("details", details_doc);
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
    payload: web::Json<payloads::UploadDatasetOptions>,
) -> ServerResponse {
    let datasets = state.database.collection("datasets");
    let dataset_details = state.database.collection("dataset_details");
    let projects = state.database.collection("projects");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

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

    let data = crypto::clean(&payload.content);
    let analysis = utils::analysis::analyse(&data);
    let (train, predict) = utils::infer_train_and_predict(&data);
    let column_types = analysis.types;
    let data_head = analysis.header;

    // Compress the input data
    let compressed = compress_vec(&train)?;
    let compressed_predict = compress_vec(&predict)?;

    log::debug!(
        "Compressed {} bytes of data to {} bytes of training and {} bytes of prediction",
        data.len(),
        compressed.len(),
        compressed_predict.len()
    );

    let details = DatasetDetails::new(
        payload.name.clone(),
        object_id.clone(),
        data_head,
        column_types,
    );
    let dataset = Dataset::new(object_id.clone(), compressed, compressed_predict);

    // Update the project status
    projects
        .update_one(
            doc! { "_id": &object_id},
            doc! {"$set": {"status": Status::Ready}},
            None,
        )
        .await?;

    // Insert the dataset details and the dataset itself
    let document = to_document(&details)?;
    dataset_details.insert_one(document, None).await?;

    let document = to_document(&dataset)?;
    let id = datasets.insert_one(document, None).await?.inserted_id;

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

/// Gets the data associated with a given project.
///
/// Queries the database for the dataset related with a given identifier and decompresses both the
/// training and prediction data. The data is then converted back to a [`str`] and cleaned before
/// being sent to the frontend for display.
pub async fn get_data(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
) -> ServerResponse {
    let datasets = state.database.collection("datasets");
    let projects = state.database.collection("projects");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;
    let filter = doc! { "project_id": &object_id };

    // Find the dataset in the database
    let document = datasets
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset itself
    let dataset: Dataset = from_document(document)?;

    let comp_train = dataset.dataset.expect("missing training dataset").bytes;
    let comp_predict = dataset.predict.expect("missing prediction dataset").bytes;

    let decomp_train = decompress_data(&comp_train)?;
    let decomp_predict = decompress_data(&comp_predict)?;

    let train = crypto::clean(std::str::from_utf8(&decomp_train)?);
    let predict = crypto::clean(std::str::from_utf8(&decomp_predict)?);

    log::debug!("Fetched {} bytes of training data", train.len());
    log::debug!("Fetched {} bytes of prediction data", predict.len());

    response_from_json(doc! {"dataset": train, "predict": predict})
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

    if forward_to_interface(&job).await.is_err() {
        log::warn!("Failed to forward job_id={} to the interface", job.id);
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
    let datasets = state.database.collection("datasets");

    // Get the project identifier and check it exists
    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    // Find the predictions for the given project
    let filter = doc! { "project_id": &object_id };
    let cursor = predictions.find(filter.clone(), None).await?;

    let result = datasets
        .find_one(filter.clone(), None)
        .await?
        .ok_or(ServerError::NotFound)?;

    // Parse the dataset detail itself
    let dataset: Dataset = from_document(result)?;

    // Given a document, extract the predictions and decompress them
    let extract = |document: Document| -> ServerResult<String> {
        let prediction: Prediction = from_document(document)?;
        let decompressed = decompress_data(&prediction.predictions.bytes)?;

        Ok(String::from_utf8(decompressed)?)
    };

    // Decompress the predictions for each instance found
    let decompressed: Vec<_> = cursor
        .filter_map(Result::ok)
        .map(extract)
        .collect::<Result<_, _>>()
        .await?;

    let comp_predict = dataset.predict.expect("missing prediction dataset").bytes;
    let decomp_predict = decompress_data(&comp_predict)?;
    let predict = crypto::clean(std::str::from_utf8(&decomp_predict)?);

    response_from_json(doc! {"predictions": decompressed, "predict_data": predict})
}

/// Forwards a job to the interface layer.
///
/// Attempts to connect to the interface layer on the specified port from the configuration before
/// sending it the job. If the interface cannot be connected to, this will return an error.
async fn forward_to_interface(job: &Job) -> tokio::io::Result<()> {
    // Get the environment variable for the interface listener
    let var = env::var("INTERFACE_LISTEN").expect("INTERFACE_LISTEN must be set");
    let port = u16::from_str(&var).expect("INTERFACE_LISTEN must be a u16");

    // Build the address to send to
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let mut stream = TcpStream::connect(addr).await?;

    log::debug!("Connected to: {}", addr);

    stream.write(&job.as_bytes()).await?;

    log::info!("Forwarded a job to the interface with id={}", job.id);

    Ok(())
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
