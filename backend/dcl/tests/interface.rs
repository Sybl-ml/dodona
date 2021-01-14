use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use mongodb::bson::oid::ObjectId;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

use messages::{InterfaceMessage, WriteLengthPrefix};

mod common;

#[tokio::test]
async fn test_interface_end() {
    // Create user/project/dataset in database.
    let (database, params) = common::initialise_with_db().await;

    let database = Arc::new(database);
    let is_clone = params.interface_socket.clone();

    // Start up interface end
    let (tx, mut rx) = mpsc::channel(20);
    let db_clone = Arc::clone(&database);

    tokio::spawn(async move {
        dcl::interface_end::run(is_clone, db_clone, tx)
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    // Create fake interface client
    let socket = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        params.interface_socket,
    );

    let mut stream = TcpStream::connect(socket.to_string()).await.unwrap();

    // write dataset id to interface end
    let config = InterfaceMessage::Config {
        id: ObjectId::with_string(common::DATASET_ID).unwrap(),
        timeout: 10,
        column_types: Vec::new(),
    };
    stream.write(&config.as_bytes()).await.unwrap();
    tokio::time::sleep(Duration::from_secs(1)).await;

    // assert on receive end of mpsc
    if let Some(msg) = rx.recv().await {
        println!("Received: {:?}", &msg.1);
        assert_eq!(&msg.1.train, common::DATASET);
    }
}
