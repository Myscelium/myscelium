// SPDX-License-Identifier: MPL-2.0
// Copyright © 2021-2026 Cristian Camargo Filho

use pyo3::exceptions;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyAny, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use pyo3::wrap_pyfunction;
use pyo3::IntoPy;
use pyo3::Py;
use pyo3::ToPyObject;
use pyo3::{PyErr, PyObject, PyResult, Python};
use serde_json::Value as JsonValue;
use serde_json::{json, Value};
use std::any::Any;
use std::collections::HashMap;
use std::result;
use std::sync::MutexGuard;
use OxidizedMyscelium::Command;
use OxidizedMyscelium::CommandError;
use OxidizedMyscelium::CommandInstructions;
use OxidizedMyscelium::CommandType;
use OxidizedMyscelium::ResultType;

use pyo3::prelude::*;

use indexmap::IndexMap;

pub fn convert_to_pydict(py: Python, data: &HashMap<String, IndexMap<String, String>>) -> PyResult<PyObject> {
    let py_dict = PyDict::new(py);
    for (key, value) in data {
        let inner_py_dict = convert_indexmap_to_pydict(py, value)?;
        py_dict.set_item(key, inner_py_dict)?;
    }
    Ok(py_dict.into())
}

fn convert_indexmap_to_pydict(py: Python<'_>, data: &IndexMap<String, String>) -> PyResult<Py<PyDict>> {
    let py_dict = PyDict::new(py);
    for (key, value) in data {
        py_dict.set_item(PyString::new(py, key), PyString::new(py, value))?;
    }
    Ok(py_dict.into())
}

// fn convert_boxed_any_to_pyany(py: Python, boxed_any: Box<Value>) -> PyResult<PyObject> {
//     if let Some(value) = boxed_any.downcast_ref::<bool>() {
//         Ok(value.into_py(py))
//     } else if let Some(value) = boxed_any.downcast_ref::<f64>() {
//         Ok(value.into_py(py))
//     } else if let Some(value) = boxed_any.downcast_ref::<String>() {
//         Ok(value.into_py(py))
//     } else if let Some(vec) = boxed_any.downcast_ref::<Vec<Box<dyn Any>>>() {
//         let py_list = PyList::empty(py);
//         for item in vec {
//             let py_item = convert_boxed_any_to_pyany(py, item)?;
//             py_list.append(py_item)?;
//         }
//         Ok(py_list.into_py(py))
//     } else if let Some(hash_map) = boxed_any.downcast_ref::<HashMap<String, Value>>() {
//         let py_dict = PyDict::new(py);
//         for (key, value) in hash_map {
//             let py_value = convert_json_value_to_pyobject(py, value)?;
//             py_dict.set_item(key, py_value)?;
//         }
//         Ok(py_dict.into_py(py))
//     } else if boxed_any.is::<()>() {
//         Ok(py.None())
//     } else {
//         Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!("Unsupported type: {:?}", boxed_any)))
//     }
// }

fn convert_json_value_to_pyobject(py: Python, value: &Value) -> PyResult<PyObject> {
    match value {
        Value::Object(map) => {
            let py_dict = PyDict::new(py);
            for (key, val) in map.iter() {
                let py_val = convert_json_value_to_pyobject(py, val)?;
                py_dict.set_item(key, py_val)?;
            }
            Ok(py_dict.into_py(py))
        },
        Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for val in arr {
                let py_val = convert_json_value_to_pyobject(py, val)?;
                py_list.append(py_val)?;
            }
            Ok(py_list.into_py(py))
        },
        Value::String(s) => Ok(s.into_py(py)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.into_py(py))
            } else if let Some(f) = n.as_f64() {
                Ok(f.into_py(py))
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyValueError, _>("Unsupported number type"))
            }
        },
        Value::Bool(b) => Ok(b.into_py(py)),
        Value::Null => Ok(py.None()),
    }
}

