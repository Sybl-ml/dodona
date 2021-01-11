//! Defines the routes specific to user operations.

use actix_web::{web, HttpResponse};
use mongodb::bson::{doc, document::Document, oid::ObjectId};
use tokio::stream::StreamExt;

use crate::dodona_error::DodonaError;
use crate::routes::{check_user_exists, get_from_doc, response_from_json};
use crate::AppState;
use crypto::clean;
use models::users::User;

/// Gets a user given their database identifier.
///
/// Given a user identifier, finds the user in the database and returns them as a JSON object. If
/// the user does not exist, the handler will panic.
pub async fn get(
    app_data: web::Data<AppState>,
    user_id: web::Path<String>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let users = database.collection("users");

    let object_id = check_user_exists(&user_id, &users).await?;

    let filter: Document = doc! { "_id": object_id };
    let document = users
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    response_from_json(document)
}

/// Gets all users who match a filter.
///
/// Given a filter query, finds all users who match the filter and returns them as a JSON array of
/// objects. For example, given `{"first_name", "John"}`, finds all the users with the first name
/// John.
pub async fn filter(
    app_data: web::Data<AppState>,
    filter: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let users = database.collection("users");

    println!("Filter: {:?}", &filter);

    let cursor = users
        .find(filter.into_inner(), None)
        .await
        .map_err(|_| DodonaError::Unknown)?;
    let documents: Result<Vec<Document>, mongodb::error::Error> = cursor.collect().await;

    response_from_json(documents.unwrap())
}

/// Creates a new user given the form information.
///
/// Given an email, password, first name and last name, peppers their password and hashes it. This
/// then gets stored in the Mongo database with a randomly generated user identifier. If the user's
/// email already exists, the route will not register any user.
/// The user's client status will be false which can be later changed
pub async fn new(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    log::debug!("Document received: {:?}", &doc);

    let pepper = app_data.pepper.clone();

    let database = app_data.client.database("sybl");
    let users = database.collection("users");

    let password = get_from_doc(&doc, "password")?;
    let email = clean(get_from_doc(&doc, "email")?);
    let first_name = clean(get_from_doc(&doc, "firstName")?);
    let last_name = clean(get_from_doc(&doc, "lastName")?);

    log::info!("Email: {}, Password: {}", email, password);
    log::info!("Name: {} {}", first_name, last_name);

    let filter = doc! { "email": &email };

    log::info!("Checking if the user exists already");

    if users
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)
        .is_ok()
    {
        log::error!("Found a user with email '{}' already", &email);
        return response_from_json(doc! {"token": "null"});
    }

    log::info!("User does not exist, registering them now");

    let peppered = format!("{}{}", &password, &pepper);
    let hash = pbkdf2::pbkdf2_simple(&peppered, app_data.pbkdf2_iterations)
        .map_err(|_| DodonaError::Unknown)?;

    log::info!("Hash: {:?}", hash);

    let user = User::new(email, hash, first_name, last_name);

    let document = mongodb::bson::ser::to_document(&user).unwrap();
    let id = users
        .insert_one(document, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .inserted_id;

    response_from_json(doc! {"token": id.as_object_id().unwrap().to_string()})
}

/// Edits a user in the database and updates their information.
///
/// Given a user identifier, finds the user in the database and updates their information based on
/// the JSON provided, returning a message based on whether it was updated.
pub async fn edit(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let users = database.collection("users");

    let user_id = clean(get_from_doc(&doc, "id")?);
    let object_id = check_user_exists(&user_id, &users).await?;

    // Get the user from the database
    let filter = doc! { "_id": &object_id };
    let user_doc = users
        .find_one(filter.clone(), None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    let mut user: User =
        mongodb::bson::de::from_document(user_doc.unwrap()).map_err(|_| DodonaError::Invalid)?;

    if let Ok(email) = doc.get_str("email") {
        user.email = clean(email);
    }

    let document = mongodb::bson::ser::to_document(&user).unwrap();
    users
        .update_one(filter, document, None)
        .await
        .map_err(|_| DodonaError::Unknown)?;

    response_from_json(doc! {"status": "changed"})
}

/// Verifies a user's password against the one in the database.
///
/// Given an email and password, finds the user in the database and checks that the two hashes
/// match. If they don't, or the user does not exist, it will not authenticate them and send back a
/// null token.
pub async fn login(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let pepper = app_data.pepper.clone();

    let users = database.collection("users");

    let password = get_from_doc(&doc, "password")?;
    let email = clean(get_from_doc(&doc, "email")?);

    println!("{}, {}", &email, &password);

    let filter = doc! {"email": email};
    let user = users
        .find_one(filter, None)
        .await
        .map_err(|_| DodonaError::Unknown)?
        .map(|doc| mongodb::bson::de::from_document::<User>(doc).unwrap());

    let user = user.ok_or_else(|| DodonaError::NotFound)?;
    let peppered = format!("{}{}", password, pepper);

    // Check the user's password
    pbkdf2::pbkdf2_check(&peppered, &user.hash).map_err(|_| DodonaError::Unauthorized)?;

    log::info!("Logged in: {:?}", user);

    let identifier = user.id.unwrap().to_string();
    response_from_json(doc! {"token": identifier})
}

/// Deletes a user from the database.
///
/// Given a user identifier, deletes the related user from the database if they exist.
pub async fn delete(
    app_data: web::Data<AppState>,
    doc: web::Json<Document>,
) -> Result<HttpResponse, DodonaError> {
    let database = app_data.client.database("sybl");
    let users = database.collection("users");

    let object_id = clean(get_from_doc(&doc, "id")?);
    let id = ObjectId::with_string(&object_id).map_err(|_| DodonaError::Invalid)?;
    let filter = doc! {"_id": id};

    users.find_one_and_delete(filter, None).await.unwrap();

    response_from_json(doc! {"status": "deleted"})
}
