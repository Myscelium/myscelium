use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use pyo3::exceptions;

use pyo3::exceptions::PyException;
use pyo3::types::{IntoPyDict, PyAny, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use pyo3::IntoPy;
use pyo3::Py;
use pyo3::ToPyObject;
use pyo3::{PyErr, PyObject, PyResult, Python};

use crate::common::enhanced_buffer::utilities::CommandType;
use crate::common::structs::results_structs::ResultType;

use std::collections::HashMap;
use std::result;
use std::sync::MutexGuard;

use crate::common::enhanced_buffer::utilities::Command;

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

pub fn json_value_to_py_object(py: Python, value: &Value) -> PyResult<PyObject> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(b) => Ok(b.into_py(py)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_py(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_py(py))
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid number type"))
            }
        },
        Value::String(s) => Ok(s.clone().into_py(py)),
        Value::Array(arr) => {
            let py_list = PyList::new(py, arr.iter().map(|v| json_value_to_py_object(py, v).unwrap()));
            Ok(py_list.into())
        },
        Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (k, v) in obj {
                py_dict.set_item(k, json_value_to_py_object(py, v)?.to_object(py))?;
            }
            Ok(py_dict.into())
        },
    }
}

pub fn dict_to_kwargs<'l>(py: Python<'l>, dict: &HashMap<String, Value>) -> PyResult<HashMap<String, PyObject>> {
    let mut kwargs: HashMap<String, PyObject> = HashMap::new();
    for (key, value) in dict.iter() {
        let py_value = json_value_to_py_object(py, value)?;
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
            let key_str = match key.extract::<String>() {
                Ok(k) => k,
                Err(e) => {
                    println!("Failed to extract key as string: {:?}", e);
                    continue; // Skip this key-value pair
                },
            };

            if let Ok(value_str) = value.extract::<String>() {
                rust_dict.insert(key_str, ResultType::Str(value_str));
            } else if let Ok(value_int) = value.extract::<i64>() {
                rust_dict.insert(key_str, ResultType::Int(value_int));
            } else if let Ok(value_list) = value.cast_as::<PyList>() {
                let rust_list: Vec<_> = value_list.iter().map(|item| extract_pyobject(py, item.to_object(py))).collect();
                rust_dict.insert(key_str, ResultType::List(rust_list));
            } else if let Ok(nested_dict) = value.cast_as::<PyDict>() {
                rust_dict.insert(key_str, extract_pyobject(py, nested_dict.into()));
            } else {
                println!("Unmatched type for key: {}", key_str);
                // You may decide how to handle other types
            }
        }

        ResultType::Map(rust_dict)
    } else if let Ok(tuple) = obj.cast_as::<PyTuple>(py) {
        let rust_list: Vec<_> = tuple.iter().map(|item| extract_pyobject(py, item.to_object(py))).collect();
        ResultType::List(rust_list)
    } else if let Ok(list) = obj.cast_as::<PyList>(py) {
        let rust_list: Vec<_> = list.iter().map(|item| extract_pyobject(py, item.to_object(py))).collect();
        ResultType::List(rust_list)
    } else if let Ok(int) = obj.cast_as::<PyInt>(py) {
        match int.extract() {
            Ok(i) => ResultType::Int(i),
            Err(e) => {
                println!("Failed to extract integer: {:?}", e);
                ResultType::Empty
            },
        }
    } else if let Ok(float) = obj.cast_as::<PyFloat>(py) {
        match float.extract() {
            Ok(f) => ResultType::Float(f),
            Err(e) => {
                println!("Failed to extract float: {:?}", e);
                ResultType::Empty
            },
        }
    } else if let Ok(string) = obj.cast_as::<PyString>(py) {
        match string.extract() {
            Ok(s) => ResultType::Str(s),
            Err(e) => {
                println!("Failed to extract string: {:?}", e);
                ResultType::Empty
            },
        }
    } else if let Ok(boolean) = obj.cast_as::<PyBool>(py) {
        match boolean.extract() {
            Ok(b) => ResultType::Bool(b),
            Err(e) => {
                println!("Failed to extract boolean: {:?}", e);
                ResultType::Empty
            },
        }
    } else if obj.is_none(py) {
        ResultType::Empty
    } else {
        println!("Unmatched type for object: {:?}", obj);
        ResultType::Empty
    }
}

