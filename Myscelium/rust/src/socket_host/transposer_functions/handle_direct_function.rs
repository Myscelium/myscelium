use serde_json::Value;
use std::collections::HashMap;

use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::utilities::{Command, CommandInstructions, CommandMode, CommandOrigin, CommandStatus, CommandTarget, CommandType};
use crate::common::structs::avaliable_commands::CommandPatterns;
use crate::common::structs::results_structs::ResultType;
use crate::socket_client::transposer::ProcessError;

use serde::{Deserialize, Serialize};

use crate::socket_host::transposer::COMMAND_PATTERNS;

use crate::socket_host::client_manager::manager::get_all_clients;

use crate::socket_host::host_logger::log_handler::Logger;
use crate::HOST_LOG_LEVEL;

use crate::socket_host::client_manager::manager::{check_if_client_key_exists, Client, ClientError};

use crate::common::functions::converters::{convert_json_map_to_hash_map, convert_value_map_to_resulttype_map, ConversionError};

use crate::common::functions::advanced_lockers::smart_lock;
use crate::CLIENTS_SYNC_CONTROLLER;

use crate::chrono::TimeZone;
use chrono::Duration;
use chrono::Utc;

use crate::socket_host::sync_controller::controller::Clients;

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let host_log_level;
        {
            let log_level = HOST_LOG_LEVEL.lock();
            host_log_level = log_level.clone()
        }
        Logger::new(host_log_level, $section_name)
    }};
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessResult {
    Empty,
    List(Vec<ProcessResult>),
    Error(String),
    CommandInstructions(CommandInstructions),
}

