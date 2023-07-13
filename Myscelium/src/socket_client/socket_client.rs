use crate::socket_client::enhanced_buffer;
use crate::socket_host::enhanced_buffer::buffer_up_mananger;
use lazy_static::lazy_static;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use serde_json::{Value, from_str};

use serde::{Serialize, Deserialize};

use crate::socket_host::socket_host::is_client_registred;

use crate::socket_host::enhanced_buffer::buffer_down_mananger::DownCommand;

use std::sync::{Condvar, atomic::{AtomicBool, Ordering}};

use pyo3::types::{IntoPyDict, PyString, PyInt, PyAny, PyDict, PyTuple, PyList, PyFunction, PyBool, PyFloat};
use pyo3::{Python, PyResult, PyObject, PyErr};

use pyo3::IntoPy;

use pyo3::Py;
use pyo3::exceptions::PyException;

use std::time::{Duration, Instant};

use std::error::Error;

use pyo3::ToPyObject;

use crate::CLIENT_IS_RUNING;

use std::fmt;

use std::net::TcpStream;

use std::io::Write;
use std::io::Read;

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


pub fn set_socket_client_callbacks_patterns (callbacks_patterns: HashMap<String, Value>) {
    let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();
    *command_patterns = callbacks_patterns;
}

pub fn initialize_client_buffer (buffer_location:String) {

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


pub fn get_socket_client_available_commands_registered () -> HashMap<String, Value> {
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
struct Command {
    client_id: String,
    parity_id: String,
    priority: u8,
    command: HashMap<String, Value>,
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

impl Command {

    fn new(client_id: &str, parity_id: &str, priority:u8, command:HashMap<String, Value>) -> Self {
        
        Self {

            client_id: client_id.to_string(),
            parity_id: parity_id.to_string(),
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

    fn from_up_command (&self, up_command: UpCommand) -> Self {

        let client_id = up_command.client_id.clone();
        let parity_id = up_command.parity_id.clone();
        let priority = up_command.priority.clone();
        let command: HashMap<String, Value> = serde_json::from_str(&up_command.command).unwrap();

        Self {

            client_id,
            parity_id,
            priority,
            command

        }


    }

}

use serde_json::to_string;

fn verify_connection (stream:&mut TcpStream) -> bool{

    let mut command_map = HashMap::new();
    command_map.insert("function".to_string(), Value::String("C202".to_string()));

    let command = Command {
        client_id: "some_client_id".to_string(),
        parity_id: "itisaspecialcase".to_string(),
        priority: 11,
        command: command_map,
    };

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
                return true
            } else {
                return false;
            } 
        }
        _ => {
            println!("The function name is not found or not a string.");
            return false
        }
    }
    
}

fn send (stream:&mut TcpStream, command:Command) -> Response {

    let conn:bool = verify_connection(stream);

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

    return Response::Command (command);
    
}

use buffer_up_mananger::UpCommand;

fn initialize_client (address:String) {
    
    let mut stream = TcpStream::connect(address).unwrap();

    let up_schedule = buffer_up_mananger::buffer_up_list_schedule();

    

    for up_command in up_schedule {
        
        loop {

            let received = send(&mut stream, );
            let mut command_received:Command;




        }



    }



    loop {

        thread::sleep(Duration::from_secs(5));

        let received = send(&mut stream, request_command.clone());

        let mut command_received;

        match received {    

            Response::None => {
                println!("Received invalid data!");
                continue;
            }
            Response::Command(c) => {
                println!("Received command: {:?}", c);
                command_received = c
            }

        }

        match command_received.command_type() {
            
            CommandType::Function(f) => {

                let function:String = serde_json::from_str(&f).unwrap();

                if command_received.parity_id != "itisaspecialcase" {
                    if function == "C210".to_string() {
                        println!("Received Confirmation!");
                        break;
                    } else if function == "Error".to_string() {
                        println!("An error ocurred in host, the error was: {}", command_received.command.get("Error").unwrap());
                        break;
                    }
                }

                println!("Receive a function: {:?}", f);
            
            }

            CommandType::Response(r) => {

                println!("Received a response!");   

                let response_parity_id = command_received.parity_id;

                if response_parity_id == request_command.parity_id {
                    println!("Response matches the request parity_id: {:?}", request_command.parity_id);

                    let response_command = command_received.command;

                    println!("The receive response is: {:?}", response_command);

                    break;

                }
            
            }

            CommandType::Unknown => {
                println!("Received a Unknown command!")
            }

        }

    }

}