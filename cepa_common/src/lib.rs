use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

pub type NodeListPointer = Arc<Mutex<NodeList>>;

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct NodeData {
    pub host: String,
    pub pub_key: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
pub struct NodeList {
    pub timestamp: u64,
    pub list: Vec<NodeData>,
}