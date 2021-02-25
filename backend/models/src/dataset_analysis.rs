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
    pub columns: HashMap<String, ColumnAnalysis>,
}

impl DatasetAnalysis {
    /// Creates a new instance of [`DatasetDetails`].
    pub fn new(project_id: ObjectId, columns: HashMap<String, ColumnAnalysis>) -> Self {
        Self {
            id: ObjectId::new(),
            project_id,
            columns
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let dataset_analysis = database.collection("dataset_analysis");

        let filter = doc! {"_id": &self.id};
        dataset_analysis.delete_one(filter, None).await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ColumnAnalysis {
    Categorical(CategoricalAnalysis),
    Numerical(NumericalAnalysis),
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct CategoricalAnalysis {
    /// All the values in the column
    pub values: HashMap<String, i64>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NumericalAnalysis {
    pub max: f64,
    pub min: f64,
    pub sum: f64,
    pub avg: f64,
}

impl Default for NumericalAnalysis {
    fn default() -> Self {
        Self {
            max: f64::MIN,
            min: f64::MAX,
            sum: 0.0,
            avg: 0.0,
        }
    }
}
