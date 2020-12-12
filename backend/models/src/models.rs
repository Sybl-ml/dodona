//! Defines the details for a model in the MongoDB instance.

use std::fmt;

use chrono::{DateTime, Duration, Utc};
use crypto::generate_access_token;
use mongodb::bson::{self, oid::ObjectId, Binary};

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Running,
    Stopped,
    NotStarted,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Running => write!(f, "Running"),
            Status::Stopped => write!(f, "Stopped"),
            Status::NotStarted => write!(f, "NotStarted"),
        }
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
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    /// The user id which this models belongs to
    pub user_id: ObjectId,
    /// name provided for the model
    pub name: String,
    /// Status of the model
    pub status: Option<Status>,
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
}

impl ClientModel {
    pub fn is_authenticated(&self, token: &[u8]) -> bool {
        // Check the easy conditions
        if !self.authenticated || self.locked {
            return false;
        }

        // Check the user's token
        match &self.access_token {
            Some(x) if x.token.bytes == token => true,
            _ => false,
        }
    }
}
