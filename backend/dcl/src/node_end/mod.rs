//! DCL functionality to allow DCNs to connect
//!
//! First place where a DCN will connect to where its connection will be created with the DCL. Once
//! the connection is formed a [`Node`] object will be created which holds that [`TcpStream`] for
//! that [`Node`]. This allows the Job End to ask for a [`TcpStream`] and receive one for a DCN.

use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::str;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use anyhow::Result;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Database,
};
use rand::Rng;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Notify, RwLock};

use messages::{ClientMessage, WriteLengthPrefix};
use models::models::Status;
use models::{job_performance::JobPerformance, jobs::JobConfiguration};

use crate::protocol;

/// Defines information about a Node
#[derive(Debug)]
pub struct Node {
    /// TCPStream for connection to node
    conn: Arc<RwLock<TcpStream>>,
    /// ID for associated model in database
    model_id: String,
    /// Counter used to determine if node is permanently dead
    pub counter: RwLock<u8>,
}

// Node Methods
impl Node {
    /// Creates a new Node object
    pub fn new(conn: TcpStream, model_id: impl Into<String>) -> Self {
        Self {
            conn: Arc::new(RwLock::new(conn)),
            model_id: model_id.into(),
            counter: RwLock::new(0),
        }
    }

    /// Gets [`TcpStream`] access
    ///
    /// Returns an Arc reference to a [`TcpStream`]. This is so access to the [`TcpStream`] can be
    /// acheived over multiple threads.
    pub fn get_tcp(&self) -> Arc<RwLock<TcpStream>> {
        Arc::clone(&self.conn)
    }

    /// Gets the model identifier for the node.
    pub fn get_model_id(&self) -> &String {
        &self.model_id
    }

    /// Increment the dead counter for node
    pub async fn inc_counter(&self) {
        let mut counter = self.counter.write().await;
        *counter += 1;
    }

    /// Reset the dead counter for node
    pub async fn reset_counter(&self) {
        let mut counter = self.counter.write().await;
        *counter = 0;
    }

    /// Get the value for dead counter for node
    pub async fn get_counter(&self) -> u8 {
        *self.counter.read().await
    }
}

/// Information about a connected [`Node`]
#[derive(Deserialize, Debug, Copy, Clone)]
pub struct NodeInfo {
    /// Flag to specify if [`Node`] is alive or not
    pub alive: bool,
    /// Flag to specify if [`Node`] is in use or not
    pub using: bool,
    /// Performance of the [`Node`] on previous jobs
    pub performance: f64,
}

impl NodeInfo {
    /// creates new [`NodeInfo`] instance
    pub fn new(performance: f64) -> Self {
        log::trace!("Creating a new `NodeInfo` with performance={}", performance);

        Self {
            alive: true,
            using: false,
            performance,
        }
    }

    /// gets the past 5 performances of node from DB
    /// and averages them out. To be used when a node
    /// is created.
    pub async fn from_database(database: Arc<Database>, model_id: &str) -> Result<NodeInfo> {
        let performances = JobPerformance::get_past_k(database, model_id, 5).await?;

        let mut perf = 0.0;

        if !performances.is_empty() {
            perf = performances.iter().sum::<f64>() / performances.len() as f64;
        }

        Ok(NodeInfo::new(perf))
    }
}

impl Default for NodeInfo {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// Struct holding all compute node connections and information about them
#[derive(Debug, Default)]
pub struct NodePool {
    /// [`HashMap`] of [`Node`] objects with unique IDs
    pub nodes: RwLock<HashMap<String, Node>>,
    /// [`HashMap`] of [`NodeInfo`] objects with unique IDs
    pub info: RwLock<HashMap<String, NodeInfo>>,
    /// Value to keep track of the number of active nodes in nodepool
    pub active: AtomicUsize,
    /// Notify struct for alerting changes to Job End
    pub job_notify: Arc<Notify>,
}

impl NodePool {
    /// Returns a [`NodePool`] instance
    pub fn new(job_notify: Arc<Notify>) -> Self {
        Self {
            nodes: RwLock::new(HashMap::new()),
            info: RwLock::new(HashMap::new()),
            active: AtomicUsize::new(0),
            job_notify,
        }
    }

