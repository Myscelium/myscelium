use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use pyo3::exceptions;

use pyo3::exceptions::PyException;
use pyo3::types::{IntoPyDict, PyAny, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use pyo3::IntoPy;
use pyo3::Py;
use pyo3::ToPyObject;
use pyo3::{PyErr, PyObject, PyResult, Python};

use crate::commom::enhanced_buffer::utilities::CommandType;
use crate::commom::structs::results_structs::ResultType;

use std::collections::HashMap;
use std::sync::MutexGuard;

use crate::commom::enhanced_buffer::utilities::Command;

use serde_json::{json, Value};

pub fn extract_arg_types(arg: &PyAny) -> PyResult<Value> {
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

pub fn dict_to_kwargs<'l>(py: Python<'l>, dict: &HashMap<String, Value>) -> PyResult<HashMap<String, PyObject>> {
    println!("map to convert to python kwargs: {:?}", dict);

    let args_map: HashMap<String, Value> = match dict.get("kwargs") {
        Some(Value::Object(map)) => map.clone().into_iter().collect(), // Convert the inner serde_json::Map to a HashMap
        Some(Value::String(s)) => serde_json::from_str(s).unwrap(),    // Deserialize the JSON string into a HashMap
        _ => {
            println!("The kwargs key is not found or not a correct type, so assuming callback doesn't need any kargs");
            return Ok(HashMap::new());
        },
    };

    let mut kwargs: HashMap<String, PyObject> = HashMap::new();
    for (key, value) in args_map.iter() {
        let py_value = match value {
            Value::Object(inner_map) => {
                let map_as_hashmap: HashMap<String, Value> = inner_map.clone().into_iter().collect();
                let inner_kwargs = dict_to_kwargs(py, &map_as_hashmap)?;
                let py_dict = PyDict::new(py);
                for (k, v) in inner_kwargs.iter() {
                    py_dict.set_item(k, v)?;
                }
                py_dict.into()
            },
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

    Ok(kwargs)
}

pub fn dict_to_tuple<'l>(py: Python<'l>, dict: &HashMap<String, Value>) -> PyResult<&'l PyTuple> {
    // let logger = acquire_logger!("Transposer - Py Dict to Tuple Converter");

    // Check if the dict contains the function name as a key
    if !dict.contains_key("kwargs") {
        // If it does not, return an empty Vec since there are no arguments
        let mut values: Vec<PyObject> = Vec::new();
        return Ok(PyTuple::new(py, values));
    }

    let args_string = match dict.get("kwargs") {
        Some(Value::String(s)) => s,
        _ => return Err(PyErr::new::<PyException, _>("The kwargs key is not found or not a string.")),
    };

    let sub_dict: HashMap<String, Value> = serde_json::from_str(args_string).unwrap();

    // logger.debug(format!("Args extracted: {:?}", sub_dict));

    let mut values: Vec<PyObject> = Vec::new();
    for value in sub_dict.values() {
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
        values.push(py_value);
    }

    let py_tuple = PyTuple::new(py, &values);

    // logger.debug(format!("py_tuple: {}", py_tuple));

    Ok(py_tuple)
}

pub fn extract_pyobject(py: Python, obj: PyObject) -> ResultType {
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
                let inner_map = extract_pyobject(py, nested_dict.into());
                rust_dict.insert(key_str, inner_map);
            } else {
                // Handle other types as needed
            }
        }

        return ResultType::Map(rust_dict);
    } else if let Ok(tuple) = obj.cast_as::<PyTuple>(py) {
        // Handle tuple
        let rust_list: Vec<_> = tuple.iter().map(|item| extract_pyobject(py, item.into())).collect();
        return ResultType::List(rust_list);
    } else if let Ok(list) = obj.cast_as::<PyList>(py) {
        // Handle list
        let rust_list: Vec<_> = list.iter().map(|item| extract_pyobject(py, item.into())).collect();
        return ResultType::List(rust_list);
    } else if let Ok(int) = obj.cast_as::<PyInt>(py) {
        // Handle int
        return ResultType::Int(int.extract().unwrap());
    } else if let Ok(float) = obj.cast_as::<PyFloat>(py) {
        // Handle float
        return ResultType::Float(float.extract().unwrap());
    } else if let Ok(string) = obj.cast_as::<PyString>(py) {
        // Handle string
        return ResultType::Str(string.extract().unwrap());
    } else if let Ok(boolean) = obj.cast_as::<PyBool>(py) {
        // Handle bool
        return ResultType::Bool(boolean.extract().unwrap());
    } else if obj.is_none(py) {
        // Handle None
        return ResultType::Empty;
    }

    return ResultType::Empty;
}

pub fn call_callback(py: Python<'_>, command: Command, callback_patterns: MutexGuard<'_, HashMap<String, (Py<PyFunction>, Value)>>) -> PyResult<PyObject> {
    println!("Command to call a callback: {:?}", command);

    let function_name: &String = match command.command.get("function") {
        Some(Value::String(function_name)) => function_name,
        _ => match command.command.get("response") {
            Some(Value::Object(inner_map)) => match inner_map.get("response_activation_function") {
                Some(Value::String(function_name)) => function_name,
                _ => return Err(PyErr::new::<PyException, _>("The function name is not found or not a string.")),
            },
            _ => return Err(PyErr::new::<PyException, _>("The response key is not found or not an object.")),
        },
    };

    // Get the function and args_types from the CALLBACK_PATTERNS
    let (function, _) = callback_patterns.get(function_name).unwrap();

    // let kwargs_map: &String = match command.command {
    //     Some(Value::String(function_name)) => dict_to_kwargs(py, &command.command).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs: {:?}", e)))?;,
    //     _ => match command.command.get("response") {
    //         Some(Value::Object(inner_map)) => match inner_map.get("response_activation_function") {
    //             Some(Value::String(function_name)) => function_name,
    //             _ => return Err(PyErr::new::<PyException, _>("The function name is not found or not a string.")),
    //         },
    //         _ => return Err(PyErr::new::<PyException, _>("The response key is not found or not an object.")),
    //     },
    // };

    let kwargs_map = match command.command_type() {
        CommandType::Response(_) => {
            let command = &command.command;

            match command.get("response") {
                Some(Value::Object(inner_map)) => {
                    let inner_hash_map: HashMap<_, _> = inner_map.clone().into_iter().collect();
                    dict_to_kwargs(py, &inner_hash_map).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs: {:?}", e)))?
                },
                _ => HashMap::new(),
            }
        },
        _ => dict_to_kwargs(py, &command.command).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs: {:?}", e)))?,
    };

    println!("Converted kwargs_map: {:?}", kwargs_map);

    // -> Convert to py dict
    let kwargs = PyDict::new(py);
    for (key, value) in kwargs_map {
        kwargs.set_item(key, value).unwrap();
    }

    // Call the Python function with the converted arguments
    let result = function.call(py, (), Some(kwargs)).map_err(|e| e)?;

    let result_obj: PyObject = result.clone().into(); // Convert the result into a PyObject

    Ok(result_obj) // Return the PyObject
}
