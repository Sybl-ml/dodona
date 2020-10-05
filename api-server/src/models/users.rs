use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use crate::models::model;

// Define a model. Simple as deriving a few traits.
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password: String,
    pub salt: String,
}

impl model::Model for User {
    const COLLECTION_NAME: &'static str = "users";

    fn id(&self) -> Option<ObjectId> {
        self.id.clone()
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
}
