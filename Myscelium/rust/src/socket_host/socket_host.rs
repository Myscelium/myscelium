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

use crate::commom::enhanced_buffer;
use crate::commom::enhanced_buffer::buffer_down_mananger::DownCommand;
use crate::commom::enhanced_buffer::buffer_up_mananger::UpCommand;
use crate::commom::enhanced_buffer::utilities::Command;

#[macro_use]
use crate::{init_thread_pool, terminate_pool, run_in_thread_pool, wait_all_threads};
use crate::commom::custom_thread_pool::thread_pool::UnifiedThreadPool;
use std::time::Duration;

// > Global Vars Core

use crate::HOST_IS_RUNING;
use std::sync::atomic::Ordering;

use pyo3::exceptions::PyException;
use pyo3::types::PyFunction;

use super::host_logger;
use super::host_logger::log_handler::Logger;
use crate::HOST_LOG_LEVEL;
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
    static ref HEARTBEAT_CALLBACK: Arc<Mutex<HashMap<String, (Py<PyFunction>, Value)>>> = {
        let command_patterns: HashMap<String, (Py<PyFunction>, Value)> = HashMap::new();
        Arc::new(Mutex::new(command_patterns))
    };
    static ref CONNECTION_HENDLER_POOL: Arc<Mutex<UnifiedThreadPool>> = {
        let mut max_connections;
        {
            let max_conns = MAX_CONS.lock().unwrap();
            max_connections = *max_conns;
        }

        init_thread_pool!(max_connections as usize)
    };
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

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let host_log_level;
        {
            host_log_level = HOST_LOG_LEVEL.lock().clone();
        }
        Logger::new(host_log_level, $section_name)
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

pub fn set_heartbeat_callback(callback_pattern: HashMap<String, (Py<PyFunction>, Value)>) {
    {
        let mut heart_beat_callback = HEARTBEAT_CALLBACK.lock().unwrap();
        *heart_beat_callback = callback_pattern;
    }
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

fn dict_to_kwargs<'l>(py: Python<'l>, dict: &HashMap<String, Value>) -> PyResult<HashMap<String, PyObject>> {
    let logger = acquire_logger!("py dict to kwargs converter");

    // Check if the dict contains the function name as a key
    if !dict.contains_key("args") {
        // If it does not, return an empty HashMap since there are no arguments
        let kwargs: HashMap<String, PyObject> = HashMap::new();
        return Ok(kwargs);
    }

    let args_string = match dict.get("args") {
        Some(Value::String(s)) => s,
        _ => return Err(PyErr::new::<PyException, _>("The args key is not found or not a string.")),
    };

    let sub_dict: HashMap<String, Value> = serde_json::from_str(args_string).unwrap();

    logger.debug(format!("Args extracted: {:?}", sub_dict));

    let mut kwargs: HashMap<String, PyObject> = HashMap::new();
    for (key, value) in sub_dict.iter() {
        let py_value = match value {
            Value::String(s) => s.into_py(py),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    i.into_py(py)
                } else if let Some(f) = n.as_f64() {
                    f.into_py(py)
                } else {
                    return Err(PyErr::new::<PyException, _>("Unsupported number type."));
                }
            },
            Value::Bool(b) => b.into_py(py),
            _ => return Err(PyErr::new::<PyException, _>("Unsupported value type.")),
        };
        kwargs.insert(key.clone(), py_value);
    }

    logger.debug(format!("kwargs: {:?}", kwargs));

    Ok(kwargs)
}

pub fn update_last_contact(py: Python<'_>, client_id: String) {
    let mut clients = CLIENTS_ALLOWED.lock().unwrap();
    if let Some(client) = clients.get_mut(&client_id) {
        client.last_contact = SystemTime::now();
    }

    let function_name = "handle_client_contact";

    let callback_patterns = HEARTBEAT_CALLBACK.lock().unwrap();

    let function = match callback_patterns.get(function_name) {
        Some(function) => function.clone(),
        _ => return,
    };

    // Get the function and args_types from the CALLBACK_PATTERNS

    // let mut command: HashMap<String, Value> = HashMap::new();

    // command.insert("client_id".to_string(), );

    // let kwargs_map = dict_to_kwargs(py, &command)
    //     .map_err(|e| {
    //         eprintln!("Error converting arguments to kwargs: {:?}", e);
    //         PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs: {:?}", e))
    //     })
    //     .unwrap();

    // let kwargs = PyDict::new(py);
    // for (key, value) in kwargs_map {
    //     kwargs.set_item(key, value).unwrap();
    // }

    let kwargs = PyDict::new(py);

    let py_client_id = &client_id.into_py(py);

    kwargs.set_item("client_id".to_string(), py_client_id).unwrap();

    let logger = acquire_logger!("Heartbeat Callback Handler");

    // Call the Python function with the converted arguments
    let result = function.0.call(py, (), Some(kwargs)).map_err(|e| {
        logger.exception(format!("Error calling function: {:?}", e));
        e
    });
}

// > Commands Manangemement & Checking

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

// > Socket Interactive Functions:

pub fn set_max_conns(n_max_conns: u32) {
    host_logger::register::register_mananger::set_workers_num(n_max_conns.clone() * 7); // 7 * n because we need 7 for each

    let mut default_max_conns = MAX_CONS.lock().unwrap();

    *default_max_conns = n_max_conns;
}

pub fn set_socket_host_callbacks(callbacks_patterns: HashMap<String, Value>) {
    let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();
    *command_patterns = callbacks_patterns;
}

