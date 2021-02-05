//! Deals with DCL connection to the interface layer
//!
//! Listens to traffic over a socket and maintains a transmitter end of
//! a mpsc channel which allows it to send data to the job end.

use anyhow::Result;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Database;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Sender;
use utils::compress::decompress_data;

use crate::DatasetPair;
use messages::{ClientMessage, InterfaceMessage, ReadLengthPrefix};
use models::datasets::Dataset;

/// Starts up interface server
///
/// Takes in socket, db connection and transmitter end of mpsc chaneel and will
/// read in data from an interface. Messages read over this are taken and the
/// corresponding dataset is found and decompressed before being passed to the
/// job end to be sent to a compute node.
pub async fn run(
    socket: u16,
    db_conn: Arc<Database>,
    tx: Sender<(ObjectId, DatasetPair, ClientMessage)>,
) -> Result<()> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), socket);
    log::info!("Interface Socket: {:?}", socket);
    let listener = TcpListener::bind(&socket).await?;

    while let Ok((inbound, _)) = listener.accept().await {
        log::info!("Interface Connection: {}", inbound.peer_addr()?);

        let db_conn_clone = Arc::clone(&db_conn);
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
    tx: Sender<(ObjectId, DatasetPair, ClientMessage)>,
) -> Result<()> {
    let mut buffer = [0_u8; 4096];
    let (object_id, timeout, column_types, prediction_column, prediction_type) =
        match InterfaceMessage::from_stream(&mut stream, &mut buffer).await? {
            InterfaceMessage::Config {
                id,
                timeout,
                column_types,
                prediction_column,
                prediction_type,
            } => (
                id,
                timeout,
                column_types,
                prediction_column,
                prediction_type,
            ),
        };

    log::info!("Received a message from the interface:");
    log::debug!("\tIdentifier: {}", object_id);
    log::debug!("\tTimeout: {}", timeout);
    log::debug!("\tColumn types: {:?}", column_types);

    let datasets = db_conn.collection("datasets");

    let filter = doc! { "_id": object_id };
    log::debug!("Finding datasets with filter: {:?}", &filter);

    let doc = datasets
        .find_one(filter, None)
        .await?
        .expect("Failed to find a document with the previous filter");
    let dataset: Dataset = mongodb::bson::de::from_document(doc)?;

    log::debug!("{:?}", &dataset);

    // Get the data from the struct
    let comp_train = dataset.dataset.unwrap().bytes;
    let comp_predict = dataset.predict.unwrap().bytes;

    // Decompress it
    let train_bytes = decompress_data(&comp_train)?;
    let predict_bytes = decompress_data(&comp_predict)?;

    // Convert it to a string
    let train = std::str::from_utf8(&train_bytes)?.to_string();
    let predict = std::str::from_utf8(&predict_bytes)?.to_string();

    log::debug!("Decompressed train: {:?}", &train);
    log::debug!("Decompressed predict: {:?}", &predict);

    tx.send((
        dataset.project_id,
        DatasetPair { train, predict },
        ClientMessage::JobConfig {
            timeout,
            column_types,
            prediction_column,
            prediction_type,
        },
    ))
    .await
    .unwrap_or_else(|error| log::error!("Error while sending over MPSC: {}", error));

    Ok(())
}
