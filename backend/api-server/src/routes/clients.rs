//! Defines routes specific to client operations

use actix_web::{web, HttpResponse};
use chrono::Utc;
use models::models::{AccessToken, ClientModel, Status};
use models::users::{Client, User};
use mongodb::bson::de::from_document;
use mongodb::bson::ser::to_document;
use mongodb::bson::{self, doc, document::Document, oid::ObjectId, Binary};
use tokio_stream::StreamExt;

use crate::auth;
use crate::dodona_error::DodonaError;
use crate::routes::response_from_json;
use crate::AppState;

/// Template for registering a new client
///
/// Will check the provided `user_id` matches with the provided email and password
pub async fn register(
    claims: auth::User,
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let pepper = app_data.pepper.clone();
    let users = database.collection("users");
    let clients = database.collection("clients");

    let password = doc.get_str("password")?;
    let email = crypto::clean(doc.get_str("email")?);

    let filter = doc! { "_id": &claims.id };
    let user_doc = users.find_one(filter, None).await?;

    let user: User = from_document(user_doc.ok_or(DodonaError::NotFound)?)?;

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
                doc! { "_id": &claims.id },
                doc! {"$set": {"client": true}},
                None,
            )
            .await?;

        // Update the user to be a client
        let client = Client::new(claims.id, public_key);

        // store client object in db
        let document = to_document(&client)?;
        clients.insert_one(document, None).await?;

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
pub async fn new_model(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let users = database.collection("users");
    let models = database.collection("models");

    let email = crypto::clean(doc.get_str("email")?);
    let model_name = doc.get_str("model_name")?.to_string();

    let filter = doc! { "email": &email };
    let user = match users.find_one(filter, None).await? {
        Some(u) => from_document::<User>(u)?,
        None => return Err(DodonaError::NotFound),
    };
    let user_id = user.id.expect("ID is none");

    if !user.client {
        return Err(DodonaError::Forbidden);
    }

    let filter = doc! { "user_id": &user_id, "name": &model_name };
    if models.find_one(filter, None).await?.is_some() {
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
        locked: true,
        authenticated: false,
        challenge: Some(Binary {
            subtype: bson::spec::BinarySubtype::Generic,
            bytes: challenge.clone(),
        }),
        times_run: 0,
    };

    // insert model into database
    let document = to_document(&temp_model)?;
    models.insert_one(document, None).await?;

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
pub async fn verify_challenge(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let users = database.collection("users");
    let clients = database.collection("clients");
    let models = database.collection("models");

    let model_name = doc.get_str("model_name")?.to_string();
    let email = crypto::clean(doc.get_str("email")?);
    let filter = doc! { "email": &email };

    let user_doc = users
        .find_one(filter, None)
        .await?
        .ok_or(DodonaError::NotFound)?;
    let user: User = from_document(user_doc)?;
    let user_id = user.id.expect("User ID is none");

    // get clients public key matching with that users id
    let filter = doc! { "user_id": &user_id };
    let client_doc = clients
        .find_one(filter, None)
        .await?
        .ok_or(DodonaError::NotFound)?;
    let client: Client = from_document(client_doc)?;

    let filter = doc! { "user_id": &user_id, "name": &model_name };
    let model_doc = models
        .find_one(filter, None)
        .await?
        .ok_or(DodonaError::NotFound)?;
    let mut model: ClientModel = from_document(model_doc)?;

    let public_key = client.public_key;
    let challenge = &model.challenge.ok_or(DodonaError::Unauthorized)?.bytes;

    // needs converting to Vec<u8>
    let challenge_response = base64::decode(doc.get_str("challenge_response")?)?;

    if !crypto::verify_challenge(challenge.to_vec(), challenge_response, public_key) {
        return Err(DodonaError::Unauthorized);
    }

    let access_token = AccessToken::new();
    model.authenticated = true;
    model.access_token = Some(access_token.clone());
    model.challenge = None;

    let filter = doc! { "user_id": &user_id, "name": &model_name };
    let update = doc! { "$set": to_document(&model)? };
    models.find_one_and_update(filter, update, None).await?;

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
///
/// To unlock a model, the frontend must query this endpoint with a valid model id `id`
/// and the password `password` of the user to whom the model is registered.
/// The model must be authenticated using `verify_challenge` before being unlocked
pub async fn unlock_model(
    claims: auth::User,
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let models = database.collection("models");
    let users = database.collection("users");

    let model_id = ObjectId::with_string(doc.get_str("id")?)?;
    let filter = doc! { "_id": &model_id };
    let model_doc = models
        .find_one(filter, None)
        .await?
        .ok_or(DodonaError::Unauthorized)?;
    let mut model: ClientModel = from_document(model_doc)?;

    // Check the current user owns this model
    if model.user_id != claims.id {
        return Err(DodonaError::Unauthorized);
    }

    let password = doc.get_str("password")?;
    let pepper = &app_data.pepper;

    let filter = doc! { "_id": &claims.id };
    let user_doc = users
        .find_one(filter, None)
        .await?
        .ok_or(DodonaError::Unauthorized)?;
    let user: User = from_document(user_doc)?;

    let peppered = format!("{}{}", password, pepper);

    if !model.authenticated || pbkdf2::pbkdf2_check(&peppered, &user.hash).is_err() {
        return Err(DodonaError::Unauthorized);
    }

    model.locked = false;

    let filter = doc! { "_id": &model_id };
    let update = doc! { "$set": to_document(&model)? };
    models.find_one_and_update(filter, update, None).await?;

    Ok(HttpResponse::Ok().body("Model successfully unlocked"))
}

/// Authenticates a model using an access token
///
/// Given a model `id` and an access token `token` and a `challenge`, verifies that the
/// model is not locked, has been authenticated and has a valid access token. If the token
/// has expired, the model should be asked to reauthenticate using a challenge response.
/// Returns 200 if authentication is successful and a new challenge if the token has expired.
/// Returns a 401 error if the model is not found or if authentication fails.
pub async fn authenticate_model(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let models = database.collection("models");

    let model_id = ObjectId::with_string(&doc.get_str("id")?)?;
    let filter = doc! { "_id": &model_id };
    let model_doc = models
        .find_one(filter, None)
        .await?
        .ok_or(DodonaError::Unauthorized)?;
    let mut model: ClientModel = from_document(model_doc)?;

    let token = base64::decode(doc.get_str("token")?)?;

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
        let update = doc! { "$set": to_document(&model)? };
        models.find_one_and_update(filter, update, None).await?;

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
pub async fn get_user_models(
    claims: auth::User,
    app_data: web::Data<AppState>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let models = database.collection("models");

    let filter = doc! { "user_id": &claims.id };
    let cursor = models.find(filter, None).await?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    response_from_json(documents?)
}
