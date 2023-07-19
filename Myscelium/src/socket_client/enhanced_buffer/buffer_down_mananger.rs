// use std::hash::Hash;
// use std::sync::Mutex;

use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use pyo3::buffer;

// use std::collections::HashMap;

use super::buffer_functions;

use buffer_functions::SQLiteConnectionPool;
use buffer_functions::UniqueIdGenerator;
use buffer_functions::UniqueParityIdGenerator;

use crate::socket_client::socket_client::Command;

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

use serde_json::{from_str, Value};
use std::collections::HashMap;

lazy_static! {
    static ref BUFFER_PATH: Arc<Mutex<String>> = Arc::new(Mutex::new("buffer.db".to_string()));
    static ref NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));
    static ref BUFFER_POOL: SQLiteConnectionPool = {
        let buffer_path_clone;
        let num_workers_clone;
        {
            let buffer_path = BUFFER_PATH.lock().unwrap();
            buffer_path_clone = buffer_path.clone();

            let num_workers = NUM_WORKERS.lock().unwrap();
            num_workers_clone = num_workers.clone() as usize
        }
        SQLiteConnectionPool::new(num_workers_clone, buffer_path_clone.as_str()).unwrap()
    };
}
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

#[derive(Serialize, Deserialize, Debug)]
pub struct DownCommand {
    pub command_id: Option<u32>,
    pub client_id: String,
    pub parity_id: String,
    pub priority: u8,
    pub command: String,
    pub created_time: f64,
}

impl DownCommand {
    pub fn from(command_id: u32, client_id: String, parity_id: String, priority: u8, command: String, created_time: f64) -> Self {
        let now = Utc::now();
        let timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

        Self {
            command_id: Some(command_id),
            client_id,
            parity_id,
            priority,
            command,
            created_time,
        }
    }

    pub fn new(command_id: u32, client_id: String, parity_id: String, priority: u8, command: String) -> Self {
        let now = Utc::now();
        let timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

        Self {
            command_id: Some(0000u32),
            client_id,
            parity_id,
            priority,
            command,
            created_time: timestamp,
        }
    }

    pub fn from_command(command: Command) -> Self {
        let client_id = command.client_id;
        let parity_id = command.parity_id;
        let priority = command.priority;
        let mut command = serde_json::to_string(&command.command).unwrap();

        let now = Utc::now();
        let timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

        Self {
            command_id: Some(0000u32),
            client_id,
            parity_id,
            priority,
            command,
            created_time: timestamp,
        }
    }
}

impl IntoPy<PyObject> for DownCommand {
    fn into_py(self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item("command_id", self.command_id).unwrap();
        dict.set_item("client_id", self.client_id).unwrap();
        dict.set_item("parity_id", self.parity_id).unwrap();
        dict.set_item("priority", self.priority).unwrap();
        dict.set_item("command", self.command).unwrap();
        dict.into()
    }
}

fn get_registred_ids() -> Vec<i32> {
    let conn = BUFFER_POOL.get_connection().unwrap();

    let mut ids: Vec<i32> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM ClientCommandsReceived").unwrap();
        let commands_iter = smtp
            .query_map(params![], |row| {
                let id: i32 = row.get(0)?;
                Ok(id)
            })
            .unwrap();

        for command in commands_iter {
            ids.push(command.unwrap());
        }
    }

    BUFFER_POOL.release_connection(conn);

    return ids;
}

pub fn buffer_down_initialize_table(buffer_path: String) {
    let mut default_buffer_path = BUFFER_PATH.lock().unwrap();

    let new_buffer_path = format!("{}{}", buffer_path, default_buffer_path);

    *default_buffer_path = new_buffer_path.clone();

    // Create the directory if it does not exist
    let dir_path = std::path::Path::new(&buffer_path);
    if !dir_path.exists() {
        std::fs::create_dir_all(&dir_path).unwrap();
    }

    println!("initializing buffer in: {}", new_buffer_path);

    let buffer_pool = SQLiteConnectionPool::new(10, default_buffer_path.as_str()).unwrap();
    let conn = buffer_pool.get_connection().unwrap();

    let result = conn.execute(
        "CREATE TABLE IF NOT EXISTS ClientCommandsReceived (ID INT PRIMARY KEY, ClientID TEXT, ParityId TEXT, Priority NUMBER, Command TEXT, CreatedTime NUMBER)",
        params![],
    );

    match result {
        Ok(_) => {
            println!("Successfully initialize ClientCommandsReceived table!");
        },
        Err(e) => {
            eprintln!("An error occurred while scheduling the command in the ClientCommandsReceived table: {}", e);
        },
    }

    buffer_pool.release_connection(conn); // Corrected line

    return;
}

pub fn buffer_down_list_schedule() -> Vec<DownCommand> {
    let conn = BUFFER_POOL.get_connection().unwrap();

    let mut commands_schedule: Vec<DownCommand> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM ClientCommandsReceived").unwrap();

        let commands_iter = smtp
            .query_map(params![], |row| {
                Ok(DownCommand::from(
                    row.get(0).unwrap(),
                    row.get(1).unwrap(),
                    row.get(2).unwrap(),
                    row.get(3).unwrap(),
                    row.get(4).unwrap(),
                    row.get(5).unwrap(),
                ))
            })
            .unwrap();

        for command in commands_iter {
            commands_schedule.push(command.unwrap());
        }
    }

    BUFFER_POOL.release_connection(conn);

    return commands_schedule;
}

