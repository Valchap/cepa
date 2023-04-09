use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

pub type NodeListPointer = Arc<Mutex<NodeList>>;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "serde")]
pub struct NodeData {
    pub host: String,
    pub pub_key: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(crate = "serde")]
pub struct NodeList {
    pub timestamp: u64,
    pub list: Vec<NodeData>,
}

impl NodeList {
    pub fn new() -> Self {
        Self {
            timestamp: 0,
            list: Vec::new(),
        }
    }
}
