use chrono::Utc;
use indexmap::IndexMap;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, json, Value};
use std::cell::OnceCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use std::{
    io::{ErrorKind, Read, Write},
    net::TcpStream,
};
use OxidizedMyscelium::{ClientState, Command, CommandInstructions, DownCommand, CLIENT_STATE_MANAGER};

use super::structs_and_types::{IpcnsError, OrderResponse, OrderVariant, StreamError};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref CLIENT_WAS_ONLINE: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

// Share state to store the TcpStream connection
static IPCNS_CONNECTION: OnceLock<Arc<std::sync::Mutex<Option<TcpStream>>>> = OnceLock::new();

// macro_rules! acquire_logger {
//     ($section_name:expr) => {{
//         let client_log_level;
//         {
//             client_log_level = CLIENT_LOG_LEVEL.lock().clone();
//         }
//         Logger::new(client_log_level, $section_name)
//     }};
// }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IpcncError {
    CantConnect(String),
    ConnectionNotInitialized,
    CannotObtainValidIpcnsAddr,
    Error(String),
}

fn connect(address: String) -> Result<Option<TcpStream>, IpcncError> {
    let stream = match TcpStream::connect(address.clone()) {
        Ok(s) => s,
        Err(e) => match e.kind() {
            ErrorKind::ConnectionRefused => {
                return Err(IpcncError::CantConnect("Connection refused!".to_string()));
            },
            _ => {
                panic!("Unhandled error: {}", e)
            },
        },
    };

    stream.set_read_timeout(Some(Duration::new(15, 0))).map_err(|e| IpcncError::CantConnect(format!("Failed to set read timeout: {}", e)))?;
    Ok(Some(stream))
}

const MAX_DATA_SIZE: usize = 10 * 1024 * 1024;

fn send(order: &OrderVariant, priority: u8) -> Result<Option<OrderResponse>, StreamError> {
    println!("Sending: {:?}", order);

    if let Some(lock) = IPCNS_CONNECTION.get() {
        let mut connection = lock.lock().unwrap();
        if let Some(ref mut stream) = *connection {
            {
                let command = json!(order).to_string();
                let data_size = command.len() as u32;
                let size_buffer = data_size.to_be_bytes();

                // Send the size of the data
                match stream.write(&size_buffer) {
                    Ok(_) => {},
                    Err(e) => {
                        return Err(StreamError::WriteSizeError(e));
                    },
                };

                println!("Data lenght: {:?}", size_buffer);

                // Send the actual data
                match stream.write(command.as_bytes()) {
                    Ok(_) => {},
                    Err(e) => {
                        return Err(StreamError::WriteError(e));
                    },
                };

                println!("Data sended!")
            }

            let mut size_buffer = [0; 4];

            // Read the size of the incoming data
            let data_size = match stream.read_exact(&mut size_buffer) {
                Ok(_) => u32::from_be_bytes(size_buffer) as usize,
                Err(e) => {
                    return Err(StreamError::ReadSizeError(e));
                },
            };

            println!("Receive incomming data lenght: {}", data_size);

            if data_size > MAX_DATA_SIZE {
                return Err(StreamError::ConnectionClosed); // TODO >>> Close connection or handle appropriately
            }

            println!("Data isn't greather than leght limit!");

            // Allocate a buffer of the appropriate size
            let mut data_buffer = vec![0; data_size];

            let buffer_string = match stream.read_exact(&mut data_buffer) {
                Ok(_) => String::from_utf8_lossy(&data_buffer).trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0').to_string(),
                Err(e) => {
                    return Err(StreamError::ReadDataError(e));
                },
            };

            println!("Received binary data");

            let command: Option<OrderResponse> = serde_json::from_str(&buffer_string).unwrap();

            println!("Data received: {:?}\n", command);

            return Ok(command);
        } else {
            return Err(StreamError::ConnectionClosed);
        }
    } else {
        return Err(StreamError::ConnectionClosed);
    }
}

fn get_init() -> bool {
    let client_states = match ClientState::load_from_storage() {
        Ok(st) => st,
        Err(_) => return false,
    };
    if let Some(init) = client_states.is_ready {
        CLIENT_WAS_ONLINE.store(true, Ordering::SeqCst);
        init
    } else {
        false
    }
}

