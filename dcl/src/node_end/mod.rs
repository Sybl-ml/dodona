use anyhow::{anyhow, Result};
use mongodb::bson::oid::ObjectId;
use mongodb::Database;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str::from_utf8;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct Server {
    conn: Arc<RwLock<TcpStream>>,
    api_key: String,
}

// Server Methods
impl Server {
    pub fn new(conn: TcpStream, api_key: String) -> Server {
        Server {
            conn: Arc::new(RwLock::new(conn)),
            api_key,
        }
    }

    pub fn get_tcp(&self) -> Arc<RwLock<TcpStream>> {
        self.conn.clone()
    }

    pub fn get_api_key(&self) -> &String {
        &self.api_key
    }
}

#[derive(Deserialize, Debug, Copy, Clone)]
pub struct ServerInfo {
    alive: bool,
    using: bool,
}

impl ServerInfo {
    pub fn new() -> ServerInfo {
        ServerInfo {
            alive: true,
            using: false,
        }
    }
    pub fn get_using(&self) -> bool {
        self.using
    }

    pub fn set_using(&mut self, using: bool) {
        self.using = using
    }

    pub fn get_alive(&self) -> bool {
        self.alive
    }

    pub fn set_alive(&mut self, alive: bool) {
        self.alive = alive
    }
}

#[derive(Debug)]
pub struct ServerPool {
    servers: RwLock<HashMap<ObjectId, Server>>,
    info: RwLock<HashMap<ObjectId, ServerInfo>>,
}
// ServerPool Methods
impl ServerPool {
    pub fn new() -> ServerPool {
        ServerPool {
            servers: RwLock::new(HashMap::new()),
            info: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add(&self, server: Server) {
        let oid: ObjectId = ObjectId::new();
        let mut server_vec = self.servers.write().await;
        let mut info_vec = self.info.write().await;
        server_vec.insert(oid.clone(), server);
        info_vec.insert(oid.clone(), ServerInfo::new());
    }

    pub async fn get(&self) -> Option<(ObjectId, Arc<RwLock<TcpStream>>)> {
        let servers_read = self.servers.read().await;
        let mut info_write = self.info.write().await;
        for (key, info) in info_write.iter_mut() {
            if info.get_alive() && !info.get_using() {
                info.set_using(true);
                return Some((key.clone(), servers_read.get(key).unwrap().get_tcp()));
            }
        }
        return None;
    }

    pub async fn end(&self, key: ObjectId) {
        let mut info_write = self.info.write().await;
        info_write.get_mut(&key).unwrap().set_using(false);
    }
}

// Run node_end method
pub async fn run(serverpool: Arc<ServerPool>, socket: u16, db_conn: Arc<Database>) -> Result<()> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), socket);
    let mut listener = TcpListener::bind(&socket).await?;
    log::info!("RUNNING NODE SERVER");

    while let Ok((inbound, _)) = listener.accept().await {
        log::info!("NODE CONNECTION");
        let db_conn_clone = db_conn.clone();
        let sp_clone = serverpool.clone();
        tokio::spawn(async move {
            process_connection(inbound, db_conn_clone, sp_clone)
                .await
                .unwrap();
        });
    }
    Ok(())
}

async fn process_connection(
    mut stream: TcpStream,
    db_conn: Arc<Database>,
    serverpool: Arc<ServerPool>,
) -> Result<()> {
    log::info!("PROCESSING");
    let mut buffer: [u8; 24] = [0_u8; 24];
    stream.read(&mut buffer).await?;
    let api_key = from_utf8(&buffer).unwrap();
    log::info!("API KEY: {}", api_key);
    if !check_api_key(&db_conn, &api_key) {
        log::info!("API KEY doesn't match a user");
        return Err(anyhow!("API Key is not valid"));
    }
    let server = Server::new(stream, String::from(api_key));

    serverpool.add(server).await;
    log::info!("PROCESSED");
    Ok(())
}

fn check_api_key(_db_conn: &Database, _api_key: &str) -> bool {
    true
}
