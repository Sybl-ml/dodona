use dcl::messages::Message;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::prelude::*;

use utils::read_stream;

mod common;

#[tokio::test]
async fn test_node_connect_and_hb() {
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
        let key = "507f1f77bcf86cd799439011";
        stream.write(key.as_bytes()).await.unwrap();
        loop {
            match read_stream(&mut stream, Duration::from_secs(5)).await {
                Ok(_) => {
                    stream
                        .write(Message::send(Message::Alive).as_bytes())
                        .await
                        .unwrap();
                }
                _ => (),
            };
        }
    });

    tokio::time::sleep(Duration::new(4, 0)).await;

    let nodes_read = nodepool.nodes.read().await;
    assert!(nodes_read.len() > 0);

    let info_read = nodepool.info.read().await;
    for (_, info) in info_read.iter() {
        assert!(info.alive);
    }
}

#[tokio::test]
async fn test_dcn_using() {
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
        let key = "507f1f77bcf86cd799439011";
        stream.write(key.as_bytes()).await.unwrap();
        let mut buffer = Vec::new();
        match stream.read(&mut buffer).await {
            Ok(_) => {
                stream
                    .write(Message::send(Message::Alive).as_bytes())
                    .await
                    .unwrap();
            }
            _ => (),
        };
    });
    // Node 2
    let sock_clone = socket.clone();
    tokio::spawn(async move {
        let mut stream = TcpStream::connect(sock_clone.to_string()).await.unwrap();
        let key = "507f1f77bcf86cd799439012";
        stream.write(key.as_bytes()).await.unwrap();
        let mut buffer = Vec::new();
        match stream.read(&mut buffer).await {
            Ok(_) => {
                stream
                    .write(Message::send(Message::Alive).as_bytes())
                    .await
                    .unwrap();
            }
            _ => (),
        };
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
