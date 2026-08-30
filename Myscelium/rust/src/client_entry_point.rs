// SPDX-License-Identifier: MPL-2.0
// Copyright © 2021-2026 Cristian Camargo Filho

// use socket_client;

use std::any::Any;
use std::collections::HashMap;

use OxidizedMyscelium::{ClientState, Command, StateManagerError};
use OxidizedMyscelium::{CommandType, WatcherError};
use OxidizedMyscelium::{HandlerStatus, NetworkMap, Node, NodeHandler, NodeStatus, NodeVersion, VersionIndentifier};

use crate::common::functions::extract_arg_types;
use crate::common::functions::translate_value_to_py;
use crate::common::functions::wrap_py_function;
use crate::common::functions::{convert_to_pydict, dict_to_object};
use indexmap::IndexMap;
use parking_lot::Mutex;
use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use serde_json::Value;
use std::sync::atomic::Ordering;
use std::thread;
use std::time::Duration;

use OxidizedMyscelium::Callback;

use OxidizedMyscelium::{CLIENT_IS_RUNNING, CLIENT_NODE_CONFIGS, CLIENT_NODE_KEY, CLIENT_NODE_NAME, CLIENT_STATE_MANAGER};

// -> Socket Client main-points:

use OxidizedMyscelium;

/// Sets the number of worker threads for the socket client transposer.
///
/// # Parameters
///
/// - `n_workers`: The desired number of worker threads.
///
/// # Behavior
///
/// This function updates the number of worker threads the client transposer will use.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn set_socket_client_transposer_num_of_workers(n_workers: &PyInt) {
    let workers_num: u32 = n_workers.extract().unwrap();
    OxidizedMyscelium::set_socket_client_transposer_num_of_workers(workers_num);
    return;
}

/// Stops the socket client.
///
/// # Behavior
///
/// Sets the global `CLIENT_IS_RUNNING` atomic flag to `false`.
///
fn stop_socket_client() {
    OxidizedMyscelium::CLIENT_IS_RUNNING.store(false, Ordering::SeqCst);
}

#[derive(Debug, Clone)]
enum ResultType {
    Empty,
    Map(HashMap<String, String>),
    Error(String),
}

/// A utility function that recursively converts a Python dictionary into a Rust `HashMap`.
///
/// # Parameters
///
/// - `py`: Python interpreter instance.
/// - `dict`: The Python dictionary to be converted.
///
/// # Returns
///
/// Returns a `HashMap<String, String>` representation of the provided Python dictionary.
///
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

/// Handles a generic Python object and extracts its data.
///
/// # Parameters
///
/// - `py`: Python interpreter instance.
/// - `obj`: The Python object to be handled.
///
/// # Returns
///
/// Returns a `ResultType` indicating the outcome of the handling and any extracted data.
///
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
pub fn is_target_ready(py: Python, node_key: String) -> PyResult<Py<PyBool>> {
    let client_status = match ClientState::load_from_storage() {
        Ok(c) => c,
        Err(_) => {
            return Ok(PyBool::new(py, false).into());
        },
    };

    if let Some(net_map) = client_status.network_map {
        let mut net_map = net_map;
        {
            match net_map.target_is_reachable(&node_key) {
                Ok(reachable) => {
                    if !reachable {
                        return Ok(PyBool::new(py, false).into());
                    }
                },
                Err(_) => {
                    return Ok(PyBool::new(py, false).into());
                },
            };
        }
        {
            match net_map.target_is_ready(&node_key) {
                Ok(redy) => {
                    if !redy {
                        return Ok(PyBool::new(py, false).into());
                    }
                },
                Err(_) => {
                    return Ok(PyBool::new(py, false).into());
                },
            };
        }
    } else {
        return Ok(PyBool::new(py, false).into());
    }

    return Ok(PyBool::new(py, true).into());
}

#[pyfunction]
pub fn is_client_ready(py: Python) -> PyResult<Py<PyBool>> {
    return Ok(PyBool::new(py, OxidizedMyscelium::is_client_ready()).into());
}

#[pyfunction]
pub fn setup_client(client_name: String, client_uid: String, buffer_path: String, log_level: String, is_main_process: bool) {
    OxidizedMyscelium::setup_socket_client(client_name, client_uid, buffer_path, log_level, is_main_process)
}

