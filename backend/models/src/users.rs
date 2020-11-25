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
    pub hash: String,
    /// The user's first name
    pub first_name: String,
    /// The user's last name
    pub last_name: String,
    /// The user's API key
    pub api_key: String,
    // The user's client status
    pub client: bool,
    /// The user's credits
    pub credits: i32,
}

impl User {
    /// Creates a new instance of [`User`].
    pub fn new<T: Into<String>>(email: T, hash: T, first_name: T, last_name: T) -> Self {
        Self {
            id: None,
            email: email.into(),
            hash: hash.into(),
            first_name: first_name.into(),
            last_name: last_name.into(),
            api_key: crypto::generate_user_api_key(),
            client: false,
            credits: 10,
        }
    }
}

/// Defines the information that should be stored with a client in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Client {
    /// The unique identifier for the client
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// The identifier of the user to which this client information belongs
    pub user_id: Option<ObjectId>,
    /// This clients public Key
    pub public_key: String,
}
