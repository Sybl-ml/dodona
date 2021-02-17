use super::*;

use std::error::Error;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;

use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn test_heartbeat() -> Result<(), Box<dyn Error>> {
    // Bind to a random unused TCP port
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(socket).await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((mut inbound, _)) = listener.accept().await {
            let mut buffer = [0_u8; 24];
            inbound.read(&mut buffer).await.unwrap();
        }
    });

    tokio::time::sleep(Duration::from_millis(1)).await;

    let stream = TcpStream::connect(addr).await.unwrap();
    let verdict = heartbeat(Arc::new(RwLock::new(stream))).await;

    assert_eq!(verdict, true);

    Ok(())
}

#[tokio::test]
async fn test_heartbeat_fail() -> Result<(), Box<dyn Error>> {
    // Bind to a random unused TCP port
    let socket = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0);
    let listener = TcpListener::bind(socket).await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        while let Ok((mut inbound, _)) = listener.accept().await {
            let mut buffer = [0_u8; 24];
            inbound.read(&mut buffer).await.unwrap();
            tokio::time::sleep(Duration::from_millis(200)).await;
            inbound.shutdown().await.unwrap();
        }
    });

    tokio::time::sleep(Duration::from_millis(1)).await;
    let stream = TcpStream::connect(addr).await.unwrap();

    tokio::time::sleep(Duration::from_millis(3)).await;
    let verdict = heartbeat(Arc::new(RwLock::new(stream))).await;

    assert_eq!(verdict, false);

    Ok(())
}
