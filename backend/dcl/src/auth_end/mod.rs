//!

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};

use crate::protocol;

///
pub async fn run(socket: u16) -> Result<()> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), socket);
    log::info!("Authentication Socket: {:?}", socket);
    let listener = TcpListener::bind(&socket).await?;

    while let Ok((inbound, _)) = listener.accept().await {
        log::info!("Auth Connection: {}", inbound.peer_addr()?);

        tokio::spawn(async move {
            process_connection(inbound).await.unwrap();
        });
    }

    Ok(())
}

///
async fn process_connection(mut stream: TcpStream) -> Result<()> {
    let mut handler = protocol::Handler::new(&mut stream);
    handler.get_access_token().await?;

    Ok(())
}
