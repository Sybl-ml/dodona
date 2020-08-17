use super::*;
use async_std::stream::StreamExt;
use tide::Request;
use tide;
use bson::{oid::ObjectId, document::Document};

pub async fn show(req: Request<State>) -> tide::Result {
    let state = &req.state();
    let db = &state.client.database("sybl");
    let coll = db.collection("users");
    let mut cursor = coll.find(None, None).await.unwrap();
    while let Some(user) = cursor.next().await {
        println!("{:?}", user?);
    }

    Ok(tide::Redirect::new(format!("/")).into())
}