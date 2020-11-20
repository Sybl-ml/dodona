//! DCL functionality to allow DCNs to connect
//!
//! First place where a DCN will connect to where its connection
//! will be created with the DCL. Once the connection is formed
//! a Node object will be created which holds that TcpStream
//! for that Node. This allows the Job End to ask for a TcpStream
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

/// Defines information about a Node
#[derive(Debug)]
pub struct Node {
    conn: Arc<RwLock<TcpStream>>,
    api_key: String,
}

// Node Methods
impl Node {
    /// Creates a new Node object
    pub fn new(conn: TcpStream, api_key: String) -> Node {
        Node {
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

/// Information about a connected Node
#[derive(Deserialize, Debug, Copy, Clone)]
pub struct NodeInfo {
    /// Flag to specify if Node is alive or not
    alive: bool,
    /// Flag to specify if Node is in use or not
    using: bool,
}

impl NodeInfo {
    /// creates new NodeInfo instance
    pub fn new() -> NodeInfo {
        NodeInfo {
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
pub struct NodePool {
    /// HashMap of Node objects with unique IDs
    pub nodes: RwLock<HashMap<ObjectId, Node>>,
    /// HashMap of NodeInfo objects with unique IDs
    info: RwLock<HashMap<ObjectId, NodeInfo>>,
}
// NodePool Methods
impl NodePool {
    /// Returns a new NodePool instance
    pub fn new() -> NodePool {
        NodePool {
            nodes: RwLock::new(HashMap::new()),
            info: RwLock::new(HashMap::new()),
        }
    }

    /// Adds new Node to NodePool
    ///
    /// Function will take in a new Node and will create an ID
    /// for it. It will also create an associated NodeInfo instance
    /// to also be stored under the same ID. These are then stored in
    /// their respective HashMaps
    pub async fn add(&self, node: Node) {
        let oid: ObjectId = ObjectId::new();
        let mut node_vec = self.nodes.write().await;
        let mut info_vec = self.info.write().await;
        node_vec.insert(oid.clone(), node);
        info_vec.insert(oid.clone(), NodeInfo::new());
    }

    /// Gets TcpStream reference and its ObjectId
    ///
    /// Function is used to choose the next Node to use. When this
    /// is found, the TcpStream is cloned and the `using` flag is set
    /// in the NodeInfo instance for that Node.
    pub async fn get(&self) -> Option<(ObjectId, Arc<RwLock<TcpStream>>)> {
        let nodes_read = self.nodes.read().await;
        let mut info_write = self.info.write().await;
        for (key, info) in info_write.iter_mut() {
            if info.get_alive() && !info.get_using() {
                info.set_using(true);
                return Some((key.clone(), nodes_read.get(key).unwrap().get_tcp()));
            }
        }
        return None;
    }

    /// Changes the `using` flag on a NodeInfo object
    ///
    /// When passed an ObjectId, this function will find the
    /// NodeInfo instance for that ID and will set its `using`
    /// flag to be false, signifying the end of its use.
    pub async fn end(&self, key: ObjectId) {
        let mut info_write = self.info.write().await;
        info_write.get_mut(&key).unwrap().set_using(false);
    }

    /// Updates a NodeInfo object
    ///
    /// Gets the correct NodeInfo struct and updates its alive
    /// field by inverting what it currently is.
    pub async fn update_node(&self, status: bool, oid: &ObjectId) {
        let mut info_write = self.info.write().await;
        let node_info = info_write.get_mut(&oid).unwrap();
        node_info.set_alive(status);
    }

    /// Checks if a node is being used
    ///
    /// Passed the ObjectId of a node and it checks if it
    /// is being used for a job, which implies it is alive.
    pub async fn is_using(&self, oid: &ObjectId) -> bool {
        let info_read = self.info.read().await;
        let node_info = info_read.get(oid).unwrap();
        node_info.get_using()
    }
}

/// Run Node for DCNs to connect to
///
/// Starts up node end which allows DCNs to register their connection. This will create a
/// Node object if given a correct API Key. This allows the job end to connect and
/// communicate with the DCNs.
pub async fn run(nodepool: Arc<NodePool>, socket: u16, db_conn: Arc<Database>) -> Result<()> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), socket);
    let mut listener = TcpListener::bind(&socket).await?;
    log::info!("RUNNING NODE END");

    while let Ok((inbound, _)) = listener.accept().await {
        log::info!("NODE CONNECTION");
        let db_conn_clone = db_conn.clone();
        let sp_clone = nodepool.clone();
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
    nodepool: Arc<NodePool>,
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
    let node = Node::new(stream, String::from(api_key));

    nodepool.add(node).await;
    log::info!("PROCESSED");
    Ok(())
}

fn check_api_key(_db_conn: &Database, _api_key: &str) -> bool {
    true
}
