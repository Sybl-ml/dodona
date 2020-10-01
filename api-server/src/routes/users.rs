use super::*;
use crate::models::model::Model;
use crate::models::users::User;
use async_std::stream::StreamExt;
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use tide::{Request, Response};

/// This route will take in a user ID in the request and
/// will return the information for that user
pub async fn get(req: Request<State>) -> tide::Result {
    let state = &req.state();
    let db = &state.client.database("sybl");
    let id = req.param::<String>("user_id")?;
    let object_id = ObjectId::with_string(&id).unwrap();
    let filter = doc! { "_id": object_id };
    let doc = User::find_one(db.clone(), filter, None).await?;
    let response = Response::builder(200)
        .body(json!(doc))
        .content_type(mime::JSON)
        .build();

    Ok(response)
}

/// More general version of get. Allows filter to be passed to
/// the find. This will return a JSON object containing multiple
/// users
pub async fn filter(mut req: Request<State>) -> tide::Result {
    let state = &req.state();
    let db = &state.client.database("sybl");
    let filter: Document = req.body_json().await?;
    println!("Filter: {:?}", &filter);
    let mut cursor = User::find(db.clone(), filter, None).await?;
    let mut docs: Vec<User> = Vec::new();
    while let Some(user) = cursor.next().await {
        docs.push(user?);
    }
    Ok(Response::builder(200)
        .body(json!(docs))
        .content_type(mime::JSON)
        .build())
}

/// A User information is passed as a JSON object and it is either updated
/// or created in the database.
/// If a JSON object is passed with an object ID, then it is saved as a user
/// under that ObjectId, updating the current existing information in the DB.
/// If the object is passed without the ObjectId, but has all other fields, it
/// is saved as a new user.
pub async fn edit(mut req: Request<State>) -> tide::Result {
    let state = &req.state();
    let db = &state.client.database("sybl");
    let mut user: User = req.body_json().await?;
    user.save(db.clone(), None).await?;
    println!("User: {:?}", &user);
    let user_id = user.id().unwrap();
    Ok(Response::builder(200)
        .body(json!(doc! {"user_id": user_id.to_string()}))
        .content_type(mime::JSON)
        .build())
}
