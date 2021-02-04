//! Defines the dataset details for a given dataset and project in the `MongoDB` instance.

use chrono::Utc;
use mongodb::bson::{self, doc, oid::ObjectId};
use utils::Columns;

/// Defines the information that should be stored as details for a project
#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetDetails {
    /// The unique identifier for the dataset
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// Name of Dataset
    pub dataset_name: Option<String>,
    /// Unique identifier for the associated project
    pub project_id: ObjectId,
    /// The date that the dataset was uploaded
    pub date_created: bson::DateTime,
    /// Head of the dataset
    pub head: Option<String>,
    /// The types of each column
    pub column_types: Columns,
}

impl DatasetDetails {
    /// Creates a new instance of [`DatasetDetails`].
    pub fn new(
        dataset_name: String,
        project_id: ObjectId,
        head: String,
        column_types: Columns,
    ) -> Self {
        Self {
            id: ObjectId::new(),
            dataset_name: Some(dataset_name),
            project_id,
            date_created: bson::DateTime(Utc::now()),
            head: Some(head),
            column_types,
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let dataset_details = database.collection("dataset_details");

        let filter = doc! {"_id": &self.id};
        dataset_details.delete_one(filter, None).await?;

        Ok(())
    }
}
