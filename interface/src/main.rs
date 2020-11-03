use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const TIMEOUT_SECS: u64 = 1;

/// Listens for incoming messages from the API server and forwards them to the queue.
fn listen(inner: &Arc<Inner>) -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5000")?;
    let incoming = listener.incoming();

    for possible_stream in incoming {
        let mut stream = possible_stream?;
        let mut buffer = [0_u8; 24];
        stream.read_exact(&mut buffer)?;

        log::info!("Received: {}", std::str::from_utf8(&buffer).unwrap());

        let mut queue = inner.queue.lock().unwrap();
        queue.push_back(buffer);
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

    loop {
        // Lock the queue so it cannot change
        let mut queue = inner.queue.lock().unwrap();

        // Try and send something onwards
        if let Some(element) = queue.pop_front() {
            let mut stream = match try_to_connect(&address, timeout, attempts) {
                Some(stream) => stream,
                None => {
                    // Readd the element back to the queue at the front
                    queue.push_front(element);
                    // Manually release the mutex and wait before continuing
                    drop(queue);
                    std::thread::sleep(timeout);
                    continue;
                }
            };

            log::info!("Sending: {}", std::str::from_utf8(&element).unwrap());

            stream.write_all(&element)?;
            stream.shutdown(Shutdown::Both)?;
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
}

fn main() -> std::io::Result<()> {
    // Setup logging
    pretty_env_logger::init();

    let inner = Arc::new(Inner::default());

    log::info!("Beginning the thread execution");
    crossbeam::scope(|s| {
        s.spawn(|_| listen(&Arc::clone(&inner)));
        s.spawn(|_| receive(&Arc::clone(&inner)));
    })
    .unwrap();

    Ok(())
}