fn convert_boxed_anys_to_pyany(py: Python, boxed_anys: Vec<Box<dyn Any>>) -> PyResult<PyObject> {
    let py_list = PyList::empty(py);
    for boxed_any in boxed_anys {
        match boxed_any.downcast::<Value>() {
            Ok(value_box) => {
                println!("String value: {}", value_box.clone());
                let val = value_box.clone();
                let py_item = convert_json_value_to_pyobject(py, &val)?;
                py_list.append(py_item)?;
            },
            Err(_) => println!("Not a value"),
        }
    }
    Ok(py_list.into_py(py))
}

fn convert_to_tuple<'a>(py: Python<'a>, obj: &'a PyObject) -> PyResult<&'a PyTuple> {
    // If obj is a tuple, return it directly.
    if let Ok(tuple) = obj.extract::<&PyTuple>(py) {
        return Ok(tuple);
    }

    // If obj is a dict, convert its values to a tuple.
    if let Ok(dict) = obj.extract::<&PyDict>(py) {
        let values = dict.values().into_iter().map(|v| v.to_object(py)).collect::<Vec<_>>();
        let tuple = PyTuple::new(py, &values);
        return Ok(tuple);
    }

    // If obj is a list, convert it to a tuple by iterating over its elements.
    if let Ok(list) = obj.extract::<&PyList>(py) {
        let elements = list.into_iter().map(|item| item.to_object(py)).collect::<Vec<_>>();
        let tuple = PyTuple::new(py, &elements);
        return Ok(tuple);
    }

    // For any other type, wrap the obj in a tuple.
    let tuple = PyTuple::new(py, &[obj]);
    Ok(tuple)
}

/// Wraps a Python function into a Rust closure that can be executed with dynamic parameters.
pub fn wrap_py_function(py_func: Py<PyFunction>) -> Box<dyn Fn(Vec<Box<dyn Any + 'static>>) -> Box<dyn Any> + Send + Sync> {
    Box::new(move |args: Vec<Box<dyn Any + 'static>>| -> Box<dyn Any> {
        // Convert args to Python objects here. You might need to dynamically check types and convert them accordingly.
        // This is a placeholder showing the concept, actual implementation may vary based on your specific needs.

        println!("[MYSCELIUM][HOST][PYTHON BRIDGE] - Callback args: {:?}", args);

        let result: Result<Py<PyAny>, PyErr>;
        let value: Value;

        {
            let getting_py = unsafe { Python::assume_gil_acquired() };
            let gil_pool = unsafe { getting_py.clone().new_pool() };
            let py = gil_pool.python();

            // for any in args {
            //     let downcasted_args: Result<Box<Value>, Box<dyn Any>> = any.downcast::<Value>();
            //     match downcasted_args {
            //         Ok(value_box) => {
            //             println!("String value: {}", value_box);
            //             let val = *value_box;
            //             let converted_map: HashMap<String, Value> = serde_json::from_value(val).unwrap();
            //             let instructions = CommandInstructions::from_value_map(converted_map).unwrap();
            //         },
            //         Err(_) => println!("Not a string"),
            //     }
            // }

            // Convert Rust `args` into Python objects. This might involve type checking and conversion.
            let py_args = match convert_boxed_anys_to_pyany(py, args) {
                Ok(r) => r,
                Err(e) => {
                    // Handle error, maybe convert to a Rust error type
                    println!("Error converting box of anys to py any, the error was: {:?}", e);
                    return Box::new(e) as Box<dyn Any>;
                },
            };

            println!("[MYSCELIUM][HOST][PYTHON BRIDGE] - py_args: {:?}", py_args);

            let args;

            if let Ok(tuple) = py_args.extract::<&PyTuple>(py) {
                args = tuple;
            }
            // If obj is a dict, convert its values to a tuple.
            else if let Ok(dict) = py_args.extract::<&PyDict>(py) {
                let values = dict.values().into_iter().map(|v| v.to_object(py)).collect::<Vec<_>>();
                let tuple = PyTuple::new(py, &values);
                args = tuple;
            }
            // If obj is a list, convert it to a tuple by iterating over its elements.
            else if let Ok(list) = py_args.extract::<&PyList>(py) {
                let elements = list.into_iter().map(|item| item.to_object(py)).collect::<Vec<_>>();
                let tuple = PyTuple::new(py, &elements);
                args = tuple;
            } else {
                // For any other type, wrap the obj in a tuple.
                args = PyTuple::new(py, &[py_args]);
            }

            // let args = convert_to_tuple(py, &py_args).unwrap();

            // let py_tuple = PyTuple::new(py, &args);
            result = py_func.call(py, args, None);

            let response = match result {
                Ok(py_result) => {
                    // Convert the Python result back to Rust
                    // This is a placeholder, actual conversion logic will depend on the expected result type
                    py_result
                },
                Err(e) => {
                    // Handle error, maybe convert to a Rust error type
                    println!("Error calling Python function: {:?}", e);
                    // TODO >>> Create a better error handling mechanism
                    return Box::new(e) as Box<dyn Any>;
                },
            };

            value = extract_pyobject(py, response);
        }

        println!("Value map extracted from callback response: {:?}", value);
        let instructions = {
            // Check if the Value is an object and convert it to HashMap
            if let Some(obj) = value.as_object() {
                let map: HashMap<String, Value> = obj.clone().into_iter().collect();
                match CommandInstructions::from_value_map(map) {
                    Ok(c) => {
                        println!("Instructions extracted in python briedge: {:?}", c);
                        Box::new(c) as Box<dyn Any>
                    }, // Successfully parsed CommandInstructions
                    Err(e) => {
                        // Handling parse failure with a more descriptive error
                        println!("Error: callback returned a non-valid response: {:?}", e);
                        Box::new(CommandError::InvalidResponse("callback returned a non-valid response".to_string())) as Box<dyn Any>
                    },
                }
            } else {
                // Handling the case where the expected JSON object is not an object
                println!("Error: The value is not a JSON object!");
                Box::new(CommandError::NotAJsonObject) as Box<dyn Any>
            }
        };

        return instructions;

        // serde_json::to_string(value)instrctions.to_value_map();

        // * The response of python should be the exactly thing necessary to cast a Commandinstruction,
        // * and maybe alwready have a method to do so

        // CommandInstructions::new(mode, command_type, target, status, origin, actf, kwargs, message)

        // TODO >>> After cast the CommandInstruction convert it in json using the internal method of it to do this
    })
}

