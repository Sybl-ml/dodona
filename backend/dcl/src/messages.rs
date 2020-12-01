//! Contains the builder functions used to generate message for DCL-DCN protcol

/// Different messages to be passed between DCL and DCN
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Hearbeat alive message
    Alive { timestamp: u64 },
    /// Message to send Job Config
    JobConfig { config: String },
    /// Message to send datasets to DCN
    Data {
        /// Main dataset
        dataset: String,
        /// Prediction dataset
        predict: String,
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
}

impl Message {
    /// Wrapper function to convert Message into other format
    pub fn send(msg: Message) -> String {
        let mut ret = String::from(serde_json::to_string(&msg).unwrap());
        ret.push_str("\0");
        ret
    }

    /// Interprets a [`Message`] from a slice of bytes.
    pub fn from_slice(bytes: &[u8]) -> Self {
        serde_json::from_slice(&bytes).unwrap()
    }

    /// Converts a [`Message`] into a vector of bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}
