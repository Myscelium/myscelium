use crate::socket_client::enhanced_buffer;
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

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    id: i32,
    client_id: String,
    parity_id: String,
    priority: i32,
    command: HashMap<String, String>
}

use serde_json::to_string;

fn main() {

    let command = Command {
        id: 1,
        client_id: "client1".to_string(),
        parity_id: "parity1".to_string(),
        priority: 2,
        command: {
            let mut m = HashMap::new();
            m.insert("key".to_string(), "value".to_string());
            m
        },
    };

    let json = to_string(&command).unwrap();
    println!("{}", json);

    // -> Socket Client:

    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();

    let command = json!({
        "function": "get_symbols_data",
        "parameters": {
            "symbols_data": {
                "data-type": "AAPL",
                "symbols": "AAPL",
                "start-ts": 162.34,
                "end-ts": 163.34
            }
        }
    });

    let command_string = command.to_string();

    // let msg = b"hello, world!";
    // stream.write(msg).unwrap();

    stream.write_all(command_string.as_bytes()).unwrap();

    // write is used to send a message to the server.

    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    // Then we read the response from the server into a buffer and print it out.

    println!("Received: {}", String::from_utf8_lossy(&buffer[..]))

}
