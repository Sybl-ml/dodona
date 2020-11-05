use std::env;
use std::sync::Arc;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::from_utf8;
use std::str::FromStr;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use anyhow::Result;
use mongodb::options::ClientOptions;
use mongodb::{Database, Client};
use mongodb::bson::{Binary, doc, document::Document, oid::ObjectId};

use crate::models::datasets::Dataset;

type OId = [u8; 24];

pub async fn run_server() -> Result<()>{
    let conn_str = env::var("CONN_STR").expect("CONN_STR must be set");
    let app_name = env::var("APP_NAME").expect("APP_NAME must be set");
    let socket = env::var("SOCKET").expect("SOCKET must be set");

    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), u16::from_str(&socket).unwrap());

    let mut listener = TcpListener::bind(&socket).await?;
    let mut client_options = ClientOptions::parse(&conn_str).await.unwrap();
    client_options.app_name = Some(app_name);
    let client = Arc::new(Client::with_options(client_options).unwrap().database("sybl"));

    while let Ok((inbound, _)) = listener.accept().await {

        let db_conn = client.clone();
        tokio::spawn(async move {
            process_connection(inbound, db_conn).await.unwrap();
        });
    }
    Ok(())
}

async fn process_connection(mut stream: TcpStream, db_conn: Arc<Database>) -> Result<()> {
    let mut buffer: OId = [0_u8; 24]; 
    stream.read(&mut buffer).await?;
    log::info!("{}", from_utf8(&buffer).unwrap());
    let datasets = db_conn.collection("datasets");
    let dataset_id = from_utf8(&buffer)?;
    let object_id = match ObjectId::with_string(&dataset_id) {
        Ok(id) => id,
        Err(_) => {log::error!("Bad dataset id received"); return Ok(())},
    };
    let filter = doc! { "_id": object_id };
    log::info!("{:?}", &filter);
    let doc = datasets.find_one(filter, None).await?.unwrap();
    let dataset: Dataset = mongodb::bson::de::from_document(doc).unwrap();
    log::info!("{:?}", dataset);
    Ok(())
}