fn get_registred_parity_ids(client_id: String) -> Vec<String> {
    let conn = BUFFER_POOL.get_connection().unwrap();

    let mut parity_ids: Vec<String> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM ClientCommandsReceived WHERE ClientID = ? ").unwrap();
        let commands_iter = smtp
            .query_map(params![client_id], |row| {
                let parity_id: String = row.get(2)?;
                Ok(parity_id)
            })
            .unwrap();

        for command in commands_iter {
            parity_ids.push(command.unwrap());
        }
    }

    BUFFER_POOL.release_connection(conn);

    return parity_ids;
}

pub fn buffer_down_gen_valid_parity_id(client_id: String) -> String {
    let registred_ids: Vec<String> = get_registred_parity_ids(client_id);

    let mut unique_parity_id_generator = UniqueParityIdGenerator::new(16, registred_ids);

    let valid_parity_id: String = unique_parity_id_generator.gen();

    return valid_parity_id;
}

pub fn buffer_down_clear_old_commands() {
    let now = Utc::now();
    let current_timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

    let schedule = buffer_down_list_schedule();

    if (schedule.is_empty()) {
        return;
    }

    for dow_command in schedule {
        let command_timestamp = dow_command.created_time;

        let time_difference = (current_timestamp - command_timestamp);

        if time_difference >= 30.0 {
            buffer_down_remove_schedule_by_id(dow_command.command_id.unwrap());
            println!(
                "\nCommand: {} from client: {}, too old, clearing from the buffer down schedule!\n",
                dow_command.parity_id, dow_command.client_id
            );
        }
    }
}

pub fn buffer_down_schedule(command: DownCommand) {
    if check_if_parity_id_is_registred(command.client_id.clone(), command.parity_id.clone()) {
        println!("Parity_id: {} alwready registred to client_id: {}, so skiping...", command.parity_id, command.client_id);
        return;
    }

    let registered_ids = get_registred_ids();

    let mut id_generator = UniqueIdGenerator {
        registered_ids: registered_ids,
    };

    let conn = BUFFER_POOL.get_connection().unwrap();

    let now = Utc::now();
    let timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

    let result = conn.execute(
        "INSERT INTO ClientCommandsReceived (ID, ClientID, ParityId, Priority, Command, CreatedTime) VALUES (?, ?, ?, ?, ?, ?);",
        params![
            id_generator.gen(),
            command.client_id,
            command.parity_id,
            command.priority,
            command.command,
            timestamp
        ],
    );

    match result {
        Ok(rows) => {
            if rows > 0 {
                println!("Successfully inserted command in the table ClientCommandsReceived. {} row(s) were affected.", rows);
            } else {
                println!("No rows were affected.");
            }
        },
        Err(e) => {
            eprintln!("An error occurred while inserting the command in the table ClientCommandsReceived: {}", e);
        },
    }

    BUFFER_POOL.release_connection(conn);
}

pub fn check_if_parity_id_is_registred(client_id: String, parity_id: String) -> bool {
    let conn = BUFFER_POOL.get_connection().unwrap();

    let mut ids: Vec<Result<String, _>> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM ClientCommandsReceived WHERE ClientID = ?").unwrap();
        let commands_iter = smtp
            .query_map(params![client_id], |row| {
                let id: String = row.get(2)?;
                Ok(id)
            })
            .unwrap();

        for command in commands_iter {
            ids.push(command)
        }
    }

    BUFFER_POOL.release_connection(conn);

    for id in ids {
        match id {
            Ok(id) => {
                if parity_id == id {
                    return true;
                }
            },
            Err(e) => {
                eprintln!("And error ocurred when obtaining the ids registred in the table ClientCommandsReceived: {}", e);
            },
        }
    }

    return false;
}

pub fn buffer_down_update_schedule(id: i32, client_id: String, parity_id: String, priority: i32, command: String) {
    let conn = BUFFER_POOL.get_connection().unwrap();

    let result = conn.execute(
        "Update ClientCommandsReceived set ClientID = ?, ParityId = ?, Priority = ?, Command = ? where ID = ?",
        params![client_id, parity_id, priority, command, id],
    );

    match result {
        Ok(rows) => {
            if rows > 0 {
                println!("Successfully update the command in the table ClientCommandsReceived. {} row(s) were affected.", rows);
            } else {
                println!("Successfully update the command in the table ClientCommandsReceived. No rows were affected.");
            }
        },
        Err(e) => {
            eprintln!("An error occurred while update the command in the table ClientCommandsReceived: {}", e);
        },
    }

    BUFFER_POOL.release_connection(conn);
}

pub fn buffer_down_remove_schedule_by_id(id: u32) {
    let conn = BUFFER_POOL.get_connection().unwrap();
    let result = conn.execute("DELETE from ClientCommandsReceived where ID = ?", params![id]);

    match result {
        Ok(rows) => {
            println!("Successfully deleted Command of ID: {}. {} rows were affected.", id, rows);
        },
        Err(e) => {
            eprintln!("An error occurred while deleting the command: {} from ClientCommandsReceived table: {}", id, e);
        },
    }

    BUFFER_POOL.release_connection(conn)
}

pub fn buffer_down_remove_schedule_by_parity_id(client_id: String, parity_id: String) {
    let conn = BUFFER_POOL.get_connection().unwrap();

    let result = conn.execute("DELETE from ClientCommandsReceived where ClientID = ? AND ParityId = ?", params![client_id, parity_id]);

    match result {
        Ok(rows) => {
            println!("Successfully deleted Command from Client: {} And ParityID: {}. {} rows were affected.", client_id, parity_id, rows);
        },
        Err(e) => {
            eprintln!(
                "An error occurred while deleting the Client: {} And ParityID: {} from ClientCommandsReceived table: {}",
                client_id, parity_id, e
            );
        },
    }

    BUFFER_POOL.release_connection(conn)
}
