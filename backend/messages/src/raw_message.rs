//! Defines a message that can be sent without the struct name, generally for raw JSON.

/// Defines a message that will not serialize the struct name into the result.
#[derive(Debug)]
pub struct RawMessage {
    content: String,
}

impl RawMessage {
    /// Creates a new [`RawMessage`] that will only serialize the inner content.
    pub fn new(content: String) -> Self {
        Self { content }
    }

    /// Converts the inner content to bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        // Convert the message to bytes
        let bytes = self.content.as_bytes().to_vec();

        // Prepend with the length
        let length = bytes.len() as u32;
        log::trace!("Creating bytes for a message with length={}", length);

        let mut message = length.to_be_bytes().to_vec();
        message.extend(bytes);

        message
    }
}
