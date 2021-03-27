//! Defines the routes specific to user operations.

use actix_web::web;
use mongodb::bson::{
    de::from_document, doc, document::Document, ser::to_document, spec::BinarySubtype, Binary,
};
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
pub async fn filter(
    state: web::Data<State>,
    payload: web::Json<payloads::FilterUsersOptions>,
) -> ServerResponse {
    let users = state.database.collection("users");

    let cursor = users.find(payload.filter.clone(), None).await?;
    let documents: Vec<Document> = cursor.collect::<Result<_, _>>().await?;

    response_from_json(documents)
}

/// Creates a new user given the form information.
///
/// Given an email, password, first name and last name, peppers their password and hashes it. This
/// then gets stored in the Mongo database with a randomly generated user identifier. If the user's
/// email already exists, the route will not register any user.
pub async fn new(
    state: web::Data<State>,
    payload: web::Json<payloads::RegistrationOptions>,
) -> ServerResponse {
    let users = state.database.collection("users");

    let email = crypto::clean(&payload.email);
    let first_name = crypto::clean(&payload.first_name);
    let last_name = crypto::clean(&payload.last_name);

    let filter = doc! { "email": &email };

    if users.find_one(filter, None).await?.is_some() {
        log::error!("Found a user with email={} already", &email);
        return response_from_json(doc! {"token": "null"});
    }

    let peppered = format!("{}{}", &payload.password, &state.pepper);
    let hash = crypto::hash_password(&peppered, state.pbkdf2_iterations)
        .expect("Failed to hash the user's password");

    let user = User::new(email, hash.to_string(), first_name, last_name);

    log::debug!("Registering a new user: {:#?}", user);

    let document = to_document(&user)?;
    let inserted_id = users.insert_one(document, None).await?.inserted_id;
    let identifier = inserted_id.as_object_id().ok_or(ServerError::Unknown)?;

    log::debug!("Created the user with id={}", identifier);

    let jwt = auth::Claims::create_token(identifier.clone())?;

    response_from_json(doc! {"token": jwt})
}

/// Uploads a profile picture to a users account
///
/// Takes a Base64 string, compresses it and then converts to a vector of u8
pub async fn new_avatar(
    claims: auth::Claims,
    state: web::Data<State>,
    payload: web::Json<payloads::AvatarOptions>,
) -> ServerResponse {
    let users = state.database.collection("users");

    let bytes = base64::decode(&payload.avatar)?;
    log::debug!("Recieved {} bytes for avatar image", bytes.len());

    let binary = Binary {
        subtype: BinarySubtype::Generic,
        bytes,
    };

    let filter = doc! { "_id": &claims.id };
    let update = doc! { "$set": { "avatar": binary } };

    users.update_one(filter, update, None).await?;

    response_from_json(doc! {"status": "changed"})
}

/// Gets the users avatar
///
/// Retrieves the avatar binary data from the database and decompresses it
/// After converting to Base64 string it is returned to the user
pub async fn get_avatar(claims: auth::Claims, state: web::Data<State>) -> ServerResponse {
    let users = state.database.collection("users");

    let filter: Document = doc! { "_id": claims.id };
    let document = users
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;

    let user: User = from_document(document)?;

    let encoded_image = user
        .avatar
        .map(|b| base64::encode(b.bytes))
        .unwrap_or_default();

    response_from_json(doc! {"img": encoded_image})
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

    let filter = doc! { "_id": &claims.id };
    let update = doc! { "$set": { "email": crypto::clean(&payload.email) } };

    users.update_one(filter, update, None).await?;

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

    let email = crypto::clean(&payload.email);
    let filter = doc! {"email": email};

    let document = users
        .find_one(filter, None)
        .await?
        .ok_or(ServerError::NotFound)?;
    let user: User = from_document(document)?;

    // Check the user's password
    let peppered = format!("{}{}", &payload.password, &state.pepper);
    crypto::verify_password(&peppered, &user.hash)?;

    log::debug!("User logged in with id={}, email={}", user.id, user.email);

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

    // Delete the user and their owned items if they exist
    if let Some(user) = user {
        let user: User = from_document(user)?;
        user.delete(&state.database).await?;
    }

    response_from_json(doc! {"status": "deleted"})
}
