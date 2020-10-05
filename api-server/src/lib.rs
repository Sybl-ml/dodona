#[macro_use]
extern crate serde_json;

use std::sync::Arc;

use mongodb::Client;

pub mod models;
pub mod routes;

#[derive(Clone, Debug)]
pub struct State {
    pub client: Arc<Client>,
    pub db_name: Arc<String>,
    pub pepper: Arc<String>,
}
