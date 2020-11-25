//! Defines routes specific to client operations

use mongodb::bson::{doc, document::Document, oid::ObjectId};
use tide::Request;

use crypto::clean;
use models::users::User;

use crate::routes::{get_from_doc, response_from_json, tide_err};
use crate::State;

/// Template for registering a new client
///
/// Will check the provided user_id matches with the
/// provided email and password
pub async fn register(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;
    let state = req.state();
    let database = state.client.database("sybl");
    let pepper = &state.pepper;

    let users = database.collection("users");

    let password = get_from_doc(&doc, "password")?;
    let email = clean(get_from_doc(&doc, "email")?);
    let id = get_from_doc(&doc, "id")?;

    let object_id = ObjectId::with_string(&id).map_err(|_| tide_err(422, "invalid object id"))?;

    let filter = doc! { "_id": object_id };
    let user = users
        .find_one(filter, None)
        .await?
        .map(|doc| mongodb::bson::de::from_document::<User>(doc).unwrap());

    if let Some(user) = user {
        let peppered = format!("{}{}", password, pepper);
        let verified = pbkdf2::pbkdf2_check(&peppered, &user.hash).is_ok();

        if verified && email == user.email {
            println!("Logged in: {:?}", user);
            Ok(response_from_json(doc! {"privKey": 1}))
        } else {
            println!("Failed login: wrong password");
            Ok(response_from_json(doc! {"token": "null"}))
        }
    } else {
        println!("Failed login: wrong email");
        Ok(response_from_json(doc! {"token": "null"}))
    }
}
