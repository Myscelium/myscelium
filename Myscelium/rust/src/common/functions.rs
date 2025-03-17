use pyo3::exceptions::PyException;
use pyo3::exceptions::PyValueError;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyAny, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use pyo3::wrap_pyfunction;
use pyo3::Py;
use pyo3::{exceptions, IntoPyObjectExt};
use pyo3::{PyErr, PyObject, PyResult, Python};
use serde_json::Number;
use serde_json::Value as JsonValue;
use serde_json::{json, Value};
use std::any::Any;
use std::collections::HashMap;
use std::os::raw::c_ulonglong;
use std::result;
use std::sync::MutexGuard;
use OxidizedMyscelium::Command;
use OxidizedMyscelium::CommandError;
use OxidizedMyscelium::CommandInstructions;
use OxidizedMyscelium::CommandType;
use OxidizedMyscelium::ResultType;

use crate::common::converters::to_python::translate_value_to_py;

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

fn convert_boxed_anys_to_pylist<'py>(py: Python<'py>, boxed_anys: Vec<Box<dyn Any>>) -> PyResult<Bound<'py, PyList>> {
    let py_list: Bound<'py, PyList> = PyList::empty(py); // Already GIL-bound

    for boxed_any in boxed_anys {
        match boxed_any.downcast::<Value>() {
            Ok(value_box) => {
                let val = value_box.clone();
                let py_item = translate_value_to_py(py, *val)?;
                py_list.append(py_item)?;
            },
            Err(_) => println!("Not a value"), // TODO >>> Improve the error handling here!
        }
    }

    Ok(py_list)
}

// fn convert_to_tuple<'a>(py: Python<'a>, obj: Bound<PyObject>) -> PyResult<&'a PyTuple> {
//     // If obj is a tuple, return it directly.

//     if let Ok(tuple) = obj.extract::<PyTuple>(py) {
//         return Ok(tuple);
//     }

//     // If obj is a dict, convert its values to a tuple.
//     if let Ok(dict) = obj.extract::<&PyDict>(py) {
//         let values = dict.values().into_iter().map(|v| v.to_object(py)).collect::<Vec<_>>();
//         let tuple = PyTuple::new(py, &values);
//         return Ok(tuple);
//     }

//     // If obj is a list, convert it to a tuple by iterating over its elements.
//     if let Ok(list) = obj.extract::<&PyList>(py) {
//         let elements = list.into_iter().map(|item| item.to_object(py)).collect::<Vec<_>>();
//         let tuple = PyTuple::new(py, &elements);
//         return Ok(tuple);
//     }

//     // For any other type, wrap the obj in a tuple.
//     let tuple = PyTuple::new(py, &[obj]);
//     Ok(tuple)
// }

fn handle_dict(py: Python, dict: Bound<PyDict>) -> HashMap<String, String> {
    let mut rust_dict = HashMap::new();

    for (key, value) in dict.iter() {
        let key_str: String = key.extract().unwrap();

        if let Ok(value_str) = value.extract::<String>() {
            rust_dict.insert(key_str, value_str);
        } else if let Ok(value_int) = value.extract::<i32>() {
            rust_dict.insert(key_str, value_int.to_string());
        } else if let Ok(value_list) = value.extract::<Vec<String>>() {
            rust_dict.insert(key_str, format!("{:?}", value_list));
        } else if let Ok(nested_dict) = value.downcast::<PyDict>() {
            rust_dict.insert(key_str, format!("{:?}", handle_dict(py, (*nested_dict).clone())));
        } else {
            // Handle other types as needed
        }
    }

    rust_dict
}

