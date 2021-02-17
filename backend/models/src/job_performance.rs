//! Defines the job performance for a job and model in the `MongoDB` instance.
use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use mongodb::bson::de::from_document;
use mongodb::{
    bson::{self, doc, document::Document, oid::ObjectId},
    Database,
};
use tokio_stream::StreamExt;

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
    /// Date Job Happened
    pub date_created: bson::DateTime,
}

impl JobPerformance {
    /// Creates a new instance of [`JobPerformance`].
    pub fn new(project_id: ObjectId, model_id: ObjectId, performance: f64) -> Self {
        Self {
            id: ObjectId::new(),
            project_id,
            model_id,
            performance,
            date_created: bson::DateTime(Utc::now()),
        }
    }

    /// Gets the past >=5 JobPerformances and returns them as a Vec
    pub async fn get_past_5(database: Arc<Database>, model_id: &str) -> Result<Vec<f64>> {
        let job_performances = database.collection("job_performances");

        let filter = doc! {"model_id": ObjectId::with_string(model_id)?};

        let build_options = mongodb::options::FindOptions::builder()
            .sort(doc! {"date_created": -1})
            .build();

        let cursor = job_performances.find(filter, Some(build_options)).await?;

        let get_performance = |doc: Document| -> Result<f64> {
            let job_performance: Self = from_document(doc)?;
            Ok(job_performance.performance)
        };

        let performances: Vec<_> = cursor
            .take(5)
            .filter_map(Result::ok)
            .map(get_performance)
            .collect::<Result<_, _>>()
            .await?;

        Ok(performances)
    }
}
