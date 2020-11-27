//! Defines routes specific to client operations
use async_std::stream::StreamExt;
use mongodb::bson::{self, doc, document::Document, oid::ObjectId, Binary};
use tide::{Request, Response};

use crate::routes::{get_from_doc, response_from_json, tide_err};
use crate::State;
use chrono::Utc;
use models::models::{AccessToken, ClientModel};
use models::users::{Client, User};
use mongodb::bson::de::from_document;
use mongodb::bson::ser::to_document;

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

    let password = get_from_doc(&doc, "password")?;
    let email = crypto::clean(get_from_doc(&doc, "email")?);
    let id = get_from_doc(&doc, "id")?;

    let object_id = ObjectId::with_string(&id).map_err(|_| tide_err(422, "invalid object id"))?;

    let filter = doc! { "_id": &object_id };
    let user = users
        .find_one(filter, None)
        .await?
        .map(|doc| from_document::<User>(doc).unwrap());

    let user = user.ok_or_else(|| tide_err(404, "failed to find user"))?;

    if user.client {
        return Ok(response_from_json(doc! {"privKey": "null"}));
    }

    let peppered = format!("{}{}", password, pepper);
    let verified = pbkdf2::pbkdf2_check(&peppered, &user.hash).is_ok();

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
            user_id: object_id,
            public_key,
        };
        // store client object in db
        let document = to_document(&client).unwrap();
        clients.insert_one(document, None).await?;

        // reponse with private key
        Ok(response_from_json(doc! {"privKey": private_key}))
    } else {
        Ok(Response::builder(403)
            .body("email or password incorrect")
            .build())
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

    let email = crypto::clean(doc.get_str("email").unwrap());
    let model_name = get_from_doc(&doc, "model_name")?.to_string();

    let filter = doc! { "email": &email };
    let user = match users.find_one(filter, None).await? {
        Some(u) => from_document::<User>(u).unwrap(),
        None => return Ok(Response::builder(404).body("User not found").build()),
    };
    let user_id = user.id.expect("ID is none");

    if !user.client {
        return Ok(Response::builder(403)
            .body("User not a client found")
            .build());
    }

    let filter = doc! { "user_id": &user_id, "name": &model_name };
    if models.find_one(filter, None).await?.is_some() {
        return Err(tide_err(409, "model with duplicate name"));
    }

    // Generate challenge
    let challenge = crypto::generate_challenge();
    // Make new model
    let temp_model = ClientModel {
        id: Some(ObjectId::new()),
        user_id: user_id.clone(),
        name: model_name,
        status: None,
        access_token: None,
        locked: true,
        authenticated: false,
        challenge: Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: challenge.clone(),
        },
    };

    // insert model into database
    let document = to_document(&temp_model).unwrap();
    models.insert_one(document, None).await?;

    // return challenge
    Ok(response_from_json(
        doc! {"challenge": base64::encode(challenge)},
    ))
}

/// Verifies a challenge response from a model
///
/// Given a `new_model`, a `challenge_response` and a `challenge`, verifies that the
/// `challenge_response` matches the `challenge` with respect to the `client`'s public key.
/// Returns a new access token for the `new_model` if verification is successful.
/// Returns a 404 error if the `client` or `model` is not found, or 401 if verification fails.
pub async fn verify_challenge(mut req: Request<State>) -> tide::Result {
    let doc: Document = req.body_json().await?;
    let database = req.state().client.database("sybl");
    let users = database.collection("users");
    let clients = database.collection("clients");
    let models = database.collection("models");

    let model_name = get_from_doc(&doc, "model_name")?.to_string();
    let email = crypto::clean(doc.get_str("email").unwrap());
    let filter = doc! { "email": &email };
    let user = users
        .find_one(filter, None)
        .await?
        .map(|doc| from_document::<User>(doc).unwrap());

    let user = user.ok_or_else(|| tide_err(404, "failed to find user"))?;
    let user_id = user.id.expect("User ID is none");
    // get clients public key matching with that users id
    let filter = doc! { "user_id": &user_id };
    let client = clients
        .find_one(filter, None)
        .await?
        .map(|doc| from_document::<Client>(doc).unwrap());
    let client = client.ok_or_else(|| tide_err(404, "failed to find client"))?;

    let filter = doc! { "user_id": &user_id, "name": &model_name };
    let model = models
        .find_one(filter, None)
        .await?
        .map(|doc| from_document::<ClientModel>(doc).unwrap());
    let mut model = model.ok_or_else(|| tide_err(404, "failed to find model"))?;

    let public_key = client.public_key;

    let challenge = &model.challenge.bytes;

    // needs converting to Vec<u8>
    let challenge_response = base64::decode(get_from_doc(&doc, "challenge_response")?).unwrap();

    if !crypto::verify_challenge(challenge.to_vec(), challenge_response, public_key) {
        return Err(tide_err(
            401,
            "Invalid signature, please use OpenSSL to sign the provided challenge \
            with your private key and the SHA256 message digest function",
        ));
    }

    let access_token = AccessToken::new();
    model.authenticated = true;
    model.access_token = Some(access_token.clone());

    let filter = doc! { "user_id": &user_id, "name": &model_name };
    let update = to_document(&model).unwrap();
    models.find_one_and_update(filter, update, None).await?;

    // return the access token to the model
    Ok(response_from_json(doc! {
        "id": model.id.expect("Model ID is none"),
        "token": base64::encode(access_token.clone().token),
        "expires": access_token.expires.to_rfc3339(),
    }))
}

/// Authenticates a model using an access token
///
/// Given a model `id` and an access token `token` and a `challenge`, verifies that the
/// model is not locked, has been authenticated and has a valid access token. If the token
/// has expired, the model should be asked to reauthenticate using a challenge response.
/// Returns 200 if authentication is successful and a new challenge if the token has expired.
/// Returns a 401 error if the model is not found or if authentication fails.
pub async fn authenticate_model(mut req: Request<State>) -> tide::Result {
    let database = req.state().client.database("sybl");
    let doc: Document = req.body_json().await?;
    let models = database.collection("models");
    let msg = "Model not found, not authenticated or locked";

    let model_id = ObjectId::with_string(&get_from_doc(&doc, "id")?).map_err(|_| tide_err(401, &msg))?;
    let filter = doc! { "_id": &model_id };
    let model = models
        .find_one(filter, None)
        .await?
        .map(|doc| from_document::<ClientModel>(doc).unwrap());
    let mut model = model.ok_or_else(|| tide_err(401, &msg))?;

    let token = base64::decode(get_from_doc(&doc, "token")?).unwrap();
    if !model.authenticated
        || model.locked
        || model.access_token.is_none()
        || model.access_token.clone().unwrap().token != token
    {
        return Err(tide_err(401, &msg));
    }

    if model.access_token.clone().unwrap().expires < Utc::now() {
        let challenge = crypto::generate_challenge();
        model.authenticated = false;
        model.challenge = Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: challenge.clone(),
        };

        let filter = doc! { "_id": &model_id };
        let update = to_document(&model).unwrap();
        models.find_one_and_update(filter, update, None).await?;

        Ok(response_from_json(
            doc! {"challenge": base64::encode(challenge)},
        ))
    } else {
        // TODO: authenticate the model in the session
        Ok(Response::builder(200)
            .body("Authentication successful")
            .build())
    }
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
