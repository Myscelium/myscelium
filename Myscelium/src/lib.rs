// use socket_client;

use std::collections::HashMap;

mod socket_host;
use socket_host::socket_host::{get_available_commands_registered, initialize_host, set_socket_host_callbacks};
use socket_host::socket_host::{initialize_host_buffer, register_client, set_max_conns};
use socket_host::transposer::{initialize_socket_host_transposer, set_socket_host_transposer_callbacks, set_socket_host_transposer_workers_num};

use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use pyo3::wrap_pyfunction;

use pyo3::exceptions;

use serde_json::{json, Value};

use ctrlc::set_handler;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::Value as JsonValue;
use std::thread;

use std::time::{Duration, Instant};

use lazy_static::lazy_static;

lazy_static! {
    pub static ref HOST_IS_RUNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    pub static ref CLIENT_IS_RUNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    pub static ref CLIENT_ID: Arc<Mutex<String>> = Arc::new(Mutex::new("".to_string()));
}

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

#[pyfunction]
fn set_socket_host_transposer_num_of_workers(n_workers: &PyInt) {
    let workers_num: u32 = n_workers.extract().unwrap();

    set_socket_host_transposer_workers_num(workers_num);

    return;
}

#[pyfunction]
fn set_socket_host_max_connections(n_max_conns: &PyInt) {
    let max_conns: u32 = n_max_conns.extract().unwrap();

    set_max_conns(max_conns);

    return;
}

fn extract_arg_types(arg: &PyAny) -> PyResult<Value> {
    if let Ok(arg_dict) = arg.downcast::<PyDict>() {
        // If the argument is a dictionary, recursively extract the argument types
        let mut args_types = HashMap::new();
        for (arg_name, arg_type) in arg_dict.iter() {
            let arg_name: String = arg_name.extract()?;
            let arg_type_value = extract_arg_types(arg_type)?;
            args_types.insert(arg_name, arg_type_value);
        }
        Ok(json!(args_types))
    } else {
        // If the argument is not a dictionary, extract it as a string
        let arg_type: String = arg.extract()?;
        Ok(json!(arg_type))
    }
}

