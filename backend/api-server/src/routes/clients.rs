//! Defines routes specific to client operations

use crate::State;
use ammonia::clean_text;
use async_std::stream::StreamExt;
use base64;
use mongodb::bson;
use mongodb::bson::{doc, document::Document, oid::ObjectId, Binary};
use tide::{Request, Response};

use crate::routes::response_from_json;
use crypto;
use models::models::ClientModel;
use models::users::{Client, User};

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
            let (private_key, public_key) = crypto::encoded_key_pair();
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
        Ok(Response::builder(404).body("User not found").build())
    }
}

/// Route for registering a new model/node
///
/// provided an email check the user exists and is a client
/// If validated generate a challenge and insert a new temp model
/// Respond with the encoded challenge
pub async fn new_model(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;
    let state = req.state();
    let database = state.client.database("sybl");
    let users = database.collection("users");
    let models = database.collection("models");

    let email = clean_text(doc.get_str("email").unwrap());

    let filter = doc! { "email": &email };
    let user = match users.find_one(filter, None).await? {
        Some(u) => mongodb::bson::de::from_document::<User>(u).unwrap(),
        None => return Ok(Response::builder(404).body("User not found").build()),
    };
    let user_id = user.id.expect("ID is none");

    if !user.client {
        return Ok(Response::builder(403)
            .body("User not a client found")
            .build());
    }

    // Generate challenge
    let challenge = crypto::generate_challenge();
    // Make new model
    let temp_model = ClientModel {
        id: Some(ObjectId::new()),
        user_id: user_id.clone(),
        name: None,
        status: None,
        locked: true,
        authenticated: false,
        challenge: Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: challenge.clone(),
        },
    };

    // insert model into database
    let document = mongodb::bson::ser::to_document(&temp_model).unwrap();
    models.insert_one(document, None).await?;

    // return challenge
    Ok(response_from_json(
        doc! {"challenge": base64::encode(challenge)},
    ))
}

/// Finds all the models related to a given user.
///
/// Given a user identifier, finds all the models in the database that the user owns. If the user
/// doesn't exist or an invalid identifier is given, returns a 404 response.
pub async fn get_user_models(req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let models = database.collection("models");
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
    let cursor = models.find(filter, None).await?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    Ok(response_from_json(documents.unwrap()))
}
