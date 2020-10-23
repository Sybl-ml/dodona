//! Defines the dataset details for a given dataset and project in the MongoDB instance.

use mongodb::bson;
use mongodb::bson::oid::ObjectId;

/// Defines the information that should be stored as details for a project
#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetDetails {
    /// The unique identifier for the dataset
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// Unique identifier for the associated project
    pub project_id: Option<ObjectId>,
    /// The date that the dataset was uploaded
    pub date_created: bson::DateTime,
    /// Head of the dataset
    pub head: Option<String>,
}