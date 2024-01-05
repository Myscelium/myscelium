use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::buffer_down_manager::DownCommand;
use crate::common::enhanced_buffer::buffer_up_manager::UpCommand;
use crate::common::enhanced_buffer::utilities::{Command, CommandInstructions, CommandMode, CommandOrigin, CommandStatus, CommandTarget, CommandType};
use crate::common::enhanced_buffer::utilities::{Command, CommandInstructions, CommandType};
use crate::common::functions::converters::convert_value_map_to_resulttype_map;
use crate::common::functions::converters::ConversionError;
use crate::common::structs::results_structs::ResultType;
use crate::socket_host::client_manager::manager::{check_if_client_key_exists, Client, ClientError};

use lazy_static::lazy_static;
use serde_json::{from_str, Value};
use std::collections::HashMap;

use crate::socket_host::transposer::COMMAND_PATTERNS;
use crate::CLIENT_ID;

use serde_json::Error;

use crate::common::functions::advanced_lockers::smart_lock;
use crate::common::structs::avaliable_commands::CommandPatterns;

use super::host_logger::log_handler::Logger;
use crate::socket_host::transposer::process_map_result;
use crate::HOST_LOG_LEVEL;

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let client_log_level;
        {
            client_log_level = HOST_LOG_LEVEL.lock().clone();
        }
        Logger::new(client_log_level, $section_name)
    }};
}

/// Sets the global client ID to the specified value.
///
/// The client ID is a unique identifier that represents the client in the communication process.
/// This function updates the global `CLIENT_ID` variable to the provided value.
///
/// # Arguments
/// - `client_uid`: The new client ID to be set.
// pub fn set_host_id(client_uid: String) { // -> Future impl
//     println!("Setting client_key to: {:?}", client_uid.clone());

//     {
//         let mut client_id_global = CLIENT_ID.lock();
//         *client_id_global = client_uid.clone();
//     }
// }

/// Requests the available commands that are registered on the host.
///
/// This function prepares a command request for the host to retrieve the list of
/// registered commands. The constructed request is then scheduled for processing.
pub fn request_client_available_commands(client_key: String) {
    let mut request_host_commands: HashMap<String, ResultType> = HashMap::new();
    request_host_commands.insert("command_type".to_string(), ResultType::Str("direct_function".to_string()));
    request_host_commands.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
    request_host_commands.insert("status".to_string(), ResultType::Str("success".to_string()));
    request_host_commands.insert("function".to_string(), ResultType::Str("get_socket_client_available_handlers".to_string()));
    request_host_commands.insert("origin".to_string(), ResultType::Str("host".to_string()));
    request_host_commands.insert("kwargs".to_string(), ResultType::Map(HashMap::new()));

    // let mut to_send = HashMap::new();

    // to_send.insert("command_type".to_string(), ResultType::Str("direct_function".to_string()));
    // to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
    // to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
    // to_send.insert("function".to_string(), ResultType::Str("update_available_host_commands".to_string())); // TODO maybe change to response_act_function
    // to_send.insert("kwargs".to_string(), filtered_resulttype_commands_map);
    // to_send.insert("origin".to_string(), ResultType::Str(client_key.clone())); // -> This will be an identifier, to know the origin of the retransmited command

    // response.push(ResultType::Map(to_send));

    schedule(request_host_commands, 11, client_key, "itisaspecialcase".to_string())
}

pub fn send_network_available_commands(client_key: String) {
    let logger = acquire_logger!("Scheduler");

    logger.info(format!("Send update_available_host_commands to client trying to sync!"));

    // Lock the COMMAND_PATTERNS and insert the new map

    let mut actual_patterns: CommandPatterns = CommandPatterns::new();
    let command_patterns = &COMMAND_PATTERNS;
    smart_lock(&*command_patterns, |patterns: &mut CommandPatterns| actual_patterns = patterns.clone());

    // -> get the client by the client key
    let client = match Client::get_by_key(&client_key) {
        Ok(c) => c,
        Err(e) => match e {
            ClientError::ClientDoesNotExist(_) => {
                // return ResultType::Error(format!("unknow client_key: {:?}", client_key));
                return;
            },
            _ => {
                // return ResultType::Error(format!("Get a error {:?}, obtaining client: {:?}", e, client_key));
                return;
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
                // return ResultType::Error(format!("Error of unsuported variant to client: {:?} in handle_direct_function, the error was: {:?}", client_key, s));
                return;
            },
        },
    };

    // logger.info(format!("Successfully actualize the host available commands!"));

    let remote_function: String = "update_available_host_commands".to_string();

    let mut to_send = HashMap::new();

    to_send.insert("command_type".to_string(), ResultType::Str("function".to_string()));
    to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
    to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
    to_send.insert("function".to_string(), ResultType::Str(remote_function.to_string())); // TODO maybe change to response_act_function
    to_send.insert("kwargs".to_string(), filtered_resulttype_commands_map);
    to_send.insert("origin".to_string(), ResultType::Str("host".to_string())); // -> This will be an identifier, to know the origin of the retransmited command

    // let response: Result<String, Error>;
    let parity_id = "itisaspecialcase".to_string();
    let priority: u8 = 11;

    let command_instructions = CommandInstructions::new(
        CommandMode::Response,
        CommandType::SpecialFunction,
        CommandTarget::Origin,
        CommandStatus::Success,
        CommandOrigin::Host,
        "".to_string(),
        HashMap::new(),
        "".to_string(),
    );

    schedule(command_instructions, priority, client_key, parity_id)
}

/// Schedules a command for processing.
///
/// The function takes in a command and its priority, then schedules it for processing
/// by converting the command to a string, generating a unique parity ID, and adding it
/// to the up buffer manager's schedule.
///
/// # Arguments
/// - `command`: A map representing the command to be scheduled.
/// - `priority`: The priority level of the command. Commands with higher priority values
///               are processed before those with lower priority values.
pub fn schedule(command: CommandInstructions, priority: u8, client_key: String, parity_id: String) {
    let response: Result<String, Error>;
    let new_client_key: String;

    (response, new_client_key) = process_map_result(command, &client_key, parity_id.clone(), priority);

    let logger = acquire_logger!("Core - Scheduler");

    logger.debug("Enter Scheduler".to_string());

    let parity_id = enhanced_buffer::buffer_up_manager::buffer_up_gen_valid_parity_id(client_key.clone());

    let command_to_schedule = UpCommand::new(&new_client_key, &parity_id, priority, &response.unwrap());

    enhanced_buffer::buffer_up_manager::buffer_up_schedule(command_to_schedule.clone());

    logger.info(format!("Command: {:?} scheduled!", command_to_schedule));
}
