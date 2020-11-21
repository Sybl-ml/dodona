use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc::{self, Receiver, Sender};

mod common;

#[tokio::test]
async fn test_interface_end() {
    // Create user/project/dataset in database.
    let (database, params) = common::initialise_with_db().await;
    let database = Arc::new(database);
    let is_clone = params.interface_socket.clone();
    // Start up interface end
    let (tx, mut rx): (Sender<String>, Receiver<String>) = mpsc::channel(20);
    let db_clone = Arc::clone(&database);
    tokio::spawn(async move {
        dcl::interface_end::run(is_clone, db_clone, tx)
            .await
            .unwrap();
    });
    // Create fake interface client
    let mut stream = TcpStream::connect(is_clone.to_string()).await.unwrap();
    // write dataset id to interface end
    stream.write(common::DATASET_ID.as_bytes()).await.unwrap();
    tokio::time::sleep(Duration::new(3, 0)).await;
    // assert on receive end of mpsc
    if let Some(msg) = rx.recv().await {
        log::info!("Received: {}", &msg);
        assert_eq!(&msg, common::DATASET);
    }
}
