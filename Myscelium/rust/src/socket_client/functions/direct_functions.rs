use crate::common::structs::available_commands::CommandPatterns;
use crate::common::structs::results_structs::ResultType;
use crate::socket_client::socket_client::COMMAND_PATTERNS;
use crate::socket_client::transposer::ProcessError;
use crate::socket_host::transposer_functions::handle_direct_function::ProcessResult;
use serde_json::{to_string, Value};
use std::collections::HashMap;

use crate::socket_client::client_logger::log_handler::Logger;
use crate::{CLIENT_IS_SYNC, CLIENT_LOG_LEVEL};

use crate::socket_client::transposer::HOST_ALLOWED_COMMANDS;

use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::utilities::{Command, CommandInstructions, CommandMode, CommandOrigin, CommandStatus, CommandTarget, CommandType};
use crate::common::functions::advanced_lockers::smart_lock;
use crate::common::functions::converters::convert_to_value_map;
use crate::common::functions::converters::convert_value_map_to_resulttype_map;
use crate::common::functions::converters::ConversionError;
use crate::socket_client::functions::direct_functions::enhanced_buffer::buffer_up_manager::UpCommand;

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let client_log_level;
        {
            let log_level = CLIENT_LOG_LEVEL.lock().clone();
            client_log_level = log_level.clone();
        }
        Logger::new(client_log_level, $section_name)
    }};
}

pub fn handle_direct_function(c: &CommandInstructions, client_key: &String, command_id: u32) -> Result<ProcessResult, ProcessError> {
    let logger = acquire_logger!("Transposer - Process");

    logger.info(format!("Initializing Direct Function processing!"));

    // TODO >>> Change this for a mathc

    // -> SESSION SYNCHRONIZATION

    match c.actf.as_str() {
        "update_available_host_commands" => {
            logger.info(format!("Receive Host Allowed Commands"));

            // Clone the object to get a HashMap<String, Value>
            let response_map: HashMap<String, Value> = c.kwargs.clone();

            // TODO >>> Maybe create a mechanism to validate the new_patterns received, maybe using regex, idk...

            // Lock the COMMAND_PATTERNS and insert the new map
            let mut new_patterns = CommandPatterns::new();
            new_patterns.add_from_map(response_map);

            println!("[CLIENT][GLOBAL][Try Lock] - HOST_ALLOWED_COMMANDS");

            {
                let mut host_allowed_commands = HOST_ALLOWED_COMMANDS.lock();
                println!("[CLIENT][GLOBAL][Lock] - HOST_ALLOWED_COMMANDS");
                *host_allowed_commands = new_patterns.clone();
            }
            println!("[CLIENT][GLOBAL][Release] - HOST_ALLOWED_COMMANDS");

            let actual_patterns: HashMap<String, Value>;

            println!("[CLIENT][GLOBAL][Try Lock] - COMMAND_PATTERNS");
            {
                let command_patterns = COMMAND_PATTERNS.lock();

                println!("[CLIENT][GLOBAL][Lock] - COMMAND_PATTERNS");

                logger.info(format!("Lock In Host Command Patterns!"));

                actual_patterns = command_patterns.extract_all_commands().clone();
            }
            println!("[CLIENT][GLOBAL][Release] - COMMAND_PATTERNS");

            logger.info(format!("Successfully actualize the host available commands!"));

            let mut filtered_commands_map = HashMap::new();
            filtered_commands_map.insert("client_handlers".to_string(), Value::Object(serde_json::Map::from_iter(actual_patterns)));

            //> VERIFY IF IS CLIENT FIRST SYNC
            if !CLIENT_IS_SYNC.load(Ordering::SeqCst) {
                // -> Only return this 'update_client_commands_ref' in case that is the first sync of the client

                // TODO >>> Maybe change this to return the command instead of schedule it manually to send to host
                let new_command_instructions = CommandInstructions::new(
                    CommandMode::Function,
                    CommandType::DirectFunction,
                    CommandTarget::Host,
                    CommandStatus::Success,
                    CommandOrigin::ClientKey(client_key.clone()),
                    "update_client_commands_ref".to_string(),
                    filtered_commands_map,
                    "".to_string(),
                );

                // > This need to be scheduled this way since this is a new command and need a new parity id, if return this will use the parity id received
                // TODO >>> A possible way to do this is by call the schedule instead of schedule by hand, maybe is a better option to avoid code repetition

                let parity_id = enhanced_buffer::buffer_up_manager::buffer_up_gen_valid_parity_id(client_key.clone());
                let up_command: UpCommand = UpCommand::new(client_key, &parity_id, 11u8, &to_string(&new_command_instructions).unwrap());
                enhanced_buffer::buffer_up_manager::buffer_up_schedule(up_command);
            }

            //> TURN CLIENT SYNC STATUS TO TRUE
            CLIENT_IS_SYNC.store(true, Ordering::SeqCst);

            enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());
            return Ok(ProcessResult::Empty);
        },
        "get_socket_client_available_handlers" => {
            logger.info(format!("Receive Available Handlers Request"));

            // Lock the COMMAND_PATTERNS and insert the new map

            let actual_patterns: HashMap<String, Value>;

            println!("[CLIENT][GLOBAL][Try Lock] - COMMAND_PATTERNS");
            {
                let command_patterns = COMMAND_PATTERNS.lock();
                println!("[CLIENT][GLOBAL][Lock] - COMMAND_PATTERNS");
                actual_patterns = command_patterns.extract_all_commands().clone();
            }
            println!("[CLIENT][GLOBAL][Release] - COMMAND_PATTERNS");

            logger.info(format!("Successfully actualize the host available commands!"));

            enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());

            let mut filtered_commands_map: HashMap<String, Value> = HashMap::new();
            filtered_commands_map.insert("client_handlers".to_string(), Value::Object(serde_json::Map::from_iter(actual_patterns)));

            let new_command_instructions = CommandInstructions::new(
                CommandMode::Response,
                CommandType::DirectFunction,
                CommandTarget::Host,
                CommandStatus::Success,
                CommandOrigin::ClientKey(client_key.clone()),
                "update_client_commands_ref".to_string(),
                filtered_commands_map,
                "".to_string(),
            ); // TODO >>> Maybe change this to return the command instead of schedule it manually to send to host

            let parity_id = enhanced_buffer::buffer_up_manager::buffer_up_gen_valid_parity_id(client_key.clone());
            let up_command: UpCommand = UpCommand::new(client_key, &parity_id, 11u8, &to_string(&new_command_instructions).unwrap());
            enhanced_buffer::buffer_up_manager::buffer_up_schedule(up_command);
            enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());

            enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());

            return Ok(ProcessResult::Empty);
        },

        // -> VITAL NETWORK COMPONENTS
        "update_client_network_rechable" => {},

        // -> GENERAL OUT OF SCOPE CASES:
        _ => {
            return Err(ProcessError::CommandNotRegistered(c.actf.clone()));
        },
    }
}
