use lazy_static::lazy_static;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use serde::{Deserialize, Serialize};

use crate::socket_host::client_mananger::mananger::check_if_client_key_exists;

use crate::commom::enhanced_buffer;
use crate::commom::enhanced_buffer::buffer_down_mananger::DownCommand;
use crate::commom::enhanced_buffer::buffer_up_mananger::UpCommand;
use crate::commom::enhanced_buffer::utilities::{Command, CommandType};

#[macro_use]
use crate::{init_thread_pool, terminate_pool, run_in_thread_pool, wait_all_threads};
use crate::commom::custom_thread_pool::thread_pool::UnifiedThreadPool;

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

use crate::HOST_IS_RUNNING;

use std::fmt;

use super::host_logger;
use super::host_logger::log_handler::Logger;
use crate::HOST_LOG_LEVEL;

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let host_log_level;
        {
            host_log_level = HOST_LOG_LEVEL.lock().clone();
        }
        Logger::new(host_log_level, $section_name)
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
    static ref CALLBACK_PATTERNS: Arc<Mutex<HashMap<String, (Py<PyFunction>, Value)>>> = {
        let command_patterns: HashMap<String, (Py<PyFunction>, Value)> = HashMap::new();
        Arc::new(Mutex::new(command_patterns))
    };
    static ref NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));
}

pub fn set_socket_host_transposer_workers_num(n_workers: u32) {
    host_logger::register::register_mananger::set_workers_num(n_workers.clone() * 7); // 7 * n because we need 7 for each
    let mut default_num_of_workers = NUM_WORKERS.lock().unwrap();

    *default_num_of_workers = n_workers;

    enhanced_buffer::buffer_down_mananger::set_workers_num(n_workers);
    enhanced_buffer::buffer_up_mananger::set_workers_num(n_workers);
}

pub fn set_socket_host_transposer_callbacks(commands_patterns: HashMap<String, Value>, callbacks_patterns: HashMap<String, (Py<PyFunction>, Value)>) {
    let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();
    *command_patterns = commands_patterns;

    let mut callback_patterns = CALLBACK_PATTERNS.lock().unwrap();
    *callback_patterns = callbacks_patterns;
}

// > Transposer:

fn dict_to_tuple<'l>(py: Python<'l>, dict: &HashMap<String, Value>) -> PyResult<&'l PyTuple> {
    let logger = acquire_logger!("Transposer - Py Dict to Tuple Converter");

    // Check if the dict contains the function name as a key
    if !dict.contains_key("args") {
        // If it does not, return an empty Vec since there are no arguments
        let mut values: Vec<PyObject> = Vec::new();
        return Ok(PyTuple::new(py, values));
    }

    let args_string = match dict.get("args") {
        Some(Value::String(s)) => s,
        _ => return Err(PyErr::new::<PyException, _>("The args key is not found or not a string.")),
    };

    let sub_dict: HashMap<String, Value> = serde_json::from_str(args_string).unwrap();

    logger.debug(format!("Args extracted: {:?}", sub_dict));

    let mut values: Vec<PyObject> = Vec::new();
    for value in sub_dict.values() {
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
        values.push(py_value);
    }

    let py_tuple = PyTuple::new(py, &values);

    logger.debug(format!("py_tuple: {}", py_tuple));

    Ok(py_tuple)
}

