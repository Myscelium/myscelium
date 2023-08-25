// use socket_client;

use std::collections::HashMap;

use crate::socket_host::socket_host::{get_available_commands_registered, initialize_host, set_socket_host_callbacks};
use crate::socket_host::socket_host::{initialize_host_buffer, set_heartbeat_callback, set_max_conns};
use crate::socket_host::transposer::{initialize_socket_host_transposer, set_socket_host_transposer_callbacks, set_socket_host_transposer_workers_num};

use crate::socket_client::client_logger::log_handler::{initialize_client_logs_databse_dir, set_client_log_level};
use crate::socket_host::host_logger::log_handler::{initialize_host_logs_databse_dir, set_host_log_level};

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};

use serde_json::{json, Value};

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;

use serde_json::Value as JsonValue;
use std::thread;

use std::time::{Duration, Instant};

use crate::commom::functions::python_functions::extract_arg_types;

use crate::HOST_IS_RUNING;
use crate::HOST_LOG_LEVEL;
use crate::HOST_NODE_NAME;

// #[pyfunction]
// fn registry_socket_host_callbacks (py: Python, commands: &PyList) -> PyResult<()> {

//     for command in commands.iter() {
//         let command_dict: &PyDict = command.downcast().unwrap();
//         let function: &PyAny = command_dict.get_item("function").unwrap();
//         let args_dict: &PyDict = command_dict.get_item("args").unwrap().downcast().unwrap();

//         // Extract the Python function name
//         let function_name: &str = function.getattr("__name__")?.extract()?;

//         let mut command_patterns = HashMap::new();
//         command_patterns.insert(function_name.to_string(), Value::String(args_dict.to_string()));

//         set_socket_host_callbacks (command_patterns);

//         // Convert the args dict to a Vec and then to a tuple
//         let args_vec: Vec<&PyAny> = args_dict.values().extract::<Vec<&PyAny>>()?;
//         let args_tuple: &PyTuple = PyTuple::new(py, args_vec);

//         // Call the Python function with the args
//         let _result = function.call1(args_tuple)?;
//     }

//     Ok(())
// }

macro_rules! process_commands {
    ($py:expr, $commands:expr, $callback_pattern:expr) => {
        for command in $commands.iter() {
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

            let function = function.downcast::<PyFunction>()?.clone();

            let function: Py<PyFunction> = function.into_py($py); // convert &PyAny to Py<PyFunction>
            $callback_pattern.insert(function_name.to_string(), (function, args_types_value));
        }
    };
}

#[pyfunction]
fn set_socket_host_transposer_num_of_workers(n_workers: &PyInt) {
    let workers_num: u32 = n_workers.extract().unwrap();

    set_socket_host_transposer_workers_num(workers_num);

    return;
}

#[pyfunction]
fn set_socket_host_max_connections(n_max_conns: &PyInt) {
    let max_conns: u32 = n_max_conns.extract().unwrap();

    set_host_clients_mananger__pool_workers_num(max_conns.clone());
    set_max_conns(max_conns);

    return;
}

#[pyfunction]
fn initalize_host_buffer_tables(path: &PyString) {
    let buffer_path: String = path.extract().unwrap();

    initialize_host_logs_databse_dir(buffer_path.clone());
    initialize_host_buffer(buffer_path.clone());
    clients_mananger_initialize_table(buffer_path.clone());

    return;
}

#[pyfunction]
fn set_socket_host_log_level(log_level: &PyString) {
    let log_level: String = log_level.extract().unwrap();

    set_host_log_level(log_level);

    return;
}

// #[pyfunction]
// fn registry_host_logs_handler(py: Python, commands: &PyList) -> PyResult<()> {
//     let mut callback_pattern = HashMap::new();

//     process_commands!(py, commands, callback_pattern);

//     set_host_logs_handler_callback(callback_pattern);

//     println!("set the log callback");

//     Ok(())
// }

#[pyfunction]
fn registry_socket_host_client_heartbeat_contact_callback(py: Python, commands: &PyList) -> PyResult<()> {
    let mut callback_pattern = HashMap::new();

    process_commands!(py, commands, callback_pattern);

    set_heartbeat_callback(callback_pattern);

    Ok(())
}

fn stop_socket_host() {
    HOST_IS_RUNING.store(false, Ordering::SeqCst);
}

#[pyfunction]
fn registry_socket_host_callbacks(py: Python, commands: &PyList) -> PyResult<()> {
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
    set_socket_host_callbacks(command_patterns.clone());
    set_socket_host_transposer_callbacks(command_patterns.clone(), callbacks_patterns);

    Ok(())
}

#[pyfunction]
fn initialize_socket_host(py: Python<'_>, ip: String, port: i32, client_id: String) {
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

    let address = format!("{}:{}", ip, port);

    thread::spawn(|| {
        ctrlc::set_handler(move || {
            if HOST_IS_RUNING.load(Ordering::SeqCst) {
                println!("\nreceived Ctrl+C!\n");
                stop_socket_host();
            }
        })
        .expect("Error setting Ctrl-C handler");

        initialize_host(address, client_id);
        println!("Socket host exited ssucefully!");
    });

    loop {
        initialize_socket_host_transposer(py);

        if !HOST_IS_RUNING.load(Ordering::SeqCst) {
            println!("Stop the core!");
            thread::sleep(Duration::from_secs(7));
            break;
        }
    }

    println!("Socket transposer exited ssucefully!");
}

