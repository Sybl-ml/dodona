//! Defines routes specific to client operations
use crate::dodona_error::DodonaError;
use crate::routes::{get_from_doc, response_from_json};
use crate::AppState;

use actix_web::{get, post, web, HttpResponse};
use chrono::Utc;
use models::models::{AccessToken, ClientModel, Status};
use models::users::{Client, User};
use mongodb::bson::de::from_document;
use mongodb::bson::ser::to_document;
use mongodb::bson::{self, doc, document::Document, oid::ObjectId, Binary};
use tokio::stream::StreamExt;

/// Template for registering a new client
///
/// Will check the provided user_id matches with the
/// provided email and password
#[post("/api/clients/register")]
pub async fn register(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let pepper = app_data.pepper.clone();
    let users = database.collection("users");
    let clients = database.collection("clients");

    let password = get_from_doc(&doc, "password")?;
    let email = crypto::clean(get_from_doc(&doc, "email")?);
    let id = get_from_doc(&doc, "id")?;

    let user_id = ObjectId::with_string(&id).map_err(|_| DodonaError::Invalid)?;

    let filter = doc! { "_id": &user_id };
    let user = users
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .map(|doc| from_document::<User>(doc).unwrap());

    let user = user.ok_or_else(|| DodonaError::NotFound)?;

    if user.client {
        return response_from_json(doc! {"privKey": "null"});
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
                doc! { "_id": &user_id },
                doc! {"$set": {"client": true}},
                None,
            )
            .await
            .map_err(|_| DodonaError::Unknown)?;

        // Update the user to be a client
        let client = Client::new(user_id, public_key);

        // store client object in db
        let document = to_document(&client).unwrap();
        clients
            .insert_one(document, None)
            .await
            .map_err(|_| DodonaError::Unknown)?;

        // reponse with private key
        response_from_json(doc! {"privKey": private_key})
    } else {
        Err(DodonaError::Forbidden)
    }
}

/// Route for registering a new model/node
///
/// provided an email check the user exists and is a client
/// If validated generate a challenge and insert a new temp model
/// Respond with the encoded challenge
#[post("/api/clients/m/new")]
pub async fn new_model(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let users = database.collection("users");
    let models = database.collection("models");

    let email = crypto::clean(doc.get_str("email").unwrap());
    let model_name = get_from_doc(&doc, "model_name")?.to_string();

    let filter = doc! { "email": &email };
    let user = match users
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
    {
        Some(u) => from_document::<User>(u).unwrap(),
        None => return Err(DodonaError::NotFound),
    };
    let user_id = user.id.expect("ID is none");

    if !user.client {
        return Err(DodonaError::Forbidden);
    }

    let filter = doc! { "user_id": &user_id, "name": &model_name };
    if models
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .is_some()
    {
        return Err(DodonaError::Conflict);
    }

    // Generate challenge
    let challenge = crypto::generate_challenge();
    // Make new model
    let temp_model = ClientModel {
        id: Some(ObjectId::new()),
        user_id: user_id.clone(),
        name: model_name,
        status: Some(Status::NotStarted),
        access_token: None,
        locked: false,
        authenticated: false,
        challenge: Some(Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: challenge.clone(),
        }),
        times_run: 0,
    };

    // insert model into database
    let document = to_document(&temp_model).unwrap();
    models
        .insert_one(document, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    // return challenge
    response_from_json(doc! {
        "Challenge": {
            "challenge": base64::encode(challenge),
        }
    })
}