/// Sends a c:ommand from the client.
///
/// # Parameters
///
/// - `py`: Python interpreter instance.
/// - `command`: The command to be sent.
/// - `priority`: Priority level of the command.
///
/// # Returns
///
/// Returns a PyResult indicating the outcome of the send operation.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn client_send(py: Python, command: PyObject, priority: &PyInt) -> PyResult<Py<PyAny>> {
    if !OxidizedMyscelium::is_client_ready() {
        println!("Error, client isn't running, pls run the client before try to send something!");
        return Err(PyErr::new::<exceptions::PyValueError, _>("Client isn't running! Please start client before try to send something."));
    }

    let extracted_priority = priority.extract::<u8>();
    let priority: u8;

    match extracted_priority {
        Ok(p) => priority = p,
        Err(e) => {
            return Err(PyErr::new::<exceptions::PyValueError, _>(format!("Failed to extract priority: {}", e)));
        },
    }

    let converted_command = handle_pyobject(py, command);
    println!("\nConverted Command to schedule: {:?}\n", converted_command);

    let parity_id_assigned: String;

    match converted_command {
        ResultType::Map(m) => {
            println!("Scheduling to send {:?}", m);
            parity_id_assigned = match OxidizedMyscelium::client_send_hashmap(m, priority) {
                Ok(o) => o,
                // TODO >>> Enhace This Error Handlings
                Err(e) => match e {
                    OxidizedMyscelium::ClientError::ClientIsNotRunning => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>(format!("Can't read client states, maybe not ready yet!")));
                    },
                    OxidizedMyscelium::ClientError::ClientNotFullyInitialized => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>(format!("Client isn't fully initialized yet, pls wait!")));
                    },
                    OxidizedMyscelium::ClientError::NotAbleToReadClientStates => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>(format!("Client isn't fully initialized yet, pls wait!")));
                    },
                    OxidizedMyscelium::ClientError::ClientDoesNotExist(c) => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>(format!("Client {} doesn't exists!", c)));
                    },
                    OxidizedMyscelium::ClientError::TargetDoesntExists => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>("Can't send a command nor a response to a target that doens't exist!"));
                    },
                    OxidizedMyscelium::ClientError::CantScheduleCommandsToItself => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>("Cant schedule a command to from this node to this node!"));
                    },
                    OxidizedMyscelium::ClientError::HandlerDoesntExist => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>("Handler does not exist in target!"));
                    },
                    OxidizedMyscelium::ClientError::HostCantSendResponseToItself => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>("Host cant send a response to itself!"));
                    },
                    OxidizedMyscelium::ClientError::TargetCantSendResponseToItself => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>("Target can't send a response to itself!"));
                    },
                    OxidizedMyscelium::ClientError::ResponseHandlerDoesntExist => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>("Response handler does not exist in target!"));
                    },
                    OxidizedMyscelium::ClientError::InvalidCommand(e) => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>(format!("Can't Schedule a invalid command, error case: {:?}!", e)));
                    },
                    // TODO >>> Add unreachable for cases that needs it
                    _ => {
                        return Err(PyErr::new::<exceptions::PyValueError, _>("Unexpected Error not covered!"));
                    },
                },
            };
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

    Ok(parity_id_assigned.into_py(py))
}

#[pyfunction]
pub fn wait_client_resp(py: Python, parity_id: String, timeout_in: u64) -> PyResult<Py<PyAny>> {
    let command: Command = match OxidizedMyscelium::client_wait_response(parity_id, timeout_in) {
        Ok(c) => c,
        Err(e) => match e {
            WatcherError::CommandNotFinded(pkey) => {
                return Err(PyErr::new::<exceptions::PyValueError, _>(format!("Command Response With ParityId: {} Not finded!", pkey)));
            },
            WatcherError::MaxTimeExceeded(pkey) => {
                return Err(PyErr::new::<exceptions::PyValueError, _>(format!("Time to get Response With ParityId: {} exceeded!", pkey)));
            },
        },
    };

    // This convertes the command into a hasmpa and converts to a python object in this case a dict
    dict_to_object(py, &command.command_to_hashmap().unwrap())
}

/// Sets the log level for the client.
///
/// # Parameters
///
/// - `log_level`: The desired log level as a string.
///
/// # Behavior
///
/// Updates the logging level of the client.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
// #[pyfunction]
// pub fn set_socket_client_log_level(log_level: &PyString) {
//     let log_level: String = log_level.extract().unwrap();
//     OxidizedMyscelium::set_socket_client_log_level(&log_level);
//     return;
// }

#[pyfunction]
pub fn get_socket_client_available_handlers(py: Python<'_>) -> PyResult<PyObject> {
    let commands = OxidizedMyscelium::get_socket_client_available_handlers();
    // Convert the HashMap values to PyObjects
    convert_to_pydict(py, &commands)
}

