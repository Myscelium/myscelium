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
pub enum CommandType {
    Function(String),
    Response(String),
    Error(String),
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

fn transform_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut new_map = HashMap::new();
            for (key, val) in map {
                if let Some(inner_val) = val.get("Map") {
                    new_map.insert(key.clone(), transform_value(inner_val));
                } else if let Some(inner_val) = val.get("List") {
                    new_map.insert(key.clone(), transform_value(inner_val));
                } else if let Some(Value::String(s)) = val.get("Str") {
                    new_map.insert(key.clone(), Value::String(s.clone()));
                } else {
                    // Handle other cases if necessary
                }
            }
            Value::Object(serde_json::Map::from_iter(new_map)) // Convert HashMap to serde_json::Map using into()
        },
        Value::Array(arr) => Value::Array(arr.iter().map(|v| transform_value(v)).collect()),
        _ => value.clone(),
    }
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

    pub fn command_type(&self) -> CommandType {
        if self.command.contains_key("function") {
            CommandType::Function(self.command.get("function").unwrap().to_string())
        } else if self.command.contains_key("response") {
            CommandType::Response(self.command.get("response").unwrap().to_string())
        } else if self.command.contains_key("error") {
            CommandType::Error(self.command.get("error").unwrap().to_string())
        } else {
            CommandType::Unknown
        }
    }

    pub fn from_down_command(down_command: DownCommand) -> Self {
        let client_id = down_command.client_id.clone();
        let parity_id = down_command.parity_id.clone();
        let priority = down_command.priority.clone();

        let outer_value: Value = serde_json::from_str(&down_command.command).unwrap();

        let mut command: HashMap<String, Value>;

        // Extract the inner JSON string and deserialize it again
        if let Value::Object(outer_map) = &outer_value {
            if let Some(Value::String(inner_json)) = outer_map.get("response") {
                command = serde_json::from_str(inner_json).unwrap();

                // Transform the command right after deserialization
                let transformed_value = transform_value(&Value::Object(serde_json::Map::from_iter(command.into_iter()))); // Convert HashMap to serde_json::Map using into()

                if let Value::Object(transformed_map) = transformed_value {
                    command = transformed_map.into_iter().collect(); // Convert serde_json::Map back to HashMap
                } else {
                    println!("Unexpected format after transformation");
                    command = HashMap::new();
                }
            } else {
                command = HashMap::new();
                println!("Command has other case other than Response");
                // Handle the case where the "response" key is not a string
            }
        } else {
            command = HashMap::new();
            println!("Command isn't an object");
            // Handle the case where the outer_value is not an object
        }
        Self {
            client_id,
            parity_id,
            priority,
            command,
        }
    }

    pub fn from_up_command(up_command: UpCommand) -> Self {
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
    let command_received;

    match received {
        Response::None => {
            println!("Received invalid data!");
            return None;
        },
        Response::Command(c) => {
            println!("\nReceived command: {:?}", c);
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
                    println!("\nAn error occurred in host, the error was: {}\n", command_received.command.get("Error").unwrap());
                    CLIENT_IS_RUNING.store(false, Ordering::SeqCst);
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

        CommandType::Error(e) => {
            let down_command = DownCommand::from_command(command_received.clone());

            buffer_up_mananger::buffer_up_remove_schedule_by_parity_id(command_received.client_id, command_received.parity_id);

            println!("\nAn error occurred in host, the error was: {}\n", command_received.command.get("error").unwrap());
            CLIENT_IS_RUNING.store(false, Ordering::SeqCst);

            return None;
        },

        CommandType::Unknown => {
            println!("Received an Unknown command!");
            return None;
        },
    }
}

pub fn initialize_client(address: String, client_id: String) {
    let mut stream = TcpStream::connect(address).unwrap();

    thread::sleep(Duration::from_secs(2));

    loop {
        if !CLIENT_IS_RUNING.load(Ordering::SeqCst) {
            print!("running is set to false, shutdown socket client main process!");
            break;
        }

        let up_schedule = buffer_up_mananger::buffer_up_list_schedule();

        if !(up_schedule.len() > 0) {
            if let Some(down_command) = send_ping(&mut stream) {
                buffer_down_mananger::buffer_down_schedule(down_command.clone());
            }
            thread::sleep(Duration::from_secs(2));
            continue;
        }

        for up_command in up_schedule {
            let command_to_request = Command::from_up_command(up_command);

            loop {
                let received = send(&mut stream, command_to_request.clone());

                if let Some(down_command) = handle_response(received) {
                    buffer_down_mananger::buffer_down_schedule(down_command.clone());
                    break;
                }

                thread::sleep(Duration::from_secs(2));
            }
        }
    }
}
