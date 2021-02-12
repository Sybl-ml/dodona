//! Contains the Messages for communication between different layers
//!
//! interface contains messages which are shared across the interface.
//! client contains messages for communication with clients (DCNs)

#![warn(missing_docs)]

#[macro_use]
extern crate serde;

pub mod client;
pub mod interface;
pub mod length_prefix;
pub mod raw_message;

pub use client::ClientMessage;
pub use interface::InterfaceMessage;
pub use length_prefix::{ReadLengthPrefix, WriteLengthPrefix};
pub use raw_message::RawMessage;

/// Different types of problem Sybl can accept
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PredictionType {
    /// Predicting a class of data
    Classification,
    /// Predicting a numerical value for data
    Regression,
}
