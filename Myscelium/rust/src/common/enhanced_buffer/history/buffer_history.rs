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

use crate::common::enhanced_buffer::history::register::register::{initialize_buffer_history, write_to_file};

pub struct BufferHistory {
    buffer_type: String,
}

impl BufferHistory {
    pub fn new(buffer_type: &str) -> Self {
        Self { buffer_type: buffer_type.to_string() }
    }

    pub fn log_add_operation(&self, client_key: &String, unique_key: &String, operation: &String) {
        let ts = Utc::now();
        let ts_stamp = ts.timestamp();

        let to_write: String = format!("[{}][{}][{}][{}][ADD] - {}", ts_stamp, self.buffer_type, client_key, unique_key, operation);

        write_to_file(to_write);
    }

    pub fn log_remove_operation(&self, client_key: &String, unique_key: &String, operation: &String) {
        let ts = Utc::now();
        let ts_stamp = ts.timestamp();

        let to_write: String = format!("[{}][{}][{}][{}][REMOVE] - {}", ts_stamp, self.buffer_type, client_key, unique_key, operation);

        write_to_file(to_write);
    }
}
