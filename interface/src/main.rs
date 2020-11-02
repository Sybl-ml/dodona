use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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
fn retry_until_connect(address: &SocketAddr, timeout: Duration) -> TcpStream {
    loop {
        if let Ok(stream) = TcpStream::connect(address) {
            break stream;
        }

        log::debug!("Failed to connect to: {}", address);
        log::debug!("Sleeping for: {:?}", timeout);
        std::thread::sleep(timeout);
    }
}

/// Receives messages from the frontend thread and communicates with the DCL.
fn receive(inner: &Arc<Inner>) -> std::io::Result<()> {
    let address = SocketAddr::from_str("127.0.0.1:6000").unwrap();
    let timeout = Duration::from_secs(5);

    loop {
        // Lock the queue so it cannot change
        let mut queue = inner.queue.lock().unwrap();

        // Try and send something onwards
        if let Some(element) = queue.pop_front() {
            log::info!("Sending: {}", std::str::from_utf8(&element).unwrap());

            let mut stream = retry_until_connect(&address, timeout);
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