fn dict_to_kwargs<'l>(py: Python<'l>, dict: &HashMap<String, Value>) -> PyResult<HashMap<String, PyObject>> {
    let logger = acquire_logger!("Transposer - Py Dict to kwargs Converter");

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

fn handle_command(py: Python<'_>, command: Command) -> PyResult<PyObject> {
    let logger = acquire_logger!("Transposer - Handle Command");

    logger.debug(format!("Getting function name..."));
    let function_name: &String = match command.command.get("function") {
        Some(Value::String(function_name)) => function_name,
        _ => return Err(PyErr::new::<PyException, _>("The function name is not found or not a string.")),
    };
    logger.debug(format!("Got function name: {}", function_name));

    // Get the function and args_types from the CALLBACK_PATTERNS
    let callback_patterns = CALLBACK_PATTERNS.lock().unwrap();
    let (function, _) = callback_patterns.get(function_name).unwrap();

    let kwargs_map = dict_to_kwargs(py, &command.command).map_err(|e| {
        logger.exception(format!("Error converting arguments to kwargs: {:?}", e));
        PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs: {:?}", e))
    })?;

    let kwargs = PyDict::new(py);
    for (key, value) in kwargs_map {
        kwargs.set_item(key, value).unwrap();
    }

    // Call the Python function with the converted arguments
    let result = function.call(py, (), Some(kwargs)).map_err(|e| {
        logger.exception(format!("Error calling function: {:?}", e));
        e
    })?;

    let result_obj: PyObject = result.clone().into(); // Convert the result into a PyObject

    Ok(result_obj) // Return the PyObject
}

// Define a custom type that can be either Empty, Map, or Error
#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum ResultType {
    Map(HashMap<String, ResultType>),
    List(Vec<ResultType>),
    Str(String),
    Int(i32),
    Float(f64),
    Bool(bool),
    Empty,
    Error(String), // Assuming Error variant holds a String
                   // ... any other variants you might have
}

// Implement Display for ResultType to be able to print it
impl fmt::Display for ResultType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResultType::Empty => write!(f, "Empty"),
            ResultType::Str(s) => write!(f, "\"{}\"", s),
            ResultType::Int(i) => write!(f, "{}", i),
            ResultType::Float(fl) => write!(f, "{}", fl),
            ResultType::Bool(b) => write!(f, "{}", b),
            ResultType::List(list) => {
                write!(f, "[")?;
                for (index, item) in list.iter().enumerate() {
                    if index != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            },
            ResultType::Map(map) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, value) in map {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                    first = false;
                }
                write!(f, "}}")
            },
            ResultType::Error(err) => write!(f, "Error: {}", err),
        }
    }
}

fn handle_pyobject(py: Python, obj: PyObject) -> ResultType {
    if let Ok(dict) = obj.cast_as::<PyDict>(py) {
        let mut rust_dict = HashMap::new();

        for (key, value) in dict.iter() {
            let key_str: String = key.extract().unwrap();
            if let Ok(value_str) = value.extract::<String>() {
                rust_dict.insert(key_str, ResultType::Str(value_str));
            } else if let Ok(value_int) = value.extract::<i32>() {
                rust_dict.insert(key_str, ResultType::Int(value_int));
            } else if let Ok(value_list) = value.extract::<Vec<String>>() {
                let rust_list = value_list.into_iter().map(ResultType::Str).collect();
                rust_dict.insert(key_str, ResultType::List(rust_list));
            } else if let Ok(nested_dict) = value.cast_as::<PyDict>() {
                let inner_map = handle_pyobject(py, nested_dict.into());
                rust_dict.insert(key_str, inner_map);
            } else {
                // Handle other types as needed
            }
        }

        return ResultType::Map(rust_dict);
    } else if let Ok(tuple) = obj.cast_as::<PyTuple>(py) {
        // Handle tuple
        let rust_list: Vec<_> = tuple.iter().map(|item| handle_pyobject(py, item.into())).collect();
        return ResultType::List(rust_list);
    } else if let Ok(list) = obj.cast_as::<PyList>(py) {
        // Handle list
        let rust_list: Vec<_> = list.iter().map(|item| handle_pyobject(py, item.into())).collect();
        return ResultType::List(rust_list);
    } else if let Ok(int) = obj.cast_as::<PyInt>(py) {
        // Handle int
        return ResultType::Int(int.extract().unwrap());
    } else if let Ok(float) = obj.cast_as::<PyFloat>(py) {
        // Handle float
        return ResultType::Float(float.extract().unwrap());
    } else if let Ok(string) = obj.cast_as::<PyString>(py) {
        // Handle string
        return ResultType::Str(string.extract().unwrap());
    } else if let Ok(boolean) = obj.cast_as::<PyBool>(py) {
        // Handle bool
        return ResultType::Bool(boolean.extract().unwrap());
    } else if obj.is_none(py) {
        // Handle None
        return ResultType::Empty;
    }

    return ResultType::Empty;
}

