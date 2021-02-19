use std::collections::VecDeque;
use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::time::Duration;

use anyhow::Result;
use mongodb::bson::{doc, oid::ObjectId};
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt;

use config::Environment;
use messages::{ReadLengthPrefix, WriteLengthPrefix};
use models::jobs::Job;
use utils::setup_logger;

const TIMEOUT_SECS: u64 = 1;

/// Listens for incoming messages from the API server and forwards them to the queue.
async fn listen(inner: UnboundedSender<Job>) -> Result<()> {
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

        let message = Job::from_stream(&mut stream, &mut buffer).await?;
        log::info!("Received: {:?}", message);

        // Send the job to the other task
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
async fn receive(mut inner: UnboundedReceiver<Job>) -> Result<()> {
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
                log::info!("Sent: {:?}", element.config);
                stream.shutdown().await?;

                // Mark the job as processed, at least by the interface layer
                mark_job_as_processed(&element.id).await?;
            }
        }
    }
}

async fn mark_job_as_processed(identifier: &ObjectId) -> Result<()> {
    log::debug!("Marking the following job as processed: {}", identifier);

    // Get a connection to the database
    let uri = std::env::var("CONN_STR").unwrap();
    let client = mongodb::Client::with_uri_str(&uri).await?;

    // Get the jobs collection
    let jobs = client.database("sybl").collection("jobs");

    // Update the job with the given identifier
    let filter = doc! { "_id": &identifier };
    let update = doc! { "$set": { "processed": true } };
    jobs.update_one(filter, update, None).await?;

    Ok(())
}

async fn get_job_queue() -> mongodb::error::Result<VecDeque<Job>> {
    // Setup the MongoDB client
    let uri = std::env::var("CONN_STR").unwrap();
    let client = mongodb::Client::with_uri_str(&uri).await?;

    // Get the jobs collection
    let database = client.database("sybl");
    let jobs = database.collection("jobs");

    // Filter only for jobs that have not been processed yet
    let filter = doc! { "processed": false };

    // Pull all the jobs and deserialize them
    let mut cursor = jobs.find(filter, None).await?;

    let mut queue = VecDeque::new();

    while let Some(Ok(item)) = cursor.next().await {
        let job: Job = mongodb::bson::de::from_document(item).unwrap();
        queue.push_back(job);
    }

    Ok(queue)
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
