use std::collections::VecDeque;
use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::str::FromStr;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::StreamExt;

use config::Environment;
use messages::InterfaceMessage;
use models::jobs::Job;
use utils::setup_logger;

const TIMEOUT_SECS: u64 = 1;

/// Listens for incoming messages from the API server and forwards them to the queue.
async fn listen(inner: &Arc<Inner>) -> Result<()> {
    // Get the environment variable for listening
    let var = env::var("INTERFACE_LISTEN").expect("INTERFACE_LISTEN must be set");
    let port = u16::from_str(&var).expect("INTERFACE_LISTEN must be a u16");

    // Build the address to listen on
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);

    // Begin listening for messages
    let listener = TcpListener::bind(addr).await?;
    log::info!("Listening for connections on: {}", addr);

    loop {
        let (mut stream, address) = listener.accept().await?;
        log::debug!("Received connection from: {}", address);

        let mut buffer = [0_u8; 24];
        let message = InterfaceMessage::from_stream(&mut stream, &mut buffer).await?;

        log::info!("Received: {}", std::str::from_utf8(&buffer).unwrap());

        let mut queue = inner.queue.lock().unwrap();
        queue.push_back(message);

        // Alert the other thread
        drop(queue);
        inner.available.notify_one();
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
        std::thread::sleep(timeout);
    }

    log::debug!("Failed to make a connection at all");

    None
}

/// Receives messages from the frontend thread and communicates with the DCL.
async fn receive(inner: &Arc<Inner>) -> Result<()> {
    // Get the environment variable for sending
    let var = env::var("INTERFACE_SOCKET").expect("INTERFACE_SOCKET must be set");
    let port = u16::from_str(&var).expect("INTERFACE_SOCKET must be a u16");

    // Build the address to send to
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);
    let timeout = Duration::from_secs(TIMEOUT_SECS);
    let attempts = 3;

    // Lock the queue so it cannot change
    let mut queue = inner.queue.lock().unwrap();

    loop {
        // Try and send something onwards
        if let Some(element) = queue.pop_front() {
            if let Some(mut stream) = try_to_connect(&addr, timeout, attempts).await {
                // Send the element to the onward node that we connected to
                stream.write_all(&element.as_bytes()).await?;
                log::info!("Sent: {:?}", element);
                stream.shutdown().await?;
            } else {
                // Readd the element back to the queue at the front
                queue.push_front(element);
            };
        } else {
            log::debug!("Found nothing in the queue");

            // Manually release the mutex and wait before continuing
            queue = inner.available.wait(queue).unwrap();

            log::debug!("Wait on available finished");
        }
    }
}

/// Represents the shared structure both threads have access to.
#[derive(Debug, Default)]
struct Inner {
    /// The internal queue of jobs
    queue: Mutex<VecDeque<InterfaceMessage>>,
    /// The semaphore for alerting threads
    available: Condvar,
}

impl Inner {
    pub fn with_state(initial: VecDeque<InterfaceMessage>) -> Self {
        log::info!("Initialising the queue with {} elements", initial.len());

        Self {
            queue: Mutex::new(initial),
            available: Default::default(),
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

    let state = get_job_queue().await.expect("Failed to get the job queue");
    let inner = Arc::new(Inner::with_state(state));

    log::info!("Beginning the thread execution");

    let left = Arc::clone(&inner);
    let right = Arc::clone(&inner);

    tokio::try_join!(listen(&left), receive(&right))?;

    Ok(())
}
