use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;

use rdkafka::config::ClientConfig;
use rdkafka::error::KafkaResult;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::util::Timeout;

pub async fn produce_message(msg: &str, key: &str, topic: &str) -> KafkaResult<()> {
    // Get the environment variable for the kafka broker
    // if not set use 9092
    let var = env::var("BROKER_PORT").unwrap_or_else(|_| "9092".to_string());
    let port = u16::from_str(&var).expect("BROKER_PORT must be a u16");

    // Build the address to send to
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).to_string();
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", &addr)
        .set("message.timeout.ms", "5000")
        .create()
        .expect("Producer creation error");

    let delivery_status = producer
        .send(
            FutureRecord::to(topic).payload(msg).key(key),
            Timeout::Never,
        )
        .await;

    log::debug!("Message sent result: {:?}", delivery_status);

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ClientCompleteMessage<'a> {
    /// Model id which node completed
    pub project_id: &'a str,
    /// the number of time the model as been run
    pub cluster_size: usize,
    /// The number of models completed for this project
    pub model_complete_count: usize,
    /// If the model was successfull
    pub success: bool,
}
