//! Contains the builder functions used to generate message for DCL-DCN protcol

use anyhow::{Error, Result};
use chrono::{Duration, Utc};
use std::time::Instant;
use tokio::net::TcpStream;

use crate::ReadLengthPrefix;
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

impl ClientMessage {
    /// Compresses the data and uses Base64 encoding to form a [`ClientMessage`].
    pub fn from_train_and_predict(train: &str, predict: &str) -> Self {
        // Compress the data
        let training_bytes = utils::compress::compress_bytes(train.as_bytes())
            .expect("Failed to compress the training data");
        let prediction_bytes = utils::compress::compress_bytes(predict.as_bytes())
            .expect("Failed to compress the prediction data");

        // Perform Base64 encoding
        let encoded_training = base64::encode(&training_bytes);
        let encoded_prediction = base64::encode(&prediction_bytes);

        Self::Dataset {
            train: encoded_training,
            predict: encoded_prediction,
        }
    }

    /// Reads from the socket until given predicate is true or until
    /// the timeout has been reached. This will return the client message
    /// if the predicate is passed, or it will propate an error back up.
    pub async fn read_until(
        stream: &mut TcpStream,
        buffer: &mut [u8],
        predicate: fn(&ClientMessage) -> bool,
    ) -> Result<Self> {
        let wait = std::time::Duration::from_millis(2000);
        let now = Instant::now();

        while wait >= now.elapsed() {
            let config_response: ClientMessage =
                ClientMessage::from_stream(&mut *stream, buffer).await?;
            if predicate(&config_response) {
                return Ok(config_response);
            }
        }
        Err(Error::msg("Predicate not satisfied within timeout"))
    }
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
