use std::collections::VecDeque;
use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt;

use config::Environment;
use messages::{InterfaceMessage, ReadLengthPrefix, WriteLengthPrefix};
use models::jobs::Job;
use utils::setup_logger;

const TIMEOUT_SECS: u64 = 1;

/// Listens for incoming messages from the API server and forwards them to the queue.
async fn listen(inner: UnboundedSender<InterfaceMessage>) -> Result<()> {
    // Get the environment variable for listening
    let var = env::var("INTERFACE_LISTEN").expect("INTERFACE_LISTEN must be set");
    let port = u16::from_str(&var).expect("INTERFACE_LISTEN must be a u16");

    // Build the address to listen on
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);

    // Begin listening for messages
    let listener = TcpListener::bind(addr).await?;
    log::info!("Listening for connections on: {}", addr);

    let mut buffer = [0_u8; 1024];

    loop {
        let (mut stream, address) = listener.accept().await?;
        log::debug!("Received connection from: {}", address);

        let message = InterfaceMessage::from_stream(&mut stream, &mut buffer).await?;
        log::info!("Received: {:?}", message);

        inner.send(message)?;
    }
}

/// Continually tries to connect until a connection is achieved.
async fn try_to_connect(
    address: &SocketAddrV4,
    timeout: Duration,
    attempts: usize,
) -> Option<TcpStream> {
    for i in 0..attempts {
        log::debug!("Connection attempt: {}", i + 1);

        if let Ok(stream) = TcpStream::connect(address).await {
            return Some(stream);
        }

        log::debug!("Failed to connect to: {}", address);
        log::debug!("Sleeping for: {:?}", timeout);
        tokio::time::sleep(timeout).await;
    }

    log::debug!("Failed to make a connection at all");

    None
}

/// Receives messages from the frontend thread and communicates with the DCL.
async fn receive(mut inner: UnboundedReceiver<InterfaceMessage>) -> Result<()> {
    // Get the environment variable for sending
    let var = env::var("INTERFACE_SOCKET").expect("INTERFACE_SOCKET must be set");
    let port = u16::from_str(&var).expect("INTERFACE_SOCKET must be a u16");

    // Build the address to send to
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let timeout = Duration::from_secs(TIMEOUT_SECS);
    let attempts = 3;

    // Lock the queue so it cannot change
    loop {
        // Try and send something onwards
        if let Some(element) = inner.recv().await {
            if let Some(mut stream) = try_to_connect(&addr, timeout, attempts).await {
                // Send the element to the onward node that we connected to
                stream.write_all(&element.as_bytes()).await?;
                log::info!("Sent: {:?}", element);
                stream.shutdown().await?;
            }
        }
    }
}

async fn get_job_queue() -> mongodb::error::Result<VecDeque<InterfaceMessage>> {
    // Setup the MongoDB client
    let uri = std::env::var("CONN_STR").unwrap();
    let client = mongodb::Client::with_uri_str(&uri).await?;

    // Get the jobs collection
    let database = client.database("sybl");
    let jobs = database.collection("jobs");

    // Pull all the jobs and deserialize them
    let cursor = jobs.find(None, None).await?;

    // TODO: Change this to collect once by iterating over the cursor
    let queue: Vec<InterfaceMessage> = cursor
        .filter_map(Result::ok)
        .filter_map(|x| mongodb::bson::de::from_document::<Job>(x).ok())
        .map(|x| x.msg)
        .collect()
        .await;

    Ok(queue.into_iter().collect())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup logging
    setup_logger("interface");

    // Load the configuration variables
    let environment = if cfg!(debug_assertions) {
        Environment::Development
    } else {
        Environment::Production
    };

    config::load(environment);

    let (tx, rx) = mpsc::unbounded_channel();

    let state = get_job_queue().await.expect("Failed to get the job queue");
    log::info!("Beginning with {} elements in the queue", state.len());
    state.into_iter().for_each(|e| tx.send(e).unwrap());

    log::info!("Beginning the thread execution");
    tokio::try_join!(listen(tx), receive(rx))?;

    Ok(())
}
