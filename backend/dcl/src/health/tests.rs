use super::*;
use std::net::Shutdown;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};

#[tokio::test]
async fn test_heartbeat() {
    tokio::spawn(async move {
        let socket: SocketAddr = "127.0.0.1:5002".parse().unwrap();
        let listener = TcpListener::bind(&socket).await.unwrap();

        while let Ok((mut inbound, _)) = listener.accept().await {
            let mut buffer = [0_u8; 24];
            inbound.read(&mut buffer).await.unwrap();
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let stream = TcpStream::connect("127.0.0.1:5002").await.unwrap();
    let verdict = heartbeat(Arc::new(RwLock::new(stream))).await;

    assert_eq!(verdict, true);
}

#[tokio::test]
async fn test_heartbeat_fail() {
    tokio::spawn(async move {
        let socket: SocketAddr = "127.0.0.1:5003".parse().unwrap();
        let listener = TcpListener::bind(&socket).await.unwrap();

        while let Ok((mut inbound, _)) = listener.accept().await {
            let mut buffer = [0_u8; 24];
            inbound.read(&mut buffer).await.unwrap();
            tokio::time::sleep(Duration::new(1, 0)).await;
            inbound.shutdown(Shutdown::Both).unwrap();
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
    let stream = TcpStream::connect("127.0.0.1:5003").await.unwrap();

    tokio::time::sleep(Duration::from_millis(300)).await;
    let verdict = heartbeat(Arc::new(RwLock::new(stream))).await;

    assert_eq!(verdict, false);
}
