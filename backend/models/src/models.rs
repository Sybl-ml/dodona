//! Defines the details for a model in the `MongoDB` instance.

use std::fmt;

use chrono::{DateTime, Duration, Utc};
use mongodb::bson::{self, doc, oid::ObjectId, Binary, Bson};

use crypto::generate_access_token;

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Running,
    Stopped,
    NotStarted,
}

impl From<Status> for Bson {
    fn from(status: Status) -> Self {
        Self::from(match status {
            Status::Running => "Running",
            Status::Stopped => "Stopped",
            Status::NotStarted => "NotStarted",
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessToken {
    pub token: Binary,
    pub expires: DateTime<Utc>,
}

impl AccessToken {
    pub fn new() -> AccessToken {
        AccessToken {
            token: Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: generate_access_token(),
            },
            expires: Utc::now() + Duration::weeks(2),
        }
    }

    /// Checks whether an [`AccessToken`] has expired at the current time.
    pub fn has_expired(&self) -> bool {
        self.will_have_expired_by(Utc::now())
    }

    /// Checks whether an [`AccessToken`] will have expired by the given time.
    fn will_have_expired_by(&self, time: chrono::DateTime<Utc>) -> bool {
        self.expires < time
    }
}

impl Default for AccessToken {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for AccessToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} ({})",
            String::from_utf8(self.token.clone().bytes).unwrap(),
            self.expires.to_rfc3339()
        )
    }
}

/// Defines the information that should be stored as details for a model
#[derive(Debug, Serialize, Deserialize)]
pub struct ClientModel {
    /// The unique identifier for the client model
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// The user id which this models belongs to
    pub user_id: ObjectId,
    /// name provided for the model
    pub name: String,
    /// Status of the model
    pub status: Status,
    /// The access token for the model, if set
    pub access_token: Option<AccessToken>,
    /// false if the model has been unlocked through web
    pub locked: bool,
    /// false if model has not been authenticated with private key
    pub authenticated: bool,
    /// The most recent challenge sent to client
    pub challenge: Option<Binary>,
    /// The number of times the model has been run
    pub times_run: i32,
    /// The amount of time spent processing predictions
    pub processing_time_secs: i64,
    /// The total number of credits earned so far
    pub credits_earned: i64,
}

impl ClientModel {
    pub fn new(user_id: ObjectId, name: String, challenge: Vec<u8>) -> Self {
        log::debug!(
            "Creating a new model for user_id={} with name={}",
            user_id,
            name
        );

        Self {
            id: ObjectId::new(),
            user_id,
            name,
            status: Status::NotStarted,
            access_token: None,
            locked: true,
            authenticated: false,
            challenge: Some(Binary {
                subtype: bson::spec::BinarySubtype::Generic,
                bytes: challenge,
            }),
            times_run: 0,
            processing_time_secs: 0,
            credits_earned: 0,
        }
    }

    pub fn is_authenticated(&self, token: &[u8]) -> bool {
        // Check the easy conditions
        if !self.authenticated || self.locked {
            return false;
        }

        // Check the user's token
        matches!(&self.access_token, Some(x) if x.token.bytes == token)
    }

    /// Checks whether the client's token has not expired.
    ///
    /// If the client does not have a token, immediately return `false` as a base condition.
    /// Otherwise, check the timestamp against the current time.
    pub fn token_has_not_expired(&self) -> bool {
        let token = match &self.access_token {
            Some(token) => token,
            None => return false,
        };

        !token.has_expired()
    }

    pub async fn delete(&self, database: &mongodb::Database) -> mongodb::error::Result<()> {
        let models = database.collection("models");

        log::debug!("Deleting model with id={}", self.id);

        let filter = doc! {"_id": &self.id};
        models.delete_one(filter, None).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::models::AccessToken;
    use chrono::{Duration, Utc};

    #[test]
    fn access_tokens_do_not_expire_immediately() {
        let token = AccessToken::new();

        let now = Utc::now();
        assert!(!token.will_have_expired_by(now));
    }

    #[test]
    fn access_tokens_will_expire() {
        let token = AccessToken::new();

        let far_future = Utc::now() + Duration::weeks(2) + Duration::days(1);
        assert!(token.will_have_expired_by(far_future));
    }
}
