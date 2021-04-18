//! Defines the structure of jobs in the `MongoDB` instance.

use chrono::Utc;
use mongodb::bson::{self, oid::ObjectId};

use utils::Columns;

/// Different types of problem Sybl can accept
#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
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
    /// The identifier of the project to be processed
    pub project_id: ObjectId,
    /// The amount of time each node is allowed to compute for
    pub node_computation_time: i32,
    /// The cluster size for a job
    pub cluster_size: i32,
    /// The types of each column in the dataset
    pub column_types: Vec<String>,
    /// The number of features in the dataset
    pub feature_dim: i8,
    /// The number of rows in train set (nearest 100)
    pub train_size: i32,
    /// The number of rows in predict set (nearest 100)
    pub predict_size: i32,
    /// The column to predict during evaluation
    pub prediction_column: String,
    /// The type of problem we are being asked to solve
    pub prediction_type: PredictionType,
    /// The total amount paid to run this job
    pub cost: i32,
}

impl JobConfiguration {
    /// Produces a new [`JobConfiguration`] with the prediction column anonymised.
    ///
    /// When sending configurations to clients, we need to avoid leaking information about the
    /// original dataset where possible. The data they receive will have an anonymised prediction
    /// column regardless, so we must also do it for the configuration itself.
    pub fn anonymise(&self, columns: &Columns) -> Self {
        let prediction_column = columns
            .get(&self.prediction_column)
            .unwrap()
            .pseudonym
            .clone();

        Self {
            prediction_column,
            ..self.clone()
        }
    }
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
        log::debug!("Creating a new job from config={:?}", config);

        Self {
            id: ObjectId::new(),
            config,
            processed: false,
            date_created: bson::DateTime(Utc::now()),
        }
    }
}
