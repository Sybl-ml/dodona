//! Defines the structure of projects in the MongoDB instance.

use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Unfinished,
    Ready,
    Processing,
    Complete,
    Read,
}

/// Defines the information that should be stored with a project in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    /// The unique identifier for the project
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// The name of the project
    pub name: String,
    /// The description of the project set by user
    pub description: String,
    /// The date and time that the project was created
    pub date_created: bson::DateTime,
    /// The identifier of the user who created the project
    pub user_id: Option<ObjectId>,
    /// The status of the project
    pub status: Status,
}
