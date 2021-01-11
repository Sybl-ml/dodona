//! Defines the routes for the API server.

use crypto::clean_json;
use mongodb::{
    bson::{doc, document::Document, oid::ObjectId},
    Collection,
};

use crate::dodona_error::DodonaError;
use actix_web::{HttpResponse, Result};

pub mod clients;
pub mod projects;
pub mod users;

/// Builds a [`Response`] with a 200 OK and JSON payload.
///
/// Defines a helper function that can be used to quickly build a JSON payload response to a user
/// request. As many API functions take and return JSON, this reduces repetition when the route has
/// been processed correctly.
///
/// # Examples
///
/// ```
/// use serde::Serialize;
/// use api_server::routes::response_from_json;
///
/// #[derive(Serialize)]
/// struct Payload {
///     name: String,
///     age: u32,
/// }
///
/// let payload = Payload { name: String::from("Freddie"), age: 22 };
/// let mut response = response_from_json(payload);
///
/// assert_eq!(response.status(), tide::StatusCode::Ok);
/// assert_eq!(response.content_type(), Some(tide::http::mime::JSON));
///
/// let body = async_std::task::block_on(response.take_body().into_string());
/// let expected = String::from(r#"{"name":"Freddie","age":22}"#);
///
/// assert!(body.is_ok());
/// assert_eq!(body.unwrap(), expected);
/// ```
pub fn response_from_json<B: serde::Serialize>(body: B) -> Result<HttpResponse, DodonaError> {
    let body = clean_json(json!(body));
    Ok(HttpResponse::Ok().json(body))
}

/// Checks whether a user exists with the given ID.
pub async fn check_user_exists(id: &str, users: &Collection) -> Result<ObjectId, DodonaError> {
    // Check the project ID to make sure it exists
    let object_id = ObjectId::with_string(&id).map_err(|_| DodonaError::Invalid)?;
    let query = doc! { "_id": &object_id };

    users
        .find_one(query, None)
        .await
        .unwrap()
        .ok_or_else(|| DodonaError::NotFound)?;

    Ok(object_id)
}

/// Checks whether a project exists with the given ID.
pub async fn check_project_exists(
    id: &str,
    projects: &Collection,
) -> Result<ObjectId, DodonaError> {
    // Check the project ID to make sure it exists
    let object_id = ObjectId::with_string(&id).map_err(|_| DodonaError::Invalid)?;
    let query = doc! { "_id": &object_id};

    projects
        .find_one(query, None)
        .await
        .unwrap()
        .ok_or_else(|| DodonaError::NotFound)?;

    Ok(object_id)
}

/// Gets a key from a document, or returns a 422 error if it doesn't exist.
pub fn get_from_doc<'a>(document: &'a Document, key: &'a str) -> Result<&'a str, DodonaError> {
    document.get_str(key).map_err(|_| DodonaError::Invalid)
}
