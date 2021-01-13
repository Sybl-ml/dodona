//! Contains the builder functions used to generate message for DCL-DCN protcol

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
        /// Types of each column in dataset for job
        column_types: Vec<String>,
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
    /// Prediction data from a node after computation
    Predictions(String),
    /// A raw JSON message, usually from the API server
    RawJSON {
        /// The raw JSON contents
        content: String,
    },
}
