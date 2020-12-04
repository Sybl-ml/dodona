use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use mockito::mock;
use tokio::net::TcpStream;
use tokio::prelude::*;

use dcl::messages::Message;

mod common;

#[tokio::test]
async fn test_node_connect_and_hb() {
    // Setup the HTTP mocking
    let _m = mock("POST", "/api/clients/m/authenticate")
        .with_status(200)
        .with_body(r#"{"message": "Authentication successful"}"#)
        .create();

    let params = common::initialise();
    let nodepool = Arc::new(dcl::node_end::NodePool::new());

    let nodepool_clone = Arc::clone(&nodepool);
    let ns_clone = params.node_socket.clone();

    // Start up node end
    tokio::spawn(async move {
        dcl::node_end::run(nodepool_clone, ns_clone).await.unwrap();
    });

    // Start up health checker
    let nodepool_clone = Arc::clone(&nodepool);
    tokio::spawn(async move {
        dcl::health::health_runner(nodepool_clone, 3).await;
    });

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), params.node_socket);

    // Create dummy node
    tokio::spawn(async move {
        let mut stream = TcpStream::connect(socket.to_string()).await.unwrap();
        let key = Message::AccessToken {
            id: "507f1f77bcf86cd799439011".into(),
            token: "".into(),
        };
        stream.write(&key.as_bytes()).await.unwrap();

        let mut buffer = [0_u8; 64];

        loop {
            let size = stream.read(&mut buffer).await.unwrap();
            stream.write(&buffer[..size]).await.unwrap();
        }
    });

    tokio::time::sleep(Duration::new(4, 0)).await;

    let nodes = nodepool.nodes.read().await;
    assert!(nodes.len() > 0);

    let node_info = nodepool.info.read().await;
    for (_, info) in node_info.iter() {
        assert!(info.alive);
    }
}

#[tokio::test]
async fn test_dcn_using() {
    // Setup the HTTP mocking
    let _m = mock("POST", "/api/clients/m/authenticate")
        .with_status(200)
        .with_body(r#"{"message": "Authentication successful"}"#)
        .create();

    let params = common::initialise();
    let nodepool = Arc::new(dcl::node_end::NodePool::new());

    let nodepool_clone = Arc::clone(&nodepool);
    let ns_clone = params.node_socket.clone() + 2;

    // Start up node end
    tokio::spawn(async move {
        dcl::node_end::run(nodepool_clone, ns_clone).await.unwrap();
    });

    let socket = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        params.node_socket + 2,
    );

    let sock_clone = socket.clone();

    // Create dummy nodes
    // Node 1
    tokio::spawn(async move {
        let mut stream = TcpStream::connect(sock_clone.to_string()).await.unwrap();
        let mut buffer = Vec::new();

        let key = Message::AccessToken {
            id: "507f1f77bcf86cd799439011".into(),
            token: "".into(),
        };
        stream.write(&key.as_bytes()).await.unwrap();

        let size = stream.read(&mut buffer).await.unwrap();
        stream.write(&buffer[..size]).await.unwrap();
    });

    // Node 2
    let sock_clone = socket.clone();
    tokio::spawn(async move {
        let mut stream = TcpStream::connect(sock_clone.to_string()).await.unwrap();
        let mut buffer = Vec::new();

        let key = Message::AccessToken {
            id: "507f1f77bcf86cd799439011".into(),
            token: "".into(),
        };
        stream.write(&key.as_bytes()).await.unwrap();

        let size = stream.read(&mut buffer).await.unwrap();
        stream.write(&buffer[..size]).await.unwrap();
    });

    tokio::time::sleep(Duration::new(4, 0)).await;

    if let Some(map1) = nodepool.get_cluster(1).await {
        if let Some(map2) = nodepool.get_cluster(1).await {
            for key in map1.keys() {
                assert!(nodepool.is_using(&key).await);
                assert!(map2.get(&key).is_none());
            }
            for key in map2.keys() {
                assert!(nodepool.is_using(&key).await);
            }
        } else {
            unreachable!();
        }
    } else {
        unreachable!();
    }
}
