use serde_json::Value;
use std::collections::HashMap;

use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::utilities::Command;
use crate::common::structs::results_structs::ResultType;
use crate::socket_client::transposer::ProcessError;

use crate::socket_host::transposer::COMMAND_PATTERNS;

use crate::socket_host::client_manager::manager::get_all_clients;

use crate::socket_host::host_logger::log_handler::Logger;
use crate::HOST_LOG_LEVEL;

use crate::socket_host::client_manager::manager::{check_if_client_key_exists, Client, ClientError};

use crate::common::functions::converters::{convert_json_map_to_hash_map, convert_value_map_to_resulttype_map, ConversionError};

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let host_log_level;
        {
            host_log_level = HOST_LOG_LEVEL.lock().clone();
        }
        Logger::new(host_log_level, $section_name)
    }};
}

pub fn handle_direct_function(client_key: &String, activation_key: &String, command: HashMap<String, Value>, command_id: u32) -> ResultType {
    let logger = acquire_logger!("Transposer - Process - Handle Direct Functions");

    logger.info(format!("Initializing processing!"));

    // Special handling for "update available host commands" command
    if activation_key == &"get_registered_commands".to_string() {
        logger.info(format!("Receive get_registered_commands in host!"));

        // Lock the COMMAND_PATTERNS and insert the new map

        let actual_patterns;

        {
            actual_patterns = COMMAND_PATTERNS.lock().unwrap().clone();
        }

        // -> get the client by the client key
        let client = match Client::get_by_key(client_key) {
            Ok(c) => c,
            Err(e) => match e {
                ClientError::ClientDoesNotExist(_) => {
                    return ResultType::Error(format!("unknow client_key: {:?}", client_key));
                },
                _ => {
                    return ResultType::Error(format!("Get a error {:?}, obtaining client: {:?}", e, client_key));
                },
            },
        };

        let client_name: String = client.get_client_name();

        let filtered_commands = actual_patterns.get_all_commands_except_for_client(client_name.as_str());

        let filtered_resulttype_commands_map = match convert_value_map_to_resulttype_map(&filtered_commands) {
            Ok(c) => c,
            Err(e) => match e {
                ConversionError::UnsuportedValueVariant(s) => {
                    logger.warn(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
                    return ResultType::Error(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
                },
            },
        };

        logger.info(format!("Successfully actualize the host available commands!"));

        enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());

        // TODO >>> See what is the correct response in this stage

        let function: String = "update_available_host_commands".to_string();

        let mut to_send = HashMap::new();

        to_send.insert("command_type".to_string(), ResultType::Str("function".to_string()));
        to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string())); // TODO See if need it
        to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
        to_send.insert("function".to_string(), ResultType::Str(function.to_string())); // TODO maybe change to response_act_function
        to_send.insert("kwargs".to_string(), filtered_resulttype_commands_map);
        to_send.insert("origin".to_string(), ResultType::Str("host".to_string())); // -> This will be an identifier, to know the origin of the retransmited command

        return ResultType::Map(to_send);
    } else if activation_key == &"update_client_commands_ref".to_string() {
        logger.info(format!("Receive update_client_commands_ref in host!"));

        // -> get the client by the client key
        let client = match Client::get_by_key(client_key) {
            Ok(c) => c,
            Err(e) => match e {
                ClientError::ClientDoesNotExist(_) => {
                    return ResultType::Error(format!("unknow client_key: {:?}", client_key));
                },
                _ => {
                    return ResultType::Error(format!("Get a error {:?}, obtaining client: {:?}", e, client_key));
                },
            },
        };

        let client_handlers;

        // Check if 'kwargs' exists and is an object
        if let Some(Value::Object(kwargs_map)) = command.get("kwargs") {
            // Check if 'client_handlers' exists within 'kwargs'
            if let Some(Value::Object(handlers)) = kwargs_map.get("client_handlers") {
                client_handlers = handlers;
            } else {
                return ResultType::Error(format!("update_client_commands_ref give the followign error: The 'client_handlers' key does not exist within 'kwargs'."));
            }
        } else {
            return ResultType::Error(format!("update_client_commands_ref command doesn't have kwargs in it!"));
        }

        let client_name: String = client.get_client_name();

        {
            let mut actual_patterns = COMMAND_PATTERNS.lock().unwrap();
            actual_patterns.add_or_update_if_exists(client_name.as_str(), convert_json_map_to_hash_map(client_handlers))
        }

        let status = client.change_sync_to(true);

        // TODO >>> Create a mechanims to dont send anything to the other clients in case of the new client doesn't bring anything new as handlers

        let mut response: Vec<ResultType> = Vec::new();

        // -> Send the commands for the first client:

        {
            let filtered_commands;

            {
                let actual_patterns = COMMAND_PATTERNS.lock().unwrap();
                filtered_commands = actual_patterns.get_all_commands_except_for_client(client_name.as_str());
            }

            let filtered_resulttype_commands_map = match convert_value_map_to_resulttype_map(&filtered_commands) {
                Ok(c) => c,
                Err(e) => match e {
                    ConversionError::UnsuportedValueVariant(s) => {
                        logger.warn(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
                        return ResultType::Error(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
                    },
                },
            };

            let mut to_send = HashMap::new();

            to_send.insert("command_type".to_string(), ResultType::Str("direct-function".to_string()));
            to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
            to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
            to_send.insert("function".to_string(), ResultType::Str("update_available_host_commands".to_string())); // TODO maybe change to response_act_function
            to_send.insert("kwargs".to_string(), filtered_resulttype_commands_map);
            to_send.insert("origin".to_string(), ResultType::Str(client_key.clone())); // -> This will be an identifier, to know the origin of the retransmited command

            response.push(ResultType::Map(to_send));
        }

        // -> Try to get the clients registred in the database
        let mut clients = match get_all_clients() {
            Ok(c) => c,
            Err(e) => match e {
                _ => {
                    let empty_map = HashMap::new();

                    let mut to_send = HashMap::new();

                    to_send.insert("command_type".to_string(), ResultType::Str("direct-function".to_string()));
                    to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
                    to_send.insert("status".to_string(), ResultType::Str("error".to_string()));
                    to_send.insert("message".to_string(), ResultType::Str("unexpect error getting clients to redirect the update commands".to_string()));
                    to_send.insert("function".to_string(), ResultType::Str("update_available_host_commands".to_string())); // TODO maybe change to response_act_function
                    to_send.insert("kwargs".to_string(), ResultType::Map(empty_map));
                    to_send.insert("origin".to_string(), ResultType::Str("host".to_string())); // -> This will be an identifier, to know the origin of the retransmited command

                    return ResultType::Map(to_send);
                },
            },
        };

        // -> Filter the actual client from the list cause it alwready was handled
        for (index, client) in clients.iter().enumerate() {
            if client.client_key == client_key.clone() {
                clients.remove(index);
                break;
            }
        }

        // -> Send the updated info for all the clients
        for client in clients {
            let filtered_commands;

            //* Any mechanism that will see the client permissions to each command may be placed here

            {
                let actual_patterns = COMMAND_PATTERNS.lock().unwrap();
                filtered_commands = actual_patterns.get_all_commands_except_for_client(client_name.as_str());
            }

            let filtered_resulttype_commands_map = match convert_value_map_to_resulttype_map(&filtered_commands) {
                Ok(c) => c,
                Err(e) => match e {
                    ConversionError::UnsuportedValueVariant(s) => {
                        logger.warn(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
                        return ResultType::Error(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
                    },
                },
            };

            let mut to_send = HashMap::new();

            // > Schedule a redirect to the other clients

            let client_key_to_redirect: String = client.client_key.clone();

            to_send.insert("command_type".to_string(), ResultType::Str("direct-function".to_string()));
            to_send.insert("response_mode".to_string(), ResultType::Str("redirect".to_string()));
            to_send.insert("redirect_to".to_string(), ResultType::Str(client_key_to_redirect.to_string()));
            to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
            to_send.insert("function".to_string(), ResultType::Str("update_available_host_commands".to_string())); // TODO maybe change to response_act_function
            to_send.insert("kwargs".to_string(), filtered_resulttype_commands_map);
            to_send.insert("origin".to_string(), ResultType::Str("host".to_string())); // -> This will be an identifier, to know the origin of the retransmited command

            response.push(ResultType::Map(to_send));
        }

        return ResultType::List(response);
    }

    return ResultType::Error(format!("unknow direct function"));
}
