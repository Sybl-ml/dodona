//! Defines the structure of datasets in the MongoDB instance.

use mongodb::bson::{self, oid::ObjectId, Binary};

/// Defines the information that should be stored with a dataset in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    /// The unique identifier for the dataset
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// Unique identifier for the associated project
    pub project_id: Option<ObjectId>,
    /// Dataset binary stored in the database
    pub dataset: Option<Binary>,
    /// Dataset to be predicted by the model
    pub predict: Option<Binary>,
}

impl Dataset {
    /// Creates a new [`Dataset`] for a project with some data.
    pub fn new(project_id: ObjectId, dataset: Vec<u8>, predict: Vec<u8>) -> Self {
        Self {
            id: None,
            project_id: Some(project_id),
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
}
