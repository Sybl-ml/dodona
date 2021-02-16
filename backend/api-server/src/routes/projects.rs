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
use models::jobs::{Job, JobConfiguration, PredictionType};
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

    log::debug!("{:?}", &response);

    response_from_json(response)
}

/// Patches a project with the provided data.
///
/// Given a project identifier, finds and updates the project in the database
/// matching new data
/// If project does not exist return a 404
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

/// Deletes a project provided a valid project id.
///
/// Given a project identifier, deletes a project from the database.
/// If the project ID is invalid return a 422
/// if project is not found return a 422
///
/// Will not currently authenticate the userid
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

/// Creates a new project related to a given user.
///
/// Given a user id, a project name and description, a project will
/// be created and saved in the database. This can fail if the user id
/// provided doesn't exist.
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

/// Saves a dataset to `MongoDB` for associated project.
///
/// This will take in a project id and a dataset. This route will
/// compress the dataset using `bzip2` and will store this compressed
/// data in the database as binary data. This can go wrong if there's
/// an error writing out the compressed data to the vector or if there
/// is an error finishing the compression stream. Both times an error
/// will return a 404 to the caller.
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
        log::debug!("Deleting existing project data");
        let data: Dataset = from_document(existing_data)?;
        data.delete(&state.database).await?;
    }

    let data = crypto::clean(&payload.content);
    let analysis = utils::analysis::analyse(&data);
    let (train, predict) = utils::infer_train_and_predict(&data);
    let column_types = analysis.types;
    let data_head = analysis.header;

    log::debug!("Dataset types: {:?}", &column_types);

    // Compress the input data
    let compressed = compress_vec(&train)?;
    let compressed_predict = compress_vec(&predict)?;

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

/// Gets the dataset details for a project
///
/// Project Id passed in as part of route and the dataset details
/// for that project are returned from the database.
pub async fn overview(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
) -> ServerResponse {
    let dataset_details = state.database.collection("dataset_details");
    let projects = state.database.collection("projects");

    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    let filter = doc! { "project_id": &object_id };
    let cursor = dataset_details.find(filter, None).await?;
    let documents: Vec<Document> = cursor.collect::<Result<_, _>>().await?;

    response_from_json(documents)
}

/// Route returns back uncompressed dataset
///
/// Makes a request to mongodb and gets dataset associated with
/// project id. Compressed data is then taken from returned struct
/// and is decompressed before being sent in a response back to the
/// user.
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

/// Removes the exisiting dataset linked to a project
///
/// Using the project id MongoDB is sent delete requests
/// for both dataset and dataset_details
/// Projects are reverted to Unfinshed status
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
/// `State::Processing`.
pub async fn begin_processing(
    claims: auth::Claims,
    state: web::Data<State>,
    project_id: web::Path<String>,
    payload: web::Json<payloads::ProcessingOptions>,
) -> ServerResponse {
    let projects = state.database.collection("projects");
    let datasets = state.database.collection("datasets");
    let dataset_details = state.database.collection("dataset_details");

    let timeout = payload.timeout as i32;

    let prediction_type: PredictionType = match payload.prediction_type.as_str() {
        "classification" => PredictionType::Classification,
        "regression" => PredictionType::Regression,
        _ => return Err(ServerError::UnprocessableEntity),
    };

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
        timeout,
        column_types,
        prediction_column: payload.prediction_column.clone(),
        prediction_type,
    };
    let job = Job::new(config);

    log::debug!("Created a new job: {:?}", job);

    // Insert to MongoDB first, so the interface can immediately mark as processed if needed
    insert_to_queue(&job, state.database.collection("jobs")).await?;

    if forward_to_interface(&job).await.is_err() {
        log::warn!("Failed to forward: {:?}", job);
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

    // Get the project identifier and check it exists
    let object_id = check_user_owns_project(&claims.id, &project_id, &projects).await?;

    // Find the predictions for the given project
    let filter = doc! { "project_id": &object_id };
    let cursor = predictions.find(filter, None).await?;

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

    response_from_json(doc! {"predictions": decompressed})
}

async fn forward_to_interface(job: &Job) -> tokio::io::Result<()> {
    // Get the environment variable for the interface listener
    let var = env::var("INTERFACE_LISTEN").expect("INTERFACE_LISTEN must be set");
    let port = u16::from_str(&var).expect("INTERFACE_LISTEN must be a u16");

    // Build the address to send to
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let mut stream = TcpStream::connect(addr).await?;

    log::debug!("Connected to: {}", addr);

    stream.write(&job.as_bytes()).await?;

    log::info!("Forwarded a message to the interface: {:?}", job);

    Ok(())
}

/// Inserts an [`JobConfiguration`] into MongoDB and returns the ID of the job.
async fn insert_to_queue(job: &Job, collection: Collection) -> ServerResult<()> {
    let document = to_document(&job)?;

    if collection.insert_one(document, None).await.is_ok() {
        log::info!("Inserted {:?} to the MongoDB queue", job);
    } else {
        log::error!("Failed to insert {:?} to the MongoDB queue", job);
    }

    Ok(())
}
