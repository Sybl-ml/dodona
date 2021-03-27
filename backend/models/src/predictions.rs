//! Defines the structure of predictions in the `MongoDB` instance.

use mongodb::bson::{doc, oid::ObjectId};

/// Defines the information that should be stored with a dataset in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Prediction {
    /// The unique identifier for the prediction
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// Unique identifier for the associated project
    pub project_id: ObjectId,
    /// Dataset predicted by the model
    pub predictions: ObjectId,
}

impl Prediction {
    /// Creates a new [`Dataset`] for a project with some data.
    pub fn new(project_id: ObjectId, predictions: ObjectId) -> Self {
        Self {
            id: ObjectId::new(),
            project_id,
            predictions,
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let predictions = database.collection("predictions");

        let filter = doc! {"_id": &self.id};
        predictions.delete_one(filter, None).await?;

        Ok(())
    }
}
