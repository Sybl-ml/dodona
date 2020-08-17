use serde::{Deserialize, Serialize};
use bson::oid::ObjectId;

// Define a model. Simple as deriving a few traits.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password: String,
    pub token: String,
}