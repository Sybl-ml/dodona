//! Contains authorisation primitives for the API server.

use std::convert::TryFrom;

use actix_web::{dev::Payload, web::HttpRequest, FromRequest};
use futures_util::future;
use jsonwebtoken::{DecodingKey, EncodingKey, TokenData, Validation};
use mongodb::bson::oid::ObjectId;

use crate::dodona_error::DodonaError;

pub fn get_encoding_key() -> EncodingKey {
    let key = include_str!("../../jwt_key");
    EncodingKey::from_secret(&key.as_bytes())
}

fn get_decoding_key() -> DecodingKey<'static> {
    let key = include_str!("../../jwt_key");
    DecodingKey::from_secret(&key.as_bytes())
}

/// An authorised user and their identifier.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct User {
    pub id: ObjectId,
    exp: u64,
}

impl User {
    /// Creates a new user with an identifier and expiry timestamp.
    pub fn new(id: ObjectId, exp: u64) -> Self {
        Self { id, exp }
    }
}

impl TryFrom<&HttpRequest> for User {
    type Error = DodonaError;

    fn try_from(req: &HttpRequest) -> Result<Self, Self::Error> {
        let value = req
            .headers()
            .get("Authorization")
            .ok_or(DodonaError::Unauthorized)?;

        let token = value.to_str().map_err(|_| DodonaError::Unauthorized)?;

        // Ensure it begins with "Bearer" and remove the prefix
        let token = token
            .strip_prefix("Bearer ")
            .ok_or(DodonaError::Unauthorized)?;

        // Get the secret key from the filesystem
        let key = get_decoding_key();
        let validation = Validation::default();
        let token_data: TokenData<User> = jsonwebtoken::decode(token, &key, &validation)?;

        Ok(token_data.claims)
    }
}

impl FromRequest for User {
    type Error = DodonaError;
    type Future = future::Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        future::ready(User::try_from(req))
    }
}

#[cfg(test)]
mod tests;
