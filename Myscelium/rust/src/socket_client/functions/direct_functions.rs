use crate::common::structs::avaliable_commands::CommandPatterns;
use crate::common::structs::results_structs::ResultType;
use crate::socket_client::socket_client::COMMAND_PATTERNS;
use crate::socket_client::transposer::ProcessError;
use serde_json::Value;
use std::collections::HashMap;

use crate::socket_client::client_logger::log_handler::Logger;
use crate::CLIENT_LOG_LEVEL;

use crate::socket_client::transposer::HOST_ALLOWED_COMMANDS;

use crate::common::enhanced_buffer::utilities::{Command, CommandType};

use crate::common::enhanced_buffer;

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let client_log_level;
        {
            client_log_level = CLIENT_LOG_LEVEL.lock().clone();
        }
        Logger::new(client_log_level, $section_name)
    }};
}

pub fn handle_direct_function(activation_key: &String, translated_command: Command, command_id: u32) -> Option<ResultType> {
    let logger = acquire_logger!("Transposer - Process");

    logger.info(format!("Initializing processing!"));

    // Special handling for "update available host commands" command
    if activation_key == &"update_available_host_commands".to_string() {
        logger.info(format!("Receive Host Allowed Commands"));

        if let Some(Value::Object(response_obj)) = translated_command.command.get("kwargs") {
            // Clone the object to get a HashMap<String, Value>
            let response_map: HashMap<String, Value> = response_obj.clone().into_iter().collect();

            // Lock the COMMAND_PATTERNS and insert the new map

            let mut new_patterns = CommandPatterns::new();
            new_patterns.add_from_map(response_map);

            {
                let mut actual_patterns = HOST_ALLOWED_COMMANDS.lock().unwrap();
                *actual_patterns = new_patterns;
            }

            logger.info(format!("Successfully actualize the host available commands!"));

            enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());

            return None;
        } else {
            return Some(ResultType::Error(format!("missing kwargs key {:?}", translated_command.clone())));
        }
    }

    // Special handling for "update available host commands" command
    if activation_key == &"get_socket_client_available_handlers".to_string() {
        logger.info(format!("Receive Available Handlers Request"));

        if let Some(Value::Object(response_obj)) = translated_command.command.get("kwargs") {
            // Clone the object to get a HashMap<String, Value>
            let response_map: HashMap<String, Value> = response_obj.clone().into_iter().collect();

            // Lock the COMMAND_PATTERNS and insert the new map

            let actual_patterns;

            {
                actual_patterns = COMMAND_PATTERNS.lock().clone();
            }

            logger.info(format!("Successfully actualize the host available commands!"));

            enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());

            return None;
        } else {
            return Some(ResultType::Error(format!("missing kwargs key {:?}", translated_command.clone())));
        }
    }

    return Some(ResultType::Error(format!("Command: {:?} not found!", translated_command.clone())));
}
