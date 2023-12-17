use crate::common::structs::avaliable_commands::CommandPatterns;
use crate::common::structs::results_structs::ResultType;
use crate::socket_client::socket_client::COMMAND_PATTERNS;
use crate::socket_client::transposer::ProcessError;
use serde_json::Value;
use std::collections::HashMap;

use crate::socket_client::client_logger::log_handler::Logger;
use crate::CLIENT_LOG_LEVEL;

use crate::socket_client::transposer::HOST_ALLOWED_COMMANDS;

use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::utilities::{Command, CommandType};
use crate::common::functions::advanced_lockers::smart_lock;
use crate::common::functions::converters::convert_value_map_to_resulttype_map;
use crate::common::functions::converters::ConversionError;

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let client_log_level;
        {
            client_log_level = CLIENT_LOG_LEVEL.lock().clone();
        }
        Logger::new(client_log_level, $section_name)
    }};
}

pub fn handle_direct_function(client_key: String, activation_key: &String, translated_command: Command, command_id: u32) -> ResultType {
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

            let host_allowed_commands = &HOST_ALLOWED_COMMANDS;
            smart_lock(&*host_allowed_commands, |actual_patterns: &mut CommandPatterns| {
                *actual_patterns = new_patterns;
            });

            logger.info(format!("Successfully actualize the host available commands!"));

            let mut actual_patterns: HashMap<String, Value> = HashMap::new();

            let command_patterns = &COMMAND_PATTERNS;
            smart_lock(&*command_patterns, |patterns: &mut CommandPatterns| {
                actual_patterns = patterns.extract_all_commands();
            });

            // {
            //     actual_patterns = COMMAND_PATTERNS.lock().clone();
            // }

            logger.info(format!("Successfully actualize the host available commands!"));

            enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());

            let handlers = match convert_value_map_to_resulttype_map(&actual_patterns) {
                Ok(c) => c,
                Err(e) => match e {
                    ConversionError::UnsuportedValueVariant(s) => {
                        logger.warn(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
                        return ResultType::Error(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
                    },
                },
            };

            let mut filtered_resulttype_commands_map = HashMap::new();

            filtered_resulttype_commands_map.insert("client_handlers".to_string(), handlers);

            let function: String = "update_client_commands_ref".to_string();

            let mut to_send = HashMap::new();

            to_send.insert("command_type".to_string(), ResultType::Str("direct_function_response".to_string()));
            to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
            to_send.insert("function".to_string(), ResultType::Str(function)); // -> Function that it will act in host
            to_send.insert("kwargs".to_string(), ResultType::Map(filtered_resulttype_commands_map));
            to_send.insert("origin".to_string(), ResultType::Str(client_key.clone())); // -> This will be an identifier, to know the origin of the retransmited command
            to_send.insert("response_mode".to_string(), ResultType::Str("to_host".to_string())); // -> This is necessary to send this response back to host

            return ResultType::Map(to_send);
        } else {
            return ResultType::Error(format!("missing kwargs key {:?}", translated_command.clone()));
        }
    }

    // Special handling for "update available host commands" command
    if activation_key == &"get_socket_client_available_handlers".to_string() {
        logger.info(format!("Receive Available Handlers Request"));

        // Lock the COMMAND_PATTERNS and insert the new map

        let mut actual_patterns: HashMap<String, Value> = HashMap::new();

        let command_patterns = &COMMAND_PATTERNS;
        smart_lock(&*command_patterns, |patterns: &mut CommandPatterns| {
            actual_patterns = patterns.extract_all_commands();
        });

        logger.info(format!("Successfully actualize the host available commands!"));

        enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());

        let handlers = match convert_value_map_to_resulttype_map(&actual_patterns) {
            Ok(c) => c,
            Err(e) => match e {
                ConversionError::UnsuportedValueVariant(s) => {
                    logger.warn(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
                    return ResultType::Error(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
                },
            },
        };

        let mut filtered_resulttype_commands_map = HashMap::new();

        filtered_resulttype_commands_map.insert("client_handlers".to_string(), handlers);

        let function: String = "update_client_commands_ref".to_string();

        let mut to_send = HashMap::new();

        to_send.insert("command_type".to_string(), ResultType::Str("direct_function_response".to_string()));
        to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
        to_send.insert("function".to_string(), ResultType::Str(function)); // -> Function that it will act in host
        to_send.insert("kwargs".to_string(), ResultType::Map(filtered_resulttype_commands_map));
        to_send.insert("origin".to_string(), ResultType::Str(client_key.clone())); // -> This will be an identifier, to know the origin of the retransmited command
        to_send.insert("response_mode".to_string(), ResultType::Str("to_host".to_string())); // -> This is necessary to send this response back to host

        return ResultType::Map(to_send);
    }

    return ResultType::Error(format!("Command: {:?} not found!", translated_command.clone()));
}
