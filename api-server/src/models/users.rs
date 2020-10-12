use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

// Define a model. Simple as deriving a few traits.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password: String,
    pub salt: String,
    pub first_name: String,
    pub last_name: String,
}
