use crate::common::structs::available_commands::NetworkMap;

pub struct ClientStates {
    name: String,
    key: String,
    network_map: NetworkMap,
    is_initialized: bool,
    is_ready: bool,
    connected: bool,
}
