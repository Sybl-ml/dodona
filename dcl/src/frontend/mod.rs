use std::sync::Arc;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use futures::future::try_join;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use anyhow::Result;
use std::str::from_utf8;

pub async fn run_server() -> Result<()>{
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
    let mut listener = TcpListener::bind(&socket).await?;

    while let Ok((inbound, _)) = listener.accept().await {

        tokio::spawn(async move {
            process_connection(inbound).await.unwrap();
        });
    }
    Ok(())
}

async fn process_connection(mut stream: TcpStream) -> Result<()> {
    let mut buffer: [u8; 128] = [0; 128]; 
    stream.read(&mut buffer).await?;
    log::info!("{}", from_utf8(&buffer).unwrap());
    Ok(())
}