use bon::Builder;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// dto
#[derive(Deserialize)]
pub struct NodeStateChangeFloat {
    // node -> server
    pub position_type: i64,
    pub position_change: Vec<f64>,
    pub state_change: HashMap<String, f64>,
}

#[derive(Deserialize)]
pub struct NodeStateChangeString {
    pub state_change: HashMap<String, String>,
}

#[derive(Deserialize)]
pub struct NodeStateRegister {
    pub id: String,
    pub position_type: i64,
    pub position: Vec<f64>,
    pub state: HashMap<String, f64>,
}

#[derive(Builder, Serialize)]
pub struct NodeCommand {
    // server -> node
    pub op: String,
    pub op_value: HashMap<String, f64>,
}
