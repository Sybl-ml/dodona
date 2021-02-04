//! Defines the structure of datasets in the `MongoDB` instance.

use mongodb::bson::{self, doc, oid::ObjectId, Binary};

use crate::dataset_details::DatasetDetails;

/// Defines the information that should be stored with a dataset in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    /// The unique identifier for the dataset
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// Unique identifier for the associated project
    pub project_id: ObjectId,
    /// Dataset binary stored in the database
    pub dataset: Option<Binary>,
    /// Dataset to be predicted by the model
    pub predict: Option<Binary>,
}

impl Dataset {
    /// Creates a new [`Dataset`] for a project with some data.
    pub fn new(project_id: ObjectId, dataset: Vec<u8>, predict: Vec<u8>) -> Self {
        Self {
            id: ObjectId::new(),
            project_id: project_id,
            dataset: Some(Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: dataset,
            }),
            predict: Some(Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: predict,
            }),
        }
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let datasets = database.collection("datasets");
        let dataset_details = database.collection("dataset_details");

        let filter = doc! { "_id": &self.id };
        // Remove project from database
        datasets.delete_one(filter, None).await?;

        let dataset_det_filter = doc! { "dataset_id": &self.id};
        let dataset_details = dataset_details.find_one(dataset_det_filter, None).await?;

        if let Some(dataset_details) = dataset_details {
            let dataset_details: DatasetDetails =
                mongodb::bson::de::from_document(dataset_details).unwrap();
            dataset_details.delete(&database).await?;
        }

        Ok(())
    }
}
