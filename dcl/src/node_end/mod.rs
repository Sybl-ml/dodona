//! DCL functionality to allow DCNs to connect
//!
//! First place where a DCN will connect to where its connection
//! will be created with the DCL. Once the connection is formed
//! a Server object will be created which holds that TcpStream
//! for that Server. This allows the Job End to ask for a TcpStream
//! and receive one for a DCN.

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

/// Defines information about a server
#[derive(Debug)]
pub struct Server {
    conn: Arc<RwLock<TcpStream>>,
    api_key: String,
}

// Server Methods
impl Server {
    /// Creates a new server object
    pub fn new(conn: TcpStream, api_key: String) -> Server {
        Server {
            conn: Arc::new(RwLock::new(conn)),
            api_key,
        }
    }

    /// Gets TcpStream access
    ///
    /// Returns an Arc reference to a TcpStream. This is so
    /// access to the TcpStream can be acheived over multiple
    /// threads.
    pub fn get_tcp(&self) -> Arc<RwLock<TcpStream>> {
        self.conn.clone()
    }

    /// Getter for API key
    pub fn get_api_key(&self) -> &String {
        &self.api_key
    }
}

/// Information about a connected server
#[derive(Deserialize, Debug, Copy, Clone)]
pub struct ServerInfo {
    /// Flag to specify if server is alive or not
    alive: bool,
    /// Flag to specify if server is in use or not
    using: bool,
}

impl ServerInfo {
    /// creates new ServerInfo instance
    pub fn new() -> ServerInfo {
        ServerInfo {
            alive: true,
            using: false,
        }
    }

    /// Getter for `using` flag
    pub fn get_using(&self) -> bool {
        self.using
    }

    /// Setter for `using` flag
    pub fn set_using(&mut self, using: bool) {
        self.using = using
    }

    /// Getter for `alive` flag
    pub fn get_alive(&self) -> bool {
        self.alive
    }

    /// Setter for `alive` flag
    pub fn set_alive(&mut self, alive: bool) {
        self.alive = alive
    }
}

/// Struct holding all Compute Node connections and information about them
#[derive(Debug)]
pub struct ServerPool {
    /// HashMap of Server objects with unique IDs
    servers: RwLock<HashMap<ObjectId, Server>>,
    /// HashMap of ServerInfo objects with unique IDs
    info: RwLock<HashMap<ObjectId, ServerInfo>>,
}
// ServerPool Methods
impl ServerPool {
    /// Returns a new ServerPool instance
    pub fn new() -> ServerPool {
        ServerPool {
            servers: RwLock::new(HashMap::new()),
            info: RwLock::new(HashMap::new()),
        }
    }

    /// Adds new Server to ServerPool
    ///
    /// Function will take in a new Server and will create an ID
    /// for it. It will also create an associated ServerInfo instance
    /// to also be stored under the same ID. These are then stored in
    /// their respective HashMaps
    pub async fn add(&self, server: Server) {
        let oid: ObjectId = ObjectId::new();
        let mut server_vec = self.servers.write().await;
        let mut info_vec = self.info.write().await;
        server_vec.insert(oid.clone(), server);
        info_vec.insert(oid.clone(), ServerInfo::new());
    }

    /// Gets TcpStream reference and its ObjectId
    ///
    /// Function is used to choose the next Server to use. When this
    /// is found, the TcpStream is cloned and the `using` flag is set
    /// in the ServerInfo instance for that Server.
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

    /// Changes the `using` flag on a ServerInfo object
    ///
    /// When passed an ObjectId, this function will find the
    /// ServerInfo instance for that ID and will set its `using`
    /// flag to be false, signifying the end of its use.
    pub async fn end(&self, key: ObjectId) {
        let mut info_write = self.info.write().await;
        info_write.get_mut(&key).unwrap().set_using(false);
    }
}

/// Run server for DCNs to connect to
///
/// Starts up server which allows DCNs to register their connection. This will create a
/// Server object if given a correct API Key. This allows the job end to connect and
/// communicate with the DCNs.
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