/// Wraps a Python function into a Rust closure that can be executed with dynamic parameters.
pub fn wrap_py_function(py_func: Py<PyFunction>, self_key: String) -> Box<dyn Fn(Vec<Box<dyn Any + 'static>>) -> Box<dyn Any> + Send + Sync> {
    Box::new(move |args: Vec<Box<dyn Any + 'static>>| -> Box<dyn Any> {
        // Convert args to Python objects here. You might need to dynamically check types and convert them accordingly.
        // This is a placeholder showing the concept, actual implementation may vary based on your specific needs.

        println!("[MYSCELIUM][HOST][PYTHON BRIDGE] - Callback args: {:?}", args);

        let mut result: Option<Result<Py<PyAny>, PyErr>> = None;
        let mut value: Option<Value> = None;

        Python::with_gil(|py| {
            // Convert Rust `args` into Python objects. This might involve type checking and conversion.
            let py_args = match convert_boxed_anys_to_pylist(py, args) {
                Ok(r) => r,
                Err(e) => {
                    // Handle error, maybe convert to a Rust error type
                    println!("Error converting box of anys to py any, the error was: {:?}", e);
                    return Box::new(e) as Box<dyn Any>;
                },
            };

            println!("[MYSCELIUM][HOST][PYTHON BRIDGE] - py_args: {:?}", py_args);

            let mut args: Option<Bound<'_, PyTuple>> = None;
            if let Ok(tuple) = py_args.extract::<Bound<'_, PyTuple>>() {
                args = Some(tuple);
            }
            // If obj is a dict, convert its values to a tuple.
            else if let Ok(dict) = py_args.extract::<Bound<'_, PyDict>>() {
                let rust_dict = handle_dict(py, dict);

                // Convert the HashMap values into a Python tuple
                let values: Vec<Bound<PyString>> = rust_dict
                    .values()
                    .map(|v| PyString::new(py, v)) // ✅ Explicitly convert `String` to `PyString`
                    .collect();

                let tuple = PyTuple::new(py, values);
                if let Ok(tuple) = tuple {
                    args = Some(tuple);
                }
            }
            // If obj is a list, convert it to a tuple by iterating over its elements.
            else if let Ok(list) = py_args.extract::<Bound<'_, PyList>>() {
                let tuple = PyTuple::new(py, list);
                if let Ok(tuple) = tuple {
                    args = Some(tuple);
                }
            } else {
                // For any other type, wrap the obj in a tuple.
                args = match PyTuple::new(py, &[py_args]) {
                    Ok(tuple) => Some(tuple),
                    Err(e) => {
                        println!("Error creating Python tuple: {:?}", e);
                        return Box::new(e) as Box<dyn Any>;
                    },
                };
            }

            // -> Call the function
            if let Some(args) = args {
                result = Some(py_func.call(py, args, None));
            } else {
                let e = PyErr::new::<pyo3::exceptions::PyValueError, _>("Args isn't some, impossible to call function!");
                println!("Error calling function: {:?}", e);
                return Box::new(e) as Box<dyn Any>;
            }

            if let Some(result) = result {
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

                value = Some(extract_pyobject(py, response));
                return Box::new(value.clone()) as Box<dyn Any>;
            } else {
                let e = PyErr::new::<pyo3::exceptions::PyValueError, _>("Function calling result isn't Some!");
                println!("Error calling function: {:?}", e);
                return Box::new(e) as Box<dyn Any>;
            }
        });

        // Verify if value is some:
        if !value.is_some() {
            let e = PyErr::new::<pyo3::exceptions::PyValueError, _>("Function calling result isn't Some!");
            println!("Error calling function: {:?}", e);
            return Box::new(e) as Box<dyn Any>;
        }

        let value = value.unwrap(); // Synce value is some, unwrap it!

        println!("Value map extracted from callback response: {:?}", value);
        let instructions = {
            // Check if the Value is an object and convert it to HashMap

            if let Some(obj) = value.as_object() {
                let mut map: HashMap<String, Value> = obj.clone().into_iter().collect();

                // > Set the origin of the response as the self key, since the response needs to have
                // > the origin pointing to the place that generate it, don't mattering from were
                // > comes the command that triggered the handler that generated it

                // TODO >>> Add a special case to the cases were the response is redirected to
                // TODO other clients this cases we need to keep the origin of the command

                map.insert("origin".to_string(), Value::from(Value::String(self_key.clone())));
                match CommandInstructions::from_value_map(map) {
                    Ok(c) => {
                        println!("Instructions extracted in python briedge: {:?}", c);
                        Box::new(Some(c)) as Box<dyn Any>
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

pub fn extract_arg_types<'py>(arg: Bound<'py, PyAny>) -> PyResult<Value> {
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

    println!("Converted to Python kwargs_map: {:?}", kwargs_map);

    // -> Convert to py dict
    let kwargs = PyDict::new(py);
    for (key, value) in kwargs_map {
        kwargs.set_item(key, value).unwrap();
    }

    // Call the Python function with the converted arguments
    let result = function.call_with_kwargs((), Some(kwargs))?;

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
    let result = function.call_with_kwargs((), Some(kwargs))?;

    let result_obj: PyObject = result.clone().into(); // Convert the result into a PyObject

    Ok(result_obj) // Return the PyObject
}
