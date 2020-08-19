use super::*;
use async_std::stream::StreamExt;
use tide::{Request, Response};
use tide;
use crate::models::users::User;
use crate::models::model::Model;
use mongodb::bson::{doc, oid::ObjectId};
use serde_json::value::Map;


pub async fn show(req: Request<State>) -> tide::Result {
    let state = &req.state();
    let db = &state.client.database(&state.db_name);

    let mut cursor = User::find(db.clone(), None, None).await.unwrap();
    let mut docs: Vec<User> = Vec::new();
    while let Some(user) = cursor.next().await {
        docs.push(user?);
    }

    let response = Response::builder(200)
                        .body(json!(&docs))
                        .content_type(mime::JSON)
                        .build();

    Ok(response)
}

pub async fn get(req: Request<State>) -> tide::Result{
    let state = &req.state();
    let db = &state.client.database("test");
    let id = req.param::<String>("user_id")?;
    println!("UserID: {}", id);

    let object_id = ObjectId::with_string(&id).unwrap();
    println!("ObjectID: {:?}", &object_id);
    let filter = doc! { "_id": object_id };

    println!("Filter: {}", &filter);

    let filter = doc!{"token": "tkn1"};

    let doc = User::find_one(db.clone(), filter, None).await?;
    println!("{:?}", doc);

    

    let response = Response::builder(200)
                        .body(json!(doc))
                        .content_type(mime::JSON)
                        .build();

    Ok(response)
}