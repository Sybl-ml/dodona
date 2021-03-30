//! Defines the structure of datasets in the `MongoDB` instance.

use mongodb::bson::{doc, oid::ObjectId};

use crate::dataset_analysis::DatasetAnalysis;
use crate::dataset_details::DatasetDetails;

/// Defines the information that should be stored with a dataset in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    /// The unique identifier for the dataset
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// Unique identifier for the associated project
    pub project_id: ObjectId,
    /// Dataset File ObjectId stored in the database
    pub dataset: Option<ObjectId>,
    /// Dataset File ObjectId for prediction stored in the database
    pub predict: Option<ObjectId>,
}

impl Dataset {
    /// Creates a new [`Dataset`] for a project with some data.
    pub fn new(project_id: ObjectId, dataset: Option<ObjectId>, predict: Option<ObjectId>) -> Self {
        Self {
            id: ObjectId::new(),
            project_id,
            dataset: dataset,
            predict: predict,
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let datasets = database.collection("datasets");
        let dataset_details = database.collection("dataset_details");
        let dataset_analysis = database.collection("dataset_analysis");
        let files = database.collection("files");

        let filter = doc! { "_id": &self.id };
        // Remove project from database
        datasets.delete_one(filter, None).await?;

        let dataset_det_filter = doc! { "project_id": &self.project_id};

        let dataset_details = dataset_details
            .find_one(dataset_det_filter.clone(), None)
            .await?;
        if let Some(dataset_details) = dataset_details {
            let dataset_details: DatasetDetails =
                mongodb::bson::de::from_document(dataset_details).unwrap();
            dataset_details.delete(&database).await?;
        }

        let dataset_analysis = dataset_analysis.find_one(dataset_det_filter, None).await?;
        if let Some(dataset_analysis) = dataset_analysis {
            let dataset_analysis: DatasetAnalysis =
                mongodb::bson::de::from_document(dataset_analysis).unwrap();
            dataset_analysis.delete(&database).await?;
        }

        Ok(())
    }
}
