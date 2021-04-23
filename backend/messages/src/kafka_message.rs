//! Contains the functions to define messages sent through Kafka and the method to produce a message

use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;

use anyhow::Result;

use mongodb::{
    bson::{doc, from_document, oid::ObjectId},
    Database,
};
use models::projects::Project;

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

#[derive(Debug, Serialize, Deserialize)]
pub enum KafkaWsMessage<'a> {

    ClientCompleteMessage {
        /// Model id which node completed
        project_id: &'a str,
        /// the number of time the model as been run
        cluster_size: usize,
        /// The number of models completed for this project
        model_complete_count: usize,
        /// If the model was successfull
        success: bool,
    },
    JobCompleteMessage {
        /// Project id which job completed
        project_id: &'a str,
    }
}

impl KafkaWsMessage<'_> {
    pub async fn produce(&self, database: &Database) -> Result<()> {
        
        match self {
            Self::ClientCompleteMessage { project_id, success, .. } => {
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

                let project: Project = from_document(doc)?;
                let message_key = project.user_id.to_string();
                let topic = "project_updates";

                produce_message(&message_str, &message_key, &topic).await;
            }
            Self::JobCompleteMessage { project_id } => {
                let projects = database.collection("projects");

                let message_str = serde_json::to_string(&self).unwrap();
                let filter = doc! {"_id": ObjectId::with_string(project_id).unwrap()};


                let doc = projects
                    .find_one(filter, None)
                    .await?
                    .expect("Failed to find project in db");

                let project: Project = from_document(doc)?;
                let message_key = project.user_id.to_string();
                let topic = "project_updates";

                produce_message(&message_str, &message_key, &topic).await;
            }
        }
        Ok(())
    }
}
// /// Message produced when a Model completes
// pub struct ClientCompleteMessage<'a> {
//     /// Model id which node completed
//     pub project_id: &'a str,
//     /// the number of time the model as been run
//     pub cluster_size: usize,
//     /// The number of models completed for this project
//     pub model_complete_count: usize,
//     /// If the model was successfull
//     pub success: bool,
// }

// /// Message produced when a job is completed
// #[derive(Debug, Serialize, Deserialize)]
// pub struct JobCompleteMessage<'a> {
//     /// Project id which job completed
//     pub project_id: &'a str,
// }