/// Registers Python callback functions for the socket client.
///
/// This function allows the registration of Python callback functions to handle specific commands from the server.
/// Each command has an associated function and expected arguments.
///
/// # Parameters
///
/// - `py`: Python interpreter instance.
/// - `commands`: A Python list of dictionaries. Each dictionary contains:
///   - `function`: The Python callback function to be executed.
///   - `args`: A dictionary describing the expected arguments for the function or the string "None" if no arguments are expected.
///
/// # Returns
///
/// Returns an empty result if successful, or a Python error if there's a problem with the provided commands.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn registry_socket_client_callbacks(py: Python, commands: &PyList) -> PyResult<()> {
    let mut callbacks: Vec<Callback> = Vec::new();

    let mut client_uid: String = "".to_string();
    {
        let node = CLIENT_NODE_CONFIGS.lock();
        if let Some(key) = &node.key {
            client_uid = key.clone()
        }
    }

    for command in commands.iter() {
        // Safely casting the command to a Python dictionary
        let command_dict: &PyDict = command.downcast().unwrap();

        // Extracting the "function" item from the command dictionary
        let function: &PyAny = command_dict.get_item("function").unwrap();

        // Extracting the "args" item from the command dictionary
        let args_item: &PyAny = command_dict.get_item("args").unwrap();

        // Initializing an optional variable to hold the arguments dictionary
        let args_dict: Option<&PyDict>;

        // Checking if the args item is a dictionary or a string with the value "None"
        if let Ok(args_as_dict) = args_item.downcast::<PyDict>() {
            args_dict = Some(args_as_dict);
        } else if let Ok(args_as_str) = args_item.extract::<String>() {
            if args_as_str == "None" {
                args_dict = None;
            } else {
                // Returning an error if the args item does not meet the expected conditions
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("args must be a dict or the string 'None'"));
            }
        } else {
            // Returning an error if the args item cannot be processed
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("args must be a dict or the string 'None'"));
        }

        // Extracting the Python function name for use in the handler
        let function_name: &str = function.getattr("__name__")?.extract()?;

        // Preparing a map to hold argument types, if any
        let mut args_types_value = IndexMap::new();

        // If args are provided as a dictionary, iterating over the dictionary to populate args_types_value
        if let Some(args_dict) = args_dict {
            for (key, value) in args_dict.into_iter() {
                let key: String = key.extract()?;
                let value: String = value.extract()?;
                args_types_value.insert(key, value);
            }
        }

        // Converting the Python function to a form that can be stored and called later
        let function: Py<PyFunction> = function.downcast::<PyFunction>()?.into_py(py);
        let wrapped_function = Box::new(wrap_py_function(function, client_uid.clone()));

        callbacks.push(Callback::new(
            function_name.to_string(),
            wrapped_function,
            args_types_value.clone(),
            CommandType::ExternalFunction,
            HandlerStatus::NotTested,
            HashMap::new(),
            "".to_string(),
        ));
    }

    // -> UPDATE CLIENT HANDLERS AND NODE

    // "callback_name" {
    //       "callback": Box<CallbackClosure>
    //       "parameters": IndexMap<String, String>
    //       "type": CallbackType
    //       "status": HandlerStatus,
    //       "response_structure": HashMap<String, Value>,
    //       "description": String
    // }

    // OxidizedMyscelium::set_client_callbacks(callbacks_patterns);
    OxidizedMyscelium::set_client_callbacks(callbacks);
    OxidizedMyscelium::change_client_to_initialized();

    Ok(())
}

#[pyfunction]
pub fn get_client_state(py: Python) -> PyResult<Py<PyBool>> {
    if CLIENT_IS_RUNNING.load(Ordering::SeqCst) {
        Ok(PyBool::new(py, true).into())
    } else {
        Ok(PyBool::new(py, false).into())
    }
}

// use RustPyNet::python_pool::pool::PythonTaskError;
// use RustPyNet::python_pool::pool::PythonTaskQueue;
// use RustPyNet::python_pool::pool::{start_processing_host_python_tasks, PythonTaskResult};

/// Initializes the socket client, sets up deadlock detection, and starts the main processing loop.
///
/// This function sets up the socket client to communicate with a server and starts the main loop
/// for processing commands and callbacks. It also spawns a thread to periodically check for deadlocks.
///
/// # Parameters
///
/// - `py`: Python interpreter instance.
/// - `ip`: IP address of the server.
/// - `port`: Port number of the server.
/// - `client_id`: A unique identifier for the client.
///
/// # Behavior
///
/// - Sets up a thread to periodically detect deadlocks.
/// - Initializes global client state.
/// - Spawns a thread to handle `Ctrl+C` and gracefully shut down the client.
/// - Initializes the socket client connection.
/// - Requests available commands from the host.
/// - Enters the main command processing loop.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn initialize_socket_client(py: Python<'_>, ip: String, port: i32) {
    OxidizedMyscelium::initialize_socket_client(ip, port);
}

// / Sets the unique identifier (UID) for the client.
// /
// / This function updates the global client UID which can be used to identify this client instance
// / in communications with the server.
// /
// / # Parameters
// /
// / - `py`: Python interpreter instance.
// / - `client_uid`: The new unique identifier for the client.
// /
// / # Python Binding
// /
// / This function is exposed to Python and can be called from a Python script.
// #[pyfunction]
// pub fn set_client_uid(py: Python<'_>, client_uid: String) {

// }
