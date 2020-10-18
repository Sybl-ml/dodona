//! Defines the structure of projects in the MongoDB instance.

use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Defines the information that should be stored with a project in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    /// The unique identifier for the project
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// The name of the project
    pub name: String,
<<<<<<< HEAD
    /// The date and time that the project was created
=======
    pub description: String,
>>>>>>> adding suggestions
    pub date_created: bson::DateTime,
    /// The identifier of the user who created the project
    pub user_id: Option<ObjectId>,
}
