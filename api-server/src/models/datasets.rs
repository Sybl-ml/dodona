use mongodb::bson;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::Binary;
use serde::{Deserialize, Serialize};

// Define a model. Simple as deriving a few traits.
#[derive(Debug, Serialize, Deserialize)]
pub struct Dataset {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub project_id: Option<ObjectId>,
    pub date_created: bson::DateTime,
    pub dataset: Option<Binary>,
}
