//! Defines the structure of jobs in the MongoDB instance.

use messages::interface::InterfaceMessage;
use mongodb::bson::oid::ObjectId;

/// Defines the information that should be stored with a job in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    /// The unique identifier for the job
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// The dataset identifier the job refers to
    pub msg: InterfaceMessage,
}

impl Job {
    /// Creates a new [`Job`] with a given InterfaceMessage.
    pub fn new(msg: InterfaceMessage) -> Self {
        Self {
            id: ObjectId::new(),
            msg,
        }
    }
}
