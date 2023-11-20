// use socket_client;

use std::collections::HashMap;

use crate::socket_client::client_logger::log_handler::{initialize_client_logs_database_dir, set_client_log_level};

use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};

use pyo3::exceptions;

use serde_json::Value;

use std::sync::atomic::Ordering;

use parking_lot::Mutex;

use std::thread;

use std::time::Duration;

use crate::CLIENT_ID;
use crate::CLIENT_IS_RUNNING;

// -> Socket Client main-points:

use crate::socket_client::scheduler::{self, schedule};
use crate::socket_client::socket_client::{get_socket_client_available_commands_registered, set_socket_client_callbacks_patterns};
use crate::socket_client::socket_client::{initialize_client, initialize_client_buffer};
use crate::socket_client::transposer::{initialize_socket_client_transposer, set_socket_client_transposer_callbacks, set_socket_client_transposer_workers_num};

use crate::common::functions::python_functions::extract_arg_types;

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

    set_socket_client_transposer_workers_num(workers_num);

    return;
}

/// Stops the socket client.
///
/// # Behavior
///
/// Sets the global `CLIENT_IS_RUNNING` atomic flag to `false`.
///
fn stop_socket_client() {
    CLIENT_IS_RUNNING.store(false, Ordering::SeqCst);
}

/// Initializes the buffer tables for the client.
///
/// # Parameters
///
/// - `path`: The path to the location where buffer tables should be initialized.
///
/// # Behavior
///
/// Initializes the directories and tables required for client logs and buffer management.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn initialize_client_buffer_tables(path: &PyString) {
    let buffer_path: String = path.extract().unwrap();

    initialize_client_logs_database_dir(buffer_path.clone());
    initialize_client_buffer(buffer_path.clone());

    return;
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

/// Sends a command from the client.
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
    if !CLIENT_IS_RUNNING.load(Ordering::SeqCst) {
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

/// Sets the target host for the client.
///
/// # TODO
///
/// This function's implementation is not provided. It needs to be implemented.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
fn set_client_host_target() {}

/// Sets the number of worker threads for the client.
///
/// # TODO
///
/// This function's implementation is not provided. It needs to be implemented.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
fn set_client_workers_num() {}

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
#[pyfunction]
pub fn set_socket_client_log_level(log_level: &PyString) {
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
    // For this given data
    //
    // special_functions = [{
    //     "function": get_registered_commands,
    //     "response_type":"same_as_origin",
    //     "args": "None",
    // }, ]
    //

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

        // Here command_patterns is a list of:
        //
        // {
        //     "function1": {
        //         "arg1": "int",
        //         "arg2": "str"
        //     },
        //     "function2": "None"
        // }

        let function = function.downcast::<PyFunction>()?.clone();

        let function: Py<PyFunction> = function.into_py(py); // convert &PyAny to Py<PyFunction>
        callbacks_patterns.insert(function_name.to_string(), (function, args_types_value));

        // Callback patterns is a list of:
        //
        // {
        //     "function1": (PyFunctionObject1, {
        //         "arg1": "int",
        //         "arg2": "str"
        //     }),
        //     "function2": (PyFunctionObject2, "None")
        // }
    }

    // Now you can use the command_patterns
    set_socket_client_callbacks_patterns(command_patterns.clone());
    set_socket_client_transposer_callbacks(command_patterns.clone(), callbacks_patterns);

    Ok(())
}

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
pub fn initialize_socket_client(py: Python<'_>, ip: String, port: i32, client_id: String) {
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

    CLIENT_IS_RUNNING.store(true, Ordering::SeqCst);

    {
        let mut client_id_global = CLIENT_ID.lock();
        *client_id_global = client_id.clone();
    }

    let address = format!("{}:{}", ip, port);

    thread::spawn(|| {
        ctrlc::set_handler(move || {
            if CLIENT_IS_RUNNING.load(Ordering::SeqCst) {
                CLIENT_IS_RUNNING.store(false, Ordering::SeqCst);
                println!("\nreceived Ctrl+C!\n");
                stop_socket_client();
            }
        })
        .expect("Error setting Ctrl-C handler");

        initialize_client(address, client_id);
        println!("Socket host exited successfully!");
    });

    scheduler::request_host_available_commands();

    loop {
        initialize_socket_client_transposer();

        if !CLIENT_IS_RUNNING.load(Ordering::SeqCst) {
            println!("Stop the core!");
            break;
        }
    }

    println!("Socket transposer exited successfully!");
}

/// Sets the unique identifier (UID) for the client.
///
/// This function updates the global client UID which can be used to identify this client instance
/// in communications with the server.
///
/// # Parameters
///
/// - `py`: Python interpreter instance.
/// - `client_uid`: The new unique identifier for the client.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn set_client_uid(py: Python<'_>, client_uid: String) {
    scheduler::set_client_id(client_uid);
}
