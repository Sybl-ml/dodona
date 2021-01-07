//! Contains the builder functions used to generate message for DCL-DCN protcol

use std::convert::TryInto;

use anyhow::Result;
use mongodb::bson::oid::ObjectId;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

/// Different messages to be passed between DCL and DCN
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ClientMessage {
    /// Hearbeat alive message
    Alive {
        /// The current timestamp
        timestamp: u64,
    },
    /// Message to send Job Config
    JobConfig {
        /// Job timeout
        timeout: u8,
        /// Types of each column in dataset for job
        column_types: Vec<String>,
    },
    /// A request to setup a new model
    NewModel {
        /// The client's email address
        email: String,
        /// The name of the model
        model_name: String,
    },
    /// A response to a challenge issued by the server
    ChallengeResponse {
        /// The user's email
        email: String,
        /// The model name corresponding to the challenge
        model_name: String,
        /// The calculated response value
        response: String,
    },
    /// A request to authenticate a model
    AccessToken {
        /// The model identifier
        id: String,
        /// The access token itself
        token: String,
    },
    /// A dataset for the node to process
    Dataset {
        /// The dataset to train on
        train: String,
        /// The dataset to predict on
        predict: String,
    },
    /// Prediction data from a node after computation
    Predictions(String),
    /// A raw JSON message, usually from the API server
    RawJSON {
        /// The raw JSON contents
        content: String,
    },
}

impl ClientMessage {
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
            ClientMessage::RawJSON { content } => content.as_bytes().to_vec(),
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

/// Different messages to be passed between Interface and DCL
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InterfaceMessage {
    /// Configuration message for Job
    Config {
        /// ID of the dataset associated with Job
        id: ObjectId,
        /// Job timeout
        timeout: u8,
        /// The columns in the dataset
        column_types: Vec<String>,
    },
    /// A raw JSON message, usually from the API server
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
