//! Defines the routes specific to project operations.

use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::stream::StreamExt;
use mongodb::{
    bson::{doc, document::Document},
    Collection,
};
use tide::{Request, Response};

use crate::routes::{check_project_exists, check_user_exists, response_from_json, tide_err};
use crate::State;
use crypto::clean;
use models::dataset_details::DatasetDetails;
use models::datasets::Dataset;
use models::predictions::Prediction;
use models::projects::{Project, Status};
use utils::compress::{compress_vec, decompress_data};

/// Finds a project in the database given an identifier.
///
/// Given a project identifier, finds the project in the database and returns it as a JSON object.
/// If the project does not exist, returns a 404 response code.
pub async fn get_project(req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let projects = database.collection("projects");
    let details = database.collection("dataset_details");

    let project_id: String = req.param("project_id")?;
    let object_id = check_project_exists(&project_id, &projects).await?;

    let filter = doc! { "_id": &object_id };
    // Unwrap is fine here as we already checked it exists
    let doc = projects.find_one(filter, None).await?.unwrap();

    // get that project from the projects collection
    let filter = doc! { "project_id": &object_id };
    let details_doc = details.find_one(filter, None).await?;

    let response = if let Some(details_doc) = details_doc {
        log::info!("{:?}", &details_doc);
        doc! {"project": &doc, "details": details_doc}
    } else {
        log::info!("{:?}", &details_doc);
        doc! {"project": &doc, "details": {}}
    };

    log::info!("{:?}", &response);
    Ok(response_from_json(response))
}

/// Patches a project with the provided data.
///
/// Given a project identifier, finds and updates the project in the database
/// matching new data
/// If project does not exist return a 404
pub async fn patch_project(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;
    let database = req.state().client.database("sybl");
    let projects = database.collection("projects");

    let project_id: String = req.param("project_id")?;
    let object_id = check_project_exists(&project_id, &projects).await?;

    let filter = doc! { "_id": &object_id };
    let update_doc = doc! { "$set": doc };
    projects.update_one(filter, update_doc, None).await?;

    Ok(Response::builder(200).build())
}

/// Deletes a project provided a valid project id.
///
/// Given a project identifier, deletes a project from the database.
/// If the project ID is invalid return a 422
/// if project is not found return a 422
///
/// Will not currently authenticate the userid
pub async fn delete_project(req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let projects = database.collection("projects");

    let project_id: String = req.param("project_id")?;
    let object_id = check_project_exists(&project_id, &projects).await?;

    let filter = doc! { "_id": &object_id };
    projects.delete_one(filter, None).await?;

    Ok(Response::builder(200).build())
}

/// Finds all the projects related to a given user.
///
/// Given a user identifier, finds all the projects in the database that the user owns. If the user
/// doesn't exist or an invalid identifier is given, returns a 404 response.
pub async fn get_user_projects(req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let projects = database.collection("projects");
    let users = database.collection("users");

    let user_id: String = req.param("user_id")?;
    let object_id = check_user_exists(&user_id, &users).await?;

    let filter = doc! { "user_id": &object_id };
    let cursor = projects.find(filter, None).await?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    Ok(response_from_json(documents.unwrap()))
}

/// Creates a new project related to a given user.
///
/// Given a user id, a project name and description, a project will
/// be created and saved in the database. This can fail if the user id
/// provided doesn't exist.
pub async fn new(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;
    let state = req.state();
    let database = state.client.database("sybl");
    let projects = database.collection("projects");
    let users = database.collection("users");

    // get user ID
    let user_id: String = req.param("user_id")?;
    let user_id = check_user_exists(&user_id, &users).await?;

    // get name
    let name = clean(doc.get_str("name")?);
    let description = clean(doc.get_str("description")?);

    let project = Project::new(&name, &description, user_id);

    let document = mongodb::bson::ser::to_document(&project)?;
    let id = projects.insert_one(document, None).await?.inserted_id;

    Ok(response_from_json(doc! {"project_id": id}))
}

