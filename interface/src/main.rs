use std::collections::VecDeque;
use std::io::Read;
use std::net::TcpListener;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// Listens for incoming messages from the API server and forwards them to the queue.
fn listen(sender: mpsc::Sender<[u8; 24]>) -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:5000")?;
    let mut incoming = listener.incoming();

    while let Some(incoming) = incoming.next() {
        let mut stream = incoming?;
        let mut buffer = [0_u8; 24];
        stream.read(&mut buffer)?;
        sender.send(buffer).expect("Failed to send a message");
    }

    Ok(())
}

/// Receives messages from the frontend thread and communicates with the DCL.
fn receive(receiver: mpsc::Receiver<[u8; 24]>) -> std::io::Result<()> {
    let mut queue = VecDeque::new();

    loop {
        let buffer = receiver.recv().expect("Failed to receive a message");
        let identifier = std::str::from_utf8(&buffer).expect("Invalid ObjectId");
        queue.push_back(String::from(identifier));
    }
}

type ObjectId = [u8; 24];
type Queue = VecDeque<ObjectId>;

struct InterfaceLayer {
    queue: Arc<Mutex<Queue>>,
}

fn main() -> std::io::Result<()> {
    // Create a channel for the threads to talk over
    let (s, r) = mpsc::channel::<[u8; 24]>();

    // Spawn 2 threads
    let mut threads = Vec::new();
    threads.push(thread::spawn(move || listen(s)));
    threads.push(thread::spawn(move || receive(r)));

    for thread in threads {
        thread.join().unwrap().unwrap();
    }

    Ok(())
}
