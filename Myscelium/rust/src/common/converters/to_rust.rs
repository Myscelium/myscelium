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

pub fn extract_pyobject<'py>(py: Python<'py>, obj: Bound<'py, PyAny>) -> Value {
    if let Ok(dict) = obj.as_ref().downcast::<PyDict>() {
        let mut rust_dict = serde_json::Map::new();
        for (key, value) in dict.iter() {
            // Try to extract the key as a Rust String.
            let key_str = match key.extract::<String>() {
                Ok(k) => k,
                Err(e) => {
                    println!("Failed to extract key as string: {:?}", e);
                    continue; // Skip this key-value pair
                },
            };
            // Convert the value (a &PyAny) into a Bound so we can recurse.
            let value_bound = unsafe { Bound::from_borrowed_ptr(py, value.as_ptr()) };
            let rust_value = extract_pyobject(py, value_bound);
            rust_dict.insert(key_str, rust_value);
        }
        Value::Object(rust_dict)
    } else if let Ok(tuple) = obj.as_ref().downcast::<PyTuple>() {
        let rust_list: Vec<Value> = tuple
            .iter()
            .map(|item| {
                let item_bound = unsafe { Bound::from_borrowed_ptr(py, item.as_ptr()) };
                extract_pyobject(py, item_bound)
            })
            .collect();
        Value::Array(rust_list)
    } else if let Ok(list) = obj.as_ref().downcast::<PyList>() {
        let rust_list: Vec<Value> = list
            .iter()
            .map(|item| {
                let item_bound = unsafe { Bound::from_borrowed_ptr(py, item.as_ptr()) };
                extract_pyobject(py, item_bound)
            })
            .collect();
        Value::Array(rust_list)
    } else if let Ok(boolean) = obj.as_ref().downcast::<PyBool>() {
        match boolean.extract::<bool>() {
            Ok(b) => Value::Bool(b),
            Err(e) => {
                println!("Failed to extract boolean: {:?}", e);
                Value::Null
            },
        }
    } else if let Ok(int) = obj.as_ref().downcast::<PyInt>() {
        match int.extract::<i64>() {
            Ok(i) => Value::Number(serde_json::Number::from(i)),
            Err(e) => {
                println!("Failed to extract integer: {:?}", e);
                Value::Null
            },
        }
    } else if let Ok(float) = obj.as_ref().downcast::<PyFloat>() {
        match float.extract::<f64>() {
            Ok(f) => {
                if let Some(num) = serde_json::Number::from_f64(f) {
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
    } else if let Ok(string) = obj.as_ref().downcast::<PyString>() {
        match string.extract::<String>() {
            Ok(s) => Value::String(s),
            Err(e) => {
                println!("Failed to extract string: {:?}", e);
                Value::Null
            },
        }
    } else if obj.as_ref().is_none() {
        Value::Null
    } else {
        println!("Unmatched type for object: {:?}", obj);
        Value::Null
    }
}
