use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::collections::HashMap;

use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::buffer_down_manager::DownCommand;
use crate::common::enhanced_buffer::buffer_up_manager::UpCommand;

#[derive(Debug)]
pub enum CommandType {
    SpecialFunction(Value),
    DirectFunction(Value),
    Function(Value),
    Response(HashMap<String, Value>),
    Redirect(HashMap<String, Value>),
    Error(Value),
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub client_key: String,
    pub parity_id: String,
    pub priority: u8,
    pub command: HashMap<String, Value>,
}

fn transform_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = HashMap::new();
            for (key, val) in map.iter() {
                // Iterating over references
                if let Some(inner_val) = val.get("Map") {
                    new_map.insert(key.clone(), transform_value(inner_val));
                } else if let Some(inner_val) = val.get("List") {
                    new_map.insert(key.clone(), transform_value(inner_val));
                } else if let Some(Value::String(s)) = val.get("Str") {
                    new_map.insert(key.clone(), Value::String(s.clone()));
                } else if let Value::Object(_) = val {
                    new_map.insert(key.clone(), transform_value(val));
                } else {
                    new_map.insert(key.clone(), transform_value(val));
                }
            }
            Value::Object(serde_json::Map::from_iter(new_map)) // Convert HashMap to serde_json::Map using into()
        },
        Value::String(s) => {
            // If the string is a JSON representation, parse it and transform it
            if let Ok(parsed) = serde_json::from_str::<Value>(s) {
                transform_value(&parsed)
            } else {
                Value::String(s.clone())
            }
        },
        Value::Array(arr) => Value::Array(arr.iter().map(|v| transform_value(v)).collect()),
        _ => value.clone(),
    }
}

impl Command {
    pub fn new(client_key: String, parity_id: String, priority: u8, command: HashMap<String, Value>) -> Self {
        Self { client_key, parity_id, priority, command }
    }

    pub fn from_down_command(down_command: DownCommand) -> Self {
        let client_key = down_command.client_key.clone();
        let parity_id = down_command.parity_id.clone();
        let priority = down_command.priority.clone();

        let outer_value: Value = serde_json::from_str(&down_command.command).unwrap();

        println!("Value extracted from DownCommand.Command: {:?}", outer_value);

        let mut command: HashMap<String, Value> = HashMap::new();

        // Extract the inner JSON string and deserialize it again
        if let Value::Object(outer_map) = &outer_value {
            command = serde_json::from_value::<HashMap<String, Value>>(Value::Object(outer_map.clone())).unwrap();
        } else {
            command = HashMap::new();
            // Handle the case where the outer_value is not an object
        }
        Self { client_key, parity_id, priority, command }
    }

    pub fn from_up_command(up_command: UpCommand) -> Self {
        let client_key = up_command.client_key.clone();
        let parity_id = up_command.parity_id.clone();
        let priority = up_command.priority.clone();
        let command: HashMap<String, Value> = serde_json::from_str(&up_command.command).unwrap();

        println!("Client -> Command from UpCommand: {:?}", command);

        Self { client_key, parity_id, priority, command }
    }

    pub fn command_type(&self) -> CommandType {
        if self.command.contains_key("command_type") {
            let command_type: String = serde_json::from_value(self.command.get("command_type").unwrap().clone()).unwrap();

            println!("Command Type: {:?}", command_type);

            return match command_type.as_str() {
                "function" => {
                    println!("Self: {:?} have function: {:?}", self.command, self.command.get("function"));
                    CommandType::Function(self.command.get("function").unwrap().clone())
                },
                "direct_function" => {
                    println!("Self: {:?} have direct function: {:?}", self.command, self.command.get("function"));
                    CommandType::DirectFunction(self.command.get("function").unwrap().clone())
                },
                "special_function" => {
                    println!("Self: {:?} have special function: {:?}", self.command, self.command.get("function"));
                    CommandType::SpecialFunction(self.command.get("function").unwrap().clone())
                },
                "response" => {
                    println!("Self: {:?} have response: {:?}", self.command, self.command.get("kwargs"));
                    CommandType::Response(self.command.clone())
                },
                "redirect" => {
                    println!("Self: {:?} have redirect: {:?}", self.command, self.command.get("response"));
                    CommandType::Redirect(self.command.clone())
                },
                "error" => {
                    // TODO >>> Verify if the error command type stil a requirement since client now receives the entire command
                    println!("Self: {:?} have error: {:?}", self.command, self.command.get("error"));
                    CommandType::Error(self.command.get("error").unwrap().clone())
                },
                _ => CommandType::Unknown,
            };
        } else {
            return CommandType::Unknown;
        }
    }
}
