use serde::{Deserialize, Serialize};
use serde_json::{from_str, Value};
use std::collections::HashMap;

use crate::commom::enhanced_buffer;
use crate::commom::enhanced_buffer::buffer_down_mananger::DownCommand;
use crate::commom::enhanced_buffer::buffer_up_mananger::UpCommand;

#[derive(Debug)]
pub enum CommandType {
    Function(String),
    Response(String),
    Redirect(String),
    Error(String),
    InternalMannangement(String),
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub client_id: String,
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
    pub fn new(client_id: String, parity_id: String, priority: u8, command: HashMap<String, Value>) -> Self {
        Self { client_id, parity_id, priority, command }
    }

    pub fn from_down_command(down_command: DownCommand) -> Self {
        let client_id = down_command.client_id.clone();
        let parity_id = down_command.parity_id.clone();
        let priority = down_command.priority.clone();

        let outer_value: Value = serde_json::from_str(&down_command.command).unwrap();

        println!("Value extracted from DownCommand.Command: {:?}", outer_value);

        let mut command: HashMap<String, Value> = HashMap::new();

        // Extract the inner JSON string and deserialize it again
        if let Value::Object(outer_map) = &outer_value {
            if let Some(Value::String(inner_json)) = outer_map.get("response") {
                command = serde_json::from_str(inner_json).unwrap();

                // Transform the command right after deserialization
                let transformed_value = transform_value(&Value::Object(serde_json::Map::from_iter(command.into_iter()))); // Convert HashMap to serde_json::Map using into()

                if let Value::Object(transformed_map) = transformed_value {
                    command = HashMap::new();
                    command.insert("response".to_string(), Value::Object(transformed_map.into_iter().collect()));
                // Convert serde_json::Map back to HashMap
                } else {
                    command = HashMap::new();
                }
            } else {
                command = serde_json::from_str(&down_command.command).unwrap();
            }
        } else {
            command = HashMap::new();
            // Handle the case where the outer_value is not an object
        }
        Self { client_id, parity_id, priority, command }
    }

    pub fn from_up_command(up_command: UpCommand) -> Self {
        let client_id = up_command.client_id.clone();
        let parity_id = up_command.parity_id.clone();
        let priority = up_command.priority.clone();
        let command: HashMap<String, Value> = serde_json::from_str(&up_command.command).unwrap();

        Self { client_id, parity_id, priority, command }
    }

    pub fn command_type(&self) -> CommandType {
        if self.command.contains_key("function") {
            CommandType::Function(self.command.get("function").unwrap().to_string())
        } else if self.command.contains_key("response") {
            CommandType::Response(self.command.get("response").unwrap().to_string())
        } else if self.command.contains_key("redirect") {
            CommandType::Redirect(self.command.get("redirect").unwrap().to_string())
        } else if self.command.contains_key("internal_mannangement") {
            CommandType::InternalMannangement(self.command.get("internal_mannangement").unwrap().to_string())
        } else if self.command.contains_key("error") {
            CommandType::Error(self.command.get("error").unwrap().to_string())
        } else {
            CommandType::Unknown
        }
    }
}
