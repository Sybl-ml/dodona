use std::collections::VecDeque;
use std::env;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, Shutdown, SocketAddrV4, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use async_std::stream::StreamExt;
use async_std::task::block_on;

use config::Environment;
use models::jobs::Job;
use utils::setup_logger;

const TIMEOUT_SECS: u64 = 1;

/// Listens for incoming messages from the API server and forwards them to the queue.
fn listen(inner: &Arc<Inner>) -> std::io::Result<()> {
    // Get the environment variable for listening
    let var = env::var("INTERFACE_LISTEN").expect("INTERFACE_LISTEN must be set");
    let port = u16::from_str(&var).expect("INTERFACE_LISTEN must be a u16");

    // Build the address to listen on
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, port);

    // Begin listening for messages
    let listener = TcpListener::bind(addr)?;
    let incoming = listener.incoming();

    log::info!("Listening for connections on: {}", addr);

    for possible_stream in incoming {
        let mut stream = possible_stream?;
        let mut buffer = [0_u8; 24];
        stream.read_exact(&mut buffer)?;

        log::info!("Received: {}", std::str::from_utf8(&buffer).unwrap());

        let mut queue = inner.queue.lock().unwrap();
        queue.push_back(buffer);

        // Alert the other thread
        drop(queue);
        inner.available.notify_one();
    }

    Ok(())
}

/// Continually tries to connect until a connection is achieved.
fn try_to_connect(address: &SocketAddrV4, timeout: Duration, attempts: usize) -> Option<TcpStream> {
    for i in 0..attempts {
        log::debug!("Connection attempt: {}", i + 1);

        if let Ok(stream) = TcpStream::connect(address) {
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
fn receive(inner: &Arc<Inner>) -> std::io::Result<()> {
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
            if let Some(mut stream) = try_to_connect(&addr, timeout, attempts) {
                // Send the element to the onward node that we connected to
                stream.write_all(&element)?;
                log::info!("Sent: {}", std::str::from_utf8(&element).unwrap());
                stream.shutdown(Shutdown::Both)?;
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

/// The expected format for an ObjectId in bytes.
type ObjectId = [u8; 24];

/// Represents the shared structure both threads have access to.
#[derive(Debug, Default)]
struct Inner {
    /// The internal queue of ObjectIds
    queue: Mutex<VecDeque<ObjectId>>,
    /// The semaphore for alerting threads
    available: Condvar,
}

impl Inner {
    pub fn with_state(initial: VecDeque<ObjectId>) -> Self {
        log::info!("Initialising the queue with {} elements", initial.len());

        Self {
            queue: Mutex::new(initial),
            available: Default::default(),
        }
    }
}

fn get_job_queue() -> mongodb::error::Result<VecDeque<ObjectId>> {
    // Setup the MongoDB client
    let uri = std::env::var("CONN_STR").unwrap();
    let client = block_on(async { mongodb::Client::with_uri_str(&uri).await })?;

    // Get the jobs collection
    let database = client.database("sybl");
    let jobs = database.collection("jobs");

    // Pull all the jobs and deserialize them
    let cursor = block_on(async { jobs.find(None, None).await })?;

    let queue = block_on(async {
        cursor
            .filter_map(Result::ok)
            .filter_map(|x| mongodb::bson::de::from_document::<Job>(x).ok())
            .map(|job| {
                let mut bytes = [0_u8; 24];
                bytes.copy_from_slice(&job.dataset_id.to_hex().as_bytes()[..]);
                bytes
            })
            .collect()
            .await
    });

    Ok(queue)
}

fn main() -> std::io::Result<()> {
    // Setup logging
    setup_logger("interface");

    // Load the configuration variables
    let environment = if cfg!(debug_assertions) {
        Environment::Development
    } else {
        Environment::Production
    };

    config::load(environment);

    let state = get_job_queue().expect("Failed to get the job queue");
    let inner = Arc::new(Inner::with_state(state));

    log::info!("Beginning the thread execution");
    crossbeam::scope(|s| {
        s.spawn(|_| listen(&Arc::clone(&inner)));
        s.spawn(|_| receive(&Arc::clone(&inner)));
    })
    .unwrap();

    Ok(())
}