pub fn initialize_host_buffer(buffer_location: String) {
    println!("inicializing the buffer database into: {}buffer.db, if not inicialized!", buffer_location);

    enhanced_buffer::buffer_down_mananger::buffer_down_initialize_table(buffer_location.clone());

    enhanced_buffer::buffer_up_mananger::buffer_up_initialize_table(buffer_location.clone());

    println!("All buffer initialized succefully!");

    return;
}

pub fn initialize_host(address: String, client_id: String) {
    let logger = acquire_logger!("Core");

    let mut actual_client_id = CLIENT_ID.lock().unwrap();
    *actual_client_id = client_id;

    let default_max_conns = MAX_CONS.lock().unwrap();

    let listener = TcpListener::bind(&address).unwrap();

    logger.info(format!("Listening: {}", address));

    loop {
        println!("Waiting conn!");

        // Keep the thread alive until HOST_IS_RUNING is set to false
        if !HOST_IS_RUNING.load(Ordering::SeqCst) {
            // Lock the pool and stop it

            terminate_pool!(CONNECTION_HENDLER_POOL);
            println!("Stopped the thread pool!");
            break;
        }

        let mut receivers = Vec::new();

        match listener.accept() {
            Ok((stream, _)) => {
                let rx = run_in_thread_pool!(CONNECTION_HENDLER_POOL, {
                    handle_connection(stream);
                });
                receivers.push(rx);
            },
            Err(e) => {
                logger.warn(format!("Failed to accept a connection: {}", e));
            },
        }

        wait_all_threads!(receivers);

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

    let down_command = DownCommand::new(command.client_id.clone(), command.parity_id, command.priority, json_command);

    enhanced_buffer::buffer_down_mananger::buffer_down_schedule(down_command);

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
    // Aquire logger to section Handle Conn
    let logger = acquire_logger!("Core");

    loop {
        let mut buffer = [0; 4096];

        match stream.read(&mut buffer) {
            Ok(0) => {
                // No data was read, break the loop
                continue;
            },
            Ok(bytes_read) => {
                logger.debug("Data received!".to_string());
            },
            Err(e) => {
                // Handle the error
                logger.exception(format!("Failed to read from the stream: {}", e));
            },
        }

        let buffer_string = String::from_utf8_lossy(&buffer)
            .trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0')
            .to_string();

        let command: Command = serde_json::from_str(&buffer_string).unwrap(); // TODO >>> Fix the error treatment in the cases that results in a error

        logger.debug(format!("Command received:\n{:?}\n", command));

        let special_functions: Vec<String> = vec!["C202".to_string(), "C206".to_string()];

        let command_patterns = COMMAND_PATTERNS.lock().unwrap();

        if !is_client_registred(&command.client_id) {
            // -> In case client isn't registred in the clients allowed

            let response = create_command_error!(command.client_id, command.parity_id, "Your client isn't registred in the whitelist!");

            let command_response_json = json!(response).to_string();

            logger.exception(format!("WARNING: Client isn't registred, sending back: {:?}", command_response_json));

            stream.write_all(command_response_json.as_bytes()).unwrap();

            return;
        }

        // ! WE CAN USE THIS PY AQUIRE UNTILL THE PYTHON POOL IS FINISHED

        // let py;

        // {
        //     let getting_py = unsafe { Python::assume_gil_acquired() };

        //     let gil_pool = unsafe { getting_py.clone().new_pool() };

        //     py = gil_pool.python();

        //     logger.debug("Aquired python to call client heart beat handler callback!".to_string());

        //     update_last_contact(py, command.client_id.clone());
        // }

        match command.command.get("function") {
            Some(Value::String(function)) => {
                logger.debug(format!("Comand function: {}", function));

                if special_functions.contains(&function) {
                    // -> Special Function Handler

                    let response = handle_special_functions(command.client_id, function.clone());

                    let command_response_json = json!(response).to_string();

                    logger.debug(format!("Sending back: {:?}", command_response_json));

                    stream.write_all(command_response_json.as_bytes()).unwrap();
                } else if command_patterns.contains_key(function) {
                    // -> Commom Function Handler

                    logger.debug("Command is in command patterns!".to_string());

                    let command_is_not_registry: bool =
                        enhanced_buffer::buffer_up_mananger::check_if_parity_id_is_registred(command.parity_id.clone());

                    let response: Command;

                    if !command_is_not_registry {
                        logger.warn(format!("Command {}, alwready have a response!", command.parity_id.clone()));

                        match get_response(command.clone()) {
                            Response::Command(c) => {
                                response = c;
                            },
                            Response::None => {
                                logger.info("Response is None!".to_string());

                                response = create_sepecial_command!(command.client_id, "C210");
                            },
                        }
                    } else {
                        response = handle_commom_function(command);
                    }

                    let command_json = json!(response).to_string();

                    logger.debug(format!("Sending back: {:?}", command_json));

                    stream.write_all(command_json.as_bytes()).unwrap();
                } else {
                    // -> None of above

                    let command = create_command_error!(command.client_id, command.parity_id, format!("Function: {}, Doesn't exist!", function));

                    let command_json = json!(command).to_string();

                    logger.debug(format!("Sending back: {:?}", command_json));

                    stream.write_all(command_json.as_bytes()).unwrap();
                }
            },
            _ => {
                logger.warn("The function name is not found or not a string.".to_string());
            },
        }
    }
}
