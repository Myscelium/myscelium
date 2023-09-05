use crate::commom::enhanced_buffer;
use crate::commom::enhanced_buffer::buffer_down_mananger::DownCommand;
use crate::commom::enhanced_buffer::buffer_up_mananger::UpCommand;
use crate::commom::enhanced_buffer::utilities::{Command, CommandType};
use crate::commom::functions::python_functions::{call_callback, dict_to_kwargs, extract_pyobject};
use crate::commom::structs::results_structs::ResultType;

use lazy_static::lazy_static;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use serde::{Deserialize, Serialize};
use std::fmt::{self, format};

// use crate::socket_client::socket_client::is_client_registred;

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

use rand::distributions::Alphanumeric;
use rand::Rng;

use super::client_logger::log_handler::Logger;
use crate::CLIENT_LOG_LEVEL;

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
    static ref CALLBACK_PATTERNS: Arc<Mutex<HashMap<String, (Py<PyFunction>, Value)>>> = {
        let command_patterns: HashMap<String, (Py<PyFunction>, Value)> = HashMap::new();
        Arc::new(Mutex::new(command_patterns))
    };
    static ref NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));
}

pub fn set_socket_client_transposer_workers_num(n_workers: u32) {
    let mut default_num_of_workers = NUM_WORKERS.lock().unwrap();

    *default_num_of_workers = n_workers;

    enhanced_buffer::buffer_down_mananger::set_workers_num(n_workers);
    enhanced_buffer::buffer_up_mananger::set_workers_num(n_workers);
}

pub fn set_socket_client_transposer_callbacks(commands_patterns: HashMap<String, Value>, callbacks_patterns: HashMap<String, (Py<PyFunction>, Value)>) {
    let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();
    *command_patterns = commands_patterns;

    let mut callback_patterns = CALLBACK_PATTERNS.lock().unwrap();
    *callback_patterns = callbacks_patterns;
}

// > Transposer:

enum ProcessError {
    CommandAlwreadyProcessed(String),
    MissingCommandFunction(String),
    CommandNotRegistred(String),
    InvalidCallbackResponse(String, String),
    Error(String),
    UnknownCommandType,
    MissingResponseKey(String),
}

fn process(py: Python, down_command: DownCommand) -> Result<(), ProcessError> {
    let logger = acquire_logger!("Transposer - Process");

    logger.info(format!("Initializing prossesing!"));

    let command_is_not_registry: bool = enhanced_buffer::buffer_up_mananger::check_if_parity_id_is_registred(down_command.parity_id.clone());
    let command_id: u32 = down_command.command_id.unwrap().clone();

    if !command_is_not_registry {
        enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id);
        return Err(ProcessError::CommandAlwreadyProcessed(down_command.parity_id.clone()));
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

    let activation_key;

    match translated_command.command_type() {
        CommandType::Function(f) => {
            if let Some(Value::Object(function_obj)) = translated_command.command.get("function") {
                activation_key = match function_obj.get("function") {
                    // Replace "desired_inner_key" with the key you want to access
                    Some(Value::String(activation_key)) => activation_key,
                    _ => {
                        return Err(ProcessError::MissingCommandFunction(format!("{:?}", translated_command.clone())));
                    },
                };
            } else {
                return Err(ProcessError::MissingCommandFunction(format!("{:?}", translated_command.clone())));
            }
        },
        CommandType::Response(r) => {
            activation_key = match translated_command.command.get("response") {
                Some(Value::Object(inner_map)) => match inner_map.get("response_activation_function") {
                    Some(Value::String(activation_key)) => activation_key,
                    _ => {
                        return Err(ProcessError::MissingResponseKey(format!("{:?}", translated_command.clone())));
                    },
                },
                _ => {
                    return Err(ProcessError::MissingResponseKey(format!("{:?}", translated_command.clone())));
                },
            };
        },
        CommandType::Error(e) => {
            return Err(ProcessError::Error(e));
        },
        CommandType::Redirect(_) => {
            return Err(ProcessError::UnknownCommandType);
        },
        CommandType::Unknown => {
            return Err(ProcessError::UnknownCommandType);
        },
    }

    if activation_key == &"update_avaliable_host_commands".to_string() {
        logger.info(format!("Receive Host Allowed Commands"));

        if let Some(Value::Object(response_obj)) = translated_command.command.get("response") {
            // Clone the object to get a HashMap<String, Value>
            let response_map: HashMap<String, Value> = response_obj.clone().into_iter().collect();

            // Lock the COMMAND_PATTERNS and insert the new map

            {
                let mut actual_patterns = HOST_ALLOWED_COMMANDS.lock().unwrap();
                *actual_patterns = response_map;
            }

            logger.info(format!("Succesfuly actualize the host avalaible commands!"));

            enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id.clone());

            return Ok(());
        } else {
            return Err(ProcessError::MissingResponseKey(format!("{:?}", translated_command.clone())));
        }
    }

    let patterns;

    {
        let command_patterns = COMMAND_PATTERNS.lock().unwrap().clone();
        patterns = command_patterns;
    }

    if !patterns.contains_key(activation_key) {
        // -> Remove command from schedule if it isn't on the patterns

        logger.warn(format!("Command isn't registred in the patterns"));

        enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id.clone());

        logger.info(format!("command skipped and remvoed from schedule"));
        return Err(ProcessError::CommandNotRegistred(activation_key.clone()));
    }

    logger.info(format!("Command function: {} is a valid function!", activation_key));
    logger.debug(format!("Calling the callback!\n"));
    logger.debug(format!("Acquired the GIL"));

    let response;

    {
        let callback_patterns = CALLBACK_PATTERNS.lock().unwrap();
        response = call_callback(py, translated_command.clone(), callback_patterns);
    }

    let result = match response {
        Ok(r) => extract_pyobject(py, r),
        Err(e) => {
            // Handle the error or log it
            eprintln!("Python error: {:?}", e);
            // You can return a default value or propagate the error further
            ResultType::Error(format!("{:?}", e))
        },
    };

    let client_id = down_command.client_id.clone();

    let response: String;

    match result {
        ResultType::Map(m) => {
            if m.contains_key("response_mode") {
                let response_mode = m.get("response_mode").unwrap();

                if *response_mode == ResultType::Str("to_host".to_string()) {
                    response = serde_json::to_string(&m).unwrap();
                } else {
                    enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id.clone());
                    return Err(ProcessError::InvalidCallbackResponse(
                        activation_key.clone(),
                        "Response mode doesn't match any known mode. Please use one of: ('to_host', 'retransmit')!".to_string(),
                    ));
                }
            } else {
                enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id.clone());
                return Err(ProcessError::InvalidCallbackResponse(activation_key.clone(), "Callback doesn't implement response mode!".to_string()));
            }
        },
        ResultType::Str(s) => {
            response = s.clone();
        },
        ResultType::Int(i) => {
            response = i.to_string();
        },
        ResultType::Float(fl) => {
            response = fl.to_string();
        },
        ResultType::Bool(b) => {
            response = b.to_string();
        },
        ResultType::List(_) => {
            // eprintln!("Error! Received a list, but expected a map!");
            enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id.clone());
            return Err(ProcessError::InvalidCallbackResponse(activation_key.clone(), "Received a list, but expected a map!".to_string()));
        },
        ResultType::Empty => {
            logger.info(format!("Response is None!"));
            enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id.clone());
            return Ok(());
        },
        ResultType::Error(e) => {
            // eprintln!();
            return Err(ProcessError::InvalidCallbackResponse(
                activation_key.clone(),
                format!("An error occurred while converting the Python callback response. The error was: {:?}", e),
            ));
        },
    }

    logger.debug(format!("Function returned: {:?}", response));
    logger.info(format!("Command: {:?}, processed!", down_command.parity_id.clone()));

    let up_command: UpCommand = UpCommand::new(client_id, down_command.parity_id.clone(), down_command.priority.clone(), response);

    enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id.clone());
    enhanced_buffer::buffer_up_mananger::buffer_up_schedule(up_command);

    return Ok(());
}

