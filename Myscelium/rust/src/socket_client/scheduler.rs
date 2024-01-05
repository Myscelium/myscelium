use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::buffer_down_manager::DownCommand;
use crate::common::enhanced_buffer::buffer_up_manager::UpCommand;
use crate::common::enhanced_buffer::utilities::{Command, CommandType};
use crate::common::functions::advanced_lockers::smart_lock;

use lazy_static::lazy_static;

use serde_json::{from_str, Value};
use std::collections::HashMap;

use crate::CLIENT_ID;

use super::client_logger::log_handler::Logger;
use crate::CLIENT_LOG_LEVEL;

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let client_log_level;
        {
            client_log_level = CLIENT_LOG_LEVEL.lock().clone();
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
pub fn set_client_id(client_uid: String) {
    println!("Setting client_id to: {:?}", client_uid.clone());

    // let client_key_storage = &CLIENT_ID;
    // smart_lock(client_key_storage, |key: &mut String| {
    //     *key = client_uid;
    // });

    {
        let mut key = CLIENT_ID.lock(); // TODO > This is using parking lot, see if need to change to smart-lock
        *key = client_uid
    }
}

/// Requests the available commands that are registered on the host.
///
/// This function prepares a command request for the host to retrieve the list of
/// registered commands. The constructed request is then scheduled for processing.
// pub fn request_host_available_commands() {
//     let mut request_host_commands: HashMap<String, String> = HashMap::new();
//     request_host_commands.insert("function".to_string(), "get_registered_commands".to_string());
//     request_host_commands.insert("command_type".to_string(), "function".to_string());
//     request_host_commands.insert("kwargs".to_string(), "{}".to_string());

//     schedule(request_host_commands, 11)
// }

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
pub fn schedule(command: HashMap<String, String>, priority: u8) {
    let logger: Logger = acquire_logger!("Core - Scheduler");

    logger.debug("Enter Scheduler".to_string());

    let mut client_key: String = "".to_string();

    // let client_key_storage = &CLIENT_ID;
    // smart_lock(&client_key_storage, |key: &mut String| {
    //     client_key = key.clone();
    // });

    {
        let key = CLIENT_ID.lock(); // TODO > This is using parking lot, see if need to change to smart-lock
        client_key = key.clone()
    }

    logger.debug(format!("Client id is: {:?}", client_key));
    let command: Result<String, serde_json::Error> = serde_json::to_string(&command);

    let unwraped_command: String;

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

    let parity_id: String = enhanced_buffer::buffer_up_manager::buffer_up_gen_valid_parity_id(client_key.clone());

    let command_to_schedule: UpCommand = UpCommand::new(&client_key, &parity_id, priority, &unwraped_command);

    enhanced_buffer::buffer_up_manager::buffer_up_schedule(command_to_schedule.clone());

    logger.info(format!("Command: {:?} scheduled!", command_to_schedule));
}
