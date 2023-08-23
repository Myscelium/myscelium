use crate::commom::enhanced_buffer;
use crate::commom::enhanced_buffer::buffer_down_mananger::DownCommand;
use crate::commom::enhanced_buffer::buffer_up_mananger::UpCommand;
use crate::commom::enhanced_buffer::utilities::{Command, CommandType};

use lazy_static::lazy_static;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::thread;

use serde::{Deserialize, Serialize};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Condvar,
};

use pyo3::types::{IntoPyDict, PyAny, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use pyo3::{PyErr, PyObject, PyResult, Python};

use pyo3::IntoPy;

use pyo3::exceptions::PyException;
use pyo3::Py;

use std::time::{Duration, Instant};

use std::error::Error;

use pyo3::ToPyObject;

use crate::CLIENT_IS_RUNING;

use std::fmt;

use std::net::TcpStream;

use std::io::Read;
use std::io::Write;

use serde_json::json;

use super::client_logger::log_handler::Logger;
use crate::CLIENT_LOG_LEVEL;

use parking_lot::Mutex;

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let client_log_level;
        {
            client_log_level = CLIENT_LOG_LEVEL.lock().clone();
        }
        Logger::new(client_log_level, $section_name)
    }};
}

lazy_static! {
    static ref COMMAND_PATTERNS: Arc<Mutex<HashMap<String, Value>>> = {
        let json_str = r#"{
            "get_symbols_data": {
                "symbols_data": {
                    "data-type": "str",
                    "symbols": "str",
                    "start-ts": "float",
                    "end-ts": "float"
                }
            },
            "get_other_symbols_data": {
                "symbols_data": {
                    "data-type": "str",
                    "symbols": "str",
                    "start-ts": "float",
                    "end-ts": "float"
                }
            }
        }"#;

        let command_patterns: HashMap<String, Value> = from_str(json_str).unwrap();
        Arc::new(Mutex::new(command_patterns))
    };
    static ref HOST_ALLOWED_COMMANDS: Arc<Mutex<HashMap<String, Value>>> = {
        let json_str = r#"{
            "get_symbols_data": {
                "symbols_data": {
                    "data-type": "str",
                    "symbols": "str",
                    "start-ts": "float",
                    "end-ts": "float"
                }
            },
            "get_other_symbols_data": {
                "symbols_data": {
                    "data-type": "str",
                    "symbols": "str",
                    "start-ts": "float",
                    "end-ts": "float"
                }
            }
        }"#;

        let command_patterns: HashMap<String, Value> = from_str(json_str).unwrap();
        Arc::new(Mutex::new(command_patterns))
    };
    static ref CLIENT_ID: Arc<Mutex<String>> = Arc::new(Mutex::new(' '.to_string()));
}

// >-------------------------------------------------------------------------------------------------------------------------------------------

// -> Socket Interactive Functions:

pub fn set_socket_client_callbacks_patterns(callbacks_patterns: HashMap<String, Value>) {
    let mut command_patterns = COMMAND_PATTERNS.lock();
    *command_patterns = callbacks_patterns;
}

pub fn initialize_client_buffer(buffer_location: String) {
    println!("inicializing the buffer database into: {}buffer.db, if not inicialized!", buffer_location);

    enhanced_buffer::buffer_down_mananger::buffer_down_initialize_table(buffer_location.clone());

    enhanced_buffer::buffer_up_mananger::buffer_up_initialize_table(buffer_location.clone());

    println!("All buffer initialized succefully!");

    return;
}

// Keep the thread alive until HOST_IS_RUNING is set to false
// if !CLIENT_IS_RUNING.load(Ordering::SeqCst) {
//     print!("runing is set to false, skipping");
//     break;
// }

// The incoming method is called on the listener, which returns an iterator that gives us a sequence of
// TCP streams (representing a series of connections). The server will then handle each connection in a loop.

// handle_connection is a function that handles each TCP stream. It reads from the stream into a buffer,
// then writes the contents of the buffer back to the stream.

pub fn get_socket_client_available_commands_registered() -> HashMap<String, Value> {
    let command_patterns = COMMAND_PATTERNS.lock();
    return command_patterns.clone();
}

// > --------------------------------------------------------------------------------------------------------------------------------------

// -> Socket client functionality structures:

// #[derive(Serialize, Deserializer, Debug)] is an attribute that automatically
// derives the Serialize and Deserialize traits from the serde crate, witch allow
// the struct to be converted to and from JSON.

// The Debug Trait, is also derived, which allows the structure to be printed fro debugging purposes

enum Response {
    Command(Command),
    None,
}

macro_rules! create_special_command {
    ($code:expr) => {{
        use std::collections::HashMap;

        let mut command_map = HashMap::new();
        command_map.insert("function".to_string(), Value::String($code.to_string()));

        Command {
            client_id: "some_client_id".to_string(),
            parity_id: "itisaspecialcase".to_string(),
            priority: 11,
            command: command_map,
        }
    }};
}

fn verify_connection(stream: &mut TcpStream) -> bool {
    let logger = acquire_logger!("Core");

    let command = create_special_command!("C202");

    let command_json = json!(command).to_string();

    stream.write_all(command_json.as_bytes()).unwrap();

    let mut buffer = [0; 4096];
    stream.read(&mut buffer).unwrap();

    let buffer_string = String::from_utf8_lossy(&buffer)
        .trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0')
        .to_string();

    let command: Command = serde_json::from_str(&buffer_string).unwrap();

    logger.debug(format!("{:?}", command));

    match command.command.get("function") {
        Some(Value::String(function)) => {
            if function == "C200" {
                return true;
            } else {
                return false;
            }
        },
        _ => {
            logger.warn(format!("The function name is not found or not a string."));
            return false;
        },
    }
}

