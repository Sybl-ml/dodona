//! Defines the structure of projects in the `MongoDB` instance.

use chrono::Utc;
use mongodb::bson::{self, doc, oid::ObjectId, Array, Bson};
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;

use crate::datasets::Dataset;
use crate::predictions::Prediction;

#[allow(missing_docs)]
/// Defines the status for a project
#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Unfinished,
    Ready,
    Processing { model_success: i32, model_err: i32 },
    Complete,
    Read,
}

impl From<Status> for Bson {
    fn from(status: Status) -> Self {
        bson::to_bson(&status).expect("Failed to convert the status to BSON")
    }
}

/// Defines the information that should be stored with a project in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    /// The unique identifier for the project
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// The name of the project
    pub name: String,
    /// The description of the project set by user
    pub description: String,
    /// The tags of the project set by user
    pub tags: Array,
    /// The date and time that the project was created
    pub date_created: bson::DateTime,
    /// The identifier of the user who created the project
    pub user_id: ObjectId,
    /// The status of the project
    pub status: Status,
}

impl Project {
    /// Creates a new instance of [`Project`].
    pub fn new<T: Into<String>>(name: T, description: T, tags: Array, user_id: ObjectId) -> Self {
        let name = name.into();
        let description = description.into();

        log::debug!(
            "Creating a new project for user_id={} with name='{}' and description='{}'",
            user_id,
            name,
            description
        );

        Self {
            id: ObjectId::new(),
            name,
            description,
            tags,
            date_created: bson::DateTime(Utc::now()),
            user_id,
            status: Status::Unfinished,
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let projects = database.collection("projects");
        let datasets = database.collection("datasets");
        let predictions = database.collection("predictions");

        log::debug!("Deleting project with id={}", self.id);

        let filter = doc! { "_id": &self.id };
        // Remove project from database
        projects.delete_one(filter, None).await?;

        let dataset_filter = doc! { "project_id": &self.id};
        let dataset = datasets.find_one(dataset_filter, None).await?;

        if let Some(dataset) = dataset {
            let dataset: Dataset = mongodb::bson::de::from_document(dataset).unwrap();
            dataset.delete(&database).await?;
        }

        let predictions_filter = doc! { "project_id": &self.id};
        let mut cursor = predictions.find(predictions_filter, None).await?;

        while let Some(Ok(prediction_doc)) = cursor.next().await {
            let prediction: Prediction = mongodb::bson::de::from_document(prediction_doc).unwrap();
            prediction.delete(database).await?;
        }

        Ok(())
    }
}
