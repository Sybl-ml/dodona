//! Defines the routes specific to project operations.
use std::io::prelude::*;

use async_std::stream::StreamExt;
use bzip2::write::BzEncoder;
use bzip2::Compression;
use chrono::Utc;
use mongodb::bson::Binary;
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use tide::{Request, Response};

use crate::models::datasets::Dataset;
use crate::models::projects::Project;
use crate::routes::response_from_json;
use crate::State;

/// Gets all the projects from the database.
///
/// Defines a catch-all testing route that will pull all available projects and their information
/// from the Mongo database.
pub async fn get_all(req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let projects = database.collection("projects");

    let cursor = projects.find(None, None).await?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    Ok(response_from_json(documents.unwrap()))
}

/// Finds a project in the database given an identifier.
///
/// Given a project identifier, finds the project in the database and returns it as a JSON object.
/// If the project does not exist, returns a 404 response code.
pub async fn get_project(req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let projects = database.collection("projects");

    let project_id: String = req.param("project_id")?;
    let object_id = match ObjectId::with_string(&project_id) {
        Ok(id) => id,
        Err(_) => return Ok(Response::builder(404).body("invalid project id").build()),
    };

    let filter = doc! { "_id": object_id };

    let doc = projects.find_one(filter, None).await?.unwrap();
    let proj: Project = mongodb::bson::de::from_document(doc).unwrap();
    Ok(response_from_json(proj))
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

    let filter = doc! { "user_id": &user_id };
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
    let name = doc.get_str("name")?;
    let description = doc.get_str("description")?;

    let project = Project {
        id: Some(ObjectId::new()),
        name: String::from(name),
        description: String::from(description),
        date_created: bson::DateTime(Utc::now()),
        user_id: Some(user_id),
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
pub async fn add(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;
    let state = req.state();
    let database = state.client.database("sybl");
    let datasets = database.collection("datasets");
    let projects = database.collection("projects");

    let data = doc.get_str("content")?;
    let project_id: String = req.param("project_id")?;
    let project_id: ObjectId = ObjectId::with_string(&project_id)?;

    // check project exists
    let found_project = projects.find_one(doc! { "_id": &project_id}, None).await?;
    if found_project.is_none() {
        return Ok(Response::builder(404).body("project not found").build());
    }

    log::info!("Dataset received: {:?}", &data);

    let mut write_compress = BzEncoder::new(vec![], Compression::best());

    if write_compress.write(data.as_bytes()).is_err() {
        return Ok(Response::builder(404)
            .body("Error finishing writing stream")
            .build());
    }

    match write_compress.finish() {
        Ok(compressed) => {
            log::info!("Compressed data: {:?}", &compressed);
            let dataset = Dataset {
                id: Some(ObjectId::new()),
                project_id: Some(project_id),
                date_created: bson::DateTime(Utc::now()),
                dataset: Some(Binary {
                    subtype: bson::spec::BinarySubtype::Generic,
                    bytes: compressed,
                }),
            };

            let document = mongodb::bson::ser::to_document(&dataset)?;
            let id = datasets.insert_one(document, None).await?.inserted_id;

            Ok(response_from_json(doc! {"dataset_id": id}))
        }
        Err(_) => Ok(Response::builder(404).build()),
    }
}
