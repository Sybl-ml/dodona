//! Contains the builder functions used to generate message for DCL-DCN protcol

/// Different messages to be passed between DCL and DCN
#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    /// Hearbeat alive message
    Alive,
    /// Message to send Job Config
    JobConfig,
    /// Message to send datasets to DCN
    Data {
        /// Main dataset
        dataset: String,
        /// Prediction dataset
        predict: String,
    },
}

impl Message {
    /// Wrapper function to convert Message into other format
    pub fn send(msg: Message) -> String {
        let mut ret = String::from(serde_json::to_string(&msg).unwrap());
        ret.push_str("\0");
        ret
    }
}
