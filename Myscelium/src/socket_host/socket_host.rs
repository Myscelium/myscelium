use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::thread;

use std::sync::{mpsc, Arc, Mutex};
use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use serde_json::{from_str, Value};
use std::collections::HashMap;

use lazy_static::lazy_static;
use serde_json::json;

use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict, PyList, PyString, PyTuple};
use pyo3::wrap_pyfunction;

use crate::socket_host::enhanced_buffer;

use crate::socket_host::enhanced_buffer::buffer_up_mananger::UpCommand;

use std::time::Duration;

// > Global Vars Core

use crate::HOST_IS_RUNING;
use std::sync::atomic::Ordering;

#[derive(Debug, Clone)]
pub struct Client {
    client_id: String,
    last_contact: SystemTime,
    client_type: String,
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
    static ref MAX_CONS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));
    static ref CLIENT_ID: Arc<Mutex<String>> = Arc::new(Mutex::new(' '.to_string()));
    static ref CLIENTS_ALLOWED: Arc<Mutex<HashMap<String, Client>>> = Arc::new(Mutex::new(HashMap::new()));
}

macro_rules! create_command_error {
    ($client_id:expr, $parity_id:expr, $error:expr) => {{
        let mut command_map = HashMap::new();
        command_map.insert("error".to_string(), Value::String($error.to_string()));

        let command = Command {
            client_id: $client_id.to_string(),
            parity_id: $parity_id.to_string(),
            priority: 11,
            command: command_map,
        };
        command
    }};
}

macro_rules! create_sepecial_command {
    ($client_id:expr, $response:expr) => {{
        let mut command_map = HashMap::new();
        command_map.insert("function".to_string(), Value::String($response.to_string()));

        let command = Command {
            client_id: $client_id.to_string(),
            parity_id: "itisaspecialcase".to_string(),
            priority: 11,
            command: command_map,
        };
        command
    }};
}

macro_rules! create_response_command {
    ($client_id:expr, $parity_id:expr, $priority:expr, $response:expr) => {{
        let mut command_map = HashMap::new();
        command_map.insert("response".to_string(), Value::String($response.to_string()));

        let command = Command {
            client_id: $client_id.to_string(),
            parity_id: $parity_id.to_string(),
            priority: $priority,
            command: command_map,
        };
        command
    }};
}

pub fn is_client_registred(client_id: &String) -> bool {
    let clients;

    {
        clients = CLIENTS_ALLOWED.lock().unwrap().clone();
    }

    clients.contains_key(client_id)
}

pub fn register_client(client_id: String, client_type: String) {
    if !is_client_registred(&client_id) {
        let mut clients = CLIENTS_ALLOWED.lock().unwrap();

        clients.insert(
            client_id.clone(),
            Client {
                client_id,
                last_contact: SystemTime::now(),
                client_type,
            },
        );
    }
}

pub fn update_last_contact(client_id: String) {
    let mut clients = CLIENTS_ALLOWED.lock().unwrap();
    if let Some(client) = clients.get_mut(&client_id) {
        client.last_contact = SystemTime::now();
    }
}

// > Commands Manangemement & Checking

#[derive(Debug)]
enum CommandType {
    Function(String),
    Response(String),
    Redirect(String),
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Command {
    client_id: String,
    parity_id: String,
    priority: u8,
    command: HashMap<String, Value>,
}

impl Command {
    fn new(client_id: String, parity_id: String, priority: u8, command: HashMap<String, Value>) -> Self {
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
        } else if self.command.contains_key("redirect") {
            CommandType::Redirect(self.command.get("redirect").unwrap().to_string())
        } else {
            CommandType::Unknown
        }
    }
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
                    },
                    None => return false,
                }
            }
            true
        },
        (Value::Array(params_arr), Value::Array(pattern_arr)) => {
            params_arr.len() == pattern_arr.len()
                && params_arr
                    .iter()
                    .zip(pattern_arr.iter())
                    .all(|(param, pattern)| validate_parameters(param, pattern))
        },
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

