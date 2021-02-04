//! Defines a message that can be sent without length prefixing, generally for raw JSON.

use serde::Serialize;

use crate::WriteLengthPrefix;

/// Defines a message that will not be length prefixed when being sent.
#[derive(Serialize, Debug)]
pub struct RawMessage {
    content: String,
}

impl RawMessage {
    /// Creates a new [`RawMessage`] that will only serialize the inner content.
    pub fn new(content: String) -> Self {
        Self { content }
    }
}

impl WriteLengthPrefix for RawMessage {
    fn as_bytes(&self) -> Vec<u8> {
        log::info!("Writing a message");

        // Convert the message to bytes
        let bytes = self.content.as_bytes().to_vec();

        // Prepend with the length
        let length = bytes.len() as u32;
        log::debug!("Sending message length prefix: {}", length);

        let mut message = length.to_be_bytes().to_vec();
        log::debug!("Message prefix: {:?}", message);
        message.extend(bytes);

        message
    }
}
