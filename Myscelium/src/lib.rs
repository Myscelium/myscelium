

// use socket_client;

use std::collections::HashMap;

mod socket_host;
use socket_host::socket_host::{set_socket_host_callbacks, print_avaliable_commands, initialize_host};
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict, PyTuple, PyList};
use pyo3::wrap_pyfunction;

use serde_json::Value;


#[pyfunction]
fn registry_socket_host_callbacks (py: Python, commands: &PyList) -> PyResult<()> {
    for command in commands.iter() {
        let command_dict: &PyDict = command.downcast().unwrap();
        let function: &PyAny = command_dict.get_item("function").unwrap();
        let args_dict: &PyDict = command_dict.get_item("args").unwrap().downcast().unwrap();

        // Extract the Python function name
        let function_name: &str = function.getattr("__name__")?.extract()?;

        let mut command_patterns = HashMap::new();
        command_patterns.insert(function_name.to_string(), Value::String(args_dict.to_string()));

        set_socket_host_callbacks (command_patterns);

        // Convert the args dict to a Vec and then to a tuple
        let args_vec: Vec<&PyAny> = args_dict.values().extract::<Vec<&PyAny>>()?;
        let args_tuple: &PyTuple = PyTuple::new(py, args_vec);

        // Call the Python function with the args
        let _result = function.call1(args_tuple)?;
    }

    Ok(())
}


#[pyfunction]
fn initialize_socket_host (ip:String, port:i32) {
    initialize_host();
}


#[pyfunction]
fn show_avaliable_commands () {
    print_avaliable_commands();
}


#[pymodule]
fn Myscelium (py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(registry_socket_host_callbacks, m)?)?;
    m.add_function(wrap_pyfunction!(initialize_socket_host, m)?)?;
    m.add_function(wrap_pyfunction!(show_avaliable_commands, m)?)?;
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