type Job = Option<Box<dyn FnOnce() + Send + 'static>>;

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
        self.sender.send(Some(job)).unwrap();
    }

    pub fn stop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(None).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = match receiver.lock().unwrap().recv() {
                Ok(Some(job)) => job,
                Ok(None) => return,
                Err(_) => return,
            };

            println!("Worker {} got a job; executing.", id);

            job();
        });

        Worker { id, thread: Some(thread) }
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        for _ in &self.workers {
            self.sender.send(None).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// > Socket Interactive Functions:

pub fn set_max_conns(n_max_conns: u32) {
    let mut default_max_conns = MAX_CONS.lock().unwrap();

    *default_max_conns = n_max_conns;
}

pub fn set_socket_host_callbacks(callbacks_patterns: HashMap<String, Value>) {
    let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();
    *command_patterns = callbacks_patterns;
}

pub fn initialize_host_buffer(buffer_location: String) {
    println!("\ninicializing the buffer database into: {}buffer.db, if not inicialized!", buffer_location);

    enhanced_buffer::buffer_down_mananger::buffer_down_initialize_table(buffer_location.clone());

    enhanced_buffer::buffer_up_mananger::buffer_up_initialize_table(buffer_location.clone());

    println!("\nAll buffer initialized succefully!\n");

    return;
}

fn pool_stoping_event_controler(pool: Arc<Mutex<ThreadPool>>) {
    loop {
        // Stop the thread pool
        if !HOST_IS_RUNING.load(Ordering::SeqCst) {
            pool.lock().unwrap().stop();
            println!("Stopped the thread pool!");
            break;
        }

        // Sleep for a while before checking again
        thread::sleep(Duration::from_millis(1));
    }

    return;
}

pub fn initialize_host(address: String, client_id: String) {
    let mut actual_client_id = CLIENT_ID.lock().unwrap();
    *actual_client_id = client_id;

    let default_max_conns = MAX_CONS.lock().unwrap();

    let listener = TcpListener::bind(&address).unwrap();

    println!("Listening: {}", address);

    let pool = Arc::new(Mutex::new(ThreadPool::new(*default_max_conns as usize)));

    let pool_clone = Arc::clone(&pool);
    thread::spawn(move || pool_stoping_event_controler(pool_clone));

    loop {
        println!("Waiting conn!");

        // Keep the thread alive until HOST_IS_RUNING is set to false
        if !HOST_IS_RUNING.load(Ordering::SeqCst) {
            print!("runing is set to false, skipping");
            break;
        }

        match listener.accept() {
            Ok((stream, _)) => {
                let pool_clone = Arc::clone(&pool);
                pool_clone.lock().unwrap().execute(move || {
                    handle_connection(stream);
                });
            },
            Err(e) => {
                eprintln!("Failed to accept a connection: {}", e);
            },
        }

        thread::sleep(Duration::from_secs(1));
    }
}
// The incoming method is called on the listener, which returns an iterator that gives us a sequence of
// TCP streams (representing a series of connections). The server will then handle each connection in a loop.

// handle_connection is a function that handles each TCP stream. It reads from the stream into a buffer,
// then writes the contents of the buffer back to the stream.

pub fn get_available_commands_registered() -> HashMap<String, Value> {
    let command_patterns = COMMAND_PATTERNS.lock().unwrap();
    return command_patterns.clone();
}

// > Socket main structure:

fn handle_special_functions(client_id: String, function: String) -> Command {
    let command;

    if function == "C202" {
        // -> Connection conf request
        command = create_sepecial_command!(client_id, "C200");
    } else if function == "C206" {
        // -> Ping request
        command = create_sepecial_command!(client_id, "C207");
        // TODO >>>  Here we can check if have some data to send back to the client or not
    } else {
        // -> Receive conf
        command = create_sepecial_command!(client_id, "C210");
    }

    return command;
}

fn handle_commom_function(command: Command) -> Command {
    // let actual_client_id = CLIENT_ID.lock().unwrap();

    let mut command_map = HashMap::new();
    command_map.insert("function".to_string(), Value::String("C210".to_string()));

    let response_command = Command::new("some_client_id".to_string(), "itisaspecialcase".to_string(), 11, command_map);

    // TODO >> If have responses in the dabase to the client here is a good idea to send back

    // let command_patterns = COMMAND_PATTERNS.lock().unwrap();

    // if !validate_command(&command, &command_patterns) {
    //     return response_command;
    // } else {

    // }

    let json_command = serde_json::to_string(&command.command).unwrap();

    enhanced_buffer::buffer_down_mananger::buffer_down_schedule(command.client_id.clone(), command.parity_id, command.priority, json_command);

    // TODO >>> Add a mecanism to get the buffer up responses and send back to client or redirect to antoher client

    return response_command;
}

enum Response {
    Command(Command),
    None,
}

fn get_response(command: Command) -> Response {
    let up_schedule: Vec<UpCommand> =
        enhanced_buffer::buffer_up_mananger::buffer_up_get_scheduled_by_parity_id(command.client_id.clone(), command.parity_id.clone());

    if !(up_schedule.len() > 0) {
        return Response::None;
    }

    let command_response = &up_schedule[0];

    let response_command =
        create_response_command!(command_response.client_id, command_response.parity_id, command_response.priority, command_response.command);

    enhanced_buffer::buffer_up_mananger::buffer_up_remove_schedule_by_parity_id(command.client_id.clone(), response_command.parity_id.clone());

    return Response::Command(response_command);
}

fn handle_connection(mut stream: TcpStream) {
    loop {
        let mut buffer = [0; 4096];

        match stream.read(&mut buffer) {
            Ok(0) => {
                // No data was read, break the loop
                continue;
            },
            Ok(bytes_read) => {
                println!("Data received!");
            },
            Err(e) => {
                // Handle the error
                eprintln!("Failed to read from the stream: {}", e);
            },
        }

        let buffer_string = String::from_utf8_lossy(&buffer)
            .trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0')
            .to_string();

        let command: Command = serde_json::from_str(&buffer_string).unwrap();

        println!("\nCommand received:\n{:?}\n", command);

        let special_functions: Vec<String> = vec!["C202".to_string(), "C206".to_string()];

        let command_patterns = COMMAND_PATTERNS.lock().unwrap();

        if !is_client_registred(&command.client_id) {
            // -> In case client isn't registred in the clients allowed

            let response = create_command_error!(command.client_id, command.parity_id, "Your client isn't registred in the whitelist!");

            let command_response_json = json!(response).to_string();

            println!("WARNING: Client isn't registred, sending back: {:?}", command_response_json);

            stream.write_all(command_response_json.as_bytes()).unwrap();

            return;
        }

        update_last_contact(command.client_id.clone());

        match command.command.get("function") {
            Some(Value::String(function)) => {
                println!("Comand function: {}", function);

                if special_functions.contains(&function) {
                    // -> Special Function Handler

                    let response = handle_special_functions(command.client_id, function.clone());

                    let command_response_json = json!(response).to_string();

                    println!("Sending back: {:?}", command_response_json);

                    stream.write_all(command_response_json.as_bytes()).unwrap();
                } else if command_patterns.contains_key(function) {
                    // -> Commom Function Handler

                    println!("Command is in command patterns!");

                    let command_is_not_registry: bool =
                        enhanced_buffer::buffer_up_mananger::check_if_parity_id_is_registred(command.parity_id.clone());

                    let response: Command;

                    if !command_is_not_registry {
                        println!("Command {}, alwready have a response!", command.parity_id.clone());

                        match get_response(command.clone()) {
                            Response::Command(c) => {
                                response = c;
                            },
                            Response::None => {
                                println!("Response is None!");

                                response = create_sepecial_command!(command.client_id, "C210");
                            },
                        }
                    } else {
                        response = handle_commom_function(command);
                    }

                    let command_json = json!(response).to_string();

                    println!("Sending back: {:?}", command_json);

                    stream.write_all(command_json.as_bytes()).unwrap();
                } else {
                    // -> None of above

                    let command = create_command_error!(command.client_id, command.parity_id, format!("Function: {}, Doesn't exist!", function));

                    let command_json = json!(command).to_string();

                    println!("Sending back: {:?}", command_json);

                    stream.write_all(command_json.as_bytes()).unwrap();
                }
            },
            _ => {
                println!("The function name is not found or not a string.");
            },
        }
    }
}