/// Saves a dataset to MongoDB for associated project.
///
/// This will take in a project id and a dataset. This route will
/// compress the dataset using BZip2 and will store this compressed
/// data in the database as binary data. This can go wrong if there's
/// an error writing out the compressed data to the vector or if there
/// is an error finishing the compression stream. Both times an error
/// will return a 404 to the caller.
pub async fn add_data(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;
    let state = req.state();
    let database = state.client.database("sybl");

    let datasets = database.collection("datasets");
    let dataset_details = database.collection("dataset_details");
    let projects = database.collection("projects");

    let data = clean(doc.get_str("content")?);
    let project_id: String = req.param("project_id")?;
    let object_id = check_project_exists(&project_id, &projects).await?;

    // Check whether the project has data already
    let project_has_data = datasets
        .find_one(doc! { "project_id": &object_id }, None)
        .await?
        .is_some();

    log::info!("Project already has data: {}", project_has_data);

    let analysis = utils::analysis::analyse(&data);
    let (train, predict) = utils::infer_train_and_predict(&data);
    let column_types = analysis.types;
    let data_head = analysis.header;

    log::info!("Dataset types: {:?}", &column_types);

    // Compress the input data
    let compressed = compress_vec(&train).map_err(|_| tide_err(422, "failed compression"))?;
    let compressed_predict =
        compress_vec(&predict).map_err(|_| tide_err(422, "failed compression"))?;

    let details = DatasetDetails::new(object_id.clone(), data_head, column_types);
    let dataset = Dataset::new(object_id.clone(), compressed, compressed_predict);

    // If the project has data, delete the existing information
    if project_has_data {
        let query = doc! { "project_id": &object_id };
        datasets.delete_one(query.clone(), None).await?;
        dataset_details.delete_one(query, None).await?;
    } else {
        // Update the project status
        projects
            .update_one(
                doc! { "_id": &object_id},
                doc! {"$set": {"status": Status::Ready}},
                None,
            )
            .await?;
    }

    // Insert the dataset details and the dataset itself
    let document = mongodb::bson::ser::to_document(&details)?;
    dataset_details.insert_one(document, None).await?;

    let document = mongodb::bson::ser::to_document(&dataset)?;
    let id = datasets.insert_one(document, None).await?.inserted_id;

    Ok(response_from_json(doc! {"dataset_id": id}))
}

/// Gets the dataset details for a project
///
/// Project Id passed in as part of route and the dataset details
/// for that project are returned from the database.
pub async fn overview(req: Request<State>) -> tide::Result {
    let state = req.state();
    let database = state.client.database("sybl");
    let dataset_details = database.collection("dataset_details");
    let projects = database.collection("projects");

    let project_id: String = req.param("project_id")?;
    let object_id = check_project_exists(&project_id, &projects).await?;

    let filter = doc! { "project_id": &object_id };
    let cursor = dataset_details.find(filter, None).await?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    Ok(response_from_json(documents.unwrap()))
}

/// Route returns back uncompressed dataset
///
/// Makes a request to mongodb and gets dataset associated with
/// project id. Compressed data is then taken from returned struct
/// and is decompressed before being sent in a response back to the
/// user.
pub async fn get_data(req: Request<State>) -> tide::Result {
    let state = req.state();
    let database = state.client.database("sybl");
    let datasets = database.collection("datasets");
    let projects = database.collection("projects");

    let project_id: String = req.param("project_id")?;
    let object_id = check_project_exists(&project_id, &projects).await?;
    let filter = doc! { "project_id": &object_id };

    // Find the dataset in the database
    let document = datasets
        .find_one(filter, None)
        .await?
        .ok_or_else(|| tide_err(404, "dataset not found"))?;

    // Parse the dataset itself
    let dataset = mongodb::bson::de::from_document::<Dataset>(document)
        .map_err(|_| tide_err(422, "failed to parse dataset"))?;

    let comp_train = dataset.dataset.expect("missing training dataset").bytes;
    let comp_predict = dataset.predict.expect("missing prediction dataset").bytes;

    let decomp_train =
        decompress_data(&comp_train).map_err(|_| tide_err(422, "failed decompression"))?;
    let decomp_predict =
        decompress_data(&comp_predict).map_err(|_| tide_err(422, "failed decompression"))?;

    let train = clean(std::str::from_utf8(&decomp_train)?);
    let predict = clean(std::str::from_utf8(&decomp_predict)?);

    log::info!("Training data: {:?}", &train);
    log::info!("Prediction data: {:?}", &predict);

    Ok(response_from_json(
        doc! {"dataset": train, "predict": predict},
    ))
}