fn translate_value_to_py(py: Python<'_>, value: JsonValue) -> PyResult<PyObject> {
    // Convert the JSON value to the appropriate Python object
    match value {
        JsonValue::Null => Ok(py.None()),
        JsonValue::Bool(b) => Ok(b.into_py(py)),
        JsonValue::Number(num) => Ok(num.as_f64().unwrap().into_py(py)),
        JsonValue::String(s) => Ok(s.into_py(py)),
        JsonValue::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                let py_item = translate_value_to_py(py, item)?;
                py_list.append(py_item)?;
            }
            Ok(py_list.into())
        },
        JsonValue::Object(obj) => {
            let py_dict: &PyDict = PyDict::new(py);
            for (k, v) in obj {
                let py_key = k.into_py(py);
                let py_value = translate_value_to_py(py, v)?;
                py_dict.set_item(py_key, py_value)?;
            }
            Ok(py_dict.into())
        },
    }
}

#[pyfunction]
fn get_socket_host_available_commands(py: Python<'_>) -> PyResult<PyObject> {
    let commands = get_available_commands_registered();

    // Convert the HashMap values to PyObjects
    let py_dict: &PyDict = PyDict::new(py);
    for (key, value) in commands {
        let py_value = translate_value_to_py(py, value)?;
        py_dict.set_item(key, py_value)?;
    }

    Ok(py_dict.into())
}

// > --------------------------------------------------------------------------------------------------------
// > Client Manangement

use crate::socket_host::client_mananger::mananger::{check_if_client_key_exists, clients_mananger_initialize_table, set_host_clients_mananger__pool_workers_num, Client, ClientError};
use crate::socket_host::permissions_mananger::mananger::{GroupError, PermissionGroup};

macro_rules! extract_string {
    ($value:expr, $err_msg:expr) => {
        $value.extract::<String>().map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>($err_msg))?
    };
}

macro_rules! extract_float {
    ($value:expr, $err_msg:expr) => {
        $value.extract::<f64>().map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>($err_msg))?
    };
}

macro_rules! extract_unsigned_int {
    ($value:expr, $err_msg:expr) => {
        $value.extract::<u32>().map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>($err_msg))?
    };
}

macro_rules! extract_string_vector {
    ($value:expr, $err_msg:expr) => {
        $value.extract::<Vec<String>>().map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>($err_msg))?
    };
}

macro_rules! extract_boolean {
    ($value:expr, $err_msg:expr) => {
        $value.extract::<bool>().map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>($err_msg))?
    };
}

#[pyfunction]
fn set_socket_host_allowed_clients(allowed_client_list: &PyList) -> PyResult<()> {
    for client_allowed in allowed_client_list.iter() {
        let allowed_clients_dict: &PyDict = client_allowed.downcast().unwrap();

        let client_name = extract_string!(allowed_clients_dict.get_item("client_name").unwrap(), "Error: client_name must be a String!");
        let client_key = extract_string!(allowed_clients_dict.get_item("client_key").unwrap(), "Error: client_key must be a String with 16 characters!");

        let client_type = extract_string!(allowed_clients_dict.get_item("client_type").unwrap(), "Error: client_type must be a String!");
        let client_permission_group = extract_string!(allowed_clients_dict.get_item("permission_group").unwrap(), "Error: permission_group must be a String!");
        let client_is_super_user = extract_boolean!(allowed_clients_dict.get_item("is_super_user").unwrap(), "Error: is_super_user must be a String!");

        let client_max_sub_channels = extract_unsigned_int!(allowed_clients_dict.get_item("max_sub_channels").unwrap(), "Error: max_sub_channels must be a String!");
        let client_owned_sub_channels_keys = extract_string_vector!(allowed_clients_dict.get_item("owned_sub_channels_keys").unwrap(), "Error: owned_sub_channels_keys must be a String!");

        // {
        //     println!(
        //         "{:?},{:?},{:?},{:?},{:?},{:?},{:?}",
        //         &client_name, &client_key, &client_type, &client_permission_group, &client_is_super_user, &client_max_sub_channels, &client_owned_sub_channels_keys,
        //     )
        // }

        if !check_if_client_key_exists(client_key.clone()) {
            let _ = Client::new(
                client_name.clone(),
                client_key.clone(),
                client_type,
                client_permission_group,
                client_is_super_user,
                client_max_sub_channels,
                client_owned_sub_channels_keys,
            );
        }

        println!("Successfully created client: {} of key: {}", client_name, client_key)
    }
    Ok(())
}

#[pyfunction]
fn remove_all_allowed_clients(allowed_client_list: &PyList) {
    let _ = Client::delete_all();
}

// fn set_socket_host_allowed_clients(allowed_clients_list: &PyList) -> PyResult<()> {
//     for client_allowed in allowed_clients_list.iter() {
//         let allowed_clients_dict: &PyDict = client_allowed.downcast().unwrap();

//         let client_type: &PyAny = allowed_clients_dict.get_item("client_type").unwrap();
//         let client_id: &PyAny = allowed_clients_dict.get_item("client_id").unwrap();

//         if let Ok(extracted_client_type) = client_type.extract::<String>() {
//             if let Ok(extracted_client_id) = client_id.extract::<String>() {
//                 register_client(extracted_client_id, extracted_client_type);
//             } else {
//                 return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Error: client_id must be a String with 16 characters!"));
//             }
//         } else {
//             return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Error: client_type must be a String!"));
//         }
//     }

//     Ok(())
// }
