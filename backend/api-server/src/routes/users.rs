//! Defines the routes specific to user operations.

use actix_web::web;
use mongodb::bson::{doc, document::Document};
use tokio_stream::StreamExt;

use models::users::User;

use crate::{
    auth,
    error::{ServerError, ServerResponse},
    routes::{payloads, response_from_json},
    State,
};

/// Gets a user given their database identifier.
///
/// Given a user identifier, finds the user in the database and returns them as a JSON object. If
/// the user does not exist, the handler will panic.
pub async fn get(claims: auth::Claims, state: web::Data<State>) -> ServerResponse {
    let users = state.database.collection("users");

    let filter: Document = doc! { "_id": claims.id };
    let document = users.find_one(filter, None).await?;

    response_from_json(document)
}

/// Gets all users who match a filter.
///
/// Given a filter query, finds all users who match the filter and returns them as a JSON array of
/// objects. For example, given `{"first_name", "John"}`, finds all the users with the first name
/// John.
pub async fn filter(state: web::Data<State>, filter: web::Json<Document>) -> ServerResponse {
    let users = state.database.collection("users");

    println!("Filter: {:?}", &filter);

    let cursor = users.find(filter.into_inner(), None).await?;
    let documents: Vec<Document> = cursor.collect::<Result<_, _>>().await?;

    response_from_json(documents)
}

/// Creates a new user given the form information.
///
/// Given an email, password, first name and last name, peppers their password and hashes it. This
/// then gets stored in the Mongo database with a randomly generated user identifier. If the user's
/// email already exists, the route will not register any user.
/// The user's client status will be false which can be later changed
pub async fn new(
    state: web::Data<State>,
    payload: web::Json<payloads::RegistrationOptions>,
) -> ServerResponse {
    let pepper = state.pepper.clone();

    let users = state.database.collection("users");

    let email = crypto::clean(&payload.email);
    let first_name = crypto::clean(&payload.first_name);
    let last_name = crypto::clean(&payload.last_name);

    log::info!("Name: {} {}", first_name, last_name);

    let filter = doc! { "email": &email };

    log::info!("Checking if the user exists already");

    if users.find_one(filter, None).await?.is_some() {
        log::error!("Found a user with email '{}' already", &email);
        return response_from_json(doc! {"token": "null"});
    }

    log::info!("User does not exist, registering them now");

    let peppered = format!("{}{}", &payload.password, &pepper);
    let hash = pbkdf2::pbkdf2_simple(&peppered, state.pbkdf2_iterations)
        .expect("Failed to hash the user's password");

    log::info!("Hash: {:?}", hash);

    let user = User::new(email, hash, first_name, last_name);

    let document = mongodb::bson::ser::to_document(&user)?;
    let inserted_id = users.insert_one(document, None).await?.inserted_id;

    let identifier = inserted_id.as_object_id().ok_or(ServerError::Unknown)?;
    let jwt = auth::Claims::create_token(identifier.clone())?;

    response_from_json(doc! {"token": jwt})
}

/// Edits a user in the database and updates their information.
///
/// Given a user identifier, finds the user in the database and updates their information based on
/// the JSON provided, returning a message based on whether it was updated.
pub async fn edit(
    claims: auth::Claims,
    state: web::Data<State>,
    payload: web::Json<payloads::EditUserOptions>,
) -> ServerResponse {
    let users = state.database.collection("users");

    // Get the user from the database
    let filter = doc! { "_id": &claims.id };
    let user_doc = users
        .find_one(filter.clone(), None)
        .await?
        .ok_or(ServerError::NotFound)?;

    let mut user: User = mongodb::bson::de::from_document(user_doc)?;
    user.email = crypto::clean(&payload.email);

    let document = mongodb::bson::ser::to_document(&user)?;
    users.update_one(filter, document, None).await?;

    response_from_json(doc! {"status": "changed"})
}

/// Verifies a user's password against the one in the database.
///
/// Given an email and password, finds the user in the database and checks that the two hashes
/// match. If they don't, or the user does not exist, it will not authenticate them and send back a
/// null token.
pub async fn login(
    state: web::Data<State>,
    payload: web::Json<payloads::LoginOptions>,
) -> ServerResponse {
    let users = state.database.collection("users");
    let pepper = state.pepper.clone();

    let email = crypto::clean(&payload.email);

    let filter = doc! {"email": email};
    let user_doc = users
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;
    let user: User = mongodb::bson::de::from_document(user_doc)?;

    let peppered = format!("{}{}", payload.password, pepper);

    // Check the user's password
    pbkdf2::pbkdf2_check(&peppered, &user.hash)?;

    log::info!("Logged in: {:?}", user);

    let jwt = auth::Claims::create_token(user.id)?;
    response_from_json(doc! {"token": jwt})
}

/// Deletes a user from the database.
///
/// Given a user identifier, deletes the related user from the database if they exist.
pub async fn delete(claims: auth::Claims, state: web::Data<State>) -> ServerResponse {
    let users = state.database.collection("users");

    let filter = doc! { "_id": claims.id };
    let user = users.find_one(filter, None).await?;

    // If the project has data, delete the existing information
    if let Some(user) = user {
        let user: User = mongodb::bson::de::from_document(user)?;
        user.delete(&state.database).await?;
    }

    response_from_json(doc! {"status": "deleted"})
}
