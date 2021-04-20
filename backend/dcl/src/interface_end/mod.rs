//! Deals with DCL connection to the interface layer
//!
//! Listens to traffic over a socket and maintains a transmitter end of
//! a mpsc channel which allows it to send data to the job end.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use anyhow::Result;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{stream_consumer::StreamConsumer, Consumer, DefaultConsumerContext};
use rdkafka::Message;
use tokio_stream::StreamExt;

use models::datasets::Dataset;
use models::gridfs;
use models::jobs::{Job, JobConfiguration};

use crate::{DatasetPair, JobControl};

/// Starts up interface server
///
/// Takes in socket, db connection and transmitter end of mpsc chaneel and will
/// read in data from an interface. Messages read over this are taken and the
/// corresponding dataset is found and decompressed before being passed to the
/// job end to be sent to a compute node.
pub async fn run(port: u16, db_conn: Arc<Database>, job_control: JobControl) -> Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).to_string();
    log::info!("Listening to messages from Kafka on: {}", addr);

    let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
        .set("group.id", "job_config")
        .set("bootstrap.servers", addr)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&["jobs"])
        .expect("Can't subscribe to jobs");

    // Ignore any errors in the stream
    let mut message_stream = consumer.stream().filter_map(Result::ok);

    while let Some(message) = message_stream.next().await {
        // Interpret the content as a string
        let payload = match message.payload_view::<[u8]>() {
            // This cannot fail, `rdkafka` always returns `Ok(bytes)`
            Some(view) => view.unwrap(),
            None => {
                log::warn!("Received an empty message from Kafka");
                continue;
            }
        };

        log::debug!(
            "Message key={:?}, timestamp={:?}",
            message.key(),
            message.timestamp()
        );

        let database = Arc::clone(&db_conn);
        let jc_clone = job_control.clone();

        let job_config = match serde_json::from_slice(&payload) {
            Ok(config) => config,
            Err(e) => {
                log::error!("Failed to deserialize a message from Kafka: {}", e);
                continue;
            }
        };

        tokio::spawn(async move {
            process_job(database, jc_clone, job_config).await.unwrap();
        });
    }

    Ok(())
}

async fn download_dataset(database: &Database, identifier: &ObjectId) -> Result<Vec<u8>> {
    let files = database.collection("files");

    let filter = doc! { "_id": identifier };
    let doc = files
        .find_one(filter, None)
        .await?
        .expect("Failed to find a document with the previous filter");

    let file: gridfs::File = mongodb::bson::de::from_document(doc)?;
    Ok(file.download_dataset(&database).await?)
}

async fn process_job(db_conn: Arc<Database>, job_control: JobControl, job: Job) -> Result<()> {
    let JobConfiguration {
        project_id,
        node_computation_time,
        ..
    } = &job.config;

    log::debug!(
        "Received a job to process: job_id={}, project_id={}, node_computation_time={}",
        job.id,
        project_id,
        node_computation_time,
    );

    let datasets = db_conn.collection("datasets");

    // Query the dataset currently associated with the project
    let filter = doc! { "project_id": &project_id };

    let doc = datasets
        .find_one(filter, None)
        .await?
        .expect("Failed to find a document with the previous filter");

    let dataset: Dataset = mongodb::bson::de::from_document(doc)?;

    log::debug!("Found the dataset with id={}", dataset.id);

    // Get the decompressed data from GridFS
    let compressed_train = download_dataset(&db_conn, &dataset.dataset).await?;
    let compressed_predict = download_dataset(&db_conn, &dataset.predict).await?;

    // Convert it to a string
    let train = String::from_utf8(compressed_train)?;
    let predict = String::from_utf8(compressed_predict)?;

    job_control
        .job_queue
        .push((dataset.project_id, DatasetPair { train, predict }, job));

    job_control.notify.notify_waiters();

    Ok(())
}
