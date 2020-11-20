//! Defines the structure of clients in the MongoDB instance.

use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

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
