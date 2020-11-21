//! Defines the routes specific to project operations.

use async_std::net::TcpStream;
use async_std::prelude::*;
use async_std::stream::StreamExt;
use chrono::Utc;
use mongodb::bson;
use mongodb::bson::{doc, document::Document, oid::ObjectId, Binary};
use tide::{Request, Response};

use crate::routes::response_from_json;
use crate::State;
use crypto::clean;
use models::dataset_details::DatasetDetails;
use models::datasets::Dataset;
use models::projects::{Project, Status};

/// Finds a project in the database given an identifier.
///
/// Given a project identifier, finds the project in the database and returns it as a JSON object.
/// If the project does not exist, returns a 404 response code.
pub async fn get_project(req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let projects = database.collection("projects");
    let details = database.collection("dataset_details");

    let project_id: String = req.param("project_id")?;
    let object_id = match ObjectId::with_string(&project_id) {
        Ok(id) => id,
        Err(_) => return Ok(Response::builder(422).body("invalid project id").build()),
    };

    let filter = doc! { "_id": &object_id };
    let doc = projects.find_one(filter, None).await?;

    // if project exists get project from projects collection and
    // get dataset details from dataset_details collection
    if let Some(doc) = doc {
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
    } else {
        Ok(Response::builder(404).body("project id not found").build())
    }
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
    let object_id = match ObjectId::with_string(&project_id) {
        Ok(id) => id,
        Err(_) => return Ok(Response::builder(422).body("invalid project id").build()),
    };

    let filter = doc! { "_id": &object_id };
    let update_doc = doc! { "$set": doc };
    let update_id = projects
        .find_one_and_update(filter, update_doc, None)
        .await?;

    if update_id.is_none() {
        return Ok(Response::builder(404).body("project not found").build());
    }

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
    let object_id = match ObjectId::with_string(&project_id) {
        Ok(id) => id,
        Err(_) => return Ok(Response::builder(422).body("invalid project id").build()),
    };

    let filter = doc! { "_id": &object_id };
    let deleted_id = projects.find_one_and_delete(filter, None).await?;

    if deleted_id.is_none() {
        return Ok(Response::builder(404).body("project not found").build());
    }

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

    let object_id = match ObjectId::with_string(&user_id) {
        Ok(id) => id,
        Err(_) => return Ok(Response::builder(404).body("invalid user id").build()),
    };

    let found_user = users.find_one(doc! { "_id": &object_id}, None).await?;

    if found_user.is_none() {
        return Ok(Response::builder(404).body("user not found").build());
    }

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
    let user_id: ObjectId = ObjectId::with_string(&user_id)?;

    // Check user ID
    let found_user = users.find_one(doc! { "_id": &user_id}, None).await?;
    if found_user.is_none() {
        return Ok(Response::builder(404).body("user not found").build());
    }

    // get name
    let name = clean(doc.get_str("name")?);
    let description = clean(doc.get_str("description")?);

    let project = Project {
        id: Some(ObjectId::new()),
        name: String::from(name),
        description: String::from(description),
        date_created: bson::DateTime(Utc::now()),
        user_id: Some(user_id),
        status: Status::Unfinished,
    };

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
    let project_id: ObjectId = ObjectId::with_string(&project_id)?;

    // check project exists
    let found_project = projects
        .find_one_and_update(
            doc! { "_id": &project_id},
            doc! {"$set": {"status": Status::Ready.to_string()}},
            None,
        )
        .await?;
    if found_project.is_none() {
        return Ok(Response::builder(404).body("project not found").build());
    }

    // Check whether the project has data already
    let project_has_data = datasets
        .find_one(doc! { "project_id": &project_id }, None)
        .await?
        .is_some();
    log::info!("Project already has data: {}", project_has_data);

    let analysis = utils::analyse(&data);
    let column_types = analysis.types;
    let data_head = analysis.header;

    log::info!("Dataset types: {:?}", &column_types);

    // Compression
    let compressed = match utils::compress_data(&data) {
        Ok(compressed) => compressed,
        Err(_) => return Ok(Response::builder(422).build()),
    };

    let details = DatasetDetails {
        id: None,
        project_id: Some(project_id.clone()),
        date_created: bson::DateTime(Utc::now()),
        head: Some(data_head),
        column_types,
    };

    let dataset = Dataset {
        id: None,
        project_id: Some(project_id.clone()),
        dataset: Some(Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: compressed,
        }),
    };

    // If the project has data, delete the existing information
    if project_has_data {
        let query = doc! { "project_id": &project_id };
        datasets.delete_one(query.clone(), None).await?;
        dataset_details.delete_one(query, None).await?;
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

    let object_id = match ObjectId::with_string(&project_id) {
        Ok(id) => id,
        Err(_) => return Ok(Response::builder(422).body("invalid project id").build()),
    };

    let found_project = projects.find_one(doc! { "_id": &object_id}, None).await?;

    if found_project.is_none() {
        return Ok(Response::builder(404).body("project not found").build());
    }

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

    let object_id = match ObjectId::with_string(&project_id) {
        Ok(id) => id,
        Err(_) => return Ok(Response::builder(422).body("invalid project id").build()),
    };

    log::info!("Project ID: {}", &project_id);

    let found_project = projects.find_one(doc! { "_id": &object_id}, None).await?;

    if found_project.is_none() {
        return Ok(Response::builder(404).body("project not found").build());
    }

    let filter = doc! { "project_id": &object_id };

    // 404 if no dataset
    let dataset = match datasets
        .find_one(filter, None)
        .await?
        .map(|doc| mongodb::bson::de::from_document::<Dataset>(doc).unwrap())
    {
        Some(x) => x,
        None => return Ok(Response::builder(404).build()),
    };

    let comp_data = dataset.dataset.unwrap().bytes;

    match utils::decompress_data(&comp_data) {
        Ok(decompressed) => {
            let decomp_data = clean(std::str::from_utf8(&decompressed)?);
            log::info!("Decompressed data: {:?}", &decomp_data);
            Ok(response_from_json(doc! {"dataset": decomp_data}))
        }
        Err(_) => Ok(Response::builder(404).build()),
    }
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

    // Check the project ID to make sure it exists
    let object_id = match ObjectId::with_string(&project_id) {
        Ok(id) => id,
        Err(_) => return Ok(Response::builder(422).body("invalid project id").build()),
    };

    let project_query = doc! { "_id": &object_id};
    let found_project = projects.find_one(project_query.clone(), None).await?;

    if found_project.is_none() {
        log::error!("Failed to find project with id: {}", &project_id);
        return Ok(Response::builder(404).body("project not found").build());
    }

    let filter = doc! { "project_id": &object_id };
    let dataset = datasets
        .find_one(filter, None)
        .await?
        .map(|doc| mongodb::bson::de::from_document::<Dataset>(doc).unwrap())
        .unwrap();

    // Send a request to the interface layer
    let hex = dataset.id.expect("Dataset with no identifier").to_hex();
    log::info!("Forwarding dataset id: {} to the interface layer", &hex);

    let mut stream = TcpStream::connect("127.0.0.1:5000").await?;
    stream.write(hex.as_bytes()).await?;
    stream.shutdown(std::net::Shutdown::Both)?;

    // Mark the project as processing
    let update = doc! { "$set": doc!{ "status": Status::Processing.to_string() } };
    projects.update_one(project_query, update, None).await?;

    Ok(response_from_json(doc! {"success": true}))
}
