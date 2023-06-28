
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

use std::sync::{mpsc, Arc, Mutex};

use serde::{Serialize, Deserialize};

use serde_json::{Value, from_str};
use std::collections::HashMap;

use serde_json::json;
use lazy_static::lazy_static;

use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyString, PyDict, PyTuple, PyList};
use pyo3::wrap_pyfunction;

use crate::socket_host::enhanced_buffer;

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
}

pub fn set_socket_host_callbacks(callbacks_patterns: HashMap<String, Value>) {
    let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();
    *command_patterns = callbacks_patterns;
}

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    client_id: String,
    parity_id: String,
    priority: i32,
    command: HashMap<String, Value>,
}

fn validate_command(command: &Command, command_patterns: &HashMap<String, Value>) -> bool {
    let function_name = match command.command.get("function") {
        Some(Value::String(name)) => name,
        _ => return false,
    };

    let parameters = match command.command.get(function_name) {
        Some(parameters) => parameters,
        None => return false,
    };

    match command_patterns.get(function_name) {
        Some(pattern) => validate_parameters(parameters, pattern),
        None => false,
    }
}

fn validate_parameters(parameters: &Value, pattern: &Value) -> bool {
    match (parameters, pattern) {
        (Value::Object(params_map), Value::Object(pattern_map)) => {
            for (key, pattern_value) in pattern_map {
                match params_map.get(key) {
                    Some(param_value) => {
                        if !validate_parameters(param_value, pattern_value) {
                            return false;
                        }
                    }
                    None => return false,
                }
            }
            true
        }
        (Value::Array(params_arr), Value::Array(pattern_arr)) => {
            params_arr.len() == pattern_arr.len()
                && params_arr
                    .iter()
                    .zip(pattern_arr.iter())
                    .all(|(param, pattern)| validate_parameters(param, pattern))
        }
        (_, Value::String(pattern_type)) => match pattern_type.as_str() {
            "str" => parameters.is_string(),
            "float" => parameters.is_f64(),
            // Add more type checks here...
            _ => false,
        },
        _ => false,
    }
}


// > thread Manangement:

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = match receiver.lock().unwrap().recv() {
                Ok(job) => job,
                Err(_) => return,
            };

            println!("Worker {} got a job; executing.", id);

            job();
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}


// > Socket Functions:



pub fn get_available_commands_registered () -> HashMap<String, Value> {
    let command_patterns = COMMAND_PATTERNS.lock().unwrap();
    return command_patterns.clone();
}

pub fn initialize_host (adrress:String) {

    let listener = TcpListener::bind(adrress).unwrap();
    // TcpListener::bind is used to create a new TCP listener which will be bound to the specified address.

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    // The incoming method is called on the listener, which returns an iterator that gives us a sequence of 
    // TCP streams (representing a series of connections). The server will then handle each connection in a loop.

    // handle_connection is a function that handles each TCP stream. It reads from the stream into a buffer, 
    // then writes the contents of the buffer back to the stream.

}

fn handle_special_functions (function:String) -> Command {

    let command;

    if function == "C202" { // -> Connection conf request
        
        let mut command_map = HashMap::new();
        command_map.insert("function".to_string(), Value::String("C200".to_string()));

       command = Command {
            client_id: "some_client_id".to_string(),
            parity_id: "itisaspecialcase".to_string(),
            priority: 11,
            command: command_map,
        };

    } else if function == "C206" { // -> Ping request

        //*  Here we can check if have some data to send back to the client or not

        let mut command_map = HashMap::new();
        command_map.insert("function".to_string(), Value::String("C207".to_string()));

        command = Command {
            client_id: "some_client_id".to_string(),
            parity_id: "itisaspecialcase".to_string(),
            priority: 11,
            command: command_map,
        };

    } else { // -> Receive conf
        
        let mut command_map = HashMap::new();
        command_map.insert("function".to_string(), Value::String("C210".to_string()));

        command = Command {
            client_id: "some_client_id".to_string(),
            parity_id: "itisaspecialcase".to_string(),
            priority: 11,
            command: command_map,
        };
        
    }
    
    return command;

}

fn handle_commom_function (command:Command) {

    let command_patterns = COMMAND_PATTERNS.lock().unwrap();

    if !validate_command(&command, &command_patterns) {
        return
    } else {

    }



    // TODO >>> inteligate the function callback with the command patterns and redirect to tyhe python when they are called

}

fn handle_connection(mut stream: TcpStream)  {


    let mut buffer = [0; 4096];

    stream.read(&mut buffer).unwrap();
    
    let buffer_string = String::from_utf8_lossy(&buffer)
    .trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0')
    .to_string();

    let command: Command = serde_json::from_str(&buffer_string).unwrap();

    let special_functions: Vec<String> = vec!["C202".to_string(), "C206".to_string()];

    let command_patterns = COMMAND_PATTERNS.lock().unwrap();

    match command.command.get("function") {
        Some(Value::String(function)) => {

            if special_functions.contains(&function) { // -> Special Function Handler

                let response = handle_special_functions (function.clone());

                let command_response_json = json!(response).to_string();

                stream.write_all(command_response_json.as_bytes()).unwrap();

            } else if command_patterns.contains_key(function) { // -> Commom Function Handler

                let response = handle_commom_function(command); 

                let command_json = json!(response).to_string();

                stream.write_all(command_json.as_bytes()).unwrap();

            } else { // -> None of above

                let mut command_map = HashMap::new();
                command_map.insert("function".to_string(), Value::String("C210".to_string()));

                let command = Command {
                    client_id: "some_client_id".to_string(),
                    parity_id: "itisaspecialcase".to_string(),
                    priority: 11,
                    command: command_map,
                };

                let command_json = json!(command).to_string();

                stream.write_all(command_json.as_bytes()).unwrap();

            }

        }
        _ => {
            println!("The function name is not found or not a string.");
        }
    }

}
