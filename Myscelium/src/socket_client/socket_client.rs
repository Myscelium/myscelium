use crate::socket_client::enhanced_buffer;
use crate::socket_client::enhanced_buffer::buffer_down_mananger;
use crate::socket_client::enhanced_buffer::buffer_down_mananger::DownCommand;
use crate::socket_client::enhanced_buffer::buffer_up_mananger;

use lazy_static::lazy_static;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
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
    static ref CLIENT_ID: Arc<Mutex<String>> = Arc::new(Mutex::new(' '.to_string()));
}

// >-------------------------------------------------------------------------------------------------------------------------------------------

// -> Socket Interactive Functions:

pub fn set_socket_client_callbacks_patterns(callbacks_patterns: HashMap<String, Value>) {
    let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();
    *command_patterns = callbacks_patterns;
}

pub fn initialize_client_buffer(buffer_location: String) {
    println!("\ninicializing the buffer database into: {}buffer.db, if not inicialized!", buffer_location);

    enhanced_buffer::buffer_down_mananger::buffer_down_initialize_table(buffer_location.clone());

    enhanced_buffer::buffer_up_mananger::buffer_up_initialize_table(buffer_location.clone());

    println!("\nAll buffer initialized succefully!\n");

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
    let command_patterns = COMMAND_PATTERNS.lock().unwrap();
    return command_patterns.clone();
}

// > --------------------------------------------------------------------------------------------------------------------------------------

// -> Socket client functionality structures:

// #[derive(Serialize, Deserializer, Debug)] is an attribute that automatically
// derives the Serialize and Deserialize traits from the serde crate, witch allow
// the struct to be converted to and from JSON.

// The Debug Trait, is also derived, which allows the structure to be printed fro debugging purposes

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    pub client_id: String,
    pub parity_id: String,
    pub priority: u8,
    pub command: HashMap<String, Value>,
}

enum Response {
    Command(Command),
    None,
}

#[derive(Serialize, Deserialize)]
enum CommandType {
    Function(String),
    Response(String),
    Unknown,
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

impl Command {
    pub fn new(client_id: String, parity_id: String, priority: u8, command: HashMap<String, Value>) -> Self {
        Self {
            client_id,
            parity_id,
            priority,
            command,
        }
    }

    fn command_type(&self) -> CommandType {
        if self.command.contains_key("function") {
            CommandType::Function(self.command.get("function").unwrap().to_string())
        } else if self.command.contains_key("response") {
            CommandType::Response(self.command.get("response").unwrap().to_string())
        } else {
            CommandType::Unknown
        }
    }

    fn from_up_command(up_command: UpCommand) -> Self {
        let client_id = up_command.client_id.clone();
        let parity_id = up_command.parity_id.clone();
        let priority = up_command.priority.clone();
        let command: HashMap<String, Value> = serde_json::from_str(&up_command.command).unwrap();

        Self {
            client_id,
            parity_id,
            priority,
            command,
        }
    }
}

use serde_json::to_string;

fn verify_connection(stream: &mut TcpStream) -> bool {
    let command = create_special_command!("C202");

    let command_json = json!(command).to_string();

    stream.write_all(command_json.as_bytes()).unwrap();

    let mut buffer = [0; 4096];
    stream.read(&mut buffer).unwrap();

    let buffer_string = String::from_utf8_lossy(&buffer)
        .trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0')
        .to_string();

    let command: Command = serde_json::from_str(&buffer_string).unwrap();

    println!("{:?}", command);

    match command.command.get("function") {
        Some(Value::String(function)) => {
            if function == "C200" {
                return true;
            } else {
                return false;
            }
        },
        _ => {
            println!("The function name is not found or not a string.");
            return false;
        },
    }
}

fn send(stream: &mut TcpStream, command: Command) -> Response {
    let conn: bool = verify_connection(stream);

    if !conn {
        println!("Not connected!");
        return Response::None;
    }

    println!("Connected!!");

    let command_json = json!(command).to_string();

    stream.write_all(command_json.as_bytes()).unwrap();

    let mut buffer = [0; 4096];
    stream.read(&mut buffer).unwrap();

    let buffer_string = String::from_utf8_lossy(&buffer)
        .trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0')
        .to_string();

    let command: Command = serde_json::from_str(&buffer_string).unwrap();

    println!("Received: {:?}", command);

    return Response::Command(command);
}

use buffer_up_mananger::UpCommand;

pub fn send_ping(mut stream: &mut TcpStream) {
    let command_to_request = create_special_command!("C207");
    let received = send(&mut stream, command_to_request.clone());
}

// This function handles the response and returns an appropriate action.
fn handle_response(received: Response) -> Option<DownCommand> {
    let command_received;

    match received {
        Response::None => {
            println!("Received invalid data!");
            return None;
        },
        Response::Command(c) => {
            println!("Received command: {:?}", c);
            command_received = c;
        },
    }

    match command_received.command_type() {
        CommandType::Function(f) => {
            let function: String = serde_json::from_str(&f).unwrap();

            if command_received.parity_id != "itisaspecialcase" {
                if function == "C210".to_string() {
                    println!("Received Confirmation!");
                    return None;
                } else if function == "Error".to_string() {
                    println!("An error occurred in host, the error was: {}", command_received.command.get("Error").unwrap());
                    return None;
                }
            }

            println!("Receive a function: {:?}", f);
            return None;
        },

        CommandType::Response(r) => {
            println!("Received a response!");

            let down_command = DownCommand::from_command(command_received.clone());

            buffer_up_mananger::buffer_up_remove_schedule_by_parity_id(command_received.client_id, command_received.parity_id);

            return Some(down_command);
        },

        CommandType::Unknown => {
            println!("Received an Unknown command!");
            return None;
        },
    }
}

pub fn initialize_client(address: String, client_id: String) {
    let mut stream = TcpStream::connect(address).unwrap();

    loop {
        let up_schedule = buffer_up_mananger::buffer_up_list_schedule();

        if !(up_schedule.len() > 0) {
            send_ping(&mut stream);

            thread::sleep(Duration::from_secs(2));
            continue;
        }

        if !CLIENT_IS_RUNING.load(Ordering::SeqCst) {
            print!("running is set to false, shutdown socket client main process!");
            break;
        }

        for up_command in up_schedule {
            let command_to_request = Command::from_up_command(up_command);

            loop {
                let received = send(&mut stream, command_to_request.clone());

                if let Some(down_command) = handle_response(received) {
                    buffer_down_mananger::buffer_down_schedule(down_command.clone());
                    break;
                }
            }
        }
    }
}
