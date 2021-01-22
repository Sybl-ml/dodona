//! Defines the routes for the API server.

use actix_web::{HttpResponse, Result};
use mongodb::{
    bson::{de::from_document, doc, oid::ObjectId},
    Collection,
};

use crate::dodona_error::DodonaError;
use crypto::clean_json;
use models::projects::Project;

pub mod clients;
pub mod projects;
pub mod users;

/// Builds a [`Response`] with a 200 OK and JSON payload.
///
/// Defines a helper function that can be used to quickly build a JSON payload response to a user
/// request. As many API functions take and return JSON, this reduces repetition when the route has
/// been processed correctly.
///
pub fn response_from_json<B: serde::Serialize>(body: B) -> Result<HttpResponse, DodonaError> {
    let body = clean_json(json!(body));
    Ok(HttpResponse::Ok().json(body))
}

/// Checks whether a user exists with the given ID.
pub async fn check_user_exists(id: &str, users: &Collection) -> Result<ObjectId, DodonaError> {
    // Check the project ID to make sure it exists
    let object_id = ObjectId::with_string(&id)?;
    let query = doc! { "_id": &object_id };

    users
        .find_one(query, None)
        .await?
        .ok_or(DodonaError::NotFound)?;

    Ok(object_id)
}

/// Checks whether a project exists with the given ID.
pub async fn check_project_exists(
    id: &str,
    projects: &Collection,
) -> Result<ObjectId, DodonaError> {
    // Check the project ID to make sure it exists
    let object_id = ObjectId::with_string(&id)?;
    let query = doc! { "_id": &object_id};

    projects
        .find_one(query, None)
        .await?
        .ok_or(DodonaError::NotFound)?;

    Ok(object_id)
}

/// Checks whether a project exists with the given ID and that the given user owns it.
pub async fn check_user_owns_project(
    user_id: &ObjectId,
    project_id: &str,
    projects: &Collection,
) -> Result<ObjectId, DodonaError> {
    // Check the project ID to make sure it exists
    let object_id = ObjectId::with_string(&project_id)?;
    let query = doc! { "_id": &object_id};

    let project_doc = projects
        .find_one(query, None)
        .await?
        .ok_or(DodonaError::NotFound)?;

    let project: Project = from_document(project_doc)?;

    match project.user_id {
        Some(id) if id == *user_id => Ok(object_id),
        _ => Err(DodonaError::Unauthorized),
    }
}
