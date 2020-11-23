//! Defines the routes for the API server.

use std::convert::TryFrom;

use crypto::clean_json;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use tide::{http::mime, Response};

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
pub fn response_from_json<B: serde::Serialize>(body: B) -> Response {
    let body = clean_json(json!(body));
    Response::builder(200)
        .body(body)
        .content_type(mime::JSON)
        .build()
}

/// Builds a [`tide::Error`] from a code and message.
pub fn tide_err(code: u16, msg: &'static str) -> tide::Error {
    let status = tide::StatusCode::try_from(code).unwrap();
    tide::Error::from_str(status, msg)
}

/// Checks whether a user exists with the given ID.
pub async fn check_user_exists(id: &str, users: &Collection) -> Result<ObjectId, tide::Error> {
    // Check the project ID to make sure it exists
    let object_id = ObjectId::with_string(&id).map_err(|_| tide_err(422, "invalid user id"))?;
    let query = doc! { "_id": &object_id };

    users
        .find_one(query, None)
        .await?
        .ok_or_else(|| tide_err(404, "user not found"))?;

    Ok(object_id)
}

/// Checks whether a project exists with the given ID.
pub async fn check_project_exists(
    id: &str,
    projects: &Collection,
) -> Result<ObjectId, tide::Error> {
    // Check the project ID to make sure it exists
    let object_id = ObjectId::with_string(&id).map_err(|_| tide_err(422, "invalid project id"))?;
    let query = doc! { "_id": &object_id};

    projects
        .find_one(query, None)
        .await?
        .ok_or_else(|| tide_err(404, "project not found"))?;

    Ok(object_id)
}
