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

use crate::handle_client_error;
use crate::socket_host::transposer_functions::helpers::cast_new_client;

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

pub fn handle_direct_function(client_key: &String, activation_key: &String, command: CommandInstructions, command_id: Option<u32>) -> ProcessResult {
    let logger = acquire_logger!("Transposer - Process - Handle Direct Functions");

    logger.info(format!("Initializing processing!"));

    logger.debug(format!("function received in handle direct function: {}", activation_key));

    // -> ----------------------------------------------------------------------------------------------------------------------------------
    // -> SYNCRONIZATION MECHANISM

    match activation_key.as_str() {
        "get_registered_commands" => {
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
                CommandType::ExternalFunction,
                CommandTarget::Origin,
                CommandStatus::Success,
                CommandOrigin::Host,
                function,
                filtered_commands,
                "".to_string(),
            );

            return ProcessResult::CommandInstructions(new_command_instructions);
        },
        "update_client_commands_ref" => {
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
        },
        "add_client" => {
            // > edit client
            // {'response_mode':'InternalManagement', 'activation_function':'add_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'new_client':clientpattern}

            // if !command.kwargs.contains_key("client_key") {
            //     logger.warn("Error! Callback response kwargs don't have client_key kwarg!".to_string());
            //     return ProcessResult::Error(format!("Error! Callback response kwargs don't have client_key kwarg!"));
            // }

            // if !command.kwargs.contains_key("new_client") {
            //     logger.warn("Error! Callback response kwargs don't have new_client kwarg!".to_string());
            //     return ProcessResult::Error(format!("Error! Callback response kwargs don't have new_client kwarg!"));
            // }

            // let client_key = command.kwargs.get("client_key").unwrap().as_str().unwrap();

            // from("client_name":"str", "client_key":"str", "client_type":"str", "permission_group":"str", "is_super_user":"bool", "max_sub_channels":"int", "owned_sub_channels_keys":"list")

            let new_client = match cast_new_client(&command.kwargs) {
                Ok(c) => c,
                Err(e) => return e, // TODO >>> Fix this error case
            };

            new_client.save_into_db(); //> It Already create the new client

            logger.debug("New client saved into the database!".to_string());

            // TODO >>> Make a verification if the client already exists or not before add it!
            // let mut resp_kwargs: HashMap<String, Value> = HashMap::new();
            // resp_kwargs.insert("client_key".to_string(), Value::String(client_key.to_string()));

            let new_command_instructions: CommandInstructions = CommandInstructions::new(
                CommandMode::Response,
                CommandType::ExternalFunction,
                CommandTarget::Origin,
                CommandStatus::Success,
                CommandOrigin::Host,
                "add_client_handler".to_string(),
                HashMap::new(),
                format!("Successfully add a client: {}!", new_client.client_key).to_string(),
            );

            logger.info(format!("Successfully add a client: {}!", new_client.client_key));

            return ProcessResult::CommandInstructions(new_command_instructions);
        },
        "update_client" => {
            // > update client
            // {'response_mode':'InternalManagement', 'activation_function':'update_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'actual_client_key':String, 'updated_client':client} // Client have to have the same client key
            // 'client': {"client_name":str, "client_key":str, "client_type":str, "permission_group":str, "is_super_user":bool, "max_sub_channels":int, "owned_sub_channels_keys":list}

            logger.debug("Receive a update client inner command!".to_string());

            if !&command.kwargs.contains_key("actual_client_key") {
                logger.warn("Error! Callback response kwargs don't have actual_client_key kwarg!".to_string());
                return ProcessResult::Error(format!("Error! Callback response kwargs don't have actual_client_key kwarg!"));
            }

            if !&command.kwargs.contains_key("updated_client") {
                logger.warn("ERROR, Error! Callback response kwargs don't have update_client kwarg!".to_string());
                return ProcessResult::Error(format!("Error! Callback response kwargs don't have update_client kwarg!"));
            }

            let actual_client_key = &command.kwargs.get("actual_client_key").unwrap().as_str().unwrap().clone();

            // from("client_name":"str", "client_key":"str", "client_type":"str", "permission_group":"str", "is_super_user":"bool", "max_sub_channels":"int", "owned_sub_channels_keys":"list")

            let new_client = match cast_new_client(&command.kwargs) {
                Ok(c) => c,
                Err(e) => return e,
            };

            let old_client = handle_client_error!(Client::get_by_key(&actual_client_key.to_string()));

            let result = old_client.update_to(&new_client); //> It already saves into the database

            // TODO >>> Maybe implement a fast result-ype to client if needed

            match result {
                Ok(_) => {
                    let mut resp_kwargs: HashMap<String, Value> = HashMap::new();

                    resp_kwargs.insert("actual_client_key".to_string(), Value::String(actual_client_key.to_string())); // TODO >>> See if this actual client key is correct

                    let new_command_instructions: CommandInstructions = CommandInstructions::new(
                        CommandMode::Response,
                        CommandType::ExternalFunction,
                        CommandTarget::Origin,
                        CommandStatus::Success,
                        CommandOrigin::Host,
                        "update_client_handler".to_string(),
                        resp_kwargs,
                        format!("Successfully executed the function: {} and remove client: {}!", activation_key, old_client.client_key).to_string(),
                    );

                    logger.info(format!("Successfully executed the function: {} and remove client: {}!", activation_key, old_client.client_key));

                    return ProcessResult::CommandInstructions(new_command_instructions);
                },

                Err(e) => match e {
                    ClientError::ClientDoesNotExist(e) => {
                        logger.warn(format!("Error! Can't Update client because client {} Don't exist!", e));
                        return ProcessResult::Error(format!("Error! Can't Update client because client {} Don't exist!", e));
                    },
                    _ => {
                        logger.warn("Error! Can Update client because a unexpected error!".to_string());
                        return ProcessResult::Error(format!("Error! Can Update client because a unexpected error!"));
                    },
                },
            }

            // TODO >>> Implement a mechanism to send back the confirmation or a error message originated from the operation
            // else {
            //     logger.warn("Error! Callback response kwargs isn't a Map!".to_string());
            //     return create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            // }
        },
        "remove_client" => {
            // > remove client
            // {'response_mode':'InternalManagement', 'activation_function':'remove_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'client_key':String}

            if !command.kwargs.contains_key("client_key") {
                return ProcessResult::Error(format!("Error! Callback response kwargs don't have client_key kwarg!"));
            }

            let client_key: String = command.kwargs.get("client_key").unwrap().as_str().map(|s| s.to_string()).unwrap();

            let client = handle_client_error!(Client::get_by_key(&client_key));

            let result = client.delete();

            match result {
                Err(e) => match e {
                    ClientError::ClientDoesNotExist(e) => {
                        logger.warn(format!("Error! Can't Remove client because client {} Don't exist!", e));
                        return ProcessResult::Error(format!("Error! Can't Remove client because client {} Don't exist!", e));
                    },
                    _ => {
                        logger.warn("Error! Can Remove client because a unexpected error!".to_string());
                        return ProcessResult::Error(format!("Error! Can Remove client because a unexpected error!"));
                    },
                },
                Ok(_) => {
                    // let mut resp_kwargs: HashMap<String, Value> = HashMap::new();
                    // resp_kwargs.insert("actual_client_key".to_string(), Value::String(client_key.to_string())); // TODO >>> See if this actual client key is correct

                    let new_command_instructions: CommandInstructions = CommandInstructions::new(
                        CommandMode::Response,
                        CommandType::ExternalFunction,
                        CommandTarget::Origin,
                        CommandStatus::Success,
                        CommandOrigin::Host,
                        "remove_client_handler".to_string(),
                        HashMap::new(),
                        format!("Successfully executed the function: {} and remove client: {}!", activation_key, client_key).to_string(),
                    );

                    logger.info(format!("Successfully executed the function: {} and remove client: {}!", activation_key, client_key));

                    return ProcessResult::CommandInstructions(new_command_instructions);
                },
            }
            // else {
            //     logger.warn("Error! Callback response kwargs isn't a Map!".to_string());
            //     return create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            // }

            // TODO >>> Implement a mechanism to send back the confirmation or a error message originated from the operation
        },
        _ => {
            return ProcessResult::Error(format!("unknow direct function"));
        },
    }
}
