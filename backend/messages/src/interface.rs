//! Contains the builder functions used to generate message for DCL-DCN protcol

use std::convert::TryInto;

use anyhow::Result;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

/// Different messages to be passed between DCL and DCN
#[derive(Debug, Serialize, Deserialize)]
pub enum InterfaceMessage {
    Config,
    RawJSON {
        /// The raw JSON contents
        content: String,
    },
}

impl InterfaceMessage {
    /// Reads a [`Message`] from a raw stream of bytes, dealing with length prefixing.
    pub async fn from_stream(stream: &mut TcpStream, mut buffer: &mut [u8]) -> Result<Self> {
        log::info!("Reading a message");

        // Read the size of the message
        let mut size_buffer = [0_u8; 4];
        stream.read_exact(&mut size_buffer).await?;

        // Convert it to a u32
        let message_size = u32::from_be_bytes(size_buffer);
        log::debug!("Received message length prefix: {}", message_size);

        // Read again from the stream, extending the vector if needed
        let mut bytes = Vec::new();
        let mut remaining_size = message_size;

        log::info!("Buffer length: {}", buffer.len());

        while buffer.len() < remaining_size.try_into().unwrap() {
            log::info!("Reading {} bytes from the stream", buffer.len());
            stream.read(&mut buffer).await?;
            bytes.extend_from_slice(buffer);
            remaining_size -= buffer.len() as u32;
        }

        // Calculate the remaining number of bytes
        let remaining = (remaining_size as usize) % buffer.len();
        log::debug!("Remaining message size: {}", remaining_size);

        // Enforce reading only `remaining` bytes
        let mut truncated = stream.take(remaining as u64);
        truncated.read(&mut buffer).await?;

        bytes.extend_from_slice(&buffer[..remaining]);

        Ok(serde_json::from_slice(&bytes)?)
    }

    /// Converts a [`Message`] into a vector of bytes.
    pub fn as_bytes(&self) -> Vec<u8> {
        log::info!("Writing a message");

        // Convert the message to bytes
        let bytes = match self {
            InterfaceMessage::RawJSON { content } => content.as_bytes().to_vec(),
            _ => serde_json::to_vec(&self).unwrap(),
        };

        // Prepend with the length
        let length = bytes.len() as u32;
        log::debug!("Sending message length prefix: {}", length);

        let mut message = length.to_be_bytes().to_vec();
        log::debug!("Message prefix: {:?}", message);
        message.extend(bytes);

        message
    }
}