/// Converts a JSON value to its corresponding Python object.
///
/// This helper function takes in a JSON value and recursively converts it to the corresponding Python object.
/// This can be useful for translating between Rust and Python data structures.
///
/// # Parameters
///
/// - `py`: The Python interpreter.
/// - `value`: The JSON value to convert.
///
/// # Returns
///
/// Returns the converted Python object.
pub fn translate_value_to_py(py: Python<'_>, value: JsonValue) -> PyResult<PyObject> {
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

pub fn dict_to_object<'l>(py: Python<'l>, dict: &HashMap<String, Value>) -> PyResult<PyObject> {
    let kwargs = PyDict::new(py); // Create a new Python dictionary

    for (key, value) in dict.iter() {
        let py_value = json_value_to_py_object(py, value)?; // Assume this function converts Rust `Value` to `PyObject`
        kwargs.set_item(key, py_value)?; // Insert the key-value pair into the PyDict
    }

    Ok(kwargs.to_object(py)) // Convert the PyDict to PyObject and return
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

pub fn extract_pyobject(py: Python, obj: PyObject) -> serde_json::Value {
    use serde_json::Value;

    if let Ok(dict) = obj.downcast::<PyDict>(py) {
        let mut rust_dict = serde_json::Map::new();
        for (key, value) in dict.iter() {
            let key_str = match key.extract::<String>() {
                Ok(k) => k,
                Err(e) => {
                    println!("Failed to extract key as string: {:?}", e);
                    continue; // Skip this key-value pair
                },
            };
            let rust_value = extract_pyobject(py, value.to_object(py));
            rust_dict.insert(key_str, rust_value);
        }
        Value::Object(rust_dict)
    } else if let Ok(tuple) = obj.downcast::<PyTuple>(py) {
        let rust_list: Vec<_> = tuple.iter().map(|item| extract_pyobject(py, item.to_object(py))).collect();
        Value::Array(rust_list)
    } else if let Ok(list) = obj.downcast::<PyList>(py) {
        let rust_list: Vec<_> = list.iter().map(|item| extract_pyobject(py, item.to_object(py))).collect();
        Value::Array(rust_list)
    } else if let Ok(boolean) = obj.downcast::<PyBool>(py) {
        match boolean.extract::<bool>() {
            Ok(b) => Value::Bool(b),
            Err(e) => {
                println!("Failed to extract boolean: {:?}", e);
                Value::Null
            },
        }
    } else if let Ok(int) = obj.downcast::<PyInt>(py) {
        match int.extract::<i64>() {
            Ok(i) => Value::Number(serde_json::Number::from(i)),
            Err(e) => {
                println!("Failed to extract integer: {:?}", e);
                Value::Null
            },
        }
    } else if let Ok(float) = obj.downcast::<PyFloat>(py) {
        match float.extract::<f64>() {
            Ok(i) => {
                if let Some(num) = serde_json::Number::from_f64(i) {
                    Value::Number(num)
                } else {
                    println!("Failed to extract float!");
                    Value::Null
                }
            },
            Err(e) => {
                println!("Failed to extract float: {:?}", e);
                Value::Null
            },
        }
    } else if let Ok(string) = obj.downcast::<PyString>(py) {
        match string.extract() {
            Ok(s) => Value::String(s),
            Err(e) => {
                println!("Failed to extract string: {:?}", e);
                Value::Null
            },
        }
    } else if obj.is_none(py) {
        Value::Null
    } else {
        println!("Unmatched type for object: {:?}", obj);
        Value::Null
    }
}

pub fn call_callback(py: Python<'_>, command: Command, callback_patterns: MutexGuard<'_, HashMap<String, (Py<PyFunction>, Value)>>) -> PyResult<PyObject> {
    println!("Command to call a callback: {:?}", command);

    let function_name: &String = &command.command.actf;

    // Get the function and args_types from the CALLBACK_PATTERNS
    let (function, _) = callback_patterns.get(function_name).unwrap();

    let command: &CommandInstructions = &command.command;

    let inner_hash_map: HashMap<_, _> = command.kwargs.clone().into_iter().collect();
    let kwargs_map: HashMap<String, Py<PyAny>> = dict_to_kwargs(py, &inner_hash_map).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?;

    // let kwargs_map = match command.command_type() {
    //     CommandType::Response(_) => {
    //         let command = &command.command;

    //         let inner_hash_map: HashMap<_, _> = command.kwargs.clone().into_iter().collect();
    //         dict_to_kwargs(py, &inner_hash_map).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?
    //     },
    //     CommandType::Function(_) => {
    //         let command = &command.command;

    //         let inner_hash_map: HashMap<_, _> = command.kwargs.clone().into_iter().collect();
    //         dict_to_kwargs(py, &inner_hash_map).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?
    //     },

    //     _ => dict_to_kwargs(py, &command.command).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?,
    // };

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

pub fn client_call_callback(py: Python<'_>, command: &Command, callback_patterns: &HashMap<std::string::String, (pyo3::Py<PyFunction>, serde_json::Value)>) -> PyResult<PyObject> {
    println!("Command to call a callback: {:?}", command);

    let function_name: &String = &command.command.actf;

    // Get the function and args_types from the CALLBACK_PATTERNS
    let (function, _) = callback_patterns.get(function_name).unwrap();

    let command: &CommandInstructions = &command.command;

    let inner_hash_map: HashMap<String, Value> = command.convert_to_hashmap_string_value();

    let mut kwargs_dict: HashMap<String, Value> = HashMap::new();

    kwargs_dict.insert("data".to_string(), Value::Object(serde_json::Map::from_iter(inner_hash_map)));

    let kwargs_map: HashMap<String, Py<PyAny>> = dict_to_kwargs(py, &kwargs_dict).map_err(|e| PyErr::new::<PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?;

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
