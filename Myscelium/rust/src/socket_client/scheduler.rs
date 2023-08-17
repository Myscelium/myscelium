use crate::commom::enhanced_buffer;
use crate::commom::enhanced_buffer::buffer_down_mananger::DownCommand;
use crate::commom::enhanced_buffer::buffer_up_mananger::UpCommand;
use crate::commom::enhanced_buffer::utilities::{Command, CommandType};

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

pub fn set_client_id(client_uid: String) {
    println!("Setting client_id to: {:?}", client_uid.clone());

    {
        let mut client_id_global = CLIENT_ID.lock();
        *client_id_global = client_uid.clone();
    }
}

pub fn request_host_avaliable_commands() {
    let mut request_host_commands: HashMap<String, String> = HashMap::new();
    request_host_commands.insert("function".to_string(), "get_registred_commands".to_string());

    schedule(request_host_commands, 11)
}

pub fn schedule(command: HashMap<String, String>, priority: u8) {
    let logger = acquire_logger!("Core - Scheduler");

    logger.debug("Enter Scheduler".to_string());

    let client_id = CLIENT_ID.lock().clone();

    logger.debug(format!("Client id is: {:?}", client_id));
    let command = serde_json::to_string(&command);

    let unwraped_command;

    match command {
        Ok(c) => {
            unwraped_command = c;
        },

        Err(e) => {
            logger.exception(format!("An error occured while trying to stringfy the command when sending it to schedule! The error was: {}", e));
            return;
        },
    }

    let parity_id = enhanced_buffer::buffer_up_mananger::buffer_up_gen_valid_parity_id(client_id.clone());

    let command_to_schedule = UpCommand::new(client_id, parity_id, priority, unwraped_command);

    enhanced_buffer::buffer_up_mananger::buffer_up_schedule(command_to_schedule.clone());

    logger.info(format!("Command: {:?} scheduled!", command_to_schedule));
}
