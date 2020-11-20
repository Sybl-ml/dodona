//! Defines routes specific to client operations

use ammonia::clean_text;
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use tide::{Request, Response};

use crate::routes::response_from_json;
use crypto::encoded_key_pair;
use models::clients::Client;
use models::users::User;

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
    let clients = database.collection("clients");

    let password = doc.get_str("password").unwrap();
    let email = clean_text(doc.get_str("email").unwrap());
    let id = doc.get_str("id").unwrap();
    let object_id = ObjectId::with_string(&id).unwrap();

    let filter = doc! { "_id": &object_id };
    let user = users
        .find_one(filter, None)
        .await?
        .map(|doc| mongodb::bson::de::from_document::<User>(doc).unwrap());

    if let Some(user) = user {
        let peppered = format!("{}{}", password, pepper);
        let verified = pbkdf2::pbkdf2_check(&peppered, &user.password).is_ok();

        // Entered and stored email and password match
        if verified && email == user.email {
            // generate public and private key pair
            let (public_key, private_key) = encoded_key_pair();
            // create a new client object
            users
                .update_one(
                    doc! { "_id": &object_id },
                    doc! {"$set": {"client": true}},
                    None,
                )
                .await?;

            // update user as client
            let client = Client {
                id: Some(ObjectId::new()),
                user_id: Some(object_id),
                public_key,
            };
            // store client object in db
            let document = mongodb::bson::ser::to_document(&client).unwrap();
            clients.insert_one(document, None).await?;

            // reponse with private key
            Ok(response_from_json(doc! {"privKey": private_key}))
        } else {
            Ok(Response::builder(403)
                .body("email or password incorrect")
                .build())
        }
    } else {
        println!("User ID does not exist");
        Ok(response_from_json(doc! {"token": "null"}))
    }
}
