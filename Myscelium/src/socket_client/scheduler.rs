use crate::socket_client::enhanced_buffer;
use crate::socket_client::enhanced_buffer::buffer_up_mananger;
use crate::socket_client::enhanced_buffer::buffer_up_mananger::UpCommand;

use crate::socket_client::socket_client::Command;

use lazy_static::lazy_static;

use std::collections::HashMap;
use serde_json::{Value, from_str};

use crate::CLIENT_ID;

pub fn schedule (command:HashMap<String, String>, priority:u8) {

    let client_id = CLIENT_ID.lock().unwrap().clone();

    let command = serde_json::to_string(&command);

    let unwraped_command;

    match command {

        Ok (c) => {
            unwraped_command = c;
        }

        Err (e) => {
            eprintln! ("An error occured while trying to stringfy the command when sending it to schedule! The error was: {}", e);
            return;
        }

    }

    let parity_id = buffer_up_mananger::buffer_up_gen_valid_parity_id(client_id.clone());

    let command_to_schedule = UpCommand::new(client_id, parity_id, priority, unwraped_command);

    buffer_up_mananger::buffer_up_schedule(command_to_schedule);

}