// SPDX-License-Identifier: MPL-2.0
// Copyright © 2021-2026 Cristian Camargo Filho

use crate::ipcns::ipcn_client::schedule_up_command_instructions;
use crate::ipcns::structs_and_types::{IpcnsError, OrderResponse, OrderVariant};

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::Ordering;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    panic, thread,
};
use OxidizedMyscelium::{DownCommand, CLIENT_IS_RUNNING, CLIENT_STATE_MANAGER};

use super::structs_and_types::StreamError;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatcherError {
    MaxTimeExceeded(String),
    CommandNotFinded(String),
}

pub fn send_response(stream: &mut TcpStream, data: OrderResponse) -> Result<(), StreamError> {
    let command_response_json = json!(data).to_string();
    let data_size = command_response_json.len() as u32;
    let size_buffer = data_size.to_be_bytes();

    // Check if the connection was closed
    if stream.peer_addr().is_err() {
        return Err(StreamError::ConnectionClosed);
    }

    // Send the size of the data
    match stream.write(&size_buffer) {
        Ok(_) => {},
        Err(e) => {
            return Err(StreamError::WriteSizeError(e));
        },
    };

    // Send the actual data
    match stream.write(command_response_json.as_bytes()) {
        Ok(_) => {},
        Err(e) => {
            return Err(StreamError::WriteError(e));
        },
    };

    Ok(())
}

/// Init the inter process communication network socket server
pub fn initialize_ipcns(client_key: String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr(); // Retrieve the assigned address
    println!("Server is listening on {:?}", addr);

    // let logger: Logger = acquire_logger!("IPCNS SERVER");

    // TODO >>> Validate the data being received from trust source, it should have the key and the token

    // Set the addr of the ipcns server into the sqlite3
    {
        let mut client_mannager = CLIENT_STATE_MANAGER.lock();
        let _ = client_mannager.set_ipcns_server_address(addr.unwrap().to_string()).unwrap();
        client_mannager.update_storage_with_self().unwrap();
    }

    fn handle_connection(stream: &mut TcpStream) {
        // TODO >>> Create a mechanism that allow schedule things in buffer up
        // TODO >>> Filter the schdule things using some rules (to doesn't need to store the network map inside the buffer up)
        // TODO >>> Create a way to receive here a parity id to look for a response to the parity id + client key that allow us to return a response that will be used as a inplace response

        const MAX_DATA_SIZE: usize = 10 * 1024 * 1024; // For example, 10 MB

        loop {
            let mut size_buffer = [0; 4];

            //> Read the size of the incoming data
            let data_size = match stream.read_exact(&mut size_buffer) {
                Ok(_) => u32::from_be_bytes(size_buffer) as usize,
                Err(e) => {
                    // logger.exception(format!("Failed to read from the stream: {:?}", e));
                    eprintln!("Failed to read from the stream: {:?}", e);
                    //> Handle the error, e.g., by returning from the function or taking corrective action
                    return; //> or handle differently
                },
            };

            if data_size > MAX_DATA_SIZE {
                return; //> Close connection or handle appropriately
            }

            //> Allocate a buffer of the appropriate size
            let mut data_buffer = vec![0; data_size];

            //> Read the data into the buffer
            let buffer_string = match stream.read_exact(&mut data_buffer) {
                Ok(_) => String::from_utf8_lossy(&data_buffer).trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0').to_string(),
                Err(e) => {
                    eprintln!("Failed to read from the stream: {:?}", e);
                    // logger.exception(format!("IPCNS Server failed to read from the stream: {:?}", e));
                    //> Handle the error, e.g., by returning from the function or taking corrective action
                    return; //> or handle differently
                },
            };

            // TODO >>> if go add any hashing add here or other encription method:
            let variant: OrderVariant = serde_json::from_str(&buffer_string).unwrap();

            let mut response: OrderResponse = OrderResponse::Error(IpcnsError::Error("Unexpected case".to_string()));

            match variant {
                // This mechanism allows to match a parity id in order to find if a inplace response has arrived
                OrderVariant::MatchParityId(pid) => {
                    // > This should not break this connection while the parity id wasn't received

                    // This should be a reactive mechanism
                    // let mut schedule: Vec<DownCommand> = enhanced_buffer::buffer_down_manager::buffer_down_list_schedule();

                    // // -> Sort commands by priority in ascending order
                    // schedule.sort_by(|a, b| b.priority.cmp(&a.priority));

                    // // -> Filter only auto collect == false (that are commands to not autocollect)
                    // schedule.retain(|s| !s.auto_collect);

                    // response = OrderResponse::IncplaceResponseNotArrivedYet;
                    // for command in schedule {
                    //     if command.parity_id == pid {
                    //         response = OrderResponse::MatchingDownCommand(command);
                    //     }
                    // }

                    // TODO >>> Verify if there is a method that may allow to watch for a parity id response inside the core
                    //* If don't have this method, then implement it and then link it to this
                },
                // Used to schedule command instructions
                OrderVariant::ScheduleCommandInstructions(uci, p) => {
                    let _ = schedule_up_command_instructions(uci, p).unwrap();
                },
            }

            send_response(stream, response);
        }
    }

    loop {
        if !CLIENT_IS_RUNNING.load(Ordering::SeqCst) {
            // logger.info("Stop ICPNS Server cause the core stoped!".to_string());
            println!("Stop the core!");
            break;
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                // Spawn a new thread for each connection
                // logger.info("New Client Connected into the IPCNS server!".to_string());
                thread::spawn(move || {
                    // Set a read timeout of 5 seconds
                    stream.set_read_timeout(Some(std::time::Duration::new(5, 0))).unwrap();

                    match panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        handle_connection(&mut stream);
                    })) {
                        Ok(_) => println!("Connection handled successfully"),
                        Err(e) => eprintln!("Connection handler panicked: {:?}", e),
                    }
                });
                // logger.info("A client terminate the connection with the IPCNS server".to_string());
            },
            Err(e) => {
                eprintln!("Failed to accept a connection: {}", e);
                // logger.info(format!("IPCNS Server failed to accept a connection with a client, the error was: {:?}", e));
            },
        }
    }
}
