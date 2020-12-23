//! Defines the dataset details for a given dataset and project in the MongoDB instance.

use std::collections::HashMap;

use chrono::Utc;
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
    /// The types of each column
    pub column_types: HashMap<String, utils::ColumnType>,
}

impl DatasetDetails {
    /// Creates a new instance of [`DatasetDetails`].
    pub fn new(
        project_id: ObjectId,
        head: String,
        column_types: HashMap<String, utils::ColumnType>,
    ) -> Self {
        Self {
            id: None,
            project_id: Some(project_id),
            date_created: bson::DateTime(Utc::now()),
            head: Some(head),
            column_types,
        }
    }
}
