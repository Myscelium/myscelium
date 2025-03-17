use pyo3::exceptions::PyValueError;
use pyo3::ffi;
use pyo3::prelude::*;
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyAny, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use pyo3::{exceptions, IntoPyObjectExt};
use pyo3::{PyErr, PyObject, PyResult, Python};
use serde_json::Number;
use serde_json::Value as JsonValue;
use serde_json::{json, Value};
use std::os::raw::c_ulonglong;

/// A trait to convert Rust numeric types into a Bound<'py, PyAny> safely.
pub trait IntoPyNumber<'py> {
    fn into_py_number(self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::PyAny>>;
}

impl<'py> IntoPyNumber<'py> for i64 {
    fn into_py_number(self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::PyAny>> {
        unsafe {
            // Use the CPython API: PyLong_FromLongLong.
            let ptr = ffi::PyLong_FromLongLong(self);
            if ptr.is_null() {
                Err(PyValueError::new_err("Failed to convert i64 to Python int"))
            } else {
                Ok(Bound::from_owned_ptr(py, ptr))
            }
        }
    }
}

impl<'py> IntoPyNumber<'py> for u64 {
    fn into_py_number(self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::PyAny>> {
        unsafe {
            // Use the CPython API: PyLong_FromUnsignedLongLong.
            let ptr = ffi::PyLong_FromUnsignedLongLong(self as c_ulonglong);
            if ptr.is_null() {
                Err(PyValueError::new_err("Failed to convert u64 to Python int"))
            } else {
                Ok(Bound::from_owned_ptr(py, ptr))
            }
        }
    }
}

impl<'py> IntoPyNumber<'py> for f64 {
    fn into_py_number(self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        unsafe {
            // Use the CPython API: PyFloat_FromDouble to create a Python float.
            let ptr = ffi::PyFloat_FromDouble(self);
            if ptr.is_null() {
                Err(PyValueError::new_err("Failed to convert f64 to Python float"))
            } else {
                Ok(Bound::from_owned_ptr(py, ptr))
            }
        }
    }
}

/// Helper function to convert a serde_json::Number into a Python object
fn translate_number<'py>(py: Python<'py>, num: Number) -> PyResult<Bound<'py, pyo3::PyAny>> {
    if let Some(n) = num.as_i64() {
        n.into_py_number(py)
    } else if let Some(n) = num.as_u64() {
        n.into_py_number(py)
    } else if let Some(n) = num.as_f64() {
        n.into_py_number(py)
    } else {
        Err(PyValueError::new_err("Invalid number type"))
    }
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
pub fn translate_value_to_py(py: pyo3::Python, value: JsonValue) -> PyResult<Bound<'_, PyAny>> {
    match value {
        JsonValue::Null => unsafe { Ok(Bound::from_borrowed_ptr(py, py.None().as_ptr())) },
        JsonValue::Bool(b) => {
            let py_bool = PyBool::new(py, b);
            Ok(py_bool.as_ref().clone())
        },
        JsonValue::Number(num) => translate_number(py, num),
        JsonValue::String(s) => Ok(s.to_owned().into_bound_py_any(py)?),
        JsonValue::Array(arr) => {
            let py_list = PyList::empty(py);
            for item in arr {
                let py_item = translate_value_to_py(py, item)?;
                py_list.append(py_item)?;
            }
            Ok(py_list.into_bound_py_any(py)?)
        },
        JsonValue::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (k, v) in obj {
                let py_value = translate_value_to_py(py, v)?;
                py_dict.set_item(k, py_value)?;
            }
            Ok(py_dict.into_bound_py_any(py)?)
        },
    }
}

#[cfg(test)]
mod json_to_py_tests {
    use super::*;
    use pyo3::prelude::*;
    use pyo3::types::{PyAny, PyDict, PyList, PyString};
    use serde_json::{json, Number, Value as JsonValue};
    use std::collections::HashMap;

    // Helper to extract a value from a Bound<'py, PyAny> by converting it into a &PyAny.
    fn extract_from_bound<'py, T: for<'a> pyo3::FromPyObject<'a>>(bound: Bound<'py, PyAny>) -> PyResult<T> {
        // Convert Bound to a &PyAny and then extract.
        bound.as_ref().extract()
    }

    #[test]
    fn test_into_py_number_i64() {
        Python::with_gil(|py| {
            let n: i64 = 42;
            let bound = n.into_py_number(py).expect("i64 conversion failed");
            let extracted: i64 = extract_from_bound(bound).expect("Extraction failed");
            assert_eq!(extracted, 42);
        });
    }

    #[test]
    fn test_into_py_number_u64() {
        Python::with_gil(|py| {
            let n: u64 = 100;
            let bound = n.into_py_number(py).expect("u64 conversion failed");
            let extracted: u64 = extract_from_bound(bound).expect("Extraction failed");
            assert_eq!(extracted, 100);
        });
    }

    #[test]
    fn test_into_py_number_f64() {
        Python::with_gil(|py| {
            let n: f64 = 3.14159;
            let bound = n.into_py_number(py).expect("f64 conversion failed");
            let extracted: f64 = extract_from_bound(bound).expect("Extraction failed");
            assert!((extracted - 3.14159).abs() < 1e-5);
        });
    }

    #[test]
    fn test_translate_value_to_py_null() {
        Python::with_gil(|py| {
            let value = JsonValue::Null;
            let bound = translate_value_to_py(py, value).expect("Null conversion failed");
            // Check that the Python object is None.
            assert!(bound.as_ref().is_none());
        });
    }

    #[test]
    fn test_translate_value_to_py_bool() {
        Python::with_gil(|py| {
            let value = JsonValue::Bool(true);
            let bound = translate_value_to_py(py, value).expect("Bool conversion failed");
            let extracted: bool = extract_from_bound(bound).expect("Extraction failed");
            assert_eq!(extracted, true);
        });
    }

    #[test]
    fn test_translate_value_to_py_number() {
        Python::with_gil(|py| {
            // Using serde_json's json! macro for convenience.
            let value = json!(42);
            let bound = translate_value_to_py(py, value).expect("Number conversion failed");
            let extracted: i64 = extract_from_bound(bound).expect("Extraction failed");
            assert_eq!(extracted, 42);
        });
    }

    #[test]
    fn test_translate_value_to_py_string() {
        Python::with_gil(|py| {
            let value = JsonValue::String("hello".to_string());
            let bound = translate_value_to_py(py, value).expect("String conversion failed");
            let extracted: String = extract_from_bound(bound).expect("Extraction failed");
            assert_eq!(extracted, "hello".to_string());
        });
    }

    #[test]
    fn test_translate_value_to_py_array() {
        Python::with_gil(|py| {
            let value = json!([1, 2, 3]);
            let bound = translate_value_to_py(py, value).expect("Array conversion failed");
            // Extract as a Vec<i64> from the Python list.
            let list: Vec<i64> = bound.as_ref().extract().expect("Extraction failed");
            assert_eq!(list, vec![1, 2, 3]);
        });
    }

    #[test]
    fn test_translate_value_to_py_object() {
        Python::with_gil(|py| {
            let value = json!({"a": 10, "b": 20});
            let bound = translate_value_to_py(py, value).expect("Object conversion failed");
            // Extract as a HashMap<String, i64> from the Python dict.
            let dict: HashMap<String, i64> = bound.as_ref().extract().expect("Extraction failed");
            assert_eq!(dict.get("a"), Some(&10));
            assert_eq!(dict.get("b"), Some(&20));
        });
    }
}
