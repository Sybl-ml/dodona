//! Defines a message that can be sent without length prefixing, generally for raw JSON.

use serde::Serialize;

use crate::WriteLengthPrefix;

/// Defines a message that will not be length prefixed when being sent.
#[derive(Serialize)]
pub struct RawMessage<T: Serialize> {
    content: T,
}

impl<T: Serialize> RawMessage<T> {
    /// Creates a new [`RawMessage`] that will only serialize the inner content.
    pub fn new(content: T) -> Self {
        Self { content }
    }
}

impl<T: Serialize> WriteLengthPrefix for RawMessage<T> {
    fn as_bytes(&self) -> Vec<u8> {
        // Convert the inner content to bytes
        let bytes = serde_json::to_vec(&self.content).unwrap();

        // Prepend with the length
        let length = bytes.len() as u32;

        let mut message = length.to_be_bytes().to_vec();
        message.extend(bytes);

        log::debug!(
            "Message created with prefix: {}, total length: {}",
            length,
            message.len()
        );

        message
    }
}