pub fn connect_in_ipcns() -> Result<(), IpcncError> {
    println!("🛜🔁 >>> Initializing IPCN Client ");

    let mut last_attempt_time: Instant = Instant::now() - Duration::from_secs(30);
    let _ = IPCNS_CONNECTION.get_or_init(|| Arc::new(std::sync::Mutex::new(None)));

    let address: String;

    {
        let client_states = CLIENT_STATE_MANAGER.lock();
        if let Some(addr) = client_states.ipcns_addr.clone() {
            address = addr.clone()
        } else {
            return Err(IpcncError::CannotObtainValidIpcnsAddr);
        }
    }

    let mut max_attempts: u64 = 0;

    while !get_init() {
        if CLIENT_WAS_ONLINE.load(Ordering::SeqCst) && max_attempts > 3 {
            return Err(IpcncError::Error("Cannot Connect into the ipcns network, it was online but not isn't".to_string()));
        };
        if max_attempts > 20 {
            return Err(IpcncError::Error("Cannot Connect into the ipcns network".to_string()));
        }
        max_attempts += 1;
        println!("Client is not running yet, waiting to connect to ipcns");
        thread::sleep(Duration::from_secs(1u64));
    }

    loop {
        if !get_init() {
            return Err(IpcncError::Error("Client is not running, so ipcns is not running too!".to_string()));
        }

        let now = Instant::now();
        if now.duration_since(last_attempt_time) >= Duration::from_secs(30) {
            // try to connect:
            match connect(address.clone()) {
                Ok(con) => {
                    if let Some(st) = con {
                        IPCNS_CONNECTION.get_or_init(|| Arc::new(std::sync::Mutex::new(None)));

                        if let Some(lock) = IPCNS_CONNECTION.get() {
                            let mut connection = lock.lock().unwrap();
                            *connection = Some(st);
                        }

                        // TODO >>> Re implement connection verification mechanism

                        break; // >! remove this when connection verifcation was re implemented
                    }
                },
                Err(e) => {
                    println!("Error: {:?}", e)
                },
            }

            // Update last attempt time
            last_attempt_time = now
        } else {
            let dif: Duration = last_attempt_time - now;
            println!("Trying to connect again in: {} secs", dif.as_secs());
        }
    }

    println!("🛜🟢 >>> Scheduler Unity Connected in the IPCNS! ");

    return Ok(());
}

pub fn schedule_up_command_instructions(up_command_instructions: CommandInstructions, priority: u8) -> Result<String, IpcnsError> {
    let order = OrderVariant::ScheduleCommandInstructions(up_command_instructions.clone(), priority);
    let response: Option<OrderResponse> = match send(&order, priority) {
        Ok(r) => r,
        Err(e) => return Err(IpcnsError::Error(format!("Get the following stream error: {:?}", e))),
    };

    if let Some(resp) = response {
        match resp {
            OrderResponse::Confirmed(parity_id) => return Ok(parity_id),
            OrderResponse::Error(e) => {
                return Err(IpcnsError::Error(format!("Had a scheduling error: {:?}", e)));
            },
            _ => {
                panic!() // > This was not expected here so it is an error
            },
        }
    }

    panic!("Response obtained by the ipcns is None and this was not expected!")
}

// TODO >>> Add an method to try to match for an parity id remotelly inside the other process trying to get the inplace response that we are waiting for in some of this process threads

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatcherError {
    MaxTimeExceeded(String),
    CommandNotFinded(String),
    CannotConnectToIpcn(String),
    AnErrorHappenedInTheIpcn(String),
}

pub fn watch_response(parity_id: String, max_time: chrono::Duration) -> Result<Command, WatcherError> {
    let mut finded = false;
    let mut response_command: Option<DownCommand> = None;

    let start_time = Utc::now();

    // let logger: Logger = acquire_logger!("IPCNS CLIENT");

    let mut max_attempts: u64 = 0;

    while !get_init() {
        if CLIENT_WAS_ONLINE.load(Ordering::SeqCst) && max_attempts > 3 {
            return Err(WatcherError::AnErrorHappenedInTheIpcn("Cannot Connect into the ipcns network, it was online but not isn't".to_string()));
        };

        if max_attempts > 20 {
            return Err(WatcherError::AnErrorHappenedInTheIpcn("Cannot Connect into the ipcns network".to_string()));
        }
        max_attempts += 1;
        println!("Client is not running yet, waiting to connect to ipcns");
        thread::sleep(Duration::from_secs(1u64));
    }

    loop {
        if !get_init() {
            return Err(WatcherError::AnErrorHappenedInTheIpcn("Client is not running, so ipcns is not running too!".to_string()));
        }

        let order = OrderVariant::MatchParityId(parity_id.clone());

        let response = match send(&order, 2u8) {
            Ok(r) => r,
            Err(e) => match e {
                _ => {
                    // logger.exception(format!("An error happened in the ipcns, the error was: {:?}", e));
                    return Err(WatcherError::AnErrorHappenedInTheIpcn(format!("{:?}", e)));
                },
            },
        };

        if let Some(resp) = response {
            match resp {
                OrderResponse::Confirmed(c) => {
                    panic!("Confirmation is not expected here!");
                    // logger.exception(format!("Confirmation is not expected here in the parity id matching mechanism!"));
                },
                OrderResponse::Error(e) => {
                    // logger.exception(format!("An error happened trying to verify if the inplace response for the command: {:?} was received", &parity_id));
                    return Err(WatcherError::AnErrorHappenedInTheIpcn(format!("{:?}", e)));
                },
                OrderResponse::IncplaceResponseNotArrivedYet => {
                    // logger.info("Inplace Response not arrived yet".to_string());
                },
                OrderResponse::MatchingDownCommand(d) => {
                    // logger.info(format!("Response to command: {:?} finded successfully!", &parity_id));
                    response_command = Some(d);
                    finded = true;
                },
            }
        }

        let current_time = Utc::now();

        if current_time > (start_time + max_time) {
            // logger.exception(format!("Max time for waiting the command: {:?} was reached! Giving a timeout in watching for this response!", &parity_id));
            return Err(WatcherError::MaxTimeExceeded(parity_id));
        }

        if finded {
            // logger.info(format!("Ipcns client finded response for command with parity id: {:?}", &parity_id));
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    if let Some(response) = response_command {
        return Ok(Command::from_down_command(&response).unwrap());
    }

    return Err(WatcherError::CommandNotFinded(parity_id));
}
