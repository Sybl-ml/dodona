//! Contains the Analytics Server for the Sybl project.
//!
//! Manages connections to a `MongoDB` database and an Interface Layer

#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]

use anyhow::Result;
use mongodb::bson::oid::ObjectId;
use mongodb::options::ClientOptions;
use mongodb::Client;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

use rdkafka::config::{ClientConfig, RDKafkaLogLevel};
use rdkafka::consumer::{stream_consumer::StreamConsumer, Consumer, DefaultConsumerContext};
use rdkafka::Message;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio_stream::StreamExt;

mod dataset_analysis;

/// Main runner function for the Analytics Server
///
/// This function is called when starting up the Analytics Server. It starts the
/// tokio runtime and sets up its connection with the MongoDB database.
/// It will then spawn threads for the different parts of the DCL to
/// offer the full functionality of the product.

pub async fn run() -> Result<()> {
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");

    let broker_socket =
        u16::from_str(&env::var("BROKER_PORT").unwrap_or_else(|_| "9092".to_string()))
            .expect("BROKER_PORT must be a u16");
    let database_name = env::var("DATABASE_NAME").unwrap_or_else(|_| String::from("sybl"));

    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);

    let client = Client::with_options(client_options).unwrap();
    let database = Arc::new(client.database(&database_name));

    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, broker_socket).to_string();
    log::info!("Broker Socket: {:?}", addr);

    let consumer: StreamConsumer<DefaultConsumerContext> = ClientConfig::new()
        .set("group.id", "analytics")
        .set("bootstrap.servers", addr)
        .set("enable.partition.eof", "false")
        .set("session.timeout.ms", "6000")
        .set("enable.auto.commit", "true")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()
        .expect("Consumer creation failed");

    consumer
        .subscribe(&["analytics"])
        .expect("Can't subscribe to analytics");

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

        let project_id: ObjectId = serde_json::from_slice(&payload).unwrap();

        log::debug!(
            "Timestamp: {:?}, Payload: {}",
            message.timestamp(),
            &project_id
        );

        dataset_analysis::prepare_dataset(&database, &project_id).await?;
    }
    Ok(())
}
