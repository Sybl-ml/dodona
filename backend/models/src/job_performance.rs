//! Defines the job performance for a job and model in the `MongoDB` instance.
use mongodb::bson::{self, doc, oid::ObjectId};

/// Defines the information that should be stored as details for a project
#[derive(Debug, Serialize, Deserialize)]
pub struct JobPerformance {
    /// The unique identifier for the JobPerformance
    #[serde(rename = "_id")]
    pub id: ObjectId,
    /// Unique identifier for the associated project
    pub project_id: ObjectId,
    /// Unique identifier for the associated model
    pub model_id: ObjectId,
    /// Model Performance
    pub performance: f64,
}
