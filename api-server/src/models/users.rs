//! Defines the structure of users in the MongoDB instance.

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Defines the information that should be stored with a user in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    /// The unique identifier for the user
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// The user's email
    pub email: String,
    /// The peppered and hashed version of the user's password
    pub password: String,
    /// The user's first name
    pub first_name: String,
    /// The user's last name
    pub last_name: String,
    /// The user's API key
    pub api_key: String,
    /// The user's credits
    pub credits: i32,
}
