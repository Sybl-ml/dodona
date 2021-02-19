//! Defines the dataset analysis for a given dataset and project in the `MongoDB` instance.

use std::collections::HashMap;

use mongodb::bson::{doc, oid::ObjectId};
/// Defines the information that should be stored as analayis for a dataset
#[derive(Debug, Serialize, Deserialize)]
pub struct DatasetAnalysis {
    /// The unique identifier for the dataset
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// Unique identifier for the associated project
    pub project_id: ObjectId,
    // Column based analysis
    pub columns: HashMap<String, utils::analysis::ColumnAnalysis>,
}

impl DatasetAnalysis {
    /// Creates a new instance of [`DatasetDetails`].
    pub fn new(project_id: ObjectId, dataset_analysis: utils::analysis::DatasetAnalysis) -> Self {
        Self {
            id: ObjectId::new(),
            project_id,
            columns: dataset_analysis.columns,
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let dataset_analysis = database.collection("dataset_analysis");

        let filter = doc! {"_id": &self.id};
        dataset_analysis.delete_one(filter, None).await?;

        Ok(())
    }
}
