use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use utils::setup_logger;

const TIMEOUT_SECS: u64 = 1;
const LISTEN_ADDR: &str = "127.0.0.1:5000";

/// Listens for incoming messages from the API server and forwards them to the queue.
fn listen(inner: &Arc<Inner>) -> std::io::Result<()> {
    let listener = TcpListener::bind(LISTEN_ADDR)?;
    let incoming = listener.incoming();

    log::info!("Listening for connections on: {}", LISTEN_ADDR);

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
fn try_to_connect(address: &SocketAddr, timeout: Duration, attempts: usize) -> Option<TcpStream> {
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
    let address = SocketAddr::from_str("127.0.0.1:6000").unwrap();
    let timeout = Duration::from_secs(TIMEOUT_SECS);
    let attempts = 3;

    // Lock the queue so it cannot change
    let mut queue = inner.queue.lock().unwrap();

    loop {
        // Try and send something onwards
        if let Some(element) = queue.pop_front() {
            if let Some(mut stream) = try_to_connect(&address, timeout, attempts) {
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

fn main() -> std::io::Result<()> {
    // Setup logging
    setup_logger("interface");

    let inner = Arc::new(Inner::default());

    log::info!("Beginning the thread execution");
    crossbeam::scope(|s| {
        s.spawn(|_| listen(&Arc::clone(&inner)));
        s.spawn(|_| receive(&Arc::clone(&inner)));
    })
    .unwrap();

    Ok(())
}
