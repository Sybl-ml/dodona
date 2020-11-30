//! DCL functionality to allow DCNs to connect
//!
//! First place where a DCN will connect to where its connection
//! will be created with the DCL. Once the connection is formed
//! a Node object will be created which holds that TcpStream
//! for that Node. This allows the Job End to ask for a TcpStream
//! and receive one for a DCN.

use std::collections::HashMap;
use std::fmt::Display;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::str;
use std::sync::Arc;

use anyhow::Result;
use mongodb::bson::{bson, oid::ObjectId};
use serde::Serialize;
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::sync::RwLock;

use crate::messages::Message;

/// Defines information about a Node
#[derive(Debug)]
pub struct Node {
    conn: Arc<RwLock<TcpStream>>,
    api_key: String,
}

// Node Methods
impl Node {
    /// Creates a new Node object
    pub fn new(conn: TcpStream, api_key: impl Into<String>) -> Self {
        Self {
            conn: Arc::new(RwLock::new(conn)),
            api_key: api_key.into(),
        }
    }

    /// Gets TcpStream access
    ///
    /// Returns an Arc reference to a TcpStream. This is so
    /// access to the TcpStream can be acheived over multiple
    /// threads.
    pub fn get_tcp(&self) -> Arc<RwLock<TcpStream>> {
        Arc::clone(&self.conn)
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
    pub alive: bool,
    /// Flag to specify if Node is in use or not
    pub using: bool,
}

impl NodeInfo {
    /// creates new NodeInfo instance
    pub fn new() -> Self {
        Self {
            alive: true,
            using: false,
        }
    }
}

/// Struct holding all Compute Node connections and information about them
#[derive(Debug)]
pub struct NodePool {
    /// HashMap of Node objects with unique IDs
    pub nodes: RwLock<HashMap<ObjectId, Node>>,
    /// HashMap of NodeInfo objects with unique IDs
    pub info: RwLock<HashMap<ObjectId, NodeInfo>>,
}

impl NodePool {
    /// Returns a new NodePool instance
    pub fn new() -> Self {
        Self {
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
            if info.alive && !info.using {
                info.using = true;

                let key = key.clone();
                let stream = nodes_read.get(&key).unwrap().get_tcp();

                return Some((key, stream));
            }
        }

        None
    }

    /// Creates a cluster of `size` nodes to use
    ///
    /// It is given a cluster size and searches the nodepool
    /// for available clusters and builds the cluster as a hashmap.
    /// When the size is reached, the cluster is output. If it is empty
    /// then the None Option is returned. If it has nodes in it, but less
    /// than the size of the cluster, it is still returned.
    pub async fn get_cluster(
        &self,
        size: usize,
    ) -> Option<HashMap<ObjectId, Arc<RwLock<TcpStream>>>> {
        let nodes_read = self.nodes.read().await;
        let mut cluster: HashMap<ObjectId, Arc<RwLock<TcpStream>>> = HashMap::new();
        let mut info_write = self.info.write().await;

        for (key, info) in info_write.iter_mut() {
            if info.alive && !info.using {
                info.using = true;
                let stream = nodes_read.get(&key).unwrap().get_tcp();
                cluster.insert(key.clone(), stream);

                if cluster.len() == size {
                    return Some(cluster);
                }
            }
        }

        match cluster.len() {
            0 => None,
            _ => Some(cluster),
        }
    }

    /// Changes the `using` flag on a NodeInfo object
    ///
    /// When passed an ObjectId, this function will find the
    /// NodeInfo instance for that ID and will set its `using`
    /// flag to be false, signifying the end of its use.
    pub async fn end(&self, key: ObjectId) {
        let mut info_write = self.info.write().await;
        info_write.get_mut(&key).unwrap().using = false;
    }

    /// Updates a NodeInfo object
    ///
    /// Gets the correct NodeInfo struct and updates its alive
    /// field by inverting what it currently is.
    pub async fn update_node(&self, status: bool, oid: &ObjectId) {
        let mut info_write = self.info.write().await;
        let node_info = info_write.get_mut(&oid).unwrap();

        node_info.alive = status;
    }

