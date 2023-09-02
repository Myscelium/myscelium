use crate::commom::enhanced_buffer;
use crate::commom::enhanced_buffer::buffer_down_mananger::DownCommand;
use crate::commom::enhanced_buffer::buffer_up_mananger::UpCommand;
use crate::commom::enhanced_buffer::utilities::{Command, CommandType};

use lazy_static::lazy_static;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use serde::{Deserialize, Serialize};

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

use std::fmt::{self, format};

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

// > thread Manangement:

// type Job = Box<dyn FnOnce() + Send + 'static>;
// type Message = Option<Job>;

// pub struct ThreadPool {
//     workers: Vec<Worker>,
//     sender: mpsc::Sender<Message>,
//     free_condvar: Arc<Condvar>,
// }

// struct Worker {
//     id: usize,
//     thread: Option<thread::JoinHandle<()>>,
//     busy: Arc<AtomicBool>,
// }

// impl ThreadPool {
//     pub fn new(size: usize) -> ThreadPool {
//         assert!(size > 0);

//         let (sender, receiver) = mpsc::channel();
//         let receiver = Arc::new(Mutex::new(receiver));
//         let free_condvar = Arc::new(Condvar::new());

//         let mut workers = Vec::with_capacity(size);

//         for id in 0..size {
//             workers.push(Worker::new(id, Arc::clone(&receiver), Arc::clone(&free_condvar)));
//         }

//         ThreadPool {
//             workers,
//             sender,
//             free_condvar,
//         }
//     }

//     pub fn execute(&self, f: Job) {
//         self.sender.send(Some(f)).unwrap();
//     }

//     pub fn wait_for_free_worker(&self, f: Job) {
//         let lock = Mutex::new(());
//         let mut guard = lock.lock().unwrap();
//         while self.free_workers().is_empty() {
//             guard = self.free_condvar.wait_timeout(guard, std::time::Duration::from_secs(1)).unwrap().0;
//         }
//         self.execute(f);
//     }

//     pub fn free_workers(&self) -> Vec<usize> {
//         self.workers
//             .iter()
//             .filter(|worker| !worker.busy.load(Ordering::SeqCst))
//             .map(|worker| worker.id)
//             .collect()
//     }

//     pub fn join(&mut self) {
//         // Send termination message to each worker.
//         for _ in &self.workers {
//             self.sender.send(None).unwrap();
//         }

//         // Wait for all workers to finish.
//         for worker in &mut self.workers {
//             if let Some(thread) = worker.thread.take() {
//                 thread.join().unwrap();
//             }
//         }
//     }
// }

// impl Worker {
//     fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>, free_condvar: Arc<Condvar>) -> Worker {
//         let busy = Arc::new(AtomicBool::new(false));
//         let busy_clone = Arc::clone(&busy);
//         let free_condvar_clone = Arc::clone(&free_condvar);

//         let thread = thread::spawn(move || loop {
//             let message = match receiver.lock().unwrap().recv() {
//                 Ok(message) => message,
//                 Err(_) => return,
//             };

//             let logger = acquire_logger!("Transposer - Workers Pool");

//             match message {
//                 Some(job) => {
//                     busy_clone.store(true, Ordering::SeqCst);
//                     logger.info(format!("Transposer Worker {} got a job; executing.", id));
//                     job();
//                     busy_clone.store(false, Ordering::SeqCst);
//                     free_condvar_clone.notify_one();
//                 },
//                 None => {
//                     logger.info(format!("Transposer Worker {} was told to terminate.", id));
//                     return;
//                 },
//             }
//         });

//         Worker {
//             id,
//             thread: Some(thread),
//             busy,
//         }
//     }
// }

// > Transposer:

// fn dict_to_tuple<'l>(py: Python<'l>, dict: &HashMap<String, Value>) -> PyResult<&'l PyTuple> {
//     let logger = acquire_logger!("Transposer - Dict To Tuple");

//     // Check if the dict contains the function name as a key
//     if !dict.contains_key("args") {
//         // If it does not, return an empty Vec since there are no arguments
//         let mut values: Vec<PyObject> = Vec::new();
//         return Ok(PyTuple::new(py, values));
//     }

//     let args_string = match dict.get("args") {
//         Some(Value::String(s)) => s,
//         _ => return Err(PyErr::new::<PyException, _>("The args key is not found or not a string.")),
//     };

//     let sub_dict: HashMap<String, Value> = serde_json::from_str(args_string).unwrap();

//     logger.debug(format!("Args extracted: {:?}", sub_dict));

