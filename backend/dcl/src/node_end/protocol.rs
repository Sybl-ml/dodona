//! Encodes the protocol for handling node connections in the DCL.

use std::fmt::Display;

use anyhow::Result;
use mongodb::bson::bson;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::messages::Message;

/// The internal state for the protocol.
#[derive(Debug)]
pub struct Handler<'a> {
    stream: &'a mut TcpStream,
    buffer: [u8; 1024],
    current_msg: Option<Message>,
}

impl<'a> Handler<'a> {
    /// Begins the protocol handling.
    pub fn new(stream: &'a mut TcpStream) -> Self {
        Self {
            stream,
            buffer: [0_u8; 1024],
            current_msg: None,
        }
    }

    /// Peeks at the current message in the channel.
    async fn peek_message(&mut self) -> Result<&Message> {
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
    async fn read_message(&mut self) -> Result<Message> {
        let size = self.stream.read(&mut self.buffer).await?;
        Ok(Message::from_slice(&self.buffer[..size]))
    }

    /// Gets the access token for the user.
    ///
    /// Begins the protocol either by getting a [`Message::NewModel`] and setting up the model for
    /// them along with the challenge response, or by instantly receiving a [`Message::AccessToken`]
    /// from the user.
    pub async fn get_access_token(&mut self) -> Result<String> {
        match self.peek_message().await? {
            Message::NewModel { .. } => {
                self.register_new_model().await?;
                self.authenticate_challenge_response().await?;
            }
            _ => (),
        };

        let token = self.verify_access_token().await?;

        Ok(token)
    }

    /// Registers a new model with the API server.
    async fn register_new_model(&mut self) -> Result<()> {
        let (model_name, email) = match self.current_msg.take().unwrap() {
            Message::NewModel { model_name, email } => (model_name, email),
            _ => unreachable!(),
        };

        log::info!("Setting up a new model '{}' for: {}", model_name, email);

        // Query the API server
        let body = bson!({
            "model_name": &model_name,
            "email": &email,
        });

        let url = "http://localhost:3001/api/clients/m/new";
        let text = get_response_text(url, body).await?;

        self.respond(text.as_bytes()).await?;

        Ok(())
    }

    /// Authenticates a user's challenge response with the API server.
    async fn authenticate_challenge_response(&mut self) -> Result<()> {
        let (response, email, model_name) = match self.peek_message().await? {
            Message::ChallengeResponse {
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

        let url = "http://localhost:3001/api/clients/m/verify";
        let text = get_response_text(url, body).await?;

        // Send the response back to the client
        self.respond(text.as_bytes()).await?;

        Ok(())
    }

    /// Verifies a user's access token with the API server.
    async fn verify_access_token(&mut self) -> Result<String> {
        let (id, token) = match self.peek_message().await? {
            Message::AccessToken { id, token } => (id, String::from(token)),
            _ => unreachable!(),
        };

        log::info!("Verifying access token {} for model {}", token, id);

        // Query the API server
        let body = bson!({
            "id": &id,
            "token": &token,
        });

        let url = "http://localhost:3001/api/clients/m/authenticate";
        let text = get_response_text(url, body).await?;

        // Send the response back to the client
        self.stream.write(text.as_bytes()).await?;

        Ok(String::from(token))
    }
}

/// Queries the API server and returns the response text.
pub async fn get_response_text<S: Display + Serialize>(url: &str, body: S) -> Result<String> {
    log::debug!("Sending: {} to {}", &body, url);

    let text = reqwest::blocking::Client::new()
        .post(url)
        .json(&body)
        .send()?
        .text()?;

    log::debug!("Response body: {:?}", text);

    Ok(text)
}
