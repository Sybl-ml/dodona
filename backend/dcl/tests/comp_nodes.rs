use dcl::messages::heartbeat_msg;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::prelude::*;

mod common;

#[tokio::test]
async fn test_node_connect_and_hb() {
    let params = common::initialise();
    let client = mongodb::Client::with_uri_str(&params.conn_str)
        .await
        .unwrap();
    let client = Arc::new(client.database("sybl"));
    let nodepool = Arc::new(dcl::node_end::NodePool::new());

    let db_conn_node = Arc::clone(&client);
    let nodepool_clone = Arc::clone(&nodepool);
    let ns_clone = params.node_socket.clone();
    // Start up node end
    tokio::spawn(async move {
        dcl::node_end::run(nodepool_clone, ns_clone, db_conn_node)
            .await
            .unwrap();
    });

    // Start up health checker
    let nodepool_clone = Arc::clone(&nodepool);
    tokio::spawn(async move {
        dcl::health::health_runner(nodepool_clone, 5).await;
    });

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), params.node_socket);

    // Create dummy node
    tokio::spawn(async move {
        let mut stream = TcpStream::connect(socket.to_string()).await.unwrap();
        let key = "507f1f77bcf86cd799439011";
        stream.write(key.as_bytes()).await.unwrap();
        let mut buffer = Vec::new();
        match stream.read(&mut buffer).await {
            Ok(_) => {
                stream.write(heartbeat_msg().as_bytes()).await.unwrap();
            }
            _ => (),
        };
    });

    tokio::time::sleep(Duration::new(3, 0)).await;

    let nodes_read = nodepool.nodes.read().await;
    assert!(nodes_read.len() > 0);

    let info_read = nodepool.info.read().await;
    for (_, info) in info_read.iter() {
        assert_eq!(info.get_alive(), true);
    }
}

#[tokio::test]
async fn test_dcn_using() {
    let params = common::initialise();
    let client = mongodb::Client::with_uri_str(&params.conn_str)
        .await
        .unwrap();
    let client = Arc::new(client.database("sybl"));
    let nodepool = Arc::new(dcl::node_end::NodePool::new());

    let db_conn_node = Arc::clone(&client);
    let nodepool_clone = Arc::clone(&nodepool);
    let ns_clone = params.node_socket.clone() + 2;
    // Start up node end
    tokio::spawn(async move {
        dcl::node_end::run(nodepool_clone, ns_clone, db_conn_node)
            .await
            .unwrap();
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
                stream.write(heartbeat_msg().as_bytes()).await.unwrap();
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
                stream.write(heartbeat_msg().as_bytes()).await.unwrap();
            }
            _ => (),
        };
    });
    tokio::time::sleep(Duration::new(4, 0)).await;
    if let Some((oid1, _)) = nodepool.get().await {
        assert!(nodepool.is_using(&oid1).await);
        if let Some((oid2, _)) = nodepool.get().await {
            assert_ne!(oid1, oid2);
            assert!(nodepool.is_using(&oid2).await);
        } else {
            unreachable!();
        }
    } else {
        unreachable!();
    }
}
