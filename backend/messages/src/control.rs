//! Contains the messages to be sent between DCL nodes.

/// Messages relating to the distribution of the control layer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ControlMessage {
    /// Request for a port to connect to.
    PortRequest,
    /// Request to be a child node in the control layer.
    ChildNodeRequest {
        /// The port the node is listening for Mallus connections on.
        port: u16,
    },
    /// Response for a port to connect to.
    PortResponse {
        /// The port to connect to.
        port: Option<u16>,
    },
    /// General heartbeating message
    Alive {
        /// The current timestamp
        timestamp: u64,
    },
}