//     let mut values: Vec<PyObject> = Vec::new();
//     for value in sub_dict.values() {
//         let py_value = match value {
//             Value::String(s) => s.into_py(py),
//             Value::Number(n) => {
//                 if let Some(i) = n.as_i64() {
//                     i.into_py(py)
//                 } else if let Some(f) = n.as_f64() {
//                     f.into_py(py)
//                 } else {
//                     return Err(PyErr::new::<PyException, _>("Unsupported number type."));
//                 }
//             },
//             Value::Bool(b) => b.into_py(py),
//             _ => return Err(PyErr::new::<PyException, _>("Unsupported value type.")),
//         };
//         values.push(py_value);
//     }

//     let py_tuple = PyTuple::new(py, &values);

//     logger.debug(format!("py_tuple: {}", py_tuple));

//     Ok(py_tuple)
// }

fn dict_to_kwargs(dict: &HashMap<String, Value>) -> PyResult<HashMap<String, PyObject>> {
    let logger = acquire_logger!("Transposer - Dict To Kwargs");

    // Check if the dict contains the function name as a key
    if !dict.contains_key("response") {
        // If it does not, return an empty HashMap since there are no arguments
        let kwargs: HashMap<String, PyObject> = HashMap::new();
        return Ok(kwargs);
    }

    let args_string = match dict.get("response") {
        Some(Value::Object(s)) => s,
        _ => return Err(PyErr::new::<PyException, _>("The args key is not found or not a object.")),
    };

    let sub_dict: HashMap<String, Value> = args_string.clone().into_iter().collect();

    logger.debug(format!("Args extracted: {:?}", sub_dict));

    let mut kwargs: HashMap<String, PyObject> = HashMap::new();

    let py;

    {
        let getting_py = unsafe { Python::assume_gil_acquired() };

        let gil_pool = unsafe { getting_py.clone().new_pool() };

        py = gil_pool.python();

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
    }

    logger.debug(format!("kwargs: {:?}", kwargs));

    Ok(kwargs)
}

fn handle_command(command: Command) -> PyResult<PyObject> {
    let logger = acquire_logger!("Transposer - Handle Command");

    logger.debug(format!("Getting function name..."));
    let function_name: &String = match command.command.get("response_activation_function") {
        Some(Value::String(function_name)) => function_name,
        _ => return Err(PyErr::new::<PyException, _>("The function name is not found or not a string.")),
    };
    logger.debug(format!("Got function name: {}", function_name));

    // Get the function and args_types from the CALLBACK_PATTERNS
    let callback_patterns = CALLBACK_PATTERNS.lock().unwrap();
    let (function, _) = callback_patterns.get(function_name).unwrap();

    let kwargs_map = dict_to_kwargs(&command.command).map_err(|e| {
        logger.exception(format!("Error converting arguments to kwargs: {:?}", e));
        PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs: {:?}", e))
    })?;

    let py;

    let result;

    {
        let getting_py = unsafe { Python::assume_gil_acquired() };

        let gil_pool = unsafe { getting_py.clone().new_pool() };

        py = gil_pool.python();

        let kwargs = PyDict::new(py);
        for (key, value) in kwargs_map {
            kwargs.set_item(key, value).unwrap();
        }

        // Call the Python function with the converted arguments
        result = function.call(py, (), Some(kwargs)).map_err(|e| {
            logger.exception(format!("Error calling function: {:?}", e));
            e
        })?;
    }

    let result_obj: PyObject = result.clone().into(); // Convert the result into a PyObject

    Ok(result_obj) // Return the PyObject
}

// Define a custom type that can be either Empty, Map, or Error
#[derive(PartialEq, Serialize, Deserialize)]
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
        for item in tuple {
            println!("Item: {}", item);
        }
    } else if let Ok(list) = obj.cast_as::<PyList>(py) {
        // Handle list
        for item in list {
            println!("Item: {}", item);
        }
    } else if let Ok(int) = obj.cast_as::<PyInt>(py) {
        // Handle int
        println!("Integer: {}", int);
    } else if let Ok(float) = obj.cast_as::<PyFloat>(py) {
        // Handle float
        println!("Float: {}", float);
    } else if let Ok(string) = obj.cast_as::<PyString>(py) {
        // Handle string
        println!("String: {}", string);
    } else if let Ok(boolean) = obj.cast_as::<PyBool>(py) {
        // Handle bool
        println!("Boolean: {}", boolean);
    } else if obj.is_none(py) {
        // Handle None
        println!("None");
    } else {
        return ResultType::Empty;
    }

    ResultType::Empty
}

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
            activation_key = match translated_command.command.get("response_activation_function") {
                Some(Value::String(activation_key)) => activation_key,
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

    let response = handle_command(translated_command.clone());

    let result = handle_pyobject(py, response.unwrap());

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
