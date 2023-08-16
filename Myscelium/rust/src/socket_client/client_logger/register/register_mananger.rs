// use std::hash::Hash;
// use std::sync::Mutex;

use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use pyo3::buffer;

// use std::collections::HashMap;

use crate::commom::sql_pool::pool;

use pool::SQLiteConnectionPool;
use pool::UniqueIdGenerator;
use pool::UniqueParityIdGenerator;

use chrono::Utc;

use rusqlite::params;
use serde::{Deserialize, Serialize};

// mod buffer_functions;

// use buffer_functions::UniqueIdGenerator;
// use buffer_functions::SQLiteConnectionPool;

// use pyo3::wrap_pyfunction;
// use pyo3::types::IntoPyDict;

use pyo3::prelude::*;
use pyo3::types::PyDict;

// TODO >>> Add a mecanism toa utomatically save the ClientLogs intoa  database and the client
//*         Add a system to store ClientLogs from host
//*         Add a system to store clients last contact

//>     	Then make a interface in the python side to retirve the ClientLogs from the database
//>         And a system to retrive the client last contact from the databse

// -> DONE
lazy_static! {
    static ref BUFFER_PATH: Arc<Mutex<String>> = Arc::new(Mutex::new("Logs.db".to_string()));
    static ref NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));
    static ref LOGS_REGISTERS_POOL: SQLiteConnectionPool = {
        let buffer_path_clone;
        let num_workers_clone;
        {
            let loggs_storage_path = BUFFER_PATH.lock().unwrap();
            buffer_path_clone = loggs_storage_path.clone();

            let num_workers = NUM_WORKERS.lock().unwrap();
            num_workers_clone = num_workers.clone() as usize
        }
        SQLiteConnectionPool::new(num_workers_clone, buffer_path_clone.as_str()).unwrap()
    };
}

// -> DONE
pub fn set_workers_num(n_workers: u32) {
    let mut default_num_of_workers = NUM_WORKERS.lock().unwrap();

    *default_num_of_workers = n_workers;
}

/*
   However, the rusqlite library in Rust automatically starts a new
   transaction before each command and commits it after the command
   is executed, unless you explicitly start a transaction. This is
   known as "autocommit mode".

*/

#[derive(Serialize, Deserialize, Debug, Clone)] // -> DONE
pub struct Log {
    pub log_id: u32,
    pub node_name: String,
    pub log_time: f64,
    pub log_name: String,
    pub log_level: String,
    pub log_msg: String,
}

// -> DONE
impl IntoPy<PyObject> for Log {
    fn into_py(self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item("log_id", self.log_id).unwrap();
        dict.set_item("node_name", self.node_name).unwrap();
        dict.set_item("log_time", self.log_time).unwrap();
        dict.set_item("log_level", self.log_level).unwrap();
        dict.set_item("log_msg", self.log_msg).unwrap();
        dict.into()
    }
}

// -> DONE
fn get_registred_ids() -> Vec<u32> {
    let conn = LOGS_REGISTERS_POOL.get_connection().unwrap();

    let mut ids: Vec<u32> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM ClientLogs").unwrap();
        let commands_iter = smtp
            .query_map(params![], |row| {
                let id: u32 = row.get(0)?;
                Ok(id)
            })
            .unwrap();

        for command in commands_iter {
            ids.push(command.unwrap());
        }
    }

    LOGS_REGISTERS_POOL.release_connection(conn);

    return ids;
}

// -> DONE
pub fn logs_registrer_initialize_table(loggs_storage_path: String) {
    let mut default_loggs_storage_path = BUFFER_PATH.lock().unwrap();

    let new_loggs_storage_path = format!("{}{}", loggs_storage_path, default_loggs_storage_path);

    *default_loggs_storage_path = new_loggs_storage_path.clone();

    // Create the directory if it does not exist
    let dir_path = std::path::Path::new(&loggs_storage_path);
    if !dir_path.exists() {
        std::fs::create_dir_all(&dir_path).unwrap();
    }

    println!("initializing ClientLogs table in: {}", new_loggs_storage_path);

    let buffer_pool = SQLiteConnectionPool::new(10, default_loggs_storage_path.as_str()).unwrap();
    let conn = buffer_pool.get_connection().unwrap();

    let result = conn.execute(
        "CREATE TABLE IF NOT EXISTS ClientLogs (ID INT PRIMARY KEY, NodeName TEXT, LogTime NUMBER, LogName TEXT, LogLevel TEXT, LogMsg TEXT)",
        params![],
    );

    match result {
        Ok(_) => {
            println!("Successfully initialize ClientLogs table!");
        },
        Err(e) => {
            eprintln!("An error occurred while scheduling the command in the ClientLogs table: {}", e);
        },
    }

    buffer_pool.release_connection(conn); // Corrected line

    return;
}

// -> DONE
pub fn registry_log(node_name: String, log_time: f64, log_name: String, log_level: String, log_msg: String) {
    let conn = LOGS_REGISTERS_POOL.get_connection().unwrap();

    // let now = Utc::now();
    // let timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

    let registered_ids = get_registred_ids();

    let mut id_generator = UniqueIdGenerator {
        registered_ids: registered_ids,
    };

    let result = conn.execute(
        "INSERT INTO ClientLogs (ID, NodeName, LogTime, LogName, LogLevel, LogMsg) VALUES (?, ?, ?, ?, ?, ?);",
        params![id_generator.gen(), node_name, log_time, log_name, log_level, log_msg],
    );

    match result {
        Ok(rows) => {
            if rows > 0 {
                println!("Successfully inserted Log in the table ClientLogs. {} row(s) were affected.", rows);
            } else {
                println!("No rows were affected.");
            }
        },
        Err(e) => {
            eprintln!("An error occurred while inserting the Log in the table ClientLogs: {}", e);
        },
    }

    LOGS_REGISTERS_POOL.release_connection(conn);
}

// -> DONE
pub fn list_logs() -> Vec<Log> {
    let conn = LOGS_REGISTERS_POOL.get_connection().unwrap();

    let mut registred_logs: Vec<Log> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM ClientLogs").unwrap();

        let logs_iter = smtp
            .query_map(params![], |row| {
                Ok(Log {
                    log_id: row.get(0).unwrap(),
                    node_name: row.get(1).unwrap(),
                    log_time: row.get(2).unwrap(),
                    log_name: row.get(3).unwrap(),
                    log_level: row.get(4).unwrap(),
                    log_msg: row.get(5).unwrap(),
                })
            })
            .unwrap();

        for log in logs_iter {
            match log {
                Ok(l) => {
                    registred_logs.push(l);
                },

                Err(e) => {
                    println!("An error occurred while getting the ClientLogs vec in list_logs, the error was: {}", e);
                },
            }
        }
    }

    LOGS_REGISTERS_POOL.release_connection(conn);

    return registred_logs;
}

pub fn remove_log_by_id(log_id: u32) {
    let conn = LOGS_REGISTERS_POOL.get_connection().unwrap();
    let result = conn.execute("DELETE from ClientLogs where ID = ?", params![log_id]);

    match result {
        Ok(rows) => {
            println!("Successfully deleted Log of ID: {}. {} rows were affected.", log_id, rows);
        },
        Err(e) => {
            eprintln!("An error occurred while deleting the Log: {} from ClientLogs table: {}", log_id, e);
        },
    }

    LOGS_REGISTERS_POOL.release_connection(conn)
}
