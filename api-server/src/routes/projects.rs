//! Defines the routes specific to project operations.

use async_std::stream::StreamExt;
use bzip2::{Action, Compress, Compression};
use chrono::Utc;
use mongodb::bson::Binary;
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use tide::{http::mime, Request, Response};

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

/// This route will create a new project in the database
/// This will take a project name and a user ID and will create 
/// a Project model and will place it in the database. 
pub async fn new(req: Request<State>) -> tide::Result {
    let state = req.state();
    let database = state.client.database("sybl");
    let projects = database.collection("projects");

    // get user ID
    let user_id: String = req.param("user_id")?;
    let user_id: ObjectId = ObjectId::with_string(&user_id).unwrap();
    // get name
    let name: String = req.param("name")?;

    let project = Project {
        id: Some(ObjectId::new()),
        name,
        date_created: bson::DateTime(Utc::now()),
        user_id: Some(user_id),
    };

    let document = mongodb::bson::ser::to_document(&project).unwrap();
    let id = projects.insert_one(document, None).await?.inserted_id;

    Ok(response_from_json(doc! {"project_id": id}))
}

/// This route will create a dataset associated with a project
/// It will take a project ID and a dataset (a file like a CSV) and 
/// will compress the file and create a Dataset model. This model is 
/// then placed into the database and is associated with a project. 
/// If something goes wrong, it will return a 404 stating that something 
/// is wrong. This is generally because the compression has failed.
pub async fn add(req: Request<State>) -> tide::Result {
    let state = req.state();
    let database = state.client.database("sybl");
    let datasets = database.collection("datasets");

    let data: String = req.param("content")?;
    let project_id: String = req.param("project_id")?;
    let project_id: ObjectId = ObjectId::with_string(&project_id).unwrap();
    log::info!("Dataset received: {:?}", &data);

    let mut comp_dataset = vec![];
    let mut compressor = Compress::new(Compression::best(), 250);
    match compressor.compress(data.as_bytes(), &mut comp_dataset, Action::Run) {
        Ok(s) => match s {
            bzip2::Status::StreamEnd => Ok(Response::builder(404).build()),
            bzip2::Status::MemNeeded => Ok(Response::builder(404).build()),
            _ => {
                let dataset = Dataset {
                    id: Some(ObjectId::new()),
                    project_id: Some(project_id),
                    date_created: bson::DateTime(Utc::now()),
                    dataset: Some(Binary {
                        subtype: bson::spec::BinarySubtype::Generic,
                        bytes: comp_dataset,
                    }),
                };

                let document = mongodb::bson::ser::to_document(&dataset).unwrap();
                let id = datasets.insert_one(document, None).await?.inserted_id;

                Ok(response_from_json(doc! {"dataset_id": id}))
            }
        },
        Err(_) => Ok(Response::builder(404).build()),
    }
}
