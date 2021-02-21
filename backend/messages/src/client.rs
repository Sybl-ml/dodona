//! Contains the builder functions used to generate message for DCL-DCN protcol

use models::jobs::PredictionType;
use utils::Columns;

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
        /// Job timeout
        timeout: i32,
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
    /// Anonymises a ClientMessage
    ///
    /// Returns an anonymised clone of a ClientMessage such that it can be
    /// sent to a model without the risk of information disclosure
    /// Currently, this is only necessary for JobConfig messages
    pub fn anonymise(&self, columns: &Columns) -> ClientMessage {
        match self {
            ClientMessage::JobConfig {
                timeout,
                cluster_size,
                column_types,
                prediction_column,
                prediction_type,
            } => ClientMessage::JobConfig {
                timeout: timeout.clone(),
                cluster_size: cluster_size.clone(),
                column_types: column_types.clone(),
                prediction_column: columns.get(prediction_column).unwrap().pseudonym.clone(),
                prediction_type: prediction_type.clone(),
            },
            _ => self.clone(),
        }
    }
}
