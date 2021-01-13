//! Defines the routes specific to project operations.

use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;

use actix_web::{web, HttpResponse};
use mongodb::{
    bson::{doc, document::Document},
    Collection,
};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;

use crate::dodona_error::DodonaError;
use crate::routes::{check_project_exists, check_user_exists, response_from_json};
use crate::AppState;
use crypto::clean;
use messages::{InterfaceMessage, WriteLengthPrefix};
use models::dataset_details::DatasetDetails;
use models::datasets::Dataset;
use models::jobs::Job;
use models::predictions::Prediction;
use models::projects::{Project, Status};
use utils::compress::{compress_vec, decompress_data};
use utils::ColumnType;

/// Finds a project in the database given an identifier.
///
/// Given a project identifier, finds the project in the database and returns it as a JSON object.
/// If the project does not exist, returns a 404 response code.
pub async fn get_project(
    app_data: web::Data<AppState>,
    project_id: web::Path<String>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let projects = database.collection("projects");
    let details = database.collection("dataset_details");

    let object_id = check_project_exists(&project_id, &projects).await?;

    let filter = doc! { "_id": &object_id };
    // Unwrap is fine here as we already checked it exists
    let doc = projects
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    // get that project from the projects collection
    let filter = doc! { "project_id": &object_id };
    let details_doc = details
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    let response = if let Some(details_doc) = details_doc {
        log::info!("{:?}", &details_doc);
        doc! {"project": doc.unwrap(), "details": details_doc}
    } else {
        log::info!("{:?}", &details_doc);
        doc! {"project": doc.unwrap(), "details": {}}
    };

    log::info!("{:?}", &response);
    response_from_json(response)
}

/// Patches a project with the provided data.
///
/// Given a project identifier, finds and updates the project in the database
/// matching new data
/// If project does not exist return a 404
pub async fn patch_project(
    app_data: web::Data<AppState>,
    project_id: web::Path<String>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let projects = database.collection("projects");

    let object_id = check_project_exists(&project_id, &projects).await?;

    let filter = doc! { "_id": &object_id };
    let update_doc = doc! { "$set": doc.into_inner() };
    projects
        .update_one(filter, update_doc, None)
        .await
        .map_err(|_| DodonaError::NotFound)?;

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
    app_data: web::Data<AppState>,
    project_id: web::Path<String>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let projects = database.collection("projects");

    let object_id = check_project_exists(&project_id, &projects).await?;

    let filter = doc! { "_id": &object_id };
    projects
        .delete_one(filter, None)
        .await
        .map_err(|_| DodonaError::NotFound)?;

    Ok(HttpResponse::Ok().finish())
}

