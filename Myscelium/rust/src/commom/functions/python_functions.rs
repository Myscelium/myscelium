use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use pyo3::exceptions;

use pyo3::exceptions::PyException;
use pyo3::types::{IntoPyDict, PyAny, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use pyo3::IntoPy;
use pyo3::Py;
use pyo3::ToPyObject;
use pyo3::{PyErr, PyObject, PyResult, Python};

use crate::commom::structs::results_structs::ResultType;

use std::collections::HashMap;

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
    // let logger = acquire_logger!("Transposer - Py Dict to kwargs Converter");

    // Check if the dict contains the function name as a key
    if !dict.contains_key("args") {
        // If it does not, return an empty HashMap since there are no arguments
        let kwargs: HashMap<String, PyObject> = HashMap::new();
        return Ok(kwargs);
    }

    let args_string = match dict.get("args") {
        Some(Value::String(s)) => s,
        _ => return Err(PyErr::new::<PyException, _>("The args key is not found or not a string.")),
    };

    let sub_dict: HashMap<String, Value> = serde_json::from_str(args_string).unwrap();

    // logger.debug(format!("Args extracted: {:?}", sub_dict));

    let mut kwargs: HashMap<String, PyObject> = HashMap::new();
    for (key, value) in sub_dict.iter() {
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
        kwargs.insert(key.clone(), py_value);
    }

    // logger.debug(format!("kwargs: {:?}", kwargs));

    Ok(kwargs)
}

pub fn dict_to_tuple<'l>(py: Python<'l>, dict: &HashMap<String, Value>) -> PyResult<&'l PyTuple> {
    // let logger = acquire_logger!("Transposer - Py Dict to Tuple Converter");

    // Check if the dict contains the function name as a key
    if !dict.contains_key("args") {
        // If it does not, return an empty Vec since there are no arguments
        let mut values: Vec<PyObject> = Vec::new();
        return Ok(PyTuple::new(py, values));
    }

    let args_string = match dict.get("args") {
        Some(Value::String(s)) => s,
        _ => return Err(PyErr::new::<PyException, _>("The args key is not found or not a string.")),
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
