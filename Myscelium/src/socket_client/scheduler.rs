use crate::socket_client::enhanced_buffer;
use crate::socket_client::enhanced_buffer::buffer_up_mananger;
use crate::socket_client::enhanced_buffer::buffer_up_mananger::UpCommand;

use crate::socket_client::socket_client::Command;

use lazy_static::lazy_static;

use serde_json::{from_str, Value};
use std::collections::HashMap;

use crate::CLIENT_ID;

pub fn set_client_id(client_uid: String) {
    println!("Setting client_id to: {:?}", client_uid.clone());

    {
        let mut client_id_global = CLIENT_ID.lock().unwrap();
        *client_id_global = client_uid.clone();
    }
}

pub fn schedule(command: HashMap<String, String>, priority: u8) {
    println!("Enter Schedule");

    let client_id = CLIENT_ID.lock().unwrap().clone();

    println!("Client id is: {:?}", client_id);
    let command = serde_json::to_string(&command);

    let unwraped_command;

    match command {
        Ok(c) => {
            unwraped_command = c;
        },

        Err(e) => {
            eprintln!("An error occured while trying to stringfy the command when sending it to schedule! The error was: {}", e);
            return;
        },
    }

    let parity_id = buffer_up_mananger::buffer_up_gen_valid_parity_id(client_id.clone());

    let command_to_schedule = UpCommand::new(client_id, parity_id, priority, unwraped_command);

    buffer_up_mananger::buffer_up_schedule(command_to_schedule);

    println!("Command scheduled!");
}
