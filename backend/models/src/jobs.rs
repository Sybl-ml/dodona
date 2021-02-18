//! Defines the structure of jobs in the `MongoDB` instance.

use chrono::Utc;
use mongodb::bson::{self, oid::ObjectId};

/// Different types of problem Sybl can accept
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PredictionType {
    /// Predicting a class of data
    Classification,
    /// Predicting a numerical value for data
    Regression,
}

/// Parameters required for configuring a job.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobConfiguration {
    /// The identifier of the dataset to be processed
    pub dataset_id: ObjectId,
    /// The timeout required for clients to process within
    pub timeout: usize,
    /// The cluster size for a job
    pub cluster_size: usize,
    /// The types of each column in the dataset
    pub column_types: Vec<String>,
    /// The column to predict during evaluation
    pub prediction_column: String,
    /// The type of problem we are being asked to solve
    pub prediction_type: PredictionType,
}

/// Defines the information that should be stored with a job in the database.
#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
    /// The unique identifier for the job
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// The message/configuration associated with the job
    pub config: JobConfiguration,
    /// Whether the job has been processed by the interface or not
    pub processed: bool,
    /// The timestamp at which the [`Job`] was created
    pub date_created: bson::DateTime,
}

impl Job {
    /// Creates a new [`Job`] with a given [`JobConfiguration`].
    pub fn new(config: JobConfiguration) -> Self {
        Self {
            id: ObjectId::new(),
            config,
            processed: false,
            date_created: bson::DateTime(Utc::now()),
        }
    }
}
