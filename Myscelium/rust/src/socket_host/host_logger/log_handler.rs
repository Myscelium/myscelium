use lazy_static::lazy_static;

use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};

use serde_json::{from_str, Value};

use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyFunction;
use pyo3::types::{IntoPyDict, PyDict, PyList, PyString, PyTuple};

use std::time::{SystemTime, UNIX_EPOCH};

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use crate::HOST_LOG_LEVEL;
use crate::HOST_NODE_NAME;

use crate::socket_host::host_logger::register::register_mananger;

// TODO >>> REMOVE CALLBACK SET AND CALLBACKS SYSTEM FOR LOG FOR NOW BECAUSE WE WILL USE CUSTOM REGISTRER

lazy_static! {
    static ref CALLBACK_SET: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref LOGS_HANDLER_CALLBACK: Arc<Mutex<HashMap<String, (Py<PyFunction>, Value)>>> = {
        let command_patterns: HashMap<String, (Py<PyFunction>, Value)> = HashMap::new();
        Arc::new(Mutex::new(command_patterns))
    };
}

// TODO >>> Add a mecanism to set the node host name, to be able to indentify in the logs

pub fn set_host_log_level(log_level: String) {
    {
        let mut current_log_level = HOST_LOG_LEVEL.lock();
        *current_log_level = log_level.clone();
        println!("Host log levels seted to: {:?}", log_level);
    }
}

pub fn initialize_host_logs_databse_dir(path: String) {
    register_mananger::logs_registrer_initialize_table(path);
}

// pub fn set_host_logs_handler_callback(callback_pattern: HashMap<String, (Py<PyFunction>, Value)>) {
//     {
//         let mut heart_beat_callback = LOGS_HANDLER_CALLBACK.lock().unwrap();
//         *heart_beat_callback = callback_pattern;
//     }
//     CALLBACK_SET.store(true, Ordering::Relaxed);
// }

fn log_event(node_name: String, log_time: f64, log_name: String, log_level: String, log_msg: String) {
    // let function_name = "logs_handler";

    // let callback_patterns = LOGS_HANDLER_CALLBACK.lock().unwrap().clone();

    // if !CALLBACK_SET.load(Ordering::Relaxed) {
    //     match log_level.as_str() {
    //         "DEBUG" | "INFO" | "WARN" => println!("Default Handler: [{}][{}] - {}", log_time, log_level, log_msg),
    //         "EXCEPTION" => eprintln!("Default Handler: [{}][{}] - {}", log_time, log_level, log_msg),
    //         _ => {},
    //     }
    //     return;
    // }

    println!("{:?}", log_msg);

    register_mananger::registry_log(node_name, log_time, log_name, log_level, log_msg);

    // let function = match callback_patterns.get(function_name) {
    //     Some(function) => function.clone(),
    //     _ => return,
    // };

    // let py;

    // {
    //     let getting_py = unsafe { Python::assume_gil_acquired() };

    //     let gil_pool = unsafe { getting_py.clone().new_pool() };

    //     py = gil_pool.python();

    //     let kwargs = PyDict::new(py);

    //     let py_node_name = &node_name.into_py(py);
    //     let py_log_time = &log_time.into_py(py);
    //     let py_log_name = &log_name.into_py(py);
    //     let py_log_msg = &log_msg.into_py(py);

    //     kwargs.set_item("node_name".to_string(), py_node_name).unwrap();
    //     kwargs.set_item("log_time".to_string(), py_log_time).unwrap();
    //     kwargs.set_item("log_name".to_string(), py_log_name).unwrap();
    //     kwargs.set_item("log_msg".to_string(), py_log_msg).unwrap();

    //     // Call the Python function with the converted arguments
    //     let result = function.0.call(py, (), Some(kwargs)).map_err(|e| {
    //         eprintln!("Error calling function: {:?}", e);
    //         e
    //     });
    // }
}

pub struct Logger {
    log_level: String,
    section: String,
    node_name: String,
}

impl Logger {
    pub fn new(log_level: String, section: &str) -> Self {
        // Placeholder for other initializations

        let node_name: String = HOST_NODE_NAME.lock().clone();

        Logger {
            log_level: log_level.to_string(),
            section: section.to_string(),
            node_name,
        }
    }

    pub fn debug(&self, log: String) {
        if self.log_level == "DEBUG" {
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
            log_event(self.node_name.clone(), ts, self.section.clone(), "DEBUG".to_string(), log.to_string());
        }
    }

    pub fn info(&self, log: String) {
        if (self.log_level == "INFO") || (self.log_level == "DEBUG") {
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
            log_event(self.node_name.clone(), ts, self.section.clone(), "INFO".to_string(), log.to_string());
        }
    }

    pub fn warn(&self, log: String) {
        if (self.log_level == "INFO") || (self.log_level == "WARN") || (self.log_level == "DEBUG") {
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
            log_event(self.node_name.clone(), ts, self.section.clone(), "WARN".to_string(), log.to_string());
        }
    }

    pub fn exception(&self, log: String) {
        if (self.log_level == "INFO") || (self.log_level == "WARN") || (self.log_level == "DEBUG") || (self.log_level == "EXCEPTION") {
            let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
            log_event(self.node_name.clone(), ts, self.section.clone(), "EXCEPTION".to_string(), log.to_string());
        }
    }
}
