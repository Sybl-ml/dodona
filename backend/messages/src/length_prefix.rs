//! Contains a trait to allow any object to become length prefixed bytes.

use std::convert::TryInto;

use anyhow::Result;
use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

use crate::{ClientMessage, InterfaceMessage};

// Implement reading and writing for our 2 message types
impl ReadLengthPrefix for ClientMessage {}
impl ReadLengthPrefix for InterfaceMessage {}

impl WriteLengthPrefix for ClientMessage {}
impl WriteLengthPrefix for InterfaceMessage {}

/// Allows any object that is [`DeserializeOwned`] to be deserialized from length prefixed form.
#[async_trait]
pub trait ReadLengthPrefix: DeserializeOwned {
    /// Reads a [`Message`] from a raw stream of bytes, dealing with length prefixing.
    async fn from_stream<'a>(stream: &mut TcpStream, mut buffer: &mut [u8]) -> Result<Self> {
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
}

/// Allows any object that is [`Serialize`] to be serialized in length prefixed form.
pub trait WriteLengthPrefix: Serialize {
    /// Converts a [`Message`] into a vector of bytes.
    fn as_bytes(&self) -> Vec<u8> {
        // Convert the message to bytes
        let bytes = serde_json::to_vec(&self).unwrap();

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
