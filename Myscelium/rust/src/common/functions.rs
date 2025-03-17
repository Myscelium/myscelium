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

use crate::common::converters::to_python::dict_to_kwargs;
use crate::common::converters::to_python::translate_value_to_py;
use crate::common::converters::to_rust::extract_pyobject;
use crate::common::converters::to_rust::handle_dict;

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
                        // `py_result` is a Py<PyAny> (an owning pointer).
                        py_result
                    },
                    Err(e) => {
                        println!("Error calling Python function: {:?}", e);
                        return Box::new(e) as Box<dyn Any>;
                    },
                };

                // Convert the owning Py<PyAny> into a Bound<'_, PyAny> by borrowing it.
                // This is safe as long as `response` remains alive for the lifetime of `py`.
                let response_bound = unsafe { Bound::from_borrowed_ptr(py, response.as_ptr()) };

                // Now extract the Python object to a serde_json::Value.
                value = Some(extract_pyobject(py, response_bound));
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

// pub fn extract_arg_types<'py>(arg: Bound<'py, PyAny>) -> PyResult<Value> {
//     if let Ok(arg_dict) = arg.downcast::<PyDict>() {
//         // If the argument is a dictionary, recursively extract the argument types
//         let mut args_types = HashMap::new();
//         for (arg_name, arg_type) in arg_dict.iter() {
//             let arg_name: String = arg_name.extract()?;
//             let arg_type_value = extract_arg_types(arg_type)?;
//             args_types.insert(arg_name, arg_type_value);
//         }
//         Ok(json!(args_types))
//     } else {
//         // If the argument is not a dictionary, extract it as a string
//         let arg_type: String = arg.extract()?;
//         Ok(json!(arg_type))
//     }
// }

pub fn call_callback<'py>(py: Python<'py>, command: Command, callback_patterns: std::sync::MutexGuard<'_, HashMap<String, (Py<PyFunction>, Value)>>) -> PyResult<Bound<'py, PyAny>> {
    println!("Command to call a callback: {:?}", command);

    let function_name = &command.command.actf;

    // Get the function (and any associated args types) from the callback_patterns map.
    let (function, _) = callback_patterns
        .get(function_name)
        .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Function {} not found in callback_patterns", function_name)))?;

    // Convert the owning Py<PyFunction> to a Bound for safe use in this call.
    let function_bound: Bound<'_, PyAny> = unsafe { Bound::from_borrowed_ptr(py, function.as_ptr()) };

    let command_instr: &CommandInstructions = &command.command;

    let inner_hash_map: HashMap<_, _> = command_instr.kwargs.clone().into_iter().collect();
    // Use our updated dict_to_kwargs that returns Bound values.
    let kwargs_map: HashMap<String, Bound<'py, PyAny>> = dict_to_kwargs(py, &inner_hash_map).map_err(|e| PyErr::new::<pyo3::exceptions::PyException, _>(format!("Error converting arguments to kwargs to call client callback: {:?}", e)))?;

    println!("Converted to Python kwargs_map: {:?}", kwargs_map);

    // Create a Python dict and fill it with our kwargs.
    let kwargs = PyDict::new(py);
    for (key, value) in kwargs_map {
        // value is a Bound<'py, PyAny> so pass a reference.
        kwargs.set_item(key, value.as_ref())?;
    }

    // Call the Python function with the kwargs.
    let function_ptr = function_bound.as_ptr();
    let py_function: &PyAny = unsafe { PyAny::from_borrowed_ptr(py, function_ptr) };
    let result_py: Py<PyAny> = py_function.call_with_kwargs((), Some(kwargs))?;

    // Convert the owning Py<PyAny> into a Bound by borrowing its pointer.
    let result_bound = unsafe { Bound::from_borrowed_ptr(py, result_py.as_ptr()) };

    Ok(result_bound)
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
