//! Defines the structure of predictions in the `MongoDB` instance.

use mongodb::bson::{self, oid::ObjectId, Binary};

/// Defines the information that should be stored with a dataset in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Prediction {
    /// The unique identifier for the prediction
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// Unique identifier for the associated project
    pub project_id: ObjectId,
    /// Dataset predicted by the model
    pub predictions: Binary,
}

impl Prediction {
    /// Creates a new [`Dataset`] for a project with some data.
    pub fn new(project_id: ObjectId, predictions: Vec<u8>) -> Self {
        Self {
            id: None,
            project_id,
            predictions: Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: predictions,
            },
        }
    }
}