fn clear_old_data() {
    enhanced_buffer::buffer_down_mananger::buffer_down_clear_old_commands();
    enhanced_buffer::buffer_up_mananger::buffer_up_clear_old_commands();
}

pub fn initialize_socket_client_transposer() {
    let logger = acquire_logger!("Transposer");

    thread::sleep(Duration::from_millis(200));

    let mut schedule: Vec<DownCommand> = enhanced_buffer::buffer_down_mananger::buffer_down_list_schedule();

    schedule.sort_by(|a, b| b.priority.cmp(&a.priority)); // put the schedule in crescent order

    logger.debug(format!("\nSchedule to process:\n{:?}\n", schedule));

    if !CLIENT_IS_RUNING.load(Ordering::SeqCst) {
        logger.info(format!("runing is set to false, shutdown transposer!"));
        return;
    }

    if !(schedule.len() > 0) {
        logger.debug(format!("Nothing in the schedule, skipping >>>"));
        clear_old_data();
        thread::sleep(Duration::from_millis(500));
        return;
    }

    logger.info(format!("\nData found in schedule!"));

    for dow_command in schedule {
        let logger = acquire_logger!("Transposer");

        logger.info(format!("get a pool worker in tranposer!"));

        let py;

        {
            let getting_py = unsafe { Python::assume_gil_acquired() };

            let gil_pool = unsafe { getting_py.clone().new_pool() };

            py = gil_pool.python();

            logger.debug(format!("Aquired python in a process task!"));

            let result = process(py, dow_command).map_err(|e| {
                let error = match e {
                    ProcessError::CommandAlwreadyProcessed(m) => {
                        format!("Command: {:?} Alwready processed! So skipping", m)
                    },

                    ProcessError::CommandNotRegistred(m) => {
                        format!("Command function {:?} no registred in the callbacks! So skipping", m)
                    },

                    ProcessError::MissingResponseKey(m) => {
                        format!("Command: {:?}, missing command response key", m)
                    },

                    ProcessError::MissingCommandFunction(m) => {
                        format!("Command: {:?}, missing command function", m)
                    },

                    ProcessError::InvalidCallbackResponse(m, r) => {
                        format!("Calback function: {:?} invalid response: {:?}", m, r)
                    },

                    ProcessError::Error(e) => {
                        format!("An error occurred while processing command, the error was: {:?}", e)
                    },

                    ProcessError::UnknownCommandType => "Unknown Command type".to_string(),
                };

                error
            });

            match result {
                Ok(()) => {
                    logger.info(format!("Finalize a process task!"));
                },
                Err(e) => {
                    logger.warn(format!("\nWarning: {:?}\n", e));
                },
            }
        }
    }

    // let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();

    return;

    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();

    //     pool.execute(|| {
    //         handle_connection(stream);
    //     });
    // }
}
