use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::RwLock;
use tokio::sync::mpsc::Receiver;
use anyhow::Result;

use crate::node_end::{ServerPool};

pub async fn run(serverpool: Arc<ServerPool>, mut rx: Receiver<String>) -> Result<()>{
    log::info!("RUNNING JOB END");
    while let Some(msg) = rx.recv().await {
        log::info!("Received: {}", msg);
        let dcn: Arc<RwLock<TcpStream>> = serverpool.get().await.unwrap();
        let mut dcn_write = dcn.write().await;
        dcn_write.write(msg.as_bytes()).await.unwrap();
    }
    Ok(())
}