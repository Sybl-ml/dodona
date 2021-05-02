//! Contains the functions to define messages sent through Kafka and the method to produce a message

use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;

use anyhow::Result;

use models::projects::Project;
use mongodb::{
    bson::{doc, from_document, oid::ObjectId},
    Database,
};

use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

/// Produces a message for Kafka accepting a message, the key and the topic to produce into
pub async fn produce_message(msg: &str, key: &str, topic: &str) {
    // Get the environment variable for the kafka broker
    // if not set use 9092
    let var = env::var("BROKER_PORT").unwrap_or_else(|_| "9092".to_string());
    let port = u16::from_str(&var).expect("BROKER_PORT must be a u16");

    log::debug!(
        "Sending msg={} to Kafka with key={} and topic={}",
        msg,
        key,
        topic
    );

    // Build the address to send to
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).to_string();
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &addr)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    if let Err(e) = producer
        .send(
            FutureRecord::to(topic).payload(msg).key(key),
            Timeout::Never,
        )
        .await
    {
        log::warn!("Failed to send the message to Kafka: {:?}", e);
    }
}

/// Enum defining all messages sent through Kafka to update a websocket
#[derive(Debug, Serialize, Deserialize)]
pub enum KafkaWsMessage<'a> {
    /// Message produced when a Model completes
    ClientCompleteMessage {
        /// project id which the client completed
        project_id: &'a str,
        /// the cluster size
        cluster_size: usize,
        /// The number of models completed for this project
        model_complete_count: usize,
        /// If the model was successfull
        success: bool,
    },
    /// Message sent when a project is completed
    JobCompleteMessage {
        /// Project id which job completed
        project_id: &'a str,
    },
}

impl KafkaWsMessage<'_> {
    /// Produce a message for Kafka
    /// if a client message increment the success status and get user key
    /// if job complete get user key
    pub async fn produce(&self, database: &Database) -> Result<()> {
        let (doc, message) = match self {
            Self::ClientCompleteMessage {
                project_id,
                success,
                ..
            } => {
                let projects = database.collection("projects");

                let message_str = serde_json::to_string(&self).unwrap();
                let filter = doc! {"_id": ObjectId::with_string(project_id).unwrap()};

                let update = if *success {
                    doc! {"$inc": {"status.Processing.model_success": 1}}
                } else {
                    doc! {"$inc": {"status.Processing.model_err": 1}}
                };

                let doc = projects
                    .find_one_and_update(filter, update, None)
                    .await?
                    .expect("Failed to find project in db");

                (doc, message_str)
            }
            Self::JobCompleteMessage { project_id } => {
                let projects = database.collection("projects");

                let message_str = serde_json::to_string(&self).unwrap();
                let filter = doc! {"_id": ObjectId::with_string(project_id).unwrap()};

                let doc = projects
                    .find_one(filter, None)
                    .await?
                    .expect("Failed to find project in db");

                (doc, message_str)
            }
        };

        let project: Project = from_document(doc)?;
        let message_key = project.user_id.to_string();
        let topic = "project_updates";

        produce_message(&message, &message_key, &topic).await;
        Ok(())
    }
}
