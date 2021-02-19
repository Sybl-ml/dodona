//! Encodes the protocol for handling node connections in the DCL.

use std::fmt::Display;

use anyhow::{anyhow, Result};
use mongodb::bson::bson;
use serde::Serialize;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use messages::{ClientMessage, RawMessage, ReadLengthPrefix};

#[cfg(test)]
mod tests;

/// The internal state for the protocol.
#[derive(Debug)]
pub struct Handler<'a> {
    stream: &'a mut TcpStream,
    buffer: [u8; 4096],
    current_msg: Option<ClientMessage>,
}

impl<'a> Handler<'a> {
    /// Begins the protocol handling.
    pub fn new(stream: &'a mut TcpStream) -> Self {
        Self {
            stream,
            buffer: [0_u8; 4096],
            current_msg: None,
        }
    }

    /// Peeks at the current message in the channel.
    async fn peek_message(&mut self) -> Result<&ClientMessage> {
        if self.current_msg.is_none() {
            let msg = self.read_message().await?;
            self.current_msg = Some(msg);
        }

        Ok(self.current_msg.as_ref().unwrap())
    }

    /// Responds to a user and waits for their next message.
    async fn respond(&mut self, bytes: &[u8]) -> Result<()> {
        self.stream.write(bytes).await?;

        let next = self.read_message().await?;
        self.current_msg = Some(next);

        Ok(())
    }

    /// Reads a [`Message`] from the TCP stream.
    async fn read_message(&mut self) -> Result<ClientMessage> {
        ClientMessage::from_stream(&mut self.stream, &mut self.buffer).await
    }

    /// Gets the access token for the user.
    ///
    /// Begins the protocol either by getting a [`Message::NewModel`] and setting up the model for
    /// them along with the challenge response, or by instantly receiving a [`Message::AccessToken`]
    /// from the user.
    pub async fn get_access_token(&mut self) -> Result<Option<(String, String)>> {
        if let ClientMessage::NewModel { .. } = self.peek_message().await? {
            self.register_new_model().await?;
            self.authenticate_challenge_response().await?;
            return Ok(None);
        };

        let (id, token) = self.verify_access_token().await?;

        Ok(Some((id, token)))
    }

    /// Registers a new model with the API server.
    async fn register_new_model(&mut self) -> Result<()> {
        let (model_name, email) = match self.current_msg.take().unwrap() {
            ClientMessage::NewModel { model_name, email } => (model_name, email),
            _ => unreachable!(),
        };

        log::info!("Setting up a new model '{}' for: {}", model_name, email);

        // Query the API server
        let body = bson!({
            "model_name": &model_name,
            "email": &email,
        });

        let endpoint = "/api/clients/models/new";
        let text = get_response_text(endpoint, body).await?;

        let message = RawMessage::new(text);

        // Send the response back to the client
        self.respond(&message.as_bytes()).await?;

        Ok(())
    }

    /// Authenticates a user's challenge response with the API server.
    async fn authenticate_challenge_response(&mut self) -> Result<()> {
        let (response, email, model_name) = match self.peek_message().await? {
            ClientMessage::ChallengeResponse {
                response,
                email,
                model_name,
            } => (response, email, model_name),
            _ => unreachable!(),
        };

        log::info!("Sending challenge response: {}", response);

        // Query the API server
        let body = bson!({
            "model_name": &model_name,
            "email": &email,
            "challenge_response": &response,
        });

        let endpoint = "/api/clients/models/verify";
        let text = get_response_text(endpoint, body).await?;

        let message = RawMessage::new(text);

        // Send the response back to the client
        self.stream.write(&message.as_bytes()).await?;

        Ok(())
    }

    /// Verifies a user's access token with the API server.
    async fn verify_access_token(&mut self) -> Result<(String, String)> {
        let (id, token) = match self.peek_message().await? {
            ClientMessage::AccessToken { id, token } => (id.to_string(), token.to_string()),
            _ => unreachable!(),
        };

        log::info!("Verifying access token {} for model {}", token, id);

        // Query the API server
        let body = bson!({
            "token": &token,
        });

        let endpoint = format!("/api/clients/models/{}/authenticate", &id);
        let text = get_response_text(&endpoint, body).await?;

        let message = RawMessage::new(text);

        // Send the response back to the client
        self.stream.write(&message.as_bytes()).await?;

        Ok((id, token))
    }
}

/// Queries the API server and returns the response text.
pub async fn get_response_text<S: Display + Serialize>(endpoint: &str, body: S) -> Result<String> {
    #[cfg(test)]
    let base = mockito::server_url();

    #[cfg(not(test))]
    let base = "http://localhost:3001";

    let url = format!("{}{}", base, endpoint);

    log::debug!("Sending: {} to {}", &body, &url);

    let request = reqwest::Client::new().post(&url).json(&body);
    let response = request.send().await?;
    let status = response.status();

    // Check the status code of the response
    if !status.is_success() {
        let error = format!("Request to {} failed: {}", url, status);
        return Err(anyhow!(error));
    }

    let text = response.text().await?;

    log::debug!("Response body: {:?}", text);

    Ok(text)
}