pub fn handle_direct_function(client_key: &String, activation_key: &String, command: CommandInstructions, command_id: u32) -> ProcessResult {
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
                    return ProcessResult::Error(format!("Unknow client_key: {:?}", client_key));
                },
                _ => {
                    return ProcessResult::Error(format!("Get a error {:?}, obtaining client: {:?}", e, client_key));
                },
            },
        };

        let client_name: String = client.get_client_name();

        let filtered_commands = actual_patterns.get_all_commands_except_for_client(client_name.as_str());

        logger.info(format!("Successfully actualize the host available commands!"));

        // enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());

        let function: String = "update_available_host_commands".to_string();

        let new_command_instructions = CommandInstructions::new(
            CommandMode::Response,
            CommandType::Default,
            CommandTarget::Origin,
            CommandStatus::Success,
            CommandOrigin::Host,
            function,
            filtered_commands,
            "".to_string(),
        );

        return ProcessResult::CommandInstructions(new_command_instructions);
    } else if activation_key == &"update_client_commands_ref".to_string() {
        logger.info(format!("Receive update_client_commands_ref in host!"));

        // -> get the client by the client key
        let client = match Client::get_by_key(client_key) {
            Ok(c) => c,
            Err(e) => match e {
                ClientError::ClientDoesNotExist(_) => {
                    return ProcessResult::Error(format!("Unknow client_key: {:?}", client_key));
                },
                _ => {
                    return ProcessResult::Error(format!("Get a error {:?}, obtaining client: {:?}", e, client_key));
                },
            },
        };

        let client_handlers;

        // Check if 'client_handlers' exists within 'kwargs'
        if let Some(Value::Object(handlers)) = command.kwargs.get("client_handlers") {
            client_handlers = handlers;
        } else {
            return ProcessResult::Error(format!("update_client_commands_ref give the followign error: The 'client_handlers' key does not exist within 'kwargs'."));
        }

        // } else {
        //     return ResultType::Error(format!("update_client_commands_ref command doesn't have kwargs in it!"));
        // }

        let client_name: String = client.get_client_name();

        let actual_patterns = &COMMAND_PATTERNS;
        smart_lock(&*actual_patterns, |patterns: &mut CommandPatterns| {
            patterns.add_or_update_if_exists(client_name.as_str(), convert_json_map_to_hash_map(client_handlers))
        });

        let controller = &CLIENTS_SYNC_CONTROLLER;
        smart_lock(&*controller, |clients: &mut Clients| {
            let status = clients.update_client_sync_status(client_key, true);
            // TODO >>> Add a mechanism to set all the other clients state to sync = false
        });

        // -> Send the commands for the first client:

        // {
        //     let mut filtered_commands: HashMap<String, Value> = HashMap::new();

        //     let actual_patterns = &COMMAND_PATTERNS;
        //     smart_lock(&*actual_patterns, |patterns: &mut CommandPatterns| {
        //         filtered_commands = patterns.get_all_commands_except_for_client(client_name.as_str());
        //     });

        //     let filtered_resulttype_commands_map = match convert_value_map_to_resulttype_map(&filtered_commands) {
        //         Ok(c) => c,
        //         Err(e) => match e {
        //             ConversionError::UnsuportedValueVariant(s) => {
        //                 logger.warn(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
        //                 return ResultType::Error(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
        //             },
        //         },
        //     };

        //     let mut to_send = HashMap::new();

        //     to_send.insert("command_type".to_string(), ResultType::Str("direct_function".to_string()));
        //     to_send.insert("response_mode".to_string(), ResultType::Str("to_host".to_string()));
        //     to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
        //     to_send.insert("function".to_string(), ResultType::Str("update_available_host_commands".to_string())); // TODO maybe change to response_act_function
        //     to_send.insert("kwargs".to_string(), filtered_resulttype_commands_map);
        //     to_send.insert("origin".to_string(), ResultType::Str(client_key.clone())); // -> This will be an identifier, to know the origin of the retransmited command

        //     response.push(ResultType::Map(to_send));
        // }

        // -> Try to get the clients registred in the database
        let mut clients = match get_all_clients() {
            Ok(c) => c,
            Err(e) => match e {
                _ => {
                    // TODO >>> Create a better error handling for this, there is no need to return this to any client

                    let new_command_instructions = CommandInstructions::new(
                        CommandMode::Function,
                        CommandType::DirectFunction,
                        CommandTarget::Origin,
                        CommandStatus::Failure,
                        CommandOrigin::Host,
                        "update_available_host_commands".to_string(),
                        HashMap::new(),
                        "unexpect error getting clients to redirect the update commands".to_string(),
                    );

                    return ProcessResult::CommandInstructions(new_command_instructions);
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

        let mut responses: Vec<ProcessResult> = Vec::new();

        logger.info(format!("Receive client: {} handlers, retransmitting to: {:?}", client_key, clients).to_string());

        // Generate confirmation to triggering client
        let new_command_instructions = CommandInstructions::new(
            CommandMode::Response,
            CommandType::SpecialFunction,
            CommandTarget::Origin,
            CommandStatus::Success,
            CommandOrigin::Host,
            "C210".to_string(),
            HashMap::new(),
            "".to_string(),
        );

        responses.push(ProcessResult::CommandInstructions(new_command_instructions));

        // -> Send the updated info for all the clients
        for client in clients {
            //> See if client has some alive signal in the last 30s:

            // Split into seconds and nanoseconds
            let seconds = client.last_contact.trunc() as i64;
            let nanoseconds = (client.last_contact.fract() * 1e9).round() as u32; // Or *1_000_000_000.0

            // Convert to DateTime<Utc>
            let last_contact = Utc.timestamp_opt(seconds, nanoseconds).unwrap();

            let current_time = Utc::now();
            if current_time - last_contact > Duration::seconds(30) {
                continue;
            }

            //> Redirect new commands to client if changed:

            let mut filtered_commands: HashMap<String, Value> = HashMap::new();

            // TODO >>> Add a mechanism to see what handlers the client will ahve permission to activate
            //* Any mechanism that will see the client permissions to each command may be placed here

            let actual_patterns = &COMMAND_PATTERNS;
            smart_lock(&*actual_patterns, |patterns: &mut CommandPatterns| {
                filtered_commands = patterns.get_all_commands_except_for_client(client_name.as_str());
            });

            // > Schedule a redirect to the other clients
            let client_key_to_redirect: String = client.client_key.clone();

            let new_command_instructions = CommandInstructions::new(
                CommandMode::Response,
                CommandType::DirectFunction,
                CommandTarget::ClientKey(client_key_to_redirect),
                CommandStatus::Success,
                CommandOrigin::Host,
                "update_available_host_commands".to_string(),
                filtered_commands,
                "".to_string(),
            );

            responses.push(ProcessResult::CommandInstructions(new_command_instructions));

            return ProcessResult::List(responses);
        }

        return ProcessResult::List(responses);
    }

    return ProcessResult::Error(format!("unknow direct function"));
}
