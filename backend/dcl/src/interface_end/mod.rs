//! Deals with DCL connection to the interface layer
//!
//! Listens to traffic over a socket and maintains a transmitter end of
//! a mpsc channel which allows it to send data to the job end.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use anyhow::Result;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Database;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{stream_consumer::StreamConsumer, Consumer, DefaultConsumerContext};
use rdkafka::Message;
use tokio::sync::mpsc::Sender;

use messages::ClientMessage;
use models::datasets::Dataset;
use models::jobs::JobConfiguration;
use utils::compress::decompress_data;

use crate::DatasetPair;

/// Starts up interface server
///
/// Takes in socket, db connection and transmitter end of mpsc chaneel and will
/// read in data from an interface. Messages read over this are taken and the
/// corresponding dataset is found and decompressed before being passed to the
/// job end to be sent to a compute node.
pub async fn run(
    port: u16,
    db_conn: Arc<Database>,
    tx: Sender<(ObjectId, DatasetPair, ClientMessage)>,
) -> Result<()> {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).to_string();
    log::info!("Broker Socket: {:?}", addr);

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

    loop {
        match consumer.recv().await {
            Err(e) => log::warn!("Kafka error: {}", e),
            Ok(m) => {
                let payload = match m.payload_view::<str>() {
                    None => "",
                    Some(Ok(s)) => s,
                    Some(Err(e)) => {
                        log::warn!("Error while deserializing message payload: {:?}", e);
                        ""
                    }
                };

                log::debug!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                      m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());

                let job_config: JobConfiguration = serde_json::from_str(&payload).unwrap();

                let db_conn_clone = Arc::clone(&db_conn);
                let tx_clone = tx.clone();

                tokio::spawn(async move {
                    process_job(db_conn_clone, tx_clone, job_config)
                        .await
                        .unwrap();
                });
            }
        };
    }
}

async fn process_job(
    db_conn: Arc<Database>,
    tx: Sender<(ObjectId, DatasetPair, ClientMessage)>,
    job_config: JobConfiguration,
) -> Result<()> {
    let JobConfiguration {
        dataset_id,
        timeout,
        cluster_size,
        column_types,
        prediction_column,
        prediction_type,
    } = job_config;

    log::info!("Received a message from the interface:");
    log::debug!("\tDataset Identifier: {}", dataset_id);
    log::debug!("\tTimeout: {}", timeout);
    log::debug!("\tColumn types: {:?}", column_types);

    let datasets = db_conn.collection("datasets");

    let filter = doc! { "_id": dataset_id };
    log::debug!("Finding datasets with filter: {:?}", &filter);

    let doc = datasets
        .find_one(filter, None)
        .await?
        .expect("Failed to find a document with the previous filter");
    let dataset: Dataset = mongodb::bson::de::from_document(doc)?;

    log::debug!("Fetched dataset with id: {}", dataset.id);

    // Get the data from the struct
    let comp_train = dataset.dataset.unwrap().bytes;
    let comp_predict = dataset.predict.unwrap().bytes;

    // Decompress it
    let train_bytes = decompress_data(&comp_train)?;
    let predict_bytes = decompress_data(&comp_predict)?;

    // Convert it to a string
    let train = std::str::from_utf8(&train_bytes)?.to_string();
    let predict = std::str::from_utf8(&predict_bytes)?.to_string();

    log::debug!("Decompressed {} bytes of training data", train.len());
    log::debug!("Decompressed {} bytes of prediction data", predict.len());

    tx.send((
        dataset.project_id,
        DatasetPair { train, predict },
        ClientMessage::JobConfig {
            timeout,
            cluster_size,
            column_types,
            prediction_column,
            prediction_type,
        },
    ))
    .await
    .unwrap_or_else(|error| log::error!("Error while sending over MPSC: {}", error));

    Ok(())
}
