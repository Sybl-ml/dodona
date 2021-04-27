//! Health checking functionality
//!
//! This will go through and will check each node to make sure each is alive and working. It will
//! update its status in the [`NodeInfo`] object for the node.

use std::sync::Arc;
use std::time::{Duration, SystemTime};

use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use tokio::net::TcpStream;
use tokio::sync::RwLock;
use tokio::{io::AsyncWriteExt, time::Instant};

use anyhow::Result;

use crate::node_end::NodePool;
use messages::{ClientMessage, WriteLengthPrefix};
use models::models::Status;

/// Runner for health checking
///
/// Runs the health checking framework to go through each node that is not currently being used and
/// makes sure it is still alive. This will be run every <delay> seconds.
pub async fn health_runner(database: Arc<Database>, nodepool: Arc<NodePool>, delay: u64) {
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

    let start_timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let message = ClientMessage::Alive {
        timestamp: start_timestamp,
    }
    .as_bytes();

    if stream.write(&message).await.is_err() {
        return false;
    }

    let mut buffer = [0_u8; 64];

    let start = Instant::now();
    let health_response = ClientMessage::read_until(&mut *stream, &mut buffer, |m| {
        matches!(m, ClientMessage::Alive { .. })
    })
    .await;
    let response_time = Instant::now() - start;

    let alive = match health_response {
        Ok(ClientMessage::Alive { timestamp }) => {
            (timestamp <= start_timestamp + 2) && (timestamp >= start_timestamp)
        }
        _ => false,
    };

    log::trace!("model_id={} took {:?} to respond", model_id, response_time);

    alive
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
