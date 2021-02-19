//! Contains the Messages for communication between different layers
//!
//! interface contains messages which are shared across the interface.
//! client contains messages for communication with clients (DCNs)

#![warn(missing_docs)]

#[macro_use]
extern crate serde;

pub mod client;
pub mod length_prefix;
pub mod raw_message;

pub use client::ClientMessage;
pub use length_prefix::{ReadLengthPrefix, WriteLengthPrefix};
pub use raw_message::RawMessage;
