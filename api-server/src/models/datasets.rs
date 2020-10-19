//! Defines the structure of projects in the MongoDB instance.
use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Binary;
use serde::{Deserialize, Serialize};

/// Defines the information that should be stored with a dataset in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    /// The unique identifier for the dataset
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// Unique identifier for the associated project
    pub project_id: Option<ObjectId>,
    /// The date that the dataset was uploaded
    pub date_created: bson::DateTime,
    /// Dataset binary stored in the db
    pub dataset: Option<Binary>,
}