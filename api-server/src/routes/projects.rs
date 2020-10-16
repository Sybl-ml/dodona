use async_std::stream::StreamExt;
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use mongodb::bson::Binary;
use tide::{Request, Response, http::mime};
use chrono::Utc;
use bzip2::{Compress, Compression, Action};

use crate::models::projects::Project;
use crate::models::datasets::Dataset;
use crate::routes::response_from_json;
use crate::State;

/// route will return all projects in database
/// mainly for testing purposes
pub async fn get_all(req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let projects = database.collection("projects");

    let cursor = projects.find(None, None).await?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    Ok(response_from_json(documents.unwrap()))
}

/// route will return a single project with the id
/// matching the request
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

/// Get all projects related to a user
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

/// This route will create a new project in the database
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
        user_id: Some(user_id)
    };

    let document = mongodb::bson::ser::to_document(&project).unwrap();
    let id = projects.insert_one(document, None).await?.inserted_id;

    Ok(response_from_json(doc! {"project_id": id}))
}


/// This route will create a dataset associated with a project
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
    match compressor.compress(data.as_bytes(), &mut comp_dataset, Action::Run){
        Ok(s) => match s {
            bzip2::Status::StreamEnd => Ok(Response::builder(404).build()),
            bzip2::Status::MemNeeded => Ok(Response::builder(404).build()),
            _ => {
                let dataset = Dataset {
                    id: Some(ObjectId::new()),
                    project_id: Some(project_id),
                    date_created: bson::DateTime(Utc::now()),
                    dataset: Some(Binary{
                        subtype: bson::spec::BinarySubtype::Generic, 
                        bytes: comp_dataset
                    })
                };
            
                let document = mongodb::bson::ser::to_document(&dataset).unwrap();
                let id = datasets.insert_one(document, None).await?.inserted_id;
            
            
                Ok(response_from_json(doc! {"dataset_id": id}))
            }
        },
        Err(_) => Ok(Response::builder(404).build()),
    }

    
}
