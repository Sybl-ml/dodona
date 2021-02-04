//! Contains the builder functions used to generate messages for Interface - DCL Communication

use mongodb::bson::oid::ObjectId;

/// Different messages to be passed between Interface and DCL
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InterfaceMessage {
    /// Configuration message for Job
    Config {
        /// ID of the dataset associated with Job
        id: ObjectId,
        /// Job timeout
        timeout: i32,
        /// The columns in the dataset
        column_types: Vec<String>,
    },
}
