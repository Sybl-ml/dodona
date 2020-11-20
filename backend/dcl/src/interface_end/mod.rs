//! Deals with DCL connection to the interface layer
//!
//! Listens to traffic over a socket and maintains a transmitter end of
//! a mpsc channel which allows it to send data to the job end.

use anyhow::Result;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Database;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::from_utf8;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::sync::mpsc::Sender;

use models::datasets::Dataset;

type OId = [u8; 24];
/// Starts up interface server
///
/// Takes in socket, db connection and transmitter end of mpsc chaneel and will
/// read in data from an interface. Messages read over this are taken and the
/// corresponding dataset is found and decompressed before being passed to the
/// job end to be sent to a compute node.
pub async fn run(socket: u16, db_conn: Arc<Database>, tx: Sender<String>) -> Result<()> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), socket);
    log::info!("Socket: {:?}", socket);

    let listener = TcpListener::bind(&socket).await?;
    log::info!("RUNNING INTERFACE SERVER");
    while let Ok((inbound, _)) = listener.accept().await {
        log::info!("INTERFACE CONNECTION");
        let db_conn_clone = db_conn.clone();
        let tx_clone = tx.clone();
        tokio::spawn(async move {
            process_connection(inbound, db_conn_clone, tx_clone)
                .await
                .unwrap();
        });
    }
    Ok(())
}

async fn process_connection(
    mut stream: TcpStream,
    db_conn: Arc<Database>,
    tx: Sender<String>,
) -> Result<()> {
    let mut buffer: OId = [0_u8; 24];
    stream.read(&mut buffer).await?;
    log::info!("{}", from_utf8(&buffer).unwrap());
    let datasets = db_conn.collection("datasets");
    let dataset_id = from_utf8(&buffer)?;
    let object_id = match ObjectId::with_string(&dataset_id) {
        Ok(id) => id,
        Err(_) => {
            log::error!("Bad dataset id received");
            return Ok(());
        }
    };
    let filter = doc! { "_id": object_id };
    log::info!("{:?}", &filter);
    let doc = datasets.find_one(filter, None).await?.unwrap();
    let dataset: Dataset = mongodb::bson::de::from_document(doc).unwrap();
    log::info!("{:?}", &dataset);
    let comp_data = dataset.dataset.unwrap().bytes;

    match utils::decompress_data(&comp_data) {
        Ok(decompressed) => {
            let decomp_data = std::str::from_utf8(&decompressed)?;
            log::info!("Decompressed data: {:?}", &decomp_data);
            tx.send(String::from(decomp_data)).await.unwrap();
        }
        Err(_) => {
            log::error!("Bad dataset id received");
            return Ok(());
        }
    };

    Ok(())
}
