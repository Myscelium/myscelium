use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::Mutex;

use crate::common::structs::available_commands::{NetworkMap, Node};

lazy_static! {
    static ref CLIENT_STATE_MANAGER: Arc<Mutex<ClientState>> = Arc::new(Mutex::new(ClientState::empty()));
}

pub struct ClientState {
    name: Option<String>,
    key: Option<String>,
    network_map: Option<NetworkMap>,
    client_node_configs: Option<Node>,
    is_initialized: Option<bool>,
    is_ready: Option<bool>,
    is_connected: Option<bool>,
    is_sync: Option<bool>,
    last_change: Option<f64>,
}

impl ClientState {
    pub fn new(name: String, key: String, network_map: NetworkMap, client_node_configs: Node, is_initialized: bool, is_ready: bool, is_connected: bool, is_sync: bool, last_change: f64) -> Self {
        Self {
            name: Some(name),
            key: Some(key),
            network_map: Some(network_map),
            client_node_configs: Some(client_node_configs),
            is_initialized: Some(is_initialized),
            is_ready: Some(is_ready),
            is_connected: Some(is_connected),
            is_sync: Some(is_sync),
            last_change: Some(last_change),
        }
    }

    pub fn empty() -> Self {
        Self {
            name: None,
            key: None,
            network_map: None,
            client_node_configs: None,
            is_initialized: None,
            is_ready: None,
            is_connected: None,
            is_sync: None,
            last_change: None,
        }
    }

    pub fn save_in_storage(&self) {
        // TODO >>> Finish this method;
    }

    pub fn load_from_storage(&self) -> Self {
        Self {
            name: None,
            key: None,
            network_map: None,
            client_node_configs: None,
            is_initialized: None,
            is_ready: None,
            is_connected: None,
            is_sync: None,
            last_change: None,
        }
    }
}
