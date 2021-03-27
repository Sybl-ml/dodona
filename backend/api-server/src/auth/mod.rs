//! Contains authorisation primitives for the API server.

use std::convert::TryFrom;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use actix_web::{dev::Payload, web::HttpRequest, FromRequest};
use futures_util::future;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, TokenData, Validation};
use mongodb::bson::oid::ObjectId;
use pbkdf2::{
    password_hash::{
        HasherError, PasswordHash, PasswordHasher, PasswordVerifier, SaltString, VerifyError,
    },
    Params, Pbkdf2,
};
use rand_core::OsRng;

use crate::error::ServerError;

/// Retrieves the encoding key used for JWT authentication.
pub fn get_encoding_key() -> EncodingKey {
    let key = include_str!("../../jwt_key");
    EncodingKey::from_secret(&key.as_bytes())
}

fn get_decoding_key() -> DecodingKey<'static> {
    let key = include_str!("../../jwt_key");
    DecodingKey::from_secret(&key.as_bytes())
}

/// Hashes a user's password.
pub fn hash_password(peppered: &str, rounds: u32) -> Result<String, HasherError> {
    let params = Params {
        rounds,
        ..Default::default()
    };

    let salt = SaltString::generate(&mut OsRng);

    Ok(Pbkdf2
        .hash_password(peppered.as_bytes(), None, None, params, salt.as_salt())?
        .to_string())
}

/// Checks a user's password is correct.
pub fn verify_password(peppered: &str, expected_hash: &str) -> Result<(), VerifyError> {
    let hash = PasswordHash::new(&expected_hash).unwrap();
    Pbkdf2.verify_password(&peppered.as_bytes(), &hash)
}

/// The claims made by a user for authentication.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Claims {
    /// The user's [`ObjectId`] from the database.
    pub id: ObjectId,
    /// The expiry timestamp of the token.
    exp: u64,
}

impl Claims {
    /// Creates a new user with an identifier and expiry timestamp.
    pub fn new(id: ObjectId, exp: u64) -> Self {
        Self { id, exp }
    }

    /// Creates a JWT for the given user identifier with a 2 week duration of usage.
    pub fn create_token(id: ObjectId) -> jsonwebtoken::errors::Result<String> {
        // Tokens default to lasting 2 weeks
        let duration = Duration::from_secs(2 * 7 * 24 * 60 * 60);

        Self::create_token_with_duration(id, duration)
    }

    /// Creates a JWT for the given user identifier with a specified duration of usage.
    ///
    /// This allows tokens to expire after the given time period, meaning the API server will
    /// reject them and force the user to login again.
    pub fn create_token_with_duration(
        id: ObjectId,
        duration: Duration,
    ) -> jsonwebtoken::errors::Result<String> {
        let since_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let exp = (since_epoch + duration).as_secs();

        let header = Header::default();
        let claims = Self::new(id, exp);
        let key = get_encoding_key();

        jsonwebtoken::encode(&header, &claims, &key)
    }
}

impl TryFrom<&HttpRequest> for Claims {
    type Error = ServerError;

    fn try_from(req: &HttpRequest) -> Result<Self, Self::Error> {
        let value = req
            .headers()
            .get("Authorization")
            .ok_or(ServerError::Unauthorized)?;

        let token = value.to_str().map_err(|_| ServerError::Unauthorized)?;

        // Ensure it begins with "Bearer" and remove the prefix
        let token = token
            .strip_prefix("Bearer ")
            .ok_or(ServerError::Unauthorized)?;

        // Get the secret key from the filesystem
        let key = get_decoding_key();
        let validation = Validation::default();
        let token_data: TokenData<Self> = jsonwebtoken::decode(token, &key, &validation)?;

        Ok(token_data.claims)
    }
}

impl FromRequest for Claims {
    type Error = ServerError;
    type Future = future::Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        future::ready(Self::try_from(req))
    }
}

#[cfg(test)]
mod tests;
