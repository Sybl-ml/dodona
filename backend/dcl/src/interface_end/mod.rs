//! Deals with DCL connection to the interface layer
//!
//! Listens to traffic over a socket and maintains a transmitter end of
//! a mpsc channel which allows it to send data to the job end.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;

use anyhow::Result;
use mongodb::bson::doc;
use mongodb::Database;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{stream_consumer::StreamConsumer, Consumer, DefaultConsumerContext};
use rdkafka::Message;
use tokio_stream::StreamExt;

use models::datasets::Dataset;
use models::gridfs;
use models::jobs::JobConfiguration;

use crate::{DatasetPair, JobControl};

/// Starts up interface server
///
/// Takes in socket, db connection and transmitter end of mpsc chaneel and will
/// read in data from an interface. Messages read over this are taken and the
/// corresponding dataset is found and decompressed before being passed to the
/// job end to be sent to a compute node.
pub async fn run(port: u16, db_conn: Arc<Database>, job_control: JobControl) -> Result<()> {
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
            "Message key: {:?}, timestamp: {:?}",
            message.key(),
            message.timestamp()
        );

        let database = Arc::clone(&db_conn);
        let jc_clone = job_control.clone();
        let job_config = serde_json::from_slice(&payload).unwrap();

        tokio::spawn(async move {
            process_job(database, jc_clone, job_config).await.unwrap();
        });
    }

    Ok(())
}

async fn process_job(
    db_conn: Arc<Database>,
    job_control: JobControl,
    job_config: JobConfiguration,
) -> Result<()> {
    let JobConfiguration {
        dataset_id,
        timeout,
        column_types,
        ..
    } = job_config.clone();

    log::info!("Received a message from the interface:");
    log::debug!("\tDataset Identifier: {}", dataset_id);
    log::debug!("\tTimeout: {}", timeout);
    log::debug!("\tColumn types: {:?}", column_types);

    let datasets = db_conn.collection("datasets");
    let files = db_conn.collection("files");

    let filter = doc! { "_id": dataset_id };
    log::debug!("Finding datasets with filter: {:?}", &filter);

    let doc = datasets
        .find_one(filter, None)
        .await?
        .expect("Failed to find a document with the previous filter");
    let dataset: Dataset = mongodb::bson::de::from_document(doc)?;

    log::debug!("Fetched dataset with id: {}", dataset.id);

    // Get the decompressed data from GridFS
    let train_filter = doc! { "_id": dataset.dataset.unwrap() };
    let doc = files
        .find_one(train_filter, None)
        .await?
        .expect("Failed to find a document with the previous filter");
    let train_file: gridfs::File = mongodb::bson::de::from_document(doc)?;
    let comp_train: Vec<u8> = train_file.download_dataset(&db_conn).await?;

    let predict_filter = doc! { "_id": dataset.predict.unwrap() };
    let doc = files
        .find_one(predict_filter, None)
        .await?
        .expect("Failed to find a document with the previous filter");
    let predict_file: gridfs::File = mongodb::bson::de::from_document(doc)?;
    let comp_predict: Vec<u8> = predict_file.download_dataset(&db_conn).await?;

    // Convert it to a string
    let train = String::from_utf8(comp_train)?;
    let predict = String::from_utf8(comp_predict)?;

    log::debug!("Decompressed {} bytes of training data", train.len());
    log::debug!("Decompressed {} bytes of prediction data", predict.len());

    job_control.job_queue.push((
        dataset.project_id,
        DatasetPair { train, predict },
        job_config,
    ));

    job_control.notify.notify_waiters();

    Ok(())
}
