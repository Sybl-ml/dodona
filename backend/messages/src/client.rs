//! Contains the builder functions used to generate message for DCL-DCN protcol

use chrono::{Duration, Utc};

use models::jobs::{JobConfiguration, PredictionType};

/// Different messages to be passed between DCL and DCN
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    /// Hearbeat alive message
    Alive {
        /// The current timestamp
        timestamp: u64,
    },
    /// Message to send Job Config
    JobConfig {
        /// The current time according to the server
        message_creation_timestamp: i64,
        /// The timestamp by which models must return predictions
        prediction_cutoff_timestamp: i64,
        /// Cluster Size
        cluster_size: i32,
        /// Types of each column in dataset for job
        column_types: Vec<String>,
        /// The name of the prediction column
        prediction_column: String,
        /// If the problem is Regression or Classification
        prediction_type: PredictionType,
    },
    /// A request to setup a new model
    NewModel {
        /// The client's email address
        email: String,
        /// The client's password
        password: String,
        /// The name of the model
        model_name: String,
    },
    /// A response to a challenge issued by the server
    ChallengeResponse {
        /// The user's email
        email: String,
        /// The model name corresponding to the challenge
        model_name: String,
        /// The calculated response value
        response: String,
    },
    /// A request to authenticate a model
    AccessToken {
        /// The model identifier
        id: String,
        /// The access token itself
        token: String,
    },
    /// A dataset for the node to process
    Dataset {
        /// The dataset to train on
        train: String,
        /// The dataset to predict on
        predict: String,
    },
    /// Response from client about job
    ConfigResponse {
        /// Field to say if job has been accepted or not
        accept: bool,
    },
    /// Prediction data from a node after computation
    Predictions(String),
}

impl From<&JobConfiguration> for ClientMessage {
    fn from(value: &JobConfiguration) -> Self {
        let JobConfiguration {
            node_computation_time,
            cluster_size,
            column_types,
            prediction_column,
            prediction_type,
            ..
        } = value;

        // Get the current time and cutoff time according to the config
        let current_time = Utc::now();
        let cutoff_time = current_time + Duration::seconds(i64::from(*node_computation_time) * 60);

        // Convert both to standard Unix timestamps in UTC
        let message_creation_timestamp = current_time.timestamp();
        let prediction_cutoff_timestamp = cutoff_time.timestamp();

        ClientMessage::JobConfig {
            message_creation_timestamp,
            prediction_cutoff_timestamp,
            cluster_size: *cluster_size,
            column_types: column_types.clone(),
            prediction_column: prediction_column.clone(),
            prediction_type: *prediction_type,
        }
    }
}
