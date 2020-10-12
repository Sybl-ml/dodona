use async_std::stream::StreamExt;
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use tide::{Request, Response, http::mime};

use crate::models::projects::Project;
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


pub async fn new(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;
    log::debug!("Document received: {:?}", &doc);

    // let state = req.state();

    // let database = state.client.database("sybl");
    // let users = database.collection("users");

    // let password = doc.get_str("password").unwrap();


    Ok(Response::builder(200)
        .body(json!(doc! {"token": "hello"}))
        .content_type(mime::JSON)
        .build())
}