#[pyfunction]
fn initalize_host_buffer_tables(path: &PyString) {
    let buffer_path: String = path.extract().unwrap();

    initialize_host_buffer(buffer_path);

    return;
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

fn stop_socket_host() {
    HOST_IS_RUNING.store(false, Ordering::SeqCst);
}

#[pyfunction]
fn initialize_socket_host(py: Python<'_>, ip: String, port: i32, client_id: String) {
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
        println!("Socket transposer exited ssucefully!");

        if !HOST_IS_RUNING.load(Ordering::SeqCst) {
            println!("Stop the core!");
            thread::sleep(Duration::from_secs(7));
            break;
        }
    }
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

#[pyfunction]
fn set_socket_host_allowed_clients(allowed_clients_list: &PyList) -> PyResult<()> {
    for client_allowed in allowed_clients_list.iter() {
        let allowed_clients_dict: &PyDict = client_allowed.downcast().unwrap();

        let client_type: &PyAny = allowed_clients_dict.get_item("client_type").unwrap();
        let client_id: &PyAny = allowed_clients_dict.get_item("client_id").unwrap();

        if let Ok(extracted_client_type) = client_type.extract::<String>() {
            if let Ok(extracted_client_id) = client_id.extract::<String>() {
                register_client(extracted_client_id, extracted_client_type);
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Error: client_id must be a String with 16 characters!"));
            }
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Error: client_type must be a String!"));
        }
    }

    Ok(())
}

// > -----------------------------------------------------------------------------------------------------------------------------------------

// -> Socket Client mainpoints:

mod socket_client;
use socket_client::scheduler::schedule;
use socket_client::socket_client::{get_socket_client_available_commands_registered, set_socket_client_callbacks_patterns, Command};
use socket_client::socket_client::{initialize_client, initialize_client_buffer};
use socket_client::transposer::{
    initialize_socket_client_transposer, set_socket_client_transposer_callbacks, set_socket_client_transposer_workers_num,
};

#[pyfunction]
fn set_socket_client_transposer_num_of_workers(n_workers: &PyInt) {
    let workers_num: u32 = n_workers.extract().unwrap();

    set_socket_client_transposer_workers_num(workers_num);

    return;
}

#[pyfunction]
fn initalize_client_buffer_tables(path: &PyString) {
    let buffer_path: String = path.extract().unwrap();

    initialize_client_buffer(buffer_path);

    return;
}

#[derive(Debug, Clone)]
enum ResultType {
    Empty,
    Map(HashMap<String, String>),
    Error(String),
}

fn handle_pyobject(py: Python, obj: PyObject) -> ResultType {
    if let Ok(dict) = obj.cast_as::<PyDict>(py) {
        // Handle dict

        let mut rust_dict = HashMap::new(); // Declare the HashMap

        for (key, value) in dict.iter() {
            let key_str: String = key.extract().unwrap();
            if let Ok(value_str) = value.extract::<String>() {
                rust_dict.insert(key_str, value_str);
            } else if let Ok(value_int) = value.extract::<i32>() {
                rust_dict.insert(key_str, value_int.to_string());
            } else if let Ok(value_list) = value.extract::<Vec<String>>() {
                rust_dict.insert(key_str, format!("{:?}", value_list));
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

#[pyfunction]
fn client_send(py: Python, command: PyObject, priority: &PyInt) -> PyResult<Py<PyAny>> {
    let client_id = CLIENT_ID.lock().unwrap();

    if !CLIENT_IS_RUNING.load(Ordering::SeqCst) {
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

    match converted_command {
        ResultType::Map(m) => {
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

    Ok("Ok".to_string().into_py(py))
}

#[pyfunction]
fn set_client_host_target() {}

#[pyfunction]
fn set_client_workers_num() {}

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
    let mut client_id_global = CLIENT_ID.lock().unwrap();

    *client_id_global = client_id.clone();

    let address = format!("{}:{}", ip, port);

    thread::spawn(|| {
        ctrlc::set_handler(move || {
            if HOST_IS_RUNING.load(Ordering::SeqCst) {
                CLIENT_IS_RUNING.store(false, Ordering::SeqCst);
                println!("\nreceived Ctrl+C!\n");
                stop_socket_host();
            }
        })
        .expect("Error setting Ctrl-C handler");

        initialize_client(address, client_id);
        println!("Socket host exited ssucefully!");
    });

    loop {
        initialize_socket_client_transposer(py);

        if !CLIENT_IS_RUNING.load(Ordering::SeqCst) {
            println!("Stop the core!");
            break;
        }
    }

    println!("Socket transposer exited ssucefully!");
}

// TODO >>> Add a protocol id in the host to check if the client is outdated compared to the host

// > -----------------------------------------------------------------------------------------------------------------------------------------

// -> Entries:

#[pymodule]
fn Myscelium(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // -> Host
    m.add_function(wrap_pyfunction!(initalize_host_buffer_tables, m)?)?;
    m.add_function(wrap_pyfunction!(registry_socket_host_callbacks, m)?)?;
    m.add_function(wrap_pyfunction!(initialize_socket_host, m)?)?;
    m.add_function(wrap_pyfunction!(get_socket_host_available_commands, m)?)?;
    m.add_function(wrap_pyfunction!(set_socket_host_max_connections, m)?)?;
    m.add_function(wrap_pyfunction!(set_socket_host_transposer_num_of_workers, m)?)?;
    m.add_function(wrap_pyfunction!(set_socket_host_allowed_clients, m)?)?;

    // -> Client
    m.add_function(wrap_pyfunction!(initalize_client_buffer_tables, m)?)?;
    m.add_function(wrap_pyfunction!(registry_socket_client_callbacks, m)?)?;
    m.add_function(wrap_pyfunction!(initialize_socket_client, m)?)?;
    m.add_function(wrap_pyfunction!(set_socket_client_transposer_num_of_workers, m)?)?;
    m.add_function(wrap_pyfunction!(client_send, m)?)?;

    Ok(())
}

// To call by the python side:

/*

import rust_module  # This is your Rust module compiled as a Python extension

def python_function(name, age, birth):
    # Your function logic here
    pass

rust_module.call_python_function({
    "function": python_function,
    "args": {
        "name": "John",
        "age": 30,
        "birth": "1990-01-01"
    }
})

 */