/// Begins the processing of data associated with a project.
///
/// Checks that the project exists, before sending the identifier of its dataset to the interface
/// layer, which will then forward it to the DCL for processing. Updates the project state to
/// `State::Processing`.
pub async fn begin_processing(req: Request<State>) -> tide::Result {
    let state = req.state();
    let database = state.client.database("sybl");

    let projects = database.collection("projects");
    let datasets = database.collection("datasets");

    let project_id: String = req.param("project_id")?;
    let object_id = check_project_exists(&project_id, &projects).await?;

    // Find the dataset in the database
    let filter = doc! { "project_id": &object_id };
    let document = datasets
        .find_one(filter, None)
        .await?
        .ok_or_else(|| tide_err(404, "dataset not found"))?;

    // Parse the dataset itself
    let dataset = mongodb::bson::de::from_document::<Dataset>(document)
        .map_err(|_| tide_err(422, "failed to parse dataset"))?;

    // Send a request to the interface layer
    let hex = dataset.id.expect("Dataset with no identifier").to_hex();

    if forward_to_interface(&hex).await.is_err() {
        log::warn!("Failed to forward: {}", hex);
        insert_to_queue(&hex, database.collection("processing_queue")).await?;
    }

    // Mark the project as processing
    let update = doc! { "$set": doc!{ "status": Status::Processing } };
    projects
        .update_one(doc! { "_id": &object_id}, update, None)
        .await?;

    Ok(response_from_json(doc! {"success": true}))
}

/// Gets the predicted data for a project.
///
/// Queries the database for all [`Prediction`] instances for a given project identifier, before
/// decompressing each and returning them.
pub async fn get_predictions(req: Request<State>) -> tide::Result {
    // Get the database instance
    let database = req.state().client.database("sybl");

    // Get the projects and the predictions collections
    let projects = database.collection("projects");
    let predictions = database.collection("predictions");

    // Get the project identifier and check it exists
    let project_id: String = req.param("project_id")?;
    let object_id = check_project_exists(&project_id, &projects).await?;

    // Find the predictions for the given project
    let filter = doc! { "project_id": &object_id };
    let cursor = predictions.find(filter, None).await?;

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

    Ok(response_from_json(doc! {"predictions": decompressed}))
}

async fn forward_to_interface(identifier: &str) -> async_std::io::Result<()> {
    log::debug!("Forwarding an identifier to the interface: {}", identifier);

    let mut stream = TcpStream::connect("127.0.0.1:5000").await?;
    stream.write(identifier.as_bytes()).await?;
    stream.shutdown(std::net::Shutdown::Both)?;

    log::info!("Forwarded an identifier to the interface: {}", identifier);

    Ok(())
}

async fn insert_to_queue(identifier: &str, collection: Collection) -> mongodb::error::Result<()> {
    log::debug!("Inserting {} to the MongoDB interface queue", identifier);

    let document = doc! { "identifier": identifier };

    match collection.insert_one(document, None).await {
        Ok(_) => log::info!("Inserted {} to the MongoDB queue", identifier),
        Err(_) => log::error!("Failed to insert {} to the MongoDB queue", identifier),
    }

    Ok(())
}
