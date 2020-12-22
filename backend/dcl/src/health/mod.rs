//! Health checking functionality
//!
//! This will go through and will check each node to make sure
//! each is alive and working. It will update its status in the
//! NodeInfo object for the node.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::RwLock;
use tokio::time::timeout;

use anyhow::Result;

use crate::messages::Message;
use crate::node_end::NodePool;

/// Runner for health checking
///
/// Runs the health checking framework to go through
/// each node that is not currently being used and makes
/// sure it is still alive. This will be run every <delay> seconds.
pub async fn health_runner(nodepool: Arc<NodePool>, delay: u64) {
    log::info!("HEALTH CHECKING UP");
    let mut interval = tokio::time::interval(Duration::from_secs(delay));

    loop {
        let np = Arc::clone(&nodepool);
        check_health(np).await.unwrap();
        log::info!("HEALTH CHECKED");
        interval.tick().await;
    }
}

/// Go through nodes and check if alive
///
/// Loops through all nodes and checks to see if they are alive.
/// This information is saved in NodeInfo.
pub async fn check_health(nodepool: Arc<NodePool>) -> Result<()> {
    let mut nodes = nodepool.nodes.write().await;
    let mut clean_list: Vec<String> = vec![];

    for (id, node) in nodes.iter() {
        if !nodepool.is_using(&id).await {
            let alive = heartbeat(node.get_tcp()).await;

            if !alive {
                log::warn!("Node: {} is presumed dead", node.get_model_id());
                node.inc_counter().await;

                if node.get_counter().await == 10 {
                    log::info!("Node: {} will be removed", node.get_model_id());

                    clean_list.push(id.clone());
                }
            } else if node.get_counter().await > 0 {
                node.reset_counter().await;
            }

            nodepool.update_node(&id, alive).await?;
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
pub async fn heartbeat(stream_lock: Arc<RwLock<TcpStream>>) -> bool {
    let mut stream = stream_lock.write().await;

    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let message = Message::Alive { timestamp }.as_bytes();

    if stream.write(&message).await.is_err() {
        return false;
    }

    let wait = Duration::from_millis(100);
    let mut buffer = [0_u8; 64];
    let future = stream.read(&mut buffer);

    timeout(wait, future).await.is_ok()
}

#[cfg(test)]
mod tests;
