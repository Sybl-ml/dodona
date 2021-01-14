//! Defines the structure of projects in the `MongoDB` instance.

use chrono::Utc;
use mongodb::bson::{self, oid::ObjectId, Bson};
use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
/// Defines the status for a project
#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Unfinished,
    Ready,
    Processing,
    Complete,
    Read,
}

impl From<Status> for Bson {
    fn from(status: Status) -> Self {
        Self::from(match status {
            Status::Unfinished => "Unfinished",
            Status::Ready => "Ready",
            Status::Processing => "Processing",
            Status::Complete => "Complete",
            Status::Read => "Read",
        })
    }
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

impl Project {
    /// Creates a new instance of [`Project`].
    pub fn new<T: Into<String>>(name: T, description: T, user_id: ObjectId) -> Self {
        Self {
            id: None,
            name: name.into(),
            description: description.into(),
            date_created: bson::DateTime(Utc::now()),
            user_id: Some(user_id),
            status: Status::Unfinished,
        }
    }
}
