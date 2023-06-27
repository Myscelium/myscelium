
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

use std::sync::{mpsc, Arc, Mutex};

use serde::{Serialize, Deserialize};

use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    function: String,
    parameters: Value,
}

fn validate_command(command: &Command, command_patterns: &HashMap<String, Value>) -> bool {
    match command_patterns.get(&command.function) {
        Some(pattern) => validate_parameters(&command.parameters, pattern),
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

fn main() {

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
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

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();

    
    let command_patterns: HashMap<String, Value> = serde_json::from_str(
        r#"{

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

        }"#,
    )
    .unwrap();

    let buffer_string = String::from_utf8_lossy(&buffer)
    .trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0')
    .to_string();

    /*
        In this code, trim_end_matches is used to remove any trailing newline,
        carriage return, or null characters from the string. 
        The |c| c == '\n' || c == '\r' || c == '\0' part is a closure that returns 
        true for newline, carriage return, and null characters, causing 
        trim_end_matches to remove them.

     */

    println!("Request: {}", buffer_string);

    println!("Length: {}", buffer_string.len());
    println!("Last characters: {:?}", &buffer_string[(buffer_string.len() - 10)..]);

    let command: Command = serde_json::from_str(&buffer_string).unwrap();
    println!("Command: {:?}", command);

    println!("{}", validate_command(&command, &command_patterns));

    stream.write(&buffer).unwrap();
    stream.flush().unwrap();
}