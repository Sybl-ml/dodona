use std::sync::Arc;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use anyhow::Result;
use mongodb::Database;

use crate::models::datasets::Dataset;
use crate::utils;

pub struct Server{
    addr: SocketAddr,
    conn: TcpStream,
    api_key: String
}
// Server Methods
impl Server{
    pub fn new(addr: SocketAddr, conn: TcpStream, api_key: String) -> Server {
        Server {addr, conn, api_key}
    }

    pub fn get_addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn get_conn(&self) -> &TcpStream {
        &self.conn
    }

    pub fn get_api_key(&self) -> &String {
        &self.api_key
    }
}

pub struct ServerPool{
    servers: Vec<Arc<Server>>
}
// ServerPool Methods
impl ServerPool{
    pub fn add(&mut self, server: Server){
        self.servers.push(Arc::new(server));
    }
}

// Run node_end method
pub fn run_server(serverpool: &ServerPool, socker: u16, db_conn: Arc<Database>) -> Result<()>{
    Ok(())
}