use crate::common::structs::results_structs::ResultType;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::{Map, Value};

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
