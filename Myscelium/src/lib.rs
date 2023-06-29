

// use socket_client;

use std::collections::HashMap;

mod socket_host;
use socket_host::socket_host::{set_socket_host_callbacks, get_available_commands_registered, initialize_host};
use socket_host::socket_host::{initialize_host_buffer, set_workers_num, set_max_conns};


use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyString, PyInt, PyDict, PyTuple, PyList};
use pyo3::wrap_pyfunction;
use serde_json::{Value, json};

use serde_json::Value as JsonValue;


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
fn set_num_of_workers (n_workers:&PyInt) {

    let workers_num:u32 = n_workers.extract().unwrap();

    set_workers_num(workers_num);

    return;

}

#[pyfunction]
fn set_max_connections (n_max_conns:&PyInt) {

    let max_conns:u32 = n_max_conns.extract().unwrap();

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
fn initalize_buffer_tables (path:&PyString) {

    let buffer_path:String = path.extract().unwrap();

    initialize_host_buffer(buffer_path);

    return;

}


#[pyfunction]
fn registry_socket_host_callbacks(py: Python, commands: &PyList) -> PyResult<()> {
    let mut command_patterns = HashMap::new();

    for command in commands.iter() {
        let command_dict: &PyDict = command.downcast().unwrap();
        let function: &PyAny = command_dict.get_item("function").unwrap();
        let args_dict: &PyDict = command_dict.get_item("args").unwrap().downcast().unwrap();

        // Extract the Python function name
        let function_name: &str = function.getattr("__name__")?.extract()?;

        // Extract the argument types
        let args_types_value = extract_arg_types(args_dict)?;

        // Store the function name and argument types in the command patterns
        command_patterns.insert(function_name.to_string(), args_types_value);
    }

    // Now you can use the command_patterns
    set_socket_host_callbacks(command_patterns, );

    Ok(())
}

#[pyfunction]
fn initialize_socket_host (ip:String, port:i32) {
    let address = format!("{}:{}", ip, port);
    initialize_host(address);
}

fn dict_to_tuple (py: Python, dict: &HashMap<String, Value>) -> PyResult<Vec<PyObject>> {
    let mut tuple = Vec::new();

    for value in dict.values() {
        match value {
            Value::String(s) => tuple.push(PyString::new(py, s).to_object(py)),
            Value::Number(n) => tuple.push(n.as_f64().unwrap().into_py(py)),
            Value::Object(map) => {
                let sub_dict: HashMap<String, Value> = map.clone().into_iter().collect();
                let py_dict = PyDict::new(py);
                for (key, value) in sub_dict {
                    let py_key = PyString::new(py, &key);
                    let py_value = PyString::new(py, &value.to_string());
                    py_dict.set_item(py_key, py_value)?;
                }
                tuple.push(py_dict.to_object(py));
            },
            // Handle other Value variants here...
            _ => (),
        }
    }

    Ok(tuple)
}

fn translate_value_to_py(py: Python, value: JsonValue) -> PyResult<PyObject> {
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
        }
        JsonValue::Object(obj) => {
            let py_dict: &PyDict = PyDict::new(py);
            for (k, v) in obj {
                let py_key = k.into_py(py);
                let py_value = translate_value_to_py(py, v)?;
                py_dict.set_item(py_key, py_value)?;
            }
            Ok(py_dict.into())
        }
    }
}

#[pyfunction]
fn get_available_commands(py: Python) -> PyResult<PyObject> {
    let commands = get_available_commands_registered();

    // Convert the HashMap values to PyObjects
    let py_dict: &PyDict = PyDict::new(py);
    for (key, value) in commands {
        let py_value = translate_value_to_py(py, value)?;
        py_dict.set_item(key, py_value)?;
    }

    Ok(py_dict.into())
}

#[pymodule]
fn Myscelium (py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(initalize_buffer_tables, m)?)?;
    m.add_function(wrap_pyfunction!(registry_socket_host_callbacks, m)?)?;
    m.add_function(wrap_pyfunction!(initialize_socket_host, m)?)?;
    m.add_function(wrap_pyfunction!(get_available_commands, m)?)?;
    m.add_function(wrap_pyfunction!(set_max_connections, m)?)?;
    m.add_function(wrap_pyfunction!(set_num_of_workers, m)?)?;
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