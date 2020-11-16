use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc::Receiver;
use anyhow::Result;

use crate::node_end::{ServerPool, Server};

pub async fn run(serverpool: Arc<ServerPool>, mut rx: Receiver<String>) -> Result<()>{
    log::info!("RUNNING JOB END");
    while let Some(msg) = rx.recv().await {
        log::info!("Received: {}", msg);
        let dcn: Server = serverpool.get().await.unwrap();
        let mut stream = TcpStream::connect(dcn.get_addr()).await.unwrap();
        stream.write(msg.as_bytes()).await.unwrap();
    }
    Ok(())
}