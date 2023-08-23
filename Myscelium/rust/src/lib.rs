mod commom;

mod socket_client;
mod socket_host;

mod host_entry_point;
use host_entry_point::*;

mod client_entry_point;
use client_entry_point::*;

use pyo3::exceptions;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use pyo3::wrap_pyfunction;

use lazy_static::lazy_static;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use parking_lot::Mutex;

lazy_static! {

    // CLIENT
    pub static ref CLIENT_IS_RUNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    pub static ref CLIENT_ID: Arc<Mutex<String>> = Arc::new(Mutex::new("".to_string()));
    pub static ref CLIENT_NODE_NAME: Arc<Mutex<String>> = Arc::new(Mutex::new("".to_string()));
    pub static ref CLIENT_LOG_LEVEL: Arc<Mutex<String>> = Arc::new(Mutex::new("".to_string()));

    // HOST:
    pub static ref HOST_IS_RUNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    pub static ref HOST_NODE_NAME: Arc<Mutex<String>> = Arc::new(Mutex::new("".to_string()));
    pub static ref HOST_LOG_LEVEL: Arc<Mutex<String>> = Arc::new(Mutex::new("".to_string()));

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

// TODO >>> Add a protocol id in the host to check if the client is outdated compared to the host
// TODO >> Create a configs file that automatically be created by Host to configure, client key, or host ip, credentials, data dir, etc..

// -> Entries:

#[pymodule]
fn myscelium_engine(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // -> Host
    m.add_function(wrap_pyfunction!(initalize_host_buffer_tables, m)?)?;
    m.add_function(wrap_pyfunction!(registry_socket_host_callbacks, m)?)?;
    m.add_function(wrap_pyfunction!(initialize_socket_host, m)?)?;
    m.add_function(wrap_pyfunction!(get_socket_host_available_commands, m)?)?;
    m.add_function(wrap_pyfunction!(set_socket_host_max_connections, m)?)?;
    m.add_function(wrap_pyfunction!(set_socket_host_transposer_num_of_workers, m)?)?;
    m.add_function(wrap_pyfunction!(set_socket_host_allowed_clients, m)?)?;
    m.add_function(wrap_pyfunction!(registry_socket_host_client_heartbeat_contact_callback, m)?)?;
    // m.add_function(wrap_pyfunction!(registry_host_logs_handler, m)?)?;
    m.add_function(wrap_pyfunction!(set_socket_host_log_level, m)?)?;

    // -> Client
    m.add_function(wrap_pyfunction!(initalize_client_buffer_tables, m)?)?;
    m.add_function(wrap_pyfunction!(registry_socket_client_callbacks, m)?)?;
    m.add_function(wrap_pyfunction!(initialize_socket_client, m)?)?;
    m.add_function(wrap_pyfunction!(set_socket_client_transposer_num_of_workers, m)?)?;
    m.add_function(wrap_pyfunction!(client_send, m)?)?;
    m.add_function(wrap_pyfunction!(set_client_uid, m)?)?;
    m.add_function(wrap_pyfunction!(set_socket_client_log_level, m)?)?;
    // m.add_function(wrap_pyfunction!(registry_client_logs_handler, m)?)?;

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
