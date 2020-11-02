use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Listens for incoming messages from the API server and forwards them to the queue.
fn listen(inner: Arc<Inner>) -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5000")?;
    let mut incoming = listener.incoming();

    while let Some(incoming) = incoming.next() {
        let mut stream = incoming?;
        let mut buffer = [0_u8; 24];
        stream.read(&mut buffer)?;

        let mut queue = inner.queue.lock().unwrap();
        queue.push_back(buffer);
    }

    Ok(())
}

/// Continually tries to connect until a connection is achieved.
fn retry_until_connect(address: &SocketAddr, timeout: Duration) -> TcpStream {
    loop {
        if let Ok(stream) = TcpStream::connect_timeout(address, timeout) {
            return stream;
        }
    }
}

/// Receives messages from the frontend thread and communicates with the DCL.
fn receive(inner: Arc<Inner>) -> std::io::Result<()> {
    let address = SocketAddr::from_str("127.0.0.1:6000").unwrap();
    let timeout = Duration::from_secs(1);

    loop {
        // Lock the queue so it cannot change
        let mut queue = inner.queue.lock().unwrap();

        // Try and send something onwards
        if let Some(element) = queue.pop_front() {
            let mut stream = retry_until_connect(&address, timeout);
            stream.write(&element)?;
            stream.shutdown(Shutdown::Both)?;
        }
    }
}

type ObjectId = [u8; 24];
type Queue = VecDeque<ObjectId>;

#[derive(Debug, Default)]
struct Inner {
    queue: Mutex<Queue>,
}

fn main() -> std::io::Result<()> {
    let inner = Arc::new(Inner::default());

    crossbeam::scope(|s| {
        s.spawn(|_| listen(Arc::clone(&inner)));
        s.spawn(|_| receive(Arc::clone(&inner)));
    })
    .unwrap();

    Ok(())
}