    /// Adds new [`Node`] to [`NodePool`]
    ///
    /// Function will take in a new [`Node`] and will create an ID for it. It will also create an
    /// associated [`NodeInfo`] instance to also be stored under the same ID. These are then stored
    /// in their respective [`HashMap`]s
    pub async fn add(&self, node: Node, database: Arc<Database>) {
        let id = node.get_model_id().to_string();

        let mut node_map = self.nodes.write().await;
        let mut info_map = self.info.write().await;

        log::info!("Adding node to the pool with id={}", id);

        if let Some(node_info) = info_map.get(&id) {
            if !node_info.alive {
                self.active.fetch_add(1, Ordering::SeqCst);
            }
        } else {
            self.active.fetch_add(1, Ordering::SeqCst);
        };

        node_map.insert(id.clone(), node);
        info_map.insert(
            id.clone(),
            NodeInfo::from_database(database, &id).await.unwrap(),
        );

        self.job_notify.notify_waiters();
    }

    /// Gets [`TcpStream`] reference and its [`ObjectId`]
    ///
    /// Function is used to choose the next Node to use. When this is found, the [`TcpStream`] is
    /// cloned and the `using` flag is set in the [`NodeInfo`] instance for that Node.
    pub async fn get(&self) -> Option<(String, Arc<RwLock<TcpStream>>)> {
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

    /// Creates a cluster based on a JobConfig `config`
    ///
    /// It is given a cluster size and searches the nodepool for available clusters and builds the
    /// cluster as a hashmap.  When the size is reached, the cluster is output. If it is empty then
    /// the None Option is returned. If it has nodes in it, but less than the size of the cluster,
    /// it is still returned. This also uses the performance metric to build well performing clusters.
    pub async fn build_cluster(
        &self,
        config: JobConfiguration,
    ) -> Option<HashMap<String, Arc<RwLock<TcpStream>>>> {
        // Convert to usize as MongoDB stores as i32
        let cluster_size = config.cluster_size as usize;

        log::debug!("Attempting to build a cluster with size={}", cluster_size);

        let nodes_read = self.nodes.read().await;
        let mut accepted_job: Vec<(String, f64)> = Vec::new();
        let mut better_nodes: Vec<(String, f64)> = Vec::new();
        let mut info_write = self.info.write().await;

        let active = self.active.load(Ordering::SeqCst);

        if active < cluster_size {
            log::warn!(
                "Only {} nodes are active, so a cluster of size={} could not be built",
                active,
                cluster_size
            );

            return None;
        }

        // Ask all alive and free nodes if they want the job
        // If they do, their ID is added to accepted_job
        for (id, info) in info_write.iter_mut() {
            if info.alive && !info.using {
                info.using = true;
                let stream = nodes_read.get(id).unwrap().get_tcp();
                let config_response = NodePool::job_accepted(&stream, &config, &id).await;

                // Check all 3 possible states:
                //      - Explicit acceptance
                //      - Explicit rejection
                //      - An error in the stream itself
                match config_response {
                    Ok(true) => {
                        log::info!("Node with id={} accepted the job", id);
                        accepted_job.push((id.clone(), info.performance));

                        if info.performance > 0.5 {
                            better_nodes.push((id.clone(), info.performance));
                        }
                        self.active.fetch_sub(1, Ordering::SeqCst);

                        continue;
                    }
                    Ok(false) => {
                        log::debug!("model_id={} explicitly rejected the job", id);
                    }
                    Err(e) => {
                        log::warn!("Error occurred asking model_id={} about the job: {}", id, e);
                    }
                }

                info.using = false;
            }
        }

        // We don't need the lock past here, so drop it to prevent deadlocks
        drop(info_write);

        // Checks if number of nodes that accepted the job is less than
        // the size of the cluster required.
        if cluster_size > accepted_job.len() {
            log::warn!(
                "Only {} node(s) accepted the job, required at least {}",
                accepted_job.len(),
                cluster_size
            );

            // Reset all the nodes that accepted to not in use
            for model_id in accepted_job.iter().map(|x| &x.0) {
                self.end(&model_id)
                    .await
                    .expect("Failed to update the node status");
            }

            return None;
        }

        // Buidling actual cluster
        let mut cluster: HashMap<String, Arc<RwLock<TcpStream>>> = HashMap::new();
        let mut cluster_performance: f64 = 0.0;

        // Build cluster of size
        while cluster.len() < cluster_size {
            // Choose a node which has accepted job
            let (chosen_node, performance) = NodePool::choose_random_node(
                &mut accepted_job,
                &mut better_nodes,
                cluster_performance,
            );

            // Get the node stream
            let stream = nodes_read.get(&chosen_node).unwrap().get_tcp();

            // Add node id with stream to cluster
            cluster.insert(chosen_node.clone(), stream);
            cluster_performance = (cluster_performance + performance) / cluster.len() as f64;

            // If accepted_job.is_empty(), output cluster
            if accepted_job.is_empty() {
                break;
            }
        }

        // Reset all the nodes that accepted to not in use
        for model_id in accepted_job.iter().map(|x| &x.0) {
            self.end(&model_id)
                .await
                .expect("Failed to update the node status");
        }

        log::info!(
            "Successfully built a cluster with size={}, cluster_performance={}",
            cluster_size,
            cluster_performance
        );

        // output cluster
        match cluster.len() {
            0 => None,
            _ => Some(cluster),
        }
    }

    /// Returns random model id from list
    ///
    /// Function is given a list of nodes which are prepared to do
    /// Job. This will randomly choose one of them, remove them from the
    /// list and will return its ID, along with its performance level.
    pub fn choose_random_node(
        nodes: &mut Vec<(String, f64)>,
        better_nodes: &mut Vec<(String, f64)>,
        cluster_performance: f64,
    ) -> (String, f64) {
        if cluster_performance == 0.0 || cluster_performance > 0.5 || better_nodes.is_empty() {
            let index = rand::thread_rng().gen_range(0..nodes.len());
            let value = nodes.remove(index);

            // Remove from better nodes if exists
            better_nodes.retain(|item| value != *item);

            value
        } else {
            let index = rand::thread_rng().gen_range(0..better_nodes.len());
            // Get node which has performance of 0.5 or better
            let value = better_nodes.remove(index);

            // Remove from nodes
            nodes.retain(|item| value != *item);

            value
        }
    }

    /// Checks with a node if it will accept a job or not
    pub async fn job_accepted(
        stream: &Arc<RwLock<TcpStream>>,
        config: &JobConfiguration,
        model_id: &str,
    ) -> Result<bool> {
        log::trace!("Sending config={:?} to model_id={}", config, model_id);

        let mut dcn_stream = stream.write().await;

        let mut buffer = [0_u8; 1024];
        let message = ClientMessage::from(config);
        dcn_stream.write(&message.as_bytes()).await?;

        let config_response = ClientMessage::read_until(&mut *dcn_stream, &mut buffer, |m| {
            matches!(m, ClientMessage::ConfigResponse { .. })
        })
        .await;

        log::trace!(
            "model_id={} responded with config_response={:?}",
            model_id,
            config_response
        );

        let accept = match config_response {
            Ok(ClientMessage::ConfigResponse { accept }) => accept,
            Err(e) => {
                log::warn!("Failed to read a `ConfigResponse` from the stream: {}", e);
                false
            }
            _ => unreachable!(),
        };

        log::debug!(
            "Node with model_id={} responsed with '{}' to the job config",
            model_id,
            accept
        );

        Ok(accept)
    }

    /// Changes the `using` flag on a [`NodeInfo`] object
    ///
    /// When passed an [`ObjectId`], this function will find the [`NodeInfo`] instance for that ID
    /// and will set its `using` flag to be false, signifying the end of its use.
    pub async fn end(&self, key: &str) -> Result<()> {
        log::trace!("Finished using model_id={}, updating its status", key);

        let mut info_write = self.info.write().await;
        info_write.get_mut(key).unwrap().using = false;

        self.active.fetch_add(1, Ordering::SeqCst);
        self.job_notify.notify_waiters();

        log::info!("Active Nodes: {:?}", self.active);

        Ok(())
    }

    /// Updates a [`NodeInfo`] object about its status
    ///
    /// Gets the correct [`NodeInfo`] struct and updates its alive field by inverting what it
    /// currently is.
    pub async fn update_node_alive(&self, id: &str, status: bool) {
        let mut info_write = self.info.write().await;
        let node_info = info_write.get_mut(id).unwrap();

        log::trace!(
            "Updating liveness status of model_id={}, setting to alive={}",
            id,
            status
        );

        if !node_info.alive && status {
            self.active.fetch_add(1, Ordering::SeqCst);
            self.job_notify.notify_waiters();
        } else if node_info.alive && !status && !node_info.using {
            self.active.fetch_sub(1, Ordering::SeqCst);
        }

        node_info.alive = status;
    }

    /// Checks if a node is being used
    ///
    /// Passed the [`ObjectId`] of a node and it checks if it is being used for a job, which
    /// implies it is alive.
    pub async fn is_using(&self, id: &str) -> bool {
        let info_read = self.info.read().await;
        let node_info = info_read.get(id).unwrap();

        node_info.using
    }

    /// Updates a [`NodeInfo`] object
    ///
    /// Gets the correct [`NodeInfo`] struct and updates its average performance.
    /// New performance has the greatest impact on the stored model performance
    /// meaning recent performance has a significant bearing on current appearance,
    /// while retaining historical performance.
    pub async fn update_node_performance(&self, id: &str, performance: f64) {
        let mut info_write = self.info.write().await;
        let node_info = info_write.get_mut(id).unwrap();

        log::trace!("Updating model_id={} with performance={}", id, performance);

        if node_info.performance == 0.0 {
            node_info.performance = performance
        } else {
            node_info.performance = (performance + node_info.performance) / 2.0;
        }
    }
}

/// Run Node for DCNs to connect to
///
/// Starts up node end which allows DCNs to register their connection. This will create a Node
/// object if given a correct API Key. This allows the job end to connect and communicate with the
/// DCNs.
pub async fn run(nodepool: Arc<NodePool>, database: Arc<Database>, port: u16) -> Result<()> {
    // Bind to the external socket in production mode
    #[cfg(not(debug_assertions))]
    let ip = Ipv4Addr::UNSPECIFIED;

    #[cfg(debug_assertions)]
    let ip = Ipv4Addr::LOCALHOST;

    let socket = SocketAddr::V4(SocketAddrV4::new(ip, port));
    let listener = TcpListener::bind(&socket).await?;

    log::info!("Listening for client connections on: {}", socket);

    while let Ok((inbound, _)) = listener.accept().await {
        let sp_clone = Arc::clone(&nodepool);
        let db_clone = Arc::clone(&database);

        log::info!("Received a node connection from: {}", inbound.peer_addr()?);

        let fut = process_connection(inbound, db_clone, sp_clone);

        if let Err(e) = tokio::spawn(async move { fut.await }).await? {
            log::error!("Error processing connection: {:?}", e);
        }
    }

    Ok(())
}

async fn process_connection(
    mut stream: TcpStream,
    database: Arc<Database>,
    nodepool: Arc<NodePool>,
) -> Result<()> {
    let mut handler = protocol::Handler::new(&mut stream);
    let model_id = match handler.get_access_token().await? {
        Some(t) => t.0,
        None => return Ok(()),
    };

    update_model_status(Arc::clone(&database), &model_id, Status::Running).await?;

    let node = Node::new(stream, model_id);
    nodepool.add(node, Arc::clone(&database)).await;

    Ok(())
}

/// Update the status of a model in the database.
///
/// When a model authenticates with the DCL correctly and is heartbeating, this will set the status
/// in the database to `Running`. This can then be displayed on the frontend.
pub async fn update_model_status(
    database: Arc<Database>,
    model_id: &str,
    status: Status,
) -> Result<()> {
    let models = database.collection("models");

    log::debug!("Setting model_id={} to status={:?}", model_id, status);

    let object_id = ObjectId::with_string(model_id)?;
    let query = doc! {"_id": &object_id};
    let update = doc! { "$set": { "status": status } };
    models.update_one(query, update, None).await?;

    Ok(())
}