macro_rules! error_response {
    ($msg:expr) => {{
        println!("{:?}", $msg);
        let mut error_map = HashMap::new();
        error_map.insert("Error".to_string(), $msg.to_string());
        serde_json::to_string(&error_map)
    }};
}

fn process(py: Python, down_command: DownCommand) {
    let logger = acquire_logger!("Transposer - Process");

    fn handle_redirect(m: HashMap<String, String>, client_id: &mut String, down_command: DownCommand) -> Result<std::string::String, serde_json::Error> {
        let response;

        if !m.contains_key("redirect_to") {
            return error_response!("Error! Callback response args don't have redirect_to client_id field!");
        }

        let redirect_to = m.get("redirect_to").unwrap();

        if !check_if_client_key_exists(redirect_to.to_string()) {
            return error_response!(format!("Error! request to redirect to client_id: {} failed, client doesn't exist!", redirect_to.to_string()));
        }

        let up_command = UpCommand::new(client_id.clone(), down_command.parity_id.clone(), down_command.priority.clone(), "C210".to_string());

        enhanced_buffer::buffer_up_mananger::buffer_up_schedule(up_command);

        *client_id = redirect_to.to_string(); // > Update the client id that it will send to

        if !m.contains_key("response") {
            return error_response!("Error! Callback response args don't have response kwarg!");
        }

        let mut to_send = HashMap::new();

        to_send.insert("response_mode".to_string(), "to_origin".to_string());
        to_send.insert("response_activation_function".to_string(), "to_origin".to_string());
        to_send.insert("response".to_string(), m.get("response").unwrap().to_string());

        response = serde_json::to_string(&to_send);

        // {'response_mode':'to_origin', 'response_activation_function':response_activation_function, 'response':response}

        return response;
    }

    logger.debug(format!("Initializing prossesing!"));

    let command_is_not_registry: bool = enhanced_buffer::buffer_up_mananger::check_if_parity_id_is_registred(down_command.parity_id.clone());
    let command_id: u32 = down_command.command_id.clone().unwrap();

    if !command_is_not_registry {
        logger.debug(format!("Command {}, alwready have a response!", down_command.parity_id.clone()));
        enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id);
        return;
    }

    // TODO >>> Use the command.command or create a require type field to redirect the command to another client

    // -> One idea is to create a obrigatory key in the command.command and instead of only function create a type kwarg field
    // > Type can be:
    // >    - same as origin
    // >    - redirect

    // > if it is redirect one extra kwarg is necessary that have the client_id to redirect
    // * This will create a need to have a local database in the host to store the clients
    // * and to store when is the last contact of some client, if it is some threshold value
    // * more it will remove the registred client, if it have a contact recent, this will redirect the message
    // * however if the message is becames too old before the client the message is redirected catches it
    // * The system have to remove this old message from the buffer too.

    let translated_command: Command = Command::from_down_command(down_command.clone());

    logger.debug(format!("Translated command: {:?}", translated_command));

    let function = match translated_command.command.get("function") {
        Some(Value::String(function)) => function,
        _ => {
            logger.warn(format!("The function name is not found or not a string."));
            return;
        },
    };

    let command_patterns = COMMAND_PATTERNS.lock().unwrap().clone();
    let patterns = command_patterns;

    if !patterns.contains_key(function) {
        // -> Remove command from schedule if it isn't on the patterns

        logger.warn(format!("Command isn't registred in the patterns"));

        enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id.clone());

        logger.warn(format!("command skipped and remvoed from schedule"));
        return;
    }

    logger.debug(format!("Command function: {} is a valid function!", function));
    logger.debug(format!("Calling the callback!"));
    logger.debug(format!("Acquired the GIL"));

    let response = handle_command(py, translated_command.clone());

    let result = handle_pyobject(py, response.unwrap());

    let mut client_id = down_command.client_id.clone();

    let response;

    match result {
        ResultType::Map(m) => {
            if m.contains_key("response_mode") {
                let response_mode = m.get("response_mode").unwrap();

                if *response_mode == ResultType::Str("to_origin".to_string()) {
                    response = Ok(serde_json::to_string(&m).unwrap());
                } else if *response_mode == ResultType::Str("redirect".to_string()) {
                    let string_map: HashMap<String, String> = m
                        .iter()
                        .filter_map(|(k, v)| {
                            match v {
                                ResultType::Str(s) => Some((k.clone(), s.clone())),
                                ResultType::Map(inner_map) => {
                                    // For demonstration, converting inner_map to a string representation
                                    // You can adjust this as needed
                                    let inner_str = format!("{:?}", inner_map);
                                    Some((k.clone(), inner_str))
                                },
                                // Add other cases for different ResultType variants if needed
                                _ => None,
                            }
                        })
                        .collect();

                    println!("Response: {:?}", string_map);
                    //* probraly the cause of redirect bug
                    // TODO >>> Verify if has the response field
                    response = handle_redirect(string_map, &mut client_id, down_command.clone());
                } else {
                    response = error_response!("Error! Response mode doesn't match any known mode. Please use one of: ('to_origin', 'redirect')!");
                }
            } else {
                response = error_response!("Error! Callback doesn't implement response mode!");
            }
        },
        ResultType::Str(s) => {
            response = Ok(s.clone());
        },
        ResultType::Int(i) => {
            response = Ok(i.to_string());
        },
        ResultType::Float(fl) => {
            response = Ok(fl.to_string());
        },
        ResultType::Bool(b) => {
            response = Ok(b.to_string());
        },
        ResultType::List(_) => {
            response = error_response!("Error! Received a list, but expected a map!");
        },
        ResultType::Empty => {
            response = Ok(serde_json::to_string(&"C210".to_string()).unwrap());
        },
        ResultType::Error(e) => {
            response = error_response!(format!("An error occurred while converting the Python callback response. The error was: {:?}", e));
        },
    }

    logger.debug(format!("Function returned: {:?}", response));
    logger.info(format!("Command: {:?}, processed!", down_command.parity_id.clone()));

    enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id.clone());

    let up_command = UpCommand::new(client_id, down_command.parity_id.clone(), down_command.priority.clone(), response.unwrap());

    enhanced_buffer::buffer_up_mananger::buffer_up_schedule(up_command);
}

