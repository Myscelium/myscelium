use crate::common::enhanced_buffer::utilities::CommandType;
use crate::common::structs::results_structs::ResultType;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::{Map, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandlerStatus {
    Working,
    NotImplemented,
    NotTested,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeHandler {
    name: String,
    parameters: HashMap<String, Value>,
    handler_type: CommandType,
    status: HandlerStatus,
    response_structure: HashMap<String, Value>,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeVersion {
    marjor: u32,
    minor: u32,
    patch: u32,
    identifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    name: String,
    key: String,
    status: NodeStatus,
    description: String,
    version: NodeVersion,
    handlers: Vec<NodeHandler>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeStatus {
    Online,
    Idle,
    NotSync,
    NotImplemented,
    Offline,
}

impl Node {
    pub fn new(name: String, key: String, status: NodeStatus, description: String, version: NodeVersion, handlers: Vec<NodeHandler>) -> Self {
        Self {
            name,
            key,
            status,
            description,
            version,
            handlers,
        }
    }

    pub fn change_node_status(&mut self, new_status: NodeStatus) {
        self.status = new_status;
    }

    pub fn update(&mut self, name: String, key: String, description: String, version: NodeVersion, handlers: Vec<NodeHandler>) {
        self.name = name;
        self.key = key;
        self.description = description;
        self.version = version;
        self.handlers = handlers;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMap {
    nodes: Vec<Node>,
}

pub enum NetworkMapError {
    NodeDoNotExists(String),
    IncorrectValueMapPattern(String),
    IncorrectValuePattern,
}

impl NetworkMap {
    pub fn new(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }

    pub fn get_node_keys(&self) -> HashMap<String, String> {
        let mut valid_keys = HashMap::new();
        for node in &self.nodes {
            valid_keys.insert(node.name.clone(), node.key.clone());
        }
        return valid_keys;
    }

    pub fn get_node_by_name(&self, name: &String) -> Result<&Node, NetworkMapError> {
        for node in &self.nodes {
            if &node.name == name {
                return Ok(node);
            }
        }
        return Err(NetworkMapError::NodeDoNotExists(name.clone()));
    }

    pub fn get_node_by_key(&self, key: &String) -> Result<&Node, NetworkMapError> {
        for node in &self.nodes {
            if &node.key == key {
                return Ok(node);
            }
        }
        return Err(NetworkMapError::NodeDoNotExists(key.clone()));
    }

    pub fn convert_to_value_map(&self) -> HashMap<String, Value> {
        let mut value_map = HashMap::new();
        value_map.insert("network_map".to_string(), serde_json::to_value(&self).unwrap());
        return value_map;
    }

    pub fn extract_to_value(&self) -> serde_json::Value {
        serde_json::to_value(&self).unwrap()
    }

    fn decode_value(value_object: Value) -> Result<NetworkMap, NetworkMapError> {
        let new_network_map: NetworkMap = match serde_json::from_value(value_object) {
            Ok(n) => n,
            Err(e) => {
                println!("Error creating network map from value: {:?}", e);
                return Err(NetworkMapError::IncorrectValuePattern);
            },
        };

        return Ok(new_network_map);
    }

    pub fn update_from_value(&mut self, value_object: Value) -> Result<(), NetworkMapError> {
        let new_network_map: NetworkMap = NetworkMap::decode_value(value_object)?;
        self.mass_update_all_nodes(&new_network_map.nodes);
        return Ok(());
    }

    pub fn gen_from_value(value_object: Value) -> Result<Self, NetworkMapError> {
        Ok(NetworkMap::decode_value(value_object)?)
    }

    pub fn update_from_value_map(&mut self, map: HashMap<String, Value>) -> Result<(), NetworkMapError> {
        if !map.contains_key("network_map") {
            return Err(NetworkMapError::IncorrectValueMapPattern("network map key not found in the map provided".to_string()));
        };

        let value_network_map = &map["network_map"];

        let network_map: NetworkMap = match serde_json::from_value(value_network_map.clone()) {
            Ok(n) => n,
            Err(e) => return Err(NetworkMapError::IncorrectValueMapPattern(e.to_string())),
        };

        self.mass_update_all_nodes(&network_map.nodes);

        return Ok(());
    }

    /// The idea of the update NetworkMap are to update the network
    /// by passing a Vec<Node> a vec of nodes, this allows to iterate in the
    /// current network map and update nodes based in the nodes contained in
    /// the vec of updated nodes, if a node exists then it will be updated
    /// with the values or the variables contained in this vec.
    pub fn mass_update_all_nodes(&mut self, updated_nodes: &Vec<Node>) -> Result<(), NetworkMapError> {
        // TODO >>> Add a better mechanism that can see if a node or function isn't implemented anymore in relation to the previous expectation

        let nnl = updated_nodes.len();
        let mut not_seen_nodes: Vec<String> = Vec::new();

        let registred_node_keys: Vec<String> = self.get_node_keys().values().cloned().collect();
        let mut not_implemented_nodes: Vec<String> = Vec::new();

        let mut new_nodes: HashMap<String, Node> = HashMap::new();
        let mut new_nodes_keys: Vec<String> = Vec::new();

        for nn in updated_nodes {
            new_nodes.insert(nn.key.clone(), nn.clone());
            new_nodes_keys.push(nn.key.clone());
        }

        not_seen_nodes = new_nodes_keys.clone();

        // -> UPDATE EXISTING NODES:

        for node in &mut self.nodes {
            if new_nodes_keys.contains(&node.key) {
                //> UPDATE NODES THAT STILL EXISTING
                let new_node = &new_nodes[&node.key];
                *node = new_node.clone();

                if let Some(index) = not_seen_nodes.iter().position(|x| x == &node.key) {
                    not_seen_nodes.remove(index); // remove seen nodes
                }
            } else {
                //> UPDATE NODES THAT DON'T EXISTS ANYMORE
                node.status = NodeStatus::NotImplemented;
            };
        }

        // -> CREATE NEW NODES:

        for key in not_seen_nodes {
            let new_node = new_nodes[&key].clone();
            self.nodes.push(Node::new(new_node.name, new_node.key, new_node.status, new_node.description, new_node.version, new_node.handlers))
        }

        return Ok(());
    }

    pub fn command_exists(&self, owner: &str, command_name: &str) -> bool {
        let node = match self.get_node_by_name(&owner.to_string()) {
            Ok(n) => n,
            Err(_) => {
                return false;
            },
        };
        for handler in &node.handlers {
            if handler.name == command_name {
                return true;
            }
        }
        return false;
    }

    pub fn add_or_update_if_exists(&mut self, new_node: Node) {
        // -> UPDATE EXISTING NODE:
        for node in &mut self.nodes {
            if new_node.key == node.key {
                *node = new_node;
                return;
            } else {
                continue;
            }
        }

        // -> CREATE NEW NODE:
        self.nodes.push(Node::new(new_node.name, new_node.key, new_node.status, new_node.description, new_node.version, new_node.handlers))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    parameters: HashMap<String, String>,
    status: CommandStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CommandStatus {
    Active,
    Inactive,
    // Add more status types as needed
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandPatterns {
    //> Structure:

    //> "owner": {
    //>     "command_name": Comamnd {
    //>         "parameters": HashMap<String, String>,
    //>         "status": CommandStatus,
    //>     }
    //> }

    // -> Wrap patterns
    patterns: HashMap<String, HashMap<String, Command>>,
}

impl CommandPatterns {
    pub fn new() -> Self {
        CommandPatterns { patterns: HashMap::new() }
    }

    pub fn command_exists(&self, owner: &str, command_name: &str) -> bool {
        self.patterns.get(owner).and_then(|commands| commands.get(command_name)).is_some()
    }

    pub fn add_command(&mut self, owner: String, command_name: String, command: Command) {
        self.patterns.entry(owner).or_insert_with(HashMap::new).insert(command_name, command);
    }

    // Function to parse the JSON and integrate it into CommandPatterns
    pub fn add_json(&mut self, owner: &str, json_str: &str) -> Result<(), serde_json::Error> {
        // Parse the JSON string
        let parsed: HashMap<String, Value> = serde_json::from_str(json_str)?;

        // Iterate over the parsed data and integrate it into CommandPatterns
        for (command_name, params) in parsed {
            let mut command_params = HashMap::new();

            match params {
                Value::Object(obj) => {
                    for (param_name, param_type) in obj {
                        if let Value::String(type_str) = param_type {
                            command_params.insert(param_name, type_str);
                        }
                    }
                },
                _ => (), // Handle other types like Array, if necessary
            }

            let command = Command {
                parameters: command_params,
                status: CommandStatus::Active, // Assuming default status as Active
            };

            self.add_command(owner.to_string(), command_name, command);
        }

        Ok(())
    }

    // Function to integrate a HashMap<String, Value> as commands for a client
    pub fn add_commands_from_map(&mut self, client: &str, commands_map: HashMap<String, Value>) {
        let client_commands = self.patterns.entry(client.to_string()).or_insert_with(HashMap::new);

        for (command_name, params) in commands_map {
            let mut command_params = HashMap::new();

            match params {
                Value::Object(obj) => {
                    for (param_name, param_type) in obj {
                        if let Value::String(type_str) = param_type {
                            command_params.insert(param_name, type_str);
                        }
                        // Handle other Value types if necessary
                    }
                },
                _ => (), // Handle non-Object types if necessary
            }

            let command = Command {
                parameters: command_params,
                status: CommandStatus::Active, // Assuming default status as Active
            };

            // Add or update the command
            client_commands.insert(command_name, command);
        }
    }

    // Function to add or update commands for a client
    pub fn add_or_update_if_exists(&mut self, client: &str, commands_map: HashMap<String, Value>) {
        if self.patterns.contains_key(client) {
            // If the client already exists, update its commands
            // You might need additional logic here to properly merge or update the commands
            self.add_commands_from_map(client, commands_map);
        } else {
            // If the client does not exist, add it
            self.patterns.insert(client.to_string(), HashMap::new());
            self.add_commands_from_map(client, commands_map);
        }
    }

    pub fn extract_command_params_for_client(&self, client: &str, command_name: &str) -> Option<HashMap<String, Value>> {
        // Attempt to retrieve the command for the specified client
        if let Some(client_commands) = &self.patterns.get(client) {
            if let Some(command) = client_commands.get(command_name) {
                let mut params_map = HashMap::new();

                // Iterate over the command parameters and convert them to Value
                for (param_name, param_type) in &command.parameters {
                    params_map.insert(param_name.clone(), Value::String(param_type.clone()));
                }

                return Some(params_map);
            }
        }
        None
    }

    // Function to extract a HashMap of all clients, each with their own commands and parameters
    pub fn extract_all_commands(&self) -> HashMap<String, Value> {
        let mut all_clients_commands = HashMap::new();

        // Iterate over all clients
        for (client_name, client_commands) in &self.patterns {
            let mut client_commands_map = serde_json::Map::new();

            // Iterate over each command for the client
            for (command_name, command) in client_commands {
                let params_value = command.parameters.iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect::<serde_json::Map<_, _>>();

                client_commands_map.insert(command_name.clone(), Value::Object(params_value));
            }

            all_clients_commands.insert(client_name.clone(), Value::Object(client_commands_map));
        }

        all_clients_commands
    }

    // Function to get all commands except for those of a specified client, formatted as a HashMap<String, Value>
    pub fn get_all_commands_except_for_client(&self, excluded_client: &str) -> HashMap<String, Value> {
        let mut filtered_commands = HashMap::new();

        for (client_name, client_commands) in &self.patterns {
            if client_name != excluded_client {
                let mut client_commands_map = Map::new();

                // Iterate over each command for the client
                for (command_name, command) in client_commands {
                    let params_value = command.parameters.iter().map(|(k, v)| (k.clone(), Value::String(v.clone()))).collect::<Map<_, _>>();

                    client_commands_map.insert(command_name.clone(), Value::Object(params_value));
                }

                filtered_commands.insert(client_name.clone(), Value::Object(client_commands_map));
            }
        }

        filtered_commands
    }

    pub fn remove_command(&mut self, owner: &str, command_name: &str) {
        if let Some(commands) = self.patterns.get_mut(owner) {
            commands.remove(command_name);
        }
    }

    // Function to create a new CommandPatterns struct from a HashMap<String, Value>
    pub fn create_command_patterns_from_map(commands_map: HashMap<String, Value>) -> Self {
        let mut command_patterns = Self::new();

        // Iterate over the outer map, where each key is a client name
        for (client_name, client_commands_value) in commands_map {
            if let Value::Object(client_commands_map) = client_commands_value {
                // Iterate over each command for the client
                for (command_name, params_value) in client_commands_map {
                    if let Value::Object(params_map) = params_value {
                        let command_params = params_map
                            .into_iter()
                            .filter_map(|(k, v)| {
                                if let Value::String(value_str) = v {
                                    Some((k, value_str))
                                } else {
                                    None // Filter out non-string values
                                }
                            })
                            .collect::<HashMap<String, String>>();

                        let command = Command {
                            parameters: command_params,
                            status: CommandStatus::Active, // Default status, can be adjusted as needed
                        };

                        command_patterns.add_command(client_name.clone(), command_name, command);
                    }
                }
            }
        }

        command_patterns
    }

    // Function to create a new CommandPatterns struct from a HashMap<String, Value>
    pub fn add_from_map(&mut self, commands_map: HashMap<String, Value>) -> Self {
        // Iterate over the outer map, where each key is a client name
        for (client_name, client_commands_value) in commands_map {
            if let Value::Object(client_commands_map) = client_commands_value {
                // Iterate over each command for the client
                for (command_name, params_value) in client_commands_map {
                    if let Value::Object(params_map) = params_value {
                        let command_params = params_map
                            .into_iter()
                            .filter_map(|(k, v)| {
                                if let Value::String(value_str) = v {
                                    Some((k, value_str))
                                } else {
                                    None // Filter out non-string values
                                }
                            })
                            .collect::<HashMap<String, String>>();

                        let command = Command {
                            parameters: command_params,
                            status: CommandStatus::Active, // Default status, can be adjusted as needed
                        };

                        &self.add_command(client_name.clone(), command_name, command);
                    }
                }
            }
        }

        self.clone()
    }
}

/*
-> Example of usage:
``` Rust
lazy_static! {
    static ref GLOBAL_COMMAND_PATTERNS: Mutex<CommandPatterns> = Mutex::new(CommandPatterns::new());
}

fn main() {
    let mut command_patterns = GLOBAL_COMMAND_PATTERNS.lock().unwrap();

    Define and add 'get_symbols_data' command to 'client1'
    let get_symbols_data_params = create_command_params(&[
        ("data-type", "str"),
        ("symbols", "str"),
        ("start-ts", "float"),
        ("end-ts", "float"),
        ]);
        let get_symbols_data_command = Command {
            parameters: get_symbols_data_params,
            status: CommandStatus::Active,
        };
        command_patterns.add_command("client1".into(), "get_symbols_data".into(), get_symbols_data_command);

        Similarly, define and add 'get_other_symbols_data' command
        let get_other_symbols_data_params = create_command_params(&[
            ("data-type", "str"),
            ("symbols", "str"),
            ("start-ts", "float"),
            ("end-ts", "float"),
            ]);
            let get_other_symbols_data_command = Command {
                parameters: get_other_symbols_data_params,
                status: CommandStatus::Active,
            };
            command_patterns.add_command("client1".into(), "get_other_symbols_data".into(), get_other_symbols_data_command);

            // Now, 'command_patterns' contains the two commands for 'client1'
        }

```
*/