pub fn call_callback(py: Python<'_>, command: Command, callback_patterns: MutexGuard<'_, HashMap<String, (Py<PyFunction>, Value)>>) -> PyResult<PyObject> {
    println!("Command to call a callback: {:?}", command);

    let function_name: &String = match command.command.get("function") {
        //> To Handle both functions and response activation function and use a single code to do so
        Some(Value::String(function_name)) => function_name,
        _ => match command.command.get("response_activation_function") {
            Some(Value::String(function_name)) => function_name,
            _ => return Err(PyErr::new::<PyException, _>("The function name is not found or not a string.")),
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

            match command.get("kwargs") {
                Some(Value::String(inner_stringfied_map)) => {
                    //* This was added because when a kwarg is a String({}) it need firstly to be parsed*/
                    let result = match serde_json::from_str::<HashMap<_, _>>(inner_stringfied_map) {
                        //> Then when we obtained the kwargs Value we parse it using the default mehod
                        Ok(r) => {
                            let inner_hash_map: HashMap<_, _> = r.clone().into_iter().collect();
                            dict_to_kwargs(py, &inner_hash_map).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?
                        },
                        Err(_) => dict_to_kwargs(py, &HashMap::new()).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?,
                    };
                    result
                },
                Some(Value::Object(inner_map)) => {
                    let inner_hash_map: HashMap<_, _> = inner_map.clone().into_iter().collect();
                    dict_to_kwargs(py, &inner_hash_map).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?
                },
                _ => HashMap::new(),
            }
        },
        CommandType::Function(_) => {
            let command = &command.command;

            match command.get("kwargs") {
                Some(Value::String(inner_stringfied_map)) => {
                    //* This was added because when a kwarg is a String({}) it need firstly to be parsed*/
                    let result = match serde_json::from_str::<HashMap<_, _>>(inner_stringfied_map) {
                        //> Then when we obtained the kwargs Value we parse it using the default mehod
                        Ok(r) => {
                            let inner_hash_map: HashMap<_, _> = r.clone().into_iter().collect();
                            dict_to_kwargs(py, &inner_hash_map).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?
                        },
                        Err(_) => dict_to_kwargs(py, &HashMap::new()).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?,
                    };
                    result
                },
                Some(Value::Object(inner_map)) => {
                    let inner_hash_map: HashMap<_, _> = inner_map.clone().into_iter().collect();
                    dict_to_kwargs(py, &inner_hash_map).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?
                },
                _ => HashMap::new(),
            }
        },
        CommandType::Redirect(_) => {
            let command = &command.command;

            match command.get("kwargs") {
                Some(Value::String(inner_stringfied_map)) => {
                    //* This was added because when a kwarg is a String({}) it need firstly to be parsed*/
                    let result = match serde_json::from_str::<HashMap<_, _>>(inner_stringfied_map) {
                        //> Then when we obtained the kwargs Value we parse it using the default mehod
                        Ok(r) => {
                            let inner_hash_map: HashMap<_, _> = r.clone().into_iter().collect();
                            dict_to_kwargs(py, &inner_hash_map).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?
                        },
                        Err(_) => dict_to_kwargs(py, &HashMap::new()).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?,
                    };
                    result
                },
                Some(Value::Object(inner_map)) => {
                    let inner_hash_map: HashMap<_, _> = inner_map.clone().into_iter().collect();
                    dict_to_kwargs(py, &inner_hash_map).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?
                },
                _ => HashMap::new(),
            }
        },
        CommandType::SpecialFunction(_) => {
            let command = &command.command;

            match command.get("kwargs") {
                // Some(Value::String(inner_map)) => {
                //     HashMap::new()
                // },
                _ => HashMap::new(),
            }
        },
        _ => dict_to_kwargs(py, &command.command).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?,
    };

    println!("Converted to Python kwargs_map: {:?}", kwargs_map);

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

pub fn client_call_callback(py: Python<'_>, command: Command, callback_patterns: MutexGuard<'_, HashMap<String, (Py<PyFunction>, Value)>>) -> PyResult<PyObject> {
    println!("Command to call a callback: {:?}", command);

    let function_name: &String = match command.command.get("function") {
        //> To Handle both functions and response activation function and use a single code to do so
        Some(Value::String(function_name)) => function_name,
        _ => match command.command.get("response_activation_function") {
            Some(Value::String(function_name)) => function_name,
            _ => return Err(PyErr::new::<PyException, _>("The function name is not found or not a string.")),
        },
    };

    // Get the function and args_types from the CALLBACK_PATTERNS
    let (function, _) = callback_patterns.get(function_name).unwrap();

    let kwargs_map = match command.command_type() {
        CommandType::Response(_) => {
            let command = &command.command;

            let mut inner_hash_map: HashMap<String, Value> = HashMap::new();

            inner_hash_map.insert("data".to_string(), Value::Object(command.clone().into_iter().collect()));

            let resultexpected = dict_to_kwargs(py, &inner_hash_map)
                .map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))
                .unwrap();

            println!("Result expected: {:?}", resultexpected);

            resultexpected
        },
        CommandType::Function(_) => {
            let command = &command.command;

            let mut inner_hash_map: HashMap<String, Value> = HashMap::new();

            inner_hash_map.insert("data".to_string(), Value::Object(command.clone().into_iter().collect()));

            let resultexpected = dict_to_kwargs(py, &inner_hash_map)
                .map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))
                .unwrap();

            println!("Result expected: {:?}", resultexpected);

            resultexpected
        },
        CommandType::Redirect(_) => {
            let command = &command.command;

            let mut inner_hash_map: HashMap<String, Value> = HashMap::new();

            inner_hash_map.insert("data".to_string(), Value::Object(command.clone().into_iter().collect()));

            let resultexpected = dict_to_kwargs(py, &inner_hash_map)
                .map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))
                .unwrap();

            println!("Result expected: {:?}", resultexpected);

            resultexpected
        },
        CommandType::SpecialFunction(_) => {
            let command = &command.command;

            match command.get("kwargs") {
                // Some(Value::String(inner_map)) => {
                //     HashMap::new()
                // },
                _ => HashMap::new(),
            }
        },
        _ => dict_to_kwargs(py, &command.command).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?,
    };

    println!("Converted to Python kwargs_map: {:?}", kwargs_map);

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
