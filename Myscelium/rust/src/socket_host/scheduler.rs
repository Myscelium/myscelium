use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::buffer_down_manager::DownCommand;
use crate::common::enhanced_buffer::buffer_up_manager::UpCommand;
use crate::common::enhanced_buffer::utilities::{Command, CommandType};

use lazy_static::lazy_static;

use serde_json::{from_str, Value};
use std::collections::HashMap;

use crate::CLIENT_ID;

use super::host_logger::log_handler::Logger;
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
    let mut request_host_commands: HashMap<String, String> = HashMap::new();
    request_host_commands.insert("command_type".to_string(), "direct-function".to_string());
    request_host_commands.insert("response_mode".to_string(), "to_origin".to_string());
    request_host_commands.insert("status".to_string(), "success".to_string());
    request_host_commands.insert("function".to_string(), "get_socket_client_available_handlers".to_string());
    request_host_commands.insert("origin".to_string(), "".to_string());
    request_host_commands.insert("kwargs".to_string(), "{}".to_string());

    // let mut to_send = HashMap::new();

    // to_send.insert("command_type".to_string(), ResultType::Str("direct-function".to_string()));
    // to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
    // to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
    // to_send.insert("function".to_string(), ResultType::Str("update_available_host_commands".to_string())); // TODO maybe change to response_act_function
    // to_send.insert("kwargs".to_string(), filtered_resulttype_commands_map);
    // to_send.insert("origin".to_string(), ResultType::Str(client_key.clone())); // -> This will be an identifier, to know the origin of the retransmited command

    // response.push(ResultType::Map(to_send));

    schedule(request_host_commands, 11, client_key)
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
pub fn schedule(command: HashMap<String, String>, priority: u8, client_key: String) {
    let logger = acquire_logger!("Core - Scheduler");

    logger.debug("Enter Scheduler".to_string());

    logger.debug(format!("Client id is: {:?}", client_key));
    let command = serde_json::to_string(&command);

    let unwraped_command;

    // TODO >>> Add mecanisms to check the structure of the command that we are trying to registry

    match command {
        Ok(c) => {
            unwraped_command = c;
        },

        Err(e) => {
            logger.exception(format!("An error occured while trying to stringfy the command when sending it to schedule! The error was: {}", e));
            return;
        },
    }

    let parity_id = enhanced_buffer::buffer_up_manager::buffer_up_gen_valid_parity_id(client_key.clone());

    let command_to_schedule = UpCommand::new(client_key, parity_id, priority, unwraped_command);

    enhanced_buffer::buffer_up_manager::buffer_up_schedule(command_to_schedule.clone());

    logger.info(format!("Command: {:?} scheduled!", command_to_schedule));
}

// {"command_type":"function", "function":command_function, "kwargs":args}
