//! Defines the structure of predictions in the `MongoDB` instance.

use mongodb::bson::{de::from_document, doc, oid::ObjectId};

use crate::gridfs;

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
        log::debug!(
            "Creating a new prediction mapping between project_id={} and predictions={}",
            project_id,
            predictions
        );

        Self {
            id: ObjectId::new(),
            project_id,
            predictions,
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let predictions = database.collection("predictions");
        let files = database.collection("files");

        log::debug!("Deleting predictions for project_id={}", self.project_id);

        let filter = doc! {"_id": &self.predictions};
        if let Some(file) = files.find_one(filter, None).await? {
            let file: gridfs::File = from_document(file)?;
            file.delete(database).await?;
        }

        let filter = doc! {"_id": &self.id};
        predictions.delete_one(filter, None).await?;

        Ok(())
    }
}
