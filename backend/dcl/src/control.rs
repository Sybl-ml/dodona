//! Contains functions and types for running the control node.

use std::collections::HashSet;
use std::env;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Result};
use rand::Rng;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};

use messages::{ControlMessage, ReadLengthPrefix, WriteLengthPrefix};

/// The shared state across control node endpoints.
type ControlState = Arc<tokio::sync::RwLock<HashSet<u16>>>;

/// Gets the response of a child node and ensures it is correct.
///
/// Reads a singular message from the stream and checks that it is equal to the message we sent
/// previously, returning an error if not.
async fn get_child_health_response(
    stream: &mut TcpStream,
    buffer: &mut [u8],
    message: &ControlMessage,
) -> Result<()> {
    stream.write(&message.as_bytes()).await?;
    let read_message = ControlMessage::from_stream(stream, buffer).await?;

    // Received a message we didn't expect
    if read_message != *message {
        return Err(anyhow!("Child node may be dead"));
    }

    Ok(())
}

/// Runs the health checking between the control node and edge nodes.
///
/// Simply sends a message every `health_period` seconds and expects the edge node to respond with
/// the same message back. If it fails 10 times, the connection is over and the node is removed
/// from the pool.
async fn run_child_health_checking(
    port: u16,
    mut stream: TcpStream,
    state: ControlState,
    health_period: u64,
) -> Result<()> {
    log::info!("Inserting a child node with port={}", port);

    // Add the port to the state
    let mut lock = state.write().await;
    lock.insert(port);
    drop(lock);

    // Just keep heartbeating with the stream
    let period = Duration::from_secs(health_period);
    let mut interval = tokio::time::interval(period);

    // Allocate a buffer for messages we receive
    let mut buffer = [0_u8; 1024];

    // Count the number of failed heartbeats
    let mut failed_beats: usize = 0;

    loop {
        interval.tick().await;

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Send a heartbeat message
        let message = ControlMessage::Alive { timestamp };

        // Expect to receive the message back before sending the next one
        let future = get_child_health_response(&mut stream, &mut buffer, &message);
        let timeout = tokio::time::timeout(period, future).await;

        match timeout {
            Ok(Err(e)) => {
                log::warn!("Failed to contact child node: {}", e);
                failed_beats += 1;
            }
            Err(e) => {
                log::warn!("Child node timed out during health checking: {}", e);
                failed_beats += 1;
            }
            _ => failed_beats = 0,
        }

        if failed_beats == 10 {
            break;
        }
    }

    // Failed to heartbeat 10 times, assume dead
    let mut lock = state.write().await;
    lock.remove(&port);

    Ok(())
}

/// Responds with an available port for an edge node, if one exists.
///
/// `mallus` clients are expected to request an available port and receive a port back to connect
/// to. This will be one of the available edge nodes that is currently heartbeating.
async fn answer_port_queries(mut stream: TcpStream, state: ControlState) -> Result<()> {
    let map = state.read().await;

    // Return a randomly picked port that another node is listening on
    let port = if map.is_empty() {
        None
    } else {
        let index = rand::thread_rng().gen_range(0..map.len());
        Some(*map.iter().nth(index).unwrap())
    };

    // Drop the lock in case sending takes a while
    drop(map);

    // Send the port back to the client
    let message = ControlMessage::PortResponse { port };
    stream.write(&message.as_bytes()).await?;

    Ok(())
}

/// Handles an incoming TCP stream for the control node.
///
/// Reads the first message from the stream to determine the connection type before running the
/// appropriate handler. Client nodes are expected to request a port to connect to and edge nodes
/// are expected to register themselves as an edge node and begin heartbeating.
async fn handle_incoming_stream(
    port: u16,
    mut stream: TcpStream,
    state: ControlState,
    health_period: u64,
) -> Result<()> {
    let mut buffer = [0_u8; 256];

    log::trace!("Attempting to read a message from port={}", port);

    // Read the first message to see how to progress
    let message = ControlMessage::from_stream(&mut stream, &mut buffer).await?;

    log::debug!(
        "Received message={:?} from an incoming stream on port={}",
        message,
        port
    );

    match message {
        ControlMessage::PortRequest => answer_port_queries(stream, Arc::clone(&state)).await?,
        ControlMessage::ChildNodeRequest { port } => {
            run_child_health_checking(port, stream, Arc::clone(&state), health_period).await?
        }
        _ => unreachable!(),
    }

    Ok(())
}

/// Runs this instance of the DCL as the central control node.
///
/// Deals with incoming messages, either from `mallus` clients or other DCL edge nodes wishing to
/// begin receiving clients.
pub async fn run_as_controller() -> Result<()> {
    // Controller nodes bind to the `NODE_SOCKET` environment variable
    let port = u16::from_str(&env::var("NODE_SOCKET").expect("NODE_SOCKET must be set")).unwrap();
    let health_period = u64::from_str(&env::var("HEALTH").expect("HEALTH must be set")).unwrap();

    // Create some shared state
    let state: ControlState = ControlState::default();

    // Bind to the external socket in production mode
    #[cfg(not(debug_assertions))]
    let ip = Ipv4Addr::UNSPECIFIED;

    #[cfg(debug_assertions)]
    let ip = Ipv4Addr::LOCALHOST;

    let socket = SocketAddr::V4(SocketAddrV4::new(ip, port));
    let listener = TcpListener::bind(&socket).await?;

    log::info!(
        "Bound to socket={}, waiting for incoming connections to health check every {} seconds",
        socket,
        health_period
    );

    // Process incoming connections
    while let Ok((stream, addr)) = listener.accept().await {
        log::debug!("Processing a connection from addr={}", addr);

        let state_clone = Arc::clone(&state);

        // Spawn a new task to handle the request
        tokio::spawn(async move {
            handle_incoming_stream(addr.port(), stream, state_clone, health_period).await
        });
    }

    Ok(())
}
