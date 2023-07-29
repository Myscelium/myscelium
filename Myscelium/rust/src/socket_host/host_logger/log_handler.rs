use lazy_static::lazy_static;

use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};

use serde_json::{from_str, Value};

use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyFunction;
use pyo3::types::{IntoPyDict, PyDict, PyList, PyString, PyTuple};

lazy_static! {
    static ref LOGS_HANDLER_CALLBACK: Arc<Mutex<HashMap<String, (Py<PyFunction>, Value)>>> = {
        let command_patterns: HashMap<String, (Py<PyFunction>, Value)> = HashMap::new();
        Arc::new(Mutex::new(command_patterns))
    };
}

pub fn set_logs_handler_callback(callback_pattern: HashMap<String, (Py<PyFunction>, Value)>) {
    {
        let mut heart_beat_callback = LOGS_HANDLER_CALLBACK.lock().unwrap();
        *heart_beat_callback = callback_pattern;
    }
}

pub fn update_last_contact(py: Python<'_>, log: String) {
    let function_name = "logs_handler";

    let callback_patterns = LOGS_HANDLER_CALLBACK.lock().unwrap();

    let function = match callback_patterns.get(function_name) {
        Some(function) => function.clone(),
        _ => return,
    };

    let kwargs = PyDict::new(py);

    let py_log = &log.into_py(py);

    kwargs.set_item("log".to_string(), py_log).unwrap();

    // TODO >>> Implement sector that the log came from
    // TODO >>> Implement log ts

    // Call the Python function with the converted arguments
    let result = function.0.call(py, (), Some(kwargs)).map_err(|e| {
        eprintln!("Error calling function: {:?}", e);
        e
    });
}