    /// Checks if a node is being used
    ///
    /// Passed the ObjectId of a node and it checks if it
    /// is being used for a job, which implies it is alive.
    pub async fn is_using(&self, oid: &ObjectId) -> bool {
        let info_read = self.info.read().await;
        let node_info = info_read.get(oid).unwrap();

        node_info.using
    }
}

/// Run Node for DCNs to connect to
///
/// Starts up node end which allows DCNs to register their connection. This will create a
/// Node object if given a correct API Key. This allows the job end to connect and
/// communicate with the DCNs.
pub async fn run(nodepool: Arc<NodePool>, socket: u16) -> Result<()> {
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), socket);
    let listener = TcpListener::bind(&socket).await?;

    log::info!("RUNNING NODE END");

    while let Ok((inbound, _)) = listener.accept().await {
        log::info!("NODE CONNECTION");

        let sp_clone = Arc::clone(&nodepool);

        tokio::spawn(async move {
            process_connection(inbound, sp_clone).await.unwrap();
        });
    }

    Ok(())
}

async fn process_connection(mut stream: TcpStream, nodepool: Arc<NodePool>) -> Result<()> {
    log::info!("PROCESSING");

    let (email, model_name) = register_new_model(&mut stream).await?;
    authenticate_challenge_response(&mut stream, &email, &model_name).await?;
    let token = verify_access_token(&mut stream).await?;

    log::info!("Token: {}", token);

    let node = Node::new(stream, token);
    nodepool.add(node).await;

    log::info!("PROCESSED");

    Ok(())
}

/// Allows a user to register a new model through the DCL.
///
/// Expects the user to send a single request in JSON of the form `{"NewModel": {"model_name":
/// <name>, "email": <email>}}` and forwards the request to the API server, forwarding the response
/// back to the client. If the message is not a `NewModel` request, this will panic.
pub async fn register_new_model(stream: &mut TcpStream) -> Result<(String, String)> {
    let mut buffer = [0_u8; 1024];

    // Read the user's email and model name
    let size = stream.read(&mut buffer).await?;

    let message = Message::from_slice(&buffer[..size]);
    let (email, model_name) = match message {
        Message::NewModel { email, model_name } => (email, model_name),
        _ => unreachable!(),
    };

    log::info!("Setting up a new model '{}' for: {}", model_name, email);

    // Query the API server
    let body = bson!({
        "model_name": &model_name,
        "email": &email,
    });

    let text = get_response_text("http://localhost:3001/api/clients/m/new", body).await?;

    // Send the response back to the client
    stream.write(text.as_bytes()).await?;

    Ok((email, model_name))
}

/// Allows a user to respond to a challenge request through the DCL.
///
/// Expects the user to send a single request in JSON of the form `{"ChallengeResponse":
/// {"response": <response>,  "model_name": <name>, "email": <email>}}` and forwards the request to
/// the API server, forwarding the response back to the client. If the message is not a
/// `ChallengeResponse` request, this will panic.
pub async fn authenticate_challenge_response(
    stream: &mut TcpStream,
    email: &str,
    model_name: &str,
) -> Result<()> {
    let mut buffer = [0_u8; 1024];

    // Read the user's challenge response
    let size = stream.read(&mut buffer).await?;

    let message = Message::from_slice(&buffer[..size]);
    let response = match message {
        Message::ChallengeResponse { response, .. } => response,
        _ => unreachable!(),
    };

    log::info!("Sending challenge response: {}", response);

    // Query the API server
    let body = bson!({
        "model_name": &model_name,
        "email": &email,
        "challenge_response": &response,
    });

    let text = get_response_text("http://localhost:3001/api/clients/m/verify", body).await?;

    // Send the response back to the client
    stream.write(text.as_bytes()).await?;

    Ok(())
}

/// Reads an access token from the stream and checks it with the API server.
pub async fn verify_access_token(stream: &mut TcpStream) -> Result<String> {
    let mut buffer = [0_u8; 1024];

    // Read the user's challenge response
    let size = stream.read(&mut buffer).await?;

    let message = Message::from_slice(&buffer[..size]);
    let (id, token) = match message {
        Message::AccessToken { id, token } => (id, token),
        _ => unreachable!(),
    };

    // Query the API server
    let body = bson!({
        "id": &id,
        "token": &token,
    });

    let text = get_response_text("http://localhost:3001/api/clients/m/authenticate", body).await?;

    // Send the response back to the client
    stream.write(text.as_bytes()).await?;

    Ok(token)
}

/// Queries the API server and returns the response text.
pub async fn get_response_text<S: Display + Serialize>(url: &str, body: S) -> Result<String> {
    log::info!("Sending: {} to {}", &body, url);

    let text = reqwest::blocking::Client::new()
        .post(url)
        .json(&body)
        .send()?
        .text()?;

    log::info!("Response body: {:?}", text);

    Ok(text)
}
