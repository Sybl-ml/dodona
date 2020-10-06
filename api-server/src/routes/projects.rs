use async_std::stream::StreamExt;
use mongodb::bson::{doc, oid::ObjectId};
use tide::http::mime;
use tide::{Request, Response};

use crate::models::model::Model;
use crate::models::projects::Project;
use crate::models::users::User;
use crate::State;

pub async fn get_all(req: Request<State>) -> tide::Result {
    let state = req.state();
    let db = state.client.database("sybl");

    let mut cursor = Project::find(db.clone(), doc! {}, None).await?;
    let mut docs: Vec<Project> = Vec::new();
    while let Some(proj) = cursor.next().await {
        docs.push(proj?);
    }
    Ok(Response::builder(200)
        .body(json!(docs))
        .content_type(mime::JSON)
        .build())
}

pub async fn get_project(req: Request<State>) -> tide::Result {
    let state = req.state();
    let db = state.client.database("sybl");

    let project_id: String = req.param("project_id")?;
    let object_id = ObjectId::with_string(&project_id).unwrap();

    let filter = doc! { "_id": object_id };

    let doc = Project::find_one(db.clone(), filter, None).await?;
    let response = Response::builder(200)
        .body(json!(doc))
        .content_type(mime::JSON)
        .build();

    Ok(response)
}

pub async fn get_user_projects(req: Request<State>) -> tide::Result {
    let state = req.state();
    let db = state.client.database("sybl");

    let user_id: String = req.param("user_id")?;
    let object_id = ObjectId::with_string(&user_id).unwrap();

    // TODO check that user exists in users table when requesting projects
    let filter = doc! { "user_id": &object_id };

    match User::find_one(db.clone(), doc! { "_id": &object_id}, None).await? {
        None => return Ok(Response::builder(404).body("user not found").build()),
        _ => (),
    };

    let mut cursor = Project::find(db.clone(), filter, None).await?;
    let mut docs: Vec<Project> = Vec::new();
    while let Some(proj) = cursor.next().await {
        docs.push(proj?);
    }
    Ok(Response::builder(200)
        .body(json!(docs))
        .content_type(mime::JSON)
        .build())
}