/// Finds all the projects related to a given user.
///
/// Given a user identifier, finds all the projects in the database that the user owns. If the user
/// doesn't exist or an invalid identifier is given, returns a 404 response.
pub async fn get_user_projects(
    app_data: web::Data<AppState>,
    user_id: web::Path<String>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let projects = database.collection("projects");
    let users = database.collection("users");

    let object_id = check_user_exists(&user_id, &users).await?;

    let filter = doc! { "user_id": &object_id };
    let cursor = projects
        .find(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    response_from_json(documents.unwrap())
}

/// Creates a new project related to a given user.
///
/// Given a user id, a project name and description, a project will
/// be created and saved in the database. This can fail if the user id
/// provided doesn't exist.
pub async fn new(
    app_data: web::Data<AppState>,
    user_id: web::Path<String>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let projects = database.collection("projects");
    let users = database.collection("users");

    // get user ID
    let user_id = check_user_exists(&user_id, &users)
        .await
        .map_err(|_| DodonaError::Invalid)?;

    // get name
    let name = clean(doc.get_str("name").map_err(|_| DodonaError::Invalid)?);
    let description = clean(
        doc.get_str("description")
            .map_err(|_| DodonaError::Invalid)?,
    );

    let project = Project::new(&name, &description, user_id);

    let document = mongodb::bson::ser::to_document(&project).map_err(|_| DodonaError::Invalid)?;
    let id = projects
        .insert_one(document, None)
        .await
        .map_err(|_| DodonaError::Invalid)?
        .inserted_id;

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
    app_data: web::Data<AppState>,
    project_id: web::Path<String>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");

    let datasets = database.collection("datasets");
    let dataset_details = database.collection("dataset_details");
    let projects = database.collection("projects");

    let data = clean(doc.get_str("content").map_err(|_| DodonaError::Unknown)?);
    let object_id = check_project_exists(&project_id, &projects).await?;

    // Check whether the project has data already
    let project_has_data = datasets
        .find_one(doc! { "project_id": &object_id }, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .is_some();

    log::info!("Project already has data: {}", project_has_data);

    let analysis = utils::analysis::analyse(&data);
    let (train, predict) = utils::infer_train_and_predict(&data);
    let column_types = analysis.types;
    let data_head = analysis.header;

    log::info!("Dataset types: {:?}", &column_types);

    // Compress the input data
    let compressed = compress_vec(&train).map_err(|_| DodonaError::Invalid)?;
    let compressed_predict = compress_vec(&predict).map_err(|_| DodonaError::Invalid)?;

    let details = DatasetDetails::new(object_id.clone(), data_head, column_types);
    let dataset = Dataset::new(object_id.clone(), compressed, compressed_predict);

    // If the project has data, delete the existing information
    if project_has_data {
        let query = doc! { "project_id": &object_id };
        datasets
            .delete_one(query.clone(), None)
            .await
            .map_err(|_| DodonaError::Unknown)?;
        dataset_details
            .delete_one(query, None)
            .await
            .map_err(|_| DodonaError::Unknown)?;
    } else {
        // Update the project status
        projects
            .update_one(
                doc! { "_id": &object_id},
                doc! {"$set": {"status": Status::Ready}},
                None,
            )
            .await
            .map_err(|_| DodonaError::Unknown)?;
    }

    // Insert the dataset details and the dataset itself
    let document = mongodb::bson::ser::to_document(&details).map_err(|_| DodonaError::Unknown)?;
    dataset_details
        .insert_one(document, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    let document = mongodb::bson::ser::to_document(&dataset).map_err(|_| DodonaError::Unknown)?;
    let id = datasets
        .insert_one(document, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .inserted_id;

    response_from_json(doc! {"dataset_id": id})
}

/// Gets the dataset details for a project
///
/// Project Id passed in as part of route and the dataset details
/// for that project are returned from the database.
pub async fn overview(
    app_data: web::Data<AppState>,
    project_id: web::Path<String>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let dataset_details = database.collection("dataset_details");
    let projects = database.collection("projects");

    let object_id = check_project_exists(&project_id, &projects).await?;

    let filter = doc! { "project_id": &object_id };
    let cursor = dataset_details
        .find(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    response_from_json(documents.unwrap())
}

/// Route returns back uncompressed dataset
///
/// Makes a request to mongodb and gets dataset associated with
/// project id. Compressed data is then taken from returned struct
/// and is decompressed before being sent in a response back to the
/// user.
pub async fn get_data(
    app_data: web::Data<AppState>,
    project_id: web::Path<String>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let datasets = database.collection("datasets");
    let projects = database.collection("projects");

    let object_id = check_project_exists(&project_id, &projects).await?;
    let filter = doc! { "project_id": &object_id };

    // Find the dataset in the database
    let document = datasets
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .ok_or(DodonaError::NotFound)?;

    // Parse the dataset itself
    let dataset =
        mongodb::bson::de::from_document::<Dataset>(document).map_err(|_| DodonaError::Invalid)?;

    let comp_train = dataset.dataset.expect("missing training dataset").bytes;
    let comp_predict = dataset.predict.expect("missing prediction dataset").bytes;

    let decomp_train = decompress_data(&comp_train).map_err(|_| DodonaError::Invalid)?;
    let decomp_predict = decompress_data(&comp_predict).map_err(|_| DodonaError::Invalid)?;

    let train = clean(std::str::from_utf8(&decomp_train).map_err(|_| DodonaError::Unknown)?);
    let predict = clean(std::str::from_utf8(&decomp_predict).map_err(|_| DodonaError::Unknown)?);

    log::info!("Training data: {:?}", &train);
    log::info!("Prediction data: {:?}", &predict);

    response_from_json(doc! {"dataset": train, "predict": predict})
}

/// Begins the processing of data associated with a project.
///
/// Checks that the project exists, before sending the identifier of its dataset to the interface
/// layer, which will then forward it to the DCL for processing. Updates the project state to
/// `State::Processing`.
pub async fn begin_processing(
    app_data: web::Data<AppState>,
    project_id: web::Path<String>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");

    let projects = database.collection("projects");
    let datasets = database.collection("datasets");
    let dataset_details = database.collection("dataset_details");

    let timeout: i32 = doc.get_str("timeout").unwrap().parse().unwrap();
    log::info!("Timeout is here: {}", &timeout);

    let object_id = check_project_exists(&project_id, &projects).await?;

    // Find the dataset in the database
    let filter = doc! { "project_id": &object_id };
    let document = datasets
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .ok_or(DodonaError::NotFound)?;

    // Parse the dataset itself
    let dataset =
        mongodb::bson::de::from_document::<Dataset>(document).map_err(|_| DodonaError::Invalid)?;

    // Send a request to the interface layer
    let identifier = dataset.id.expect("Dataset with no identifier");

    let filter = doc! { "project_id": &object_id };
    let document = dataset_details
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .ok_or(DodonaError::NotFound)?;

    // Parse the dataset detail itself
    let dataset_detail = mongodb::bson::de::from_document::<DatasetDetails>(document)
        .map_err(|_| DodonaError::NotFound)?;

    let types = dataset_detail
        .column_types
        .values()
        .map(|x| match x.column_type {
            ColumnType::Categorical(..) => String::from("Categorical"),
            ColumnType::Numerical(..) => String::from("Numerical"),
        })
        .collect();

    let config = InterfaceMessage::Config {
        id: identifier.clone(),
        timeout,
        column_types: types,
    };

    if forward_to_interface(&config).await.is_err() {
        log::warn!("Failed to forward: {:?}", config);
        insert_to_queue(&config, database.collection("jobs"))
            .await
            .map_err(|_| DodonaError::Unknown)?;
    }

    // Mark the project as processing
    let update = doc! { "$set": doc!{ "status": Status::Processing } };
    projects
        .update_one(doc! { "_id": &object_id}, update, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    response_from_json(doc! {"success": true})
}

/// Gets the predicted data for a project.
///
/// Queries the database for all [`Prediction`] instances for a given project identifier, before
/// decompressing each and returning them.
pub async fn get_predictions(
    app_data: web::Data<AppState>,
    project_id: web::Path<String>,
) -> Result<HttpResponse, DodonaError> {
    // Get the database instance
    let database = app_data.client.database("sybl");

    // Get the projects and the predictions collections
    let projects = database.collection("projects");
    let predictions = database.collection("predictions");

    // Get the project identifier and check it exists
    let object_id = check_project_exists(&project_id, &projects).await?;

    // Find the predictions for the given project
    let filter = doc! { "project_id": &object_id };
    let cursor = predictions
        .find(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    // Decompress the predictions for each instance found
    let decompressed: Vec<_> = cursor
        .filter_map(Result::ok)
        .map(|document| {
            let prediction: Prediction = mongodb::bson::de::from_document(document).unwrap();
            let decompressed = decompress_data(&prediction.predictions.bytes).unwrap();

            String::from_utf8(decompressed).unwrap()
        })
        .collect()
        .await;

    response_from_json(doc! {"predictions": decompressed})
}

async fn forward_to_interface(msg: &InterfaceMessage) -> tokio::io::Result<()> {
    log::debug!("Forwarding an message to the interface: {:?}", msg);

    // Get the environment variable for the interface listener
    let var = env::var("INTERFACE_LISTEN").expect("INTERFACE_LISTEN must be set");
    let port = u16::from_str(&var).expect("INTERFACE_LISTEN must be a u16");

    // Build the address to send to
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let mut stream = TcpStream::connect(addr).await?;

    log::info!("Connected to: {}", addr);

    stream.write(&msg.as_bytes()).await?;

    log::info!("Forwarded a message to the interface: {:?}", msg);

    Ok(())
}

async fn insert_to_queue(
    msg: &InterfaceMessage,
    collection: Collection,
) -> mongodb::error::Result<()> {
    log::debug!("Inserting {:?} to the MongoDB interface queue", msg);

    let job = Job::new(msg.clone());
    let document = mongodb::bson::ser::to_document(&job).unwrap();

    if collection.insert_one(document, None).await.is_ok() {
        log::info!("Inserted {:?} to the MongoDB queue", msg);
    } else {
        log::error!("Failed to insert {:?} to the MongoDB queue", msg);
    }

    Ok(())
}
