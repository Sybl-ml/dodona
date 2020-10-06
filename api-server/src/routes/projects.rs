use async_std::stream::StreamExt;
use mongodb::bson::{doc, oid::ObjectId};
use tide::http::mime;
use tide::{Request, Response};

use crate::models::model::Model;
use crate::models::projects::Project;
use crate::State;

pub async fn get_user_projects(req: Request<State>) -> tide::Result {
    let state = req.state();
    let db = state.client.database("sybl");

    let user_id: String = req.param("user_id")?;
    let object_id = ObjectId::with_string(&user_id).unwrap();

    let filter = doc! { "user_id": object_id };
    let mut cursor = Project::find(db.clone(), filter, None).await?;
    let mut docs: Vec<Project> = Vec::new();
    while let Some(user) = cursor.next().await {
        docs.push(user?);
    }
    Ok(Response::builder(200)
        .body(json!(docs))
        .content_type(mime::JSON)
        .build())
}

pub async fn get_all(req: Request<State>) -> tide::Result {
    let state = req.state();
    let db = state.client.database("sybl");

    let mut cursor = Project::find(db.clone(), doc! {}, None).await?;
    let mut docs: Vec<Project> = Vec::new();
    while let Some(user) = cursor.next().await {
        log::info!("{:?}", &user);
        docs.push(user?);
    }
    Ok(Response::builder(200)
        .body(json!(docs))
        .content_type(mime::JSON)
        .build())
}