fn clear_old_data() {
    enhanced_buffer::buffer_down_mananger::buffer_down_clear_old_commands();
    enhanced_buffer::buffer_up_mananger::buffer_up_clear_old_commands();
}

pub fn initialize_socket_host_transposer(py: Python<'_>) {
    let logger = acquire_logger!("Transposer");

    let num_of_workers = NUM_WORKERS.lock().unwrap();

    let mut schedule: Vec<DownCommand> = enhanced_buffer::buffer_down_mananger::buffer_down_list_schedule();

    schedule.sort_by(|a, b| b.priority.cmp(&a.priority)); // put the schedule in crescent order

    // logger.debug(format!("Schedule to process:\n{:?}\n", schedule));

    if !(schedule.len() > 0) {
        // logger.debug(format!("Nothing in the schedule, skipping >>>"));
        clear_old_data();
        thread::sleep(Duration::from_millis(500));
        return;
    }

    logger.info(format!("Data found in schedule!"));

    for dow_command in schedule {
        let logger = acquire_logger!("Transposer");

        logger.info(format!("get a pool worker in tranposer!"));

        let py;

        {
            let getting_py = unsafe { Python::assume_gil_acquired() };

            let gil_pool = unsafe { getting_py.clone().new_pool() };

            py = gil_pool.python();

            logger.debug(format!("Aquired python in a process task!"));

            process(py, dow_command);

            logger.debug(format!("Finalize a process task!"));
        }
    }

    thread::sleep(Duration::from_millis(100));

    let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();

    return;

    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();

    //     pool.execute(|| {
    //         handle_connection(stream);
    //     });
    // }
}
