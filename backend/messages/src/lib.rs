//! Contains the Messages for communication between different layers
//!
//! interface contains messages which are shared across the interface.
//! client contains messages for communication with clients (DCNs)

#![warn(missing_docs)]

#[macro_use]
extern crate serde;

pub mod client;
pub mod interface;
