use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;

use chrono::Utc;

use lazy_static::lazy_static;

use std::collections::HashMap;

use serde_json::{from_str, Value};

use pyo3::prelude::*;
use pyo3::types::PyFunction;

use std::time::{SystemTime, UNIX_EPOCH};

use std::sync::atomic::AtomicBool;

use crate::CLIENT_LOG_LEVEL;
use crate::CLIENT_NODE_NAME;

// TODO >>> REMOVE CALLBACK SET AND CALLBACKS SYSTEM FOR LOG FOR NOW BECAUSE WE WILL USE CUSTOM REGISTER

lazy_static! {
    static ref CALLBACK_SET: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref BUFFER_HISTORY: Arc<Mutex<String>>> = {
        let command_patterns: HashMap<String, (Py<PyFunction>, Value)> = HashMap::new();
        Arc::new(Mutex::new(command_patterns))
    };
}

pub struct BufferHistory {
    node_name: String,
    buffer_type: String,
}

impl BufferHistory {
    pub fn new(node_name: String, buffer_type: String) -> Self {
        Self { node_name, buffer_type }
    }

    pub fn log_add_data(&self, unique_key: String, operation: String) {
        let ts = Utc::now();
        let ts_stamp = ts.timestamp();

        let to_write: String = format!("[{}][{}][{}][{}][ADD] - {}", ts_stamp, self.node_name, self.buffer_type, unique_key, operation);
    }

    pub fn log_remove_data(&self, unique_key: String, operation: String) {
        let ts = Utc::now();
        let ts_stamp = ts.timestamp();

        let to_write: String = format!("[{}][{}][{}][{}][REMOVE] - {}", ts_stamp, self.node_name, self.buffer_type, unique_key, operation);
    }
}
