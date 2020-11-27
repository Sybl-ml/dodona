//! Part of DCL that takes a DCN and a dataset and comunicates with node

use crate::messages::Message;
use anyhow::Result;
use mongodb::bson::oid::ObjectId;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc::Receiver;
use tokio::sync::RwLock;
use utils::read_stream;

use crate::node_end::NodePool;

/// Starts up and runs the job end
///
/// Takes in nodepool and mpsc receiver and will listen for incoming datasets.
/// When a dataset is received, a node will be selected from the nodepool and
/// the dataset will be written to that node. The node will then do computation
/// on that dataset and will read in information from comp node.
pub async fn run(
    nodepool: Arc<NodePool>,
    mut rx: Receiver<String>,
    job_timeout: u64,
) -> Result<()> {
    let timeout = Duration::from_secs(job_timeout);
    log::info!("RUNNING JOB END");

    while let Some(msg) = rx.recv().await {
        log::info!("Received: {}", &msg);

        let cluster = nodepool.get_cluster(1).await.unwrap();
        for (key, dcn) in cluster {
            let np_clone = Arc::clone(&nodepool);
            let msg_clone = msg.clone();
            tokio::spawn(async move {
                dcl_protcol(np_clone, timeout.clone(), key, dcn, msg_clone).await;
            });
        }
    }
    Ok(())
}

/// Function to execute DCL protocol
pub async fn dcl_protcol(
    nodepool: Arc<NodePool>,
    timeout: Duration,
    key: ObjectId,
    stream: Arc<RwLock<TcpStream>>,
    dataset: String,
) -> String {
    let mut dcn_stream = stream.write().await;
    // This is temporary, planning on creating seperate place for defining messages
    dcn_stream
        .write(Message::send(Message::JobConfig).as_bytes())
        .await
        .unwrap();
    let check_res: Vec<u8> = read_stream(&mut dcn_stream, timeout.clone()).await.unwrap();
    log::info!("Check Result: {}", from_utf8(&check_res).unwrap());
    dcn_stream.write(dataset.as_bytes()).await.unwrap();
    let dataset: Vec<u8> = read_stream(&mut dcn_stream, timeout.clone()).await.unwrap();
    log::info!("Computed Data: {}", from_utf8(&dataset).unwrap());
    nodepool.end(key).await;
    String::from(from_utf8(&dataset).unwrap())
}
