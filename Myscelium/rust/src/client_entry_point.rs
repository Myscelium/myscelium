// use socket_client;

use std::collections::HashMap;

use crate::socket_host::socket_host::{get_available_commands_registered, initialize_host, set_socket_host_callbacks};
use crate::socket_host::socket_host::{initialize_host_buffer, register_client, set_heartbeat_callback, set_max_conns};
use crate::socket_host::transposer::{initialize_socket_host_transposer, set_socket_host_transposer_callbacks, set_socket_host_transposer_workers_num};

use crate::socket_client::client_logger::log_handler::{initialize_client_logs_databse_dir, set_client_log_level};
use crate::socket_host::client_mananger::mananger::{Client, ClientError};
use crate::socket_host::host_logger::log_handler::{initialize_host_logs_databse_dir, set_host_log_level};
use crate::socket_host::permissions_mananger::mananger::{GroupError, PermissionGroup};

use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use pyo3::wrap_pyfunction;

use pyo3::exceptions;

use serde_json::{json, Value};

use ctrlc::set_handler;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;

use serde_json::Value as JsonValue;
use std::thread;

use std::time::{Duration, Instant};

use lazy_static::lazy_static;

use crate::CLIENT_ID;
use crate::CLIENT_IS_RUNING;
use crate::CLIENT_LOG_LEVEL;
use crate::CLIENT_NODE_NAME;

// -> Socket Client mainpoints:

use crate::socket_client::scheduler::{self, schedule};
use crate::socket_client::socket_client::{get_socket_client_available_commands_registered, set_socket_client_callbacks_patterns};
use crate::socket_client::socket_client::{initialize_client, initialize_client_buffer};
use crate::socket_client::transposer::{initialize_socket_client_transposer, set_socket_client_transposer_callbacks, set_socket_client_transposer_workers_num};

use crate::commom::functions::python_functions::extract_arg_types;

#[pyfunction]
fn set_socket_client_transposer_num_of_workers(n_workers: &PyInt) {
    let workers_num: u32 = n_workers.extract().unwrap();

    set_socket_client_transposer_workers_num(workers_num);

    return;
}

fn stop_socket_client() {
    CLIENT_IS_RUNING.store(false, Ordering::SeqCst);
}

#[pyfunction]
fn initalize_client_buffer_tables(path: &PyString) {
    let buffer_path: String = path.extract().unwrap();

    initialize_client_logs_databse_dir(buffer_path.clone());
    initialize_client_buffer(buffer_path.clone());

    return;
}

#[derive(Debug, Clone)]
enum ResultType {
    Empty,
    Map(HashMap<String, String>),
    Error(String),
}

fn handle_dict(py: Python, dict: &PyDict) -> HashMap<String, String> {
    let mut rust_dict = HashMap::new();

    for (key, value) in dict.iter() {
        let key_str: String = key.extract().unwrap();
        if let Ok(value_str) = value.extract::<String>() {
            rust_dict.insert(key_str, value_str);
        } else if let Ok(value_int) = value.extract::<i32>() {
            rust_dict.insert(key_str, value_int.to_string());
        } else if let Ok(value_list) = value.extract::<Vec<String>>() {
            rust_dict.insert(key_str, format!("{:?}", value_list));
        } else if let Ok(nested_dict) = value.cast_as::<PyDict>() {
            rust_dict.insert(key_str, format!("{:?}", handle_dict(py, &nested_dict)));
        } else {
            // Handle other types as needed
        }
    }

    rust_dict
}