fn send(stream: &mut TcpStream, command: Command) -> Response {
    let logger = acquire_logger!("Core");

    let conn: bool = verify_connection(stream);

    if !conn {
        logger.info(format!("Not connected!"));
        return Response::None;
    }

    let command_json = json!(command).to_string();

    stream.write_all(command_json.as_bytes()).unwrap();

    let mut buffer = [0; 4096];
    stream.read(&mut buffer).unwrap();

    let buffer_string = String::from_utf8_lossy(&buffer)
        .trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0')
        .to_string();

    let command: Command = serde_json::from_str(&buffer_string).unwrap();

    logger.debug(format!("Received: {:?}", command));

    return Response::Command(command);
}

pub fn send_ping(mut stream: &mut TcpStream) -> Option<DownCommand> {
    if !CLIENT_IS_RUNING.load(Ordering::SeqCst) {
        return None;
    }

    let command_to_request = create_special_command!("C206");
    let received = send(&mut stream, command_to_request.clone());
    if let Some(down_command) = handle_response(received) {
        return Some(down_command);
    } else {
        return None;
    }
}

// This function handles the response and returns an appropriate action.
fn handle_response(received: Response) -> Option<DownCommand> {
    let logger = acquire_logger!("Core");

    let command_received;

    match received {
        Response::None => {
            logger.warn(format!("Received invalid data!"));
            return None;
        },
        Response::Command(c) => {
            logger.debug(format!("\nReceived command: {:?}", c));
            command_received = c;
        },
    }

    match command_received.command_type() {
        CommandType::Function(f) => {
            let function: String = serde_json::from_str(&f).unwrap();

            if command_received.parity_id != "itisaspecialcase" {
                if function == "C210".to_string() {
                    logger.info(format!("Received Confirmation!"));
                    return None;
                } else if function == "Error".to_string() {
                    logger.exception(format!("\nAn error occurred in host, the error was: {}\n", command_received.command.get("Error").unwrap()));
                    CLIENT_IS_RUNING.store(false, Ordering::SeqCst);
                    return None;
                }
            }

            logger.debug(format!("Receive a function: {:?}", f));
            return None;
        },

        CommandType::Response(r) => {
            logger.info(format!("Received a response!"));

            let down_command = DownCommand::from_command(command_received.clone());

            enhanced_buffer::buffer_up_mananger::buffer_up_remove_schedule_by_parity_id(command_received.client_id, command_received.parity_id);

            return Some(down_command);
        },

        CommandType::Error(e) => {
            let down_command = DownCommand::from_command(command_received.clone());

            enhanced_buffer::buffer_up_mananger::buffer_up_remove_schedule_by_parity_id(command_received.client_id, command_received.parity_id);

            logger.exception(format!("\nAn error occurred in host, the error was: {}\n", command_received.command.get("error").unwrap()));
            CLIENT_IS_RUNING.store(false, Ordering::SeqCst);

            return None;
        },

        CommandType::Redirect(_) => {
            logger.warn(format!("Received an Unknown command!"));
            return None;
        },

        CommandType::Unknown => {
            logger.warn(format!("Received an Unknown command!"));
            return None;
        },
    }
}

pub fn initialize_client(address: String, client_id: String) {
    // Create a global Mutex for demonstration
    let mutex1 = Mutex::new(0);
    let mutex2 = Mutex::new(0);

    // Spawn a thread to periodically check for deadlocks
    thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_secs(5)); // Check every 5 seconds
            let deadlocks = parking_lot::deadlock::check_deadlock();
            if deadlocks.is_empty() {
                continue;
            }

            println!("{} deadlocks detected", deadlocks.len());
            for (i, threads) in deadlocks.iter().enumerate() {
                println!("Deadlock #{}", i);
                for t in threads {
                    println!("Thread Id {:?}", t.thread_id());
                    println!("{:?}", t.backtrace());
                }
            }
        }
    });

    let logger = acquire_logger!("Core");

    let mut stream = TcpStream::connect(address).unwrap();

    thread::sleep(Duration::from_millis(200));

    loop {
        if !CLIENT_IS_RUNING.load(Ordering::SeqCst) {
            logger.info(format!("running is set to false, shutdown socket client main process!"));
            break;
        }

        let up_schedule = enhanced_buffer::buffer_up_mananger::buffer_up_list_schedule();

        if !(up_schedule.len() > 0) {
            if let Some(down_command) = send_ping(&mut stream) {
                enhanced_buffer::buffer_down_mananger::buffer_down_schedule(down_command.clone());
            }
            thread::sleep(Duration::from_millis(500));
            continue;
        }

        for up_command in up_schedule {
            let command_to_request = Command::from_up_command(up_command);

            loop {
                let received = send(&mut stream, command_to_request.clone());

                if let Some(down_command) = handle_response(received) {
                    enhanced_buffer::buffer_down_mananger::buffer_down_schedule(down_command.clone());
                    break;
                }

                thread::sleep(Duration::from_millis(200));
            }
        }
    }
}