/// Verifies a challenge response from a model
///
/// Given a `new_model`, a `challenge_response` and a `challenge`, verifies that the
/// `challenge_response` matches the `challenge` with respect to the `client`'s public key.
/// Returns a new access token for the `new_model` if verification is successful.
/// Returns a 404 error if the `client` or `model` is not found, or 401 if verification fails.
#[post("/api/clients/m/verify")]
pub async fn verify_challenge(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let users = database.collection("users");
    let clients = database.collection("clients");
    let models = database.collection("models");

    let model_name = get_from_doc(&doc, "model_name")?.to_string();
    let email = crypto::clean(doc.get_str("email").unwrap());
    let filter = doc! { "email": &email };
    let user = users
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .map(|doc| from_document::<User>(doc).unwrap());

    let user = user.ok_or_else(|| DodonaError::NotFound)?;
    let user_id = user.id.expect("User ID is none");
    // get clients public key matching with that users id
    let filter = doc! { "user_id": &user_id };
    let client = clients
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .map(|doc| from_document::<Client>(doc).unwrap());
    let client = client.ok_or_else(|| DodonaError::NotFound)?;

    let filter = doc! { "user_id": &user_id, "name": &model_name };
    let model = models
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .map(|doc| from_document::<ClientModel>(doc).unwrap());
    let mut model = model.ok_or_else(|| DodonaError::NotFound)?;

    let public_key = client.public_key;

    let challenge = &model
        .challenge
        .ok_or_else(|| DodonaError::Unauthorized)?
        .bytes;

    // needs converting to Vec<u8>
    let challenge_response = base64::decode(get_from_doc(&doc, "challenge_response")?).unwrap();

    if !crypto::verify_challenge(challenge.to_vec(), challenge_response, public_key) {
        return Err(DodonaError::Unauthorized);
    }

    let access_token = AccessToken::new();
    model.authenticated = true;
    model.access_token = Some(access_token.clone());
    model.challenge = None;

    let filter = doc! { "user_id": &user_id, "name": &model_name };
    let update = doc! { "$set": to_document(&model).unwrap() };
    models
        .find_one_and_update(filter, update, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    // return the access token to the model
    response_from_json(doc! {
        "AccessToken": {
            "id": model.id.expect("Model ID is none").to_string(),
            "token": base64::encode(access_token.clone().token.bytes),
            "expires": access_token.expires.to_rfc3339()
        }
    })
}

/// Unlocks a model using MFA
///
/// When MFA is used such that a client approves a model for use on their dashboard,
/// then given a model `id`, unlocks the model for authentication and use by the DCL.
/// TODO: implement safeguards, such as a OTP request parameter, to prevent clients
/// (or mailicious actors) contacting this endpoint from outside of the dashboard.
#[post("/api/clients/m/unlock")]
pub async fn unlock_model(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let models = database.collection("models");

    let model_id =
        ObjectId::with_string(&get_from_doc(&doc, "id")?).map_err(|_| DodonaError::Unauthorized)?;
    let filter = doc! { "_id": &model_id };
    let model = models
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .map(|doc| from_document::<ClientModel>(doc).unwrap());
    let mut model = model.ok_or_else(|| DodonaError::Unauthorized)?;

    //TODO: implement additional safeguards to prevent arbitrary access and unlocking

    model.locked = false;

    let filter = doc! { "_id": &model_id };
    let update = doc! { "$set": to_document(&model).unwrap() };
    models
        .find_one_and_update(filter, update, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    Ok(HttpResponse::Ok().body("Model successfully unlocked"))
}

/// Authenticates a model using an access token
///
/// Given a model `id` and an access token `token` and a `challenge`, verifies that the
/// model is not locked, has been authenticated and has a valid access token. If the token
/// has expired, the model should be asked to reauthenticate using a challenge response.
/// Returns 200 if authentication is successful and a new challenge if the token has expired.
/// Returns a 401 error if the model is not found or if authentication fails.
#[post("/api/clients/m/authenticate")]
pub async fn authenticate_model(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let models = database.collection("models");

    let model_id =
        ObjectId::with_string(&get_from_doc(&doc, "id")?).map_err(|_| DodonaError::Unauthorized)?;
    let filter = doc! { "_id": &model_id };
    let model = models
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .map(|doc| from_document::<ClientModel>(doc).unwrap());
    let mut model = model.ok_or_else(|| DodonaError::Unauthorized)?;

    let token = base64::decode(get_from_doc(&doc, "token")?).unwrap();

    if !model.is_authenticated(&token) {
        return Err(DodonaError::Unauthorized);
    }

    if model.access_token.clone().unwrap().expires < Utc::now() {
        let challenge = crypto::generate_challenge();
        model.authenticated = false;
        model.challenge = Some(Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: challenge.clone(),
        });

        let filter = doc! { "_id": &model_id };
        let update = doc! { "$set": to_document(&model).unwrap() };
        models
            .find_one_and_update(filter, update, None)
            .await
            .map_err(|_| DodonaError::Unknown)?;

        response_from_json(doc! {"challenge": base64::encode(challenge)})
    } else {
        // TODO: authenticate the model in the session
        response_from_json(doc! {"message": "Authentication successful"})
    }
}

/// Finds all the models related to a given user.
///
/// Given a user identifier, finds all the models in the database that the user owns. If the user
/// doesn't exist or an invalid identifier is given, returns a 404 response.
#[get("/api/clients/u/{:user_id}")]
pub async fn get_user_models(
    app_data: web::Data<AppState>,
    user_id: web::Path<String>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let models = database.collection("models");
    let users = database.collection("users");

    let object_id = match ObjectId::with_string(&user_id) {
        Ok(id) => id,
        Err(_) => return Err(DodonaError::NotFound),
    };

    let found_user = users
        .find_one(doc! { "_id": &object_id}, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    if found_user.is_none() {
        return Err(DodonaError::NotFound);
    }
    let filter = doc! { "user_id": &object_id };
    let cursor = models
        .find(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    response_from_json(documents.unwrap())
}
