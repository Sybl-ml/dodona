//! Health checking functionality
//!
//! This will go through and will check each node to make sure each is alive and working. It will
//! update its status in the [`NodeInfo`] object for the node.

use std::net::{Ipv4Addr, SocketAddrV4};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::time::timeout;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    time::Instant,
};

use anyhow::Result;

use crate::node_end::NodePool;
use messages::{ClientMessage, ControlMessage, ReadLengthPrefix, WriteLengthPrefix};
use models::models::Status;

/// Runs the health checking with the control node, assuming this is an edge node.
///
/// Connects to the control node and sends it some information about its location, before expecting
/// heartbeat messages and echoing them back when it receives them.
async fn health_check_with_control_node(node_port: u16, control_port: u16) -> Result<()> {
    // Connect to the control node
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, control_port);
    let mut stream = TcpStream::connect(socket).await?;

    // Send a message identifying where we can be found
    let message = ControlMessage::ChildNodeRequest { port: node_port };
    stream.write(&message.as_bytes()).await?;

    // Begin heartbeating
    let mut buffer = [0_u8; 1024];

    loop {
        // Read a message from the control node
        let message = ControlMessage::from_stream(&mut stream, &mut buffer).await?;

        // Send it back
        stream.write(&message.as_bytes()).await?;
    }
}

/// Entry point for various health checking protocols.
///
/// Runs the health checking framework to go through each node that is not currently being used and
/// makes sure it is still alive. This will be run every <delay> seconds. Additionally runs the
/// health checking between the current edge node and the control node itself.
pub async fn health_runner(
    database: Arc<Database>,
    nodepool: Arc<NodePool>,
    node_port: u16,
    control_port: u16,
    delay: u64,
) {
    // Spawn a task to health check with the control node
    tokio::spawn(async move {
        if let Err(e) = health_check_with_control_node(node_port, control_port).await {
            log::error!("Error when health checking with the control node: {}", e);
        }
    });

    let duration = Duration::from_secs(delay);
    log::info!("Running health checking with a delay of {:?}", duration);

    let mut interval = tokio::time::interval(duration);

    loop {
        let np = Arc::clone(&nodepool);
        check_health(Arc::clone(&database), np).await.unwrap();

        interval.tick().await;
    }
}

/// Go through nodes and check if alive
///
/// Loops through all nodes and checks to see if they are alive.  This information is saved [`in`]
/// [`NodeInfo`].
pub async fn check_health(database: Arc<Database>, nodepool: Arc<NodePool>) -> Result<()> {
    let mut nodes = nodepool.nodes.write().await;
    let mut clean_list: Vec<String> = Vec::new();

    for (id, node) in nodes.iter() {
        if !nodepool.is_using(&id).await {
            let alive = heartbeat(&id, node.get_tcp()).await;

            if !alive {
                log::trace!("Node with id={} failed to respond", node.get_model_id());
                node.inc_counter().await;

                if node.get_counter().await == 10 {
                    log::warn!("Node with id={} is assumed to be dead", node.get_model_id());

                    change_model_status(
                        Arc::clone(&database),
                        node.get_model_id(),
                        Status::Stopped,
                    )
                    .await?;

                    clean_list.push(id.clone());
                }
            } else if node.get_counter().await > 0 {
                node.reset_counter().await;
            }

            nodepool.update_node_alive(&id, alive).await;
        }
    }

    // clean dead nodes from nodepool
    for id in clean_list {
        nodes.remove(&id);
    }

    Ok(())
}

/// Checks to see if a Node is still alive
///
/// Checks to see if a node is still alive by sending it a
/// small bit of JSON and it waits for its response. If it fails
/// then it is treated as dead. If not then it is treated as alive.
pub async fn heartbeat(model_id: &str, stream_lock: Arc<RwLock<TcpStream>>) -> bool {
    let mut stream = stream_lock.write().await;

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let message = ClientMessage::Alive { timestamp }.as_bytes();

    if stream.write(&message).await.is_err() {
        return false;
    }

    let wait = Duration::from_millis(100);
    let mut buffer = [0_u8; 64];
    let future = stream.read(&mut buffer);

    let start = Instant::now();
    let response = timeout(wait, future).await;
    let response_time = Instant::now() - start;

    log::trace!(
        "model_id={} took {:?} to respond, given {:?}",
        model_id,
        response_time,
        wait
    );

    response.is_ok()
}

///
pub async fn change_model_status(
    database: Arc<Database>,
    model_id: &str,
    status: Status,
) -> Result<()> {
    let models = database.collection("models");

    log::debug!(
        "Updating the status of model with id={} to status={:?}",
        model_id,
        status
    );

    let query = doc! {"_id": ObjectId::with_string(model_id).unwrap()};
    let update = doc! {"$set": {"status": status}};
    models.update_one(query, update, None).await?;

    Ok(())
}

#[cfg(test)]
mod tests;