fn handle_pyobject(py: Python, obj: PyObject) -> ResultType {
    if let Ok(dict) = obj.cast_as::<PyDict>(py) {
        return ResultType::Map(handle_dict(py, &dict));
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

#[pyfunction]
fn client_send(py: Python, command: PyObject, priority: &PyInt) -> PyResult<Py<PyAny>> {
    let mut client_id;

    {
        client_id = CLIENT_ID.lock().clone();
    }

    if !CLIENT_IS_RUNING.load(Ordering::SeqCst) {
        println!("Error, client isn't runing, pls run the client before try to send something!");
        return Err(PyErr::new::<exceptions::PyValueError, _>("Client isn't running! Please start client before try to send something."));
    }

    let extracted_priority = priority.extract::<u8>();
    let mut priority: u8 = 0;

    match extracted_priority {
        Ok(p) => priority = p,
        Err(e) => {
            return Err(PyErr::new::<exceptions::PyValueError, _>(format!("Failed to extract priority: {}", e)));
        },
    }

    let converted_command = handle_pyobject(py, command);

    println!("\nConverted Command to schedule: {:?}\n", converted_command);

    match converted_command {
        ResultType::Map(m) => {
            println!("Scheduling to send {:?}", m);
            schedule(m, priority);
        },
        ResultType::Empty => {
            return Err(PyErr::new::<exceptions::PyValueError, _>("Command to send is empty!"));
        },
        ResultType::Error(e) => {
            return Err(PyErr::new::<exceptions::PyValueError, _>(format!(
                "An error occurred while trying to convert command to send in the myscelium engine, the error was: {}",
                e
            )));
        },
    }

    Ok("Sended!".to_string().into_py(py))
}

#[pyfunction]
fn set_client_host_target() {}

#[pyfunction]
fn set_client_workers_num() {}

#[pyfunction]
fn set_socket_client_log_level(log_level: &PyString) {
    let log_level: String = log_level.extract().unwrap();

    set_client_log_level(log_level);

    return;
}

// #[pyfunction]
// fn registry_client_logs_handler(py: Python, commands: &PyList) -> PyResult<()> {
//     let mut callback_pattern = HashMap::new();

//     process_commands!(py, commands, callback_pattern);

//     set_client_logs_handler_callback(callback_pattern);

//     println!("set the log callback");

//     Ok(())
// }

#[pyfunction]
fn registry_socket_client_callbacks(py: Python, commands: &PyList) -> PyResult<()> {
    let mut command_patterns = HashMap::new();

    let mut callbacks_patterns = HashMap::new();

    for command in commands.iter() {
        let command_dict: &PyDict = command.downcast().unwrap();
        let function: &PyAny = command_dict.get_item("function").unwrap();

        let args_item: &PyAny = command_dict.get_item("args").unwrap();

        // Check if args_item is a dict or a string with the value "None"
        let args_dict: Option<&PyDict>;

        if let Ok(args_as_dict) = args_item.downcast::<PyDict>() {
            args_dict = Some(args_as_dict);
        } else if let Ok(args_as_str) = args_item.extract::<String>() {
            if args_as_str == "None" {
                args_dict = None;
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("args must be a dict or the string 'None'"));
            }
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("args must be a dict or the string 'None'"));
        }

        // Extract the Python function name
        let function_name: &str = function.getattr("__name__")?.extract()?;

        // Extract the argument types
        let args_types_value;
        if let Some(args_dict) = args_dict {
            args_types_value = extract_arg_types(args_dict)?;
        } else {
            args_types_value = Value::Array(Vec::new()); // or whatever default value you want to use
        }

        // Store the function name and argument types in the command patterns
        command_patterns.insert(function_name.to_string(), args_types_value.clone());

        let function = function.downcast::<PyFunction>()?.clone();

        let function: Py<PyFunction> = function.into_py(py); // convert &PyAny to Py<PyFunction>
        callbacks_patterns.insert(function_name.to_string(), (function, args_types_value));
    }

    // Now you can use the command_patterns
    set_socket_client_callbacks_patterns(command_patterns.clone());
    set_socket_client_transposer_callbacks(command_patterns.clone(), callbacks_patterns);

    Ok(())
}

#[pyfunction]
fn initialize_socket_client(py: Python<'_>, ip: String, port: i32, client_id: String) {
    // Create a global Mutex for demonstration
    let mutex1 = Mutex::new(0);
    let mutex2 = Mutex::new(0);

    // Spawn a thread to periodically check for deadlocks
    thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_secs(5)); // Check every 5 seconds
            let deadlocks = parking_lot::deadlock::check_deadlock();
            if deadlocks.is_empty() {
                continue;
            }

            println!("{} deadlocks detected", deadlocks.len());
            for (i, threads) in deadlocks.iter().enumerate() {
                println!("Deadlock #{}", i);
                for t in threads {
                    println!("Thread Id {:?}", t.thread_id());
                    println!("{:?}", t.backtrace());
                }
            }
        }
    });

    CLIENT_IS_RUNING.store(true, Ordering::SeqCst);

    {
        let mut client_id_global = CLIENT_ID.lock();
        *client_id_global = client_id.clone();
    }

    let address = format!("{}:{}", ip, port);

    thread::spawn(|| {
        ctrlc::set_handler(move || {
            if CLIENT_IS_RUNING.load(Ordering::SeqCst) {
                CLIENT_IS_RUNING.store(false, Ordering::SeqCst);
                println!("\nreceived Ctrl+C!\n");
                stop_socket_client();
            }
        })
        .expect("Error setting Ctrl-C handler");

        initialize_client(address, client_id);
        println!("Socket host exited ssucefully!");
    });

    scheduler::request_host_avaliable_commands();

    loop {
        initialize_socket_client_transposer();

        if !CLIENT_IS_RUNING.load(Ordering::SeqCst) {
            println!("Stop the core!");
            break;
        }
    }

    println!("Socket transposer exited ssucefully!");
}

#[pyfunction]
fn set_client_uid(py: Python<'_>, client_uid: String) {
    scheduler::set_client_id(client_uid);
}
