use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::buffer_down_manager::DownCommand;
use crate::common::enhanced_buffer::buffer_up_manager::UpCommand;
use crate::socket_host::client_manager::manager::Client;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, Map, Value};
use std::collections::HashMap;

use std::fmt;

trait Stringifiable: Sized {
    fn eq_str(&self, other: &str) -> bool;
}

// Implement the trait for each of your enums
macro_rules! impl_stringfiable_for_enum {
    ($($t:ty),+) => {
        $(
            impl Stringifiable for $t {
                fn eq_str(&self, other:&str) -> bool {
                    format!("{:?}", self) == other
                }
            }

            impl fmt::Display for $t {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{:?}", self)
                }
            }

            impl PartialEq<&str> for $t {
                fn eq(&self, other:&&str) -> bool {
                    self.eq_str(other)
                }
            }
        )+
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandMode {
    Function,
    Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandType {
    SpecialFunction,
    DirectFunction,
    InternalManagement,
    Default,
    Redirect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandStatus {
    Success,
    Failure,
}

// impl PartialEq<&str> for CommandStatus {
//     fn eq(&self, other: &&str) -> bool {
//         match self {
//             CommandStatus::Success => *other == "Success",
//             CommandStatus::Failure => *other == "Failure",
//         }
//     }
// }

#[derive(Debug, Clone, Serialize, Deserialize)]

pub enum CommandTarget {
    Origin,
    ClientKey(String),
    Host,
}

impl_stringfiable_for_enum!(CommandMode, CommandType, CommandStatus, CommandTarget);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandOrigin {
    Host,
    ClientKey(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandInstructions {
    pub mode: CommandMode,
    #[serde(rename = "type")]
    pub command_type: CommandType,
    pub target: CommandTarget,
    pub status: CommandStatus,
    pub origin: CommandOrigin,
    pub actf: String,
    pub kwargs: HashMap<String, serde_json::Value>,
    pub message: String,
}

#[derive(Debug)]
pub enum CommandError {
    InvalidCommand(String),
}

impl CommandInstructions {
    pub fn new(mode: CommandMode, command_type: CommandType, target: CommandTarget, status: CommandStatus, origin: CommandOrigin, actf: String, kwargs: HashMap<String, serde_json::Value>, message: String) -> Self {
        Self {
            mode,
            command_type,
            target,
            status,
            origin,
            actf,
            kwargs,
            message,
        }
    }

    pub fn to_value_map(&self) -> Value {
        serde_json::to_value(&self).unwrap()
    }

    pub fn to_string_value(&self) -> HashMap<String, Value> {
        // Serialize the struct to a JSON string
        let json_str: String = serde_json::to_string(&self).unwrap();

        // Deserialize the JSON string into a HashMap
        serde_json::from_str(&json_str).unwrap()
    }

    pub fn from_value_map(mut map: HashMap<String, Value>) -> Result<Self, CommandError> {
        let mode = match map.get("type").and_then(Value::as_str) {
            Some("function") => CommandMode::Function,
            Some("response") => CommandMode::Response,
            _ => return Err(CommandError::InvalidCommand("Invalid or missing type".to_string())),
        };

        let command_type = match map.get("mode").and_then(Value::as_str) {
            Some("special_function") => CommandType::SpecialFunction,
            Some("direct_function") => CommandType::DirectFunction,
            Some("internal_management") => CommandType::InternalManagement,
            Some("default") => CommandType::Default,
            Some("redirect") => CommandType::Redirect,
            _ => return Err(CommandError::InvalidCommand("Invalid or missing mode".to_string())),
        };

        // let target = map.get("target").and_then(Value::as_str).map(String::from).ok_or_else(|| CommandError::InvalidCommand("Missing target".to_string()))?;

        let target = match map.get("target").and_then(Value::as_str) {
            Some("host") => CommandTarget::Host,
            Some("origin") => CommandTarget::Origin,
            Some(c) => {
                if c != "" {
                    return Err(CommandError::InvalidCommand("Invalid or missing target".to_string()));
                }

                CommandTarget::ClientKey(c.to_string())
            },
        };

        let status = match map.get("status").and_then(Value::as_str) {
            Some("success") => CommandStatus::Success,
            Some("failure") => CommandStatus::Failure,
            _ => return Err(CommandError::InvalidCommand("Invalid or missing status".to_string())),
        };

        let origin = match map.get("origin").and_then(Value::as_str) {
            Some("host") => CommandOrigin::Host,
            Some(client_id) => CommandOrigin::ClientKey(client_id.to_string()),
            _ => return Err(CommandError::InvalidCommand("Invalid or missing origin".to_string())),
        };

        let actf = map.get("actf").and_then(Value::as_str).map(String::from).ok_or_else(|| CommandError::InvalidCommand("Missing actf".to_string()))?;
        let message = map.get("message").and_then(Value::as_str).map(String::from).ok_or_else(|| CommandError::InvalidCommand("Missing message".to_string()))?;

        // Extract kwargs directly
        let kwargs = map
            .remove("kwargs")
            .map_or_else(|| HashMap::new(), |v| v.as_object().map_or_else(|| HashMap::new(), |map| map.into_iter().map(|(k, v)| (k.clone(), v.clone())).collect()));

        Ok(CommandInstructions {
            mode,
            command_type,
            target,
            status,
            origin,
            actf,
            kwargs,
            message,
        })
    }

    pub fn from_string_hashmap(mut map: HashMap<String, String>) -> Result<Self, CommandError> {
        let mode = match map.get("type").map(String::as_str) {
            Some("function") => CommandMode::Function,
            Some("response") => CommandMode::Response,
            _ => return Err(CommandError::InvalidCommand("Invalid or missing type".to_string())),
        };

        let command_type = match map.get("mode").map(String::as_str) {
            Some("special_function") => CommandType::SpecialFunction,
            Some("direct_function") => CommandType::DirectFunction,
            Some("internal_management") => CommandType::InternalManagement,
            Some("default") => CommandType::Default,
            Some("redirect") => CommandType::Redirect,
            _ => return Err(CommandError::InvalidCommand("Invalid or missing mode".to_string())),
        };

        // let target = map.get("target").cloned().ok_or_else(|| "Missing target".to_string())?;

        let target = match map.get("target").map(String::as_str) {
            Some("host") => CommandTarget::Host,
            Some("origin") => CommandTarget::Origin,
            Some(c) => {
                if c != "" {
                    return Err(CommandError::InvalidCommand("Invalid or missing target".to_string()));
                }

                CommandTarget::ClientKey(c.to_string())
            },
        };

        let status = match map.get("status").map(String::as_str) {
            Some("success") => CommandStatus::Success,
            Some("failure") => CommandStatus::Failure,
            _ => return Err(CommandError::InvalidCommand("Invalid or missing status".to_string())),
        };

        let origin = match map.get("origin").map(String::as_str) {
            Some("host") => CommandOrigin::Host,
            Some(client_id) => CommandOrigin::ClientKey(client_id.to_string()),
            _ => return Err(CommandError::InvalidCommand("Invalid or missing origin".to_string())),
        };

        let actf = map.get("actf").cloned().ok_or_else(|| CommandError::InvalidCommand("Missing actf".to_string()))?;
        let message = map.get("message").cloned().ok_or_else(|| CommandError::InvalidCommand("Missing message".to_string()))?;

        // Extract and parse the kwargs field
        let kwargs = if let Some(kwargs_str) = map.remove("kwargs") {
            serde_json::from_str::<HashMap<String, Value>>(&kwargs_str).map_err(|e| CommandError::InvalidCommand(format!("Failed to parse kwargs: {}", e)))?
        } else {
            HashMap::new()
        };

        Ok(CommandInstructions {
            mode,
            command_type,
            target,
            status,
            origin,
            actf,
            kwargs,
            message,
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub client_key: String,
    pub parity_id: String,
    pub priority: u8,
    pub command: CommandInstructions,
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
    pub fn new(client_key: String, parity_id: String, priority: u8, command: CommandInstructions) -> Self {
        Self { client_key, parity_id, priority, command }
    }

    /// Converts a `DownCommand` into `Command`.
    ///
    /// This function takes a `DownCommand` and attempts to convert it into an instance of `YourStruct`.
    /// It extracts the `client_key`, `parity_id`, and `priority` fields from the `DownCommand`,
    /// and then tries to deserialize the `command` JSON string into `CommandInstructions`.
    ///
    /// # Arguments
    /// * `down_command` - A `DownCommand` instance containing the necessary information.
    ///
    /// # Returns
    /// * `Ok(YourStruct)` - If the conversion is successful.
    /// * `Err(CommandError::InvalidCommand)` - If the `command` field cannot be parsed into `CommandInstructions`.
    ///
    /// # Examples
    /// ```
    /// // Example usage
    /// let down_command = DownCommand {
    ///     client_key: "client_key_value".to_string(),
    ///     parity_id: "parity_id_value".to_string(),
    ///     priority: "priority_value".to_string(),
    ///     command: "{\"mode\":\"default\", ...}".to_string(),
    /// };
    /// let result = YourStruct::from_down_command(down_command);
    /// ```
    pub fn from_down_command(down_command: DownCommand) -> Result<Self, CommandError> {
        let client_key = down_command.client_key.clone();
        let parity_id = down_command.parity_id.clone();
        let priority = down_command.priority.clone();

        let command: CommandInstructions = match serde_json::from_str(&down_command.command) {
            Ok(c) => c,
            Err(_) => return Err(CommandError::InvalidCommand("".to_string())),
        };

        Ok(Self { client_key, parity_id, priority, command })
    }

    /// Converts an `UpCommand` into `Command`.
    ///
    /// This function takes an `UpCommand` and attempts to convert it into an instance of `YourStruct`.
    /// It extracts the `client_key`, `parity_id`, and `priority` fields from the `UpCommand`,
    /// and then tries to deserialize the `command` JSON string into `CommandInstructions`.
    ///
    /// # Arguments
    /// * `up_command` - An `UpCommand` instance containing the necessary information.
    ///
    /// # Returns
    /// * `Ok(YourStruct)` - If the conversion is successful.
    /// * `Err(CommandError::InvalidCommand)` - If the `command` field cannot be parsed into `CommandInstructions`.
    ///
    /// # Examples
    /// ```
    /// // Example usage
    /// let up_command = UpCommand {
    ///     client_key: "client_key_value".to_string(),
    ///     parity_id: "parity_id_value".to_string(),
    ///     priority: "priority_value".to_string(),
    ///     command: "{\"mode\":\"default\", ...}".to_string(),
    /// };
    /// let result = YourStruct::from_up_command(up_command);
    /// ```
    pub fn from_up_command(up_command: &UpCommand) -> Result<Self, CommandError> {
        let client_key = up_command.client_key.clone();
        let parity_id = up_command.parity_id.clone();
        let priority = up_command.priority.clone();

        let command: CommandInstructions = match serde_json::from_str(&up_command.command) {
            Ok(c) => c,
            Err(_) => return Err(CommandError::InvalidCommand("".to_string())),
        };

        println!("Client -> Command from UpCommand: {:?}", command);

        Ok(Self { client_key, parity_id, priority, command })
    }

    pub fn command_type(&self) -> CommandType {
        return self.command.command_type.clone();
    }
}
