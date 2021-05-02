//! Contains a trait to allow any object to become length prefixed bytes.

use anyhow::Result;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::AsyncReadExt;

impl<T: DeserializeOwned> ReadLengthPrefix for T {}
impl<T: Serialize> WriteLengthPrefix for T {}

/// Allows any object that is [`DeserializeOwned`] to be deserialized from length prefixed form.
#[async_trait]
pub trait ReadLengthPrefix: DeserializeOwned {
    /// Reads a [`Message`] from a raw stream of bytes, dealing with length prefixing.
    async fn from_stream<R: AsyncReadExt + Send + Unpin>(
        stream: &mut R,
        mut buffer: &mut [u8],
    ) -> Result<Self> {
        // Read the size of the message
        let mut size_buffer = [0_u8; 4];
        stream.read_exact(&mut size_buffer).await?;

        // Convert it to a u32
        let message_size = u32::from_be_bytes(size_buffer);
        log::trace!("Received a message length prefix of size={}", message_size);

        // Read again from the stream, extending the vector if needed
        let mut bytes = Vec::new();
        let mut remaining_size = message_size;

        // Enforce only reading the given size
        let mut truncated = stream.take(u64::from(remaining_size));

        while remaining_size != 0 {
            let size = truncated.read(&mut buffer).await?;
            bytes.extend_from_slice(&buffer[..size]);
            remaining_size -= size as u32;
        }

        Ok(serde_json::from_slice(&bytes)?)
    }
}

/// Allows any object that is [`Serialize`] to be serialized in length prefixed form.
pub trait WriteLengthPrefix: Serialize {
    /// Converts a [`Message`] into a vector of bytes.
    fn as_bytes(&self) -> Vec<u8> {
        // Convert the message to bytes
        let bytes = serde_json::to_vec(&self).unwrap();

        // Prepend with the length
        let length = bytes.len() as u32;
        log::trace!("Creating a message with length={}", length);

        let mut message = length.to_be_bytes().to_vec();
        message.extend(bytes);

        message
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    // Create a basic type for testing
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Basic {
        id: u32,
        value: u64,
    }

    // Create a larger type for testing
    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct Large {
        content: String,
    }

    #[test]
    fn length_prefixed_messages_can_be_created() {
        let message = Basic { id: 10, value: 24 };
        let prefixed = message.as_bytes();

        // Includes the length prefix itself
        let expected = b"\x00\x00\x00\x14{\"id\":10,\"value\":24}";

        assert_eq!(prefixed, expected);
    }

    #[test]
    fn larger_length_prefixed_messages_can_be_created() {
        let message = Large {
            content: String::from("some longer string of characters"),
        };

        let prefixed = message.as_bytes();

        // Includes the length prefix itself
        let expected = b"\x00\x00\x00\x2e{\"content\":\"some longer string of characters\"}";

        assert_eq!(prefixed, expected);
    }

    #[tokio::test]
    async fn length_prefixed_messages_can_be_read() -> Result<()> {
        // Includes the length prefix itself
        let mut cursor = Cursor::new(b"\x00\x00\x00\x14{\"id\":10,\"value\":24}");
        let mut buffer = [0_u8; 64];
        let message = Basic::from_stream(&mut cursor, &mut buffer).await?;

        let expected = Basic { id: 10, value: 24 };

        assert_eq!(message, expected);

        Ok(())
    }

    #[tokio::test]
    async fn length_prefixed_messages_can_be_read_with_smaller_buffers() -> Result<()> {
        // Includes the length prefix itself
        let mut cursor = Cursor::new(b"\x00\x00\x00\x14{\"id\":10,\"value\":24}");

        // Use a buffer smaller than the message size
        let mut buffer = [0_u8; 5];
        let message = Basic::from_stream(&mut cursor, &mut buffer).await?;

        let expected = Basic { id: 10, value: 24 };

        assert_eq!(message, expected);

        Ok(())
    }

    #[tokio::test]
    async fn larger_length_prefixed_messages_can_be_read_with_smaller_buffers() -> Result<()> {
        // Includes the length prefix itself
        let mut cursor =
            Cursor::new(b"\x00\x00\x00\x2e{\"content\":\"some longer string of characters\"}");

        // Use a buffer smaller than the message size
        let mut buffer = [0_u8; 7];
        let message = Large::from_stream(&mut cursor, &mut buffer).await?;

        let expected = Large {
            content: String::from("some longer string of characters"),
        };

        assert_eq!(message, expected);

        Ok(())
    }
}
