
use lazy_static::lazy_static;

use super::buffer_functions;

use buffer_functions::UniqueIdGenerator;
use buffer_functions::SQLiteConnectionPool;
use buffer_functions::UniqueParityIdGenerator;

use rusqlite::params;

use serde::{Serialize, Deserialize};

use pyo3::prelude::*;
use pyo3::types::PyDict;

use std::sync::{Arc, Mutex};

use std::collections::HashMap;

use chrono::Utc;

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

/*
    However, the rusqlite library in Rust automatically starts a new 
    transaction before each command and commits it after the command 
    is executed, unless you explicitly start a transaction. This is 
    known as "autocommit mode".
    
 */

 pub fn set_workers_num (n_workers:u32) {
    
    let mut default_num_of_workers = NUM_WORKERS.lock().unwrap();

    *default_num_of_workers = n_workers;

 }

 #[derive(Serialize, Deserialize, Debug)]
 pub struct UpCommand {
    pub command_id:i32,
    pub client_id:String,
    pub parity_id:String,
    pub priority:u8,
    pub command:String,
    pub created_time:f64
}

impl IntoPy <PyObject> for UpCommand {
    fn into_py (self, py:Python) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item("command_id", self.command_id).unwrap();
        dict.set_item("client_id", self.client_id).unwrap();
        dict.set_item("parity_id", self.parity_id).unwrap();
        dict.set_item("priority", self.priority).unwrap();
        dict.set_item("command", self.command).unwrap();
        dict.into()
    }
}

fn get_registred_ids () -> Vec<i32> {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let mut ids:Vec<i32> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM ClientCommandsTosend").unwrap();
        let commands_iter = smtp.query_map(params![], |row| {
            let id:i32 = row.get(0).unwrap();
            Ok(id)
        }).unwrap();

        for id in commands_iter {
            ids.push(id.unwrap());
        }

    }

    BUFFER_POOL.release_connection(conn);

    return ids;
}


pub fn buffer_up_initialize_table(buffer_path: String) {
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
        "CREATE TABLE IF NOT EXISTS ClientCommandsTosend (ID INT PRIMARY KEY, ClientID TEXT, ParityId TEXT, Priority NUMBER, Command TEXT, CreatedTime NUMBER)",
        params![],
    );

    match result {
        Ok(_) => {
            println!("Successfully initialize ClientCommandsTosend table!");
        }
        Err(e) => {
            eprintln!("An error occurred while scheduling the command in the ClientCommandsTosend table: {}", e);
        }
    }

    buffer_pool.release_connection(conn);

    return;
}

fn get_registred_parity_ids (client_id:String) -> Vec<String> {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let mut parity_ids:Vec<String> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM ClientCommandsTosend WHERE ClientID = ? ").unwrap();
        let commands_iter = smtp.query_map(params![client_id], |row| {
            let parity_id: String = row.get(2)?;
            Ok(parity_id)
        }).unwrap();

        for command in commands_iter {
            parity_ids.push(command.unwrap());
        }

    }

    BUFFER_POOL.release_connection(conn);

    return parity_ids;

}

pub fn buffer_up_gen_valid_parity_id (client_id:String) -> String {

    let registred_ids:Vec<String> = get_registred_parity_ids(client_id);

    let mut unique_parity_id_generator = UniqueParityIdGenerator::new(16, registred_ids);

    let valid_parity_id:String = unique_parity_id_generator.gen();

    return valid_parity_id;

}

pub fn buffer_up_get_scheduled_by_parity_id (client_id:String, parity_id:String) -> Vec<UpCommand> {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let mut commands_schedule:Vec<UpCommand> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM ClientCommandsTosend WHERE ClientID = ? AND ParityId = ?").unwrap();

        let commands_iter = smtp.query_map(params![client_id, parity_id], |row| {
    
            Ok (
    
                UpCommand{command_id:row.get(0).unwrap(), 
                client_id:row.get(1).unwrap(),
                parity_id:row.get(2).unwrap(),
                priority:row.get(3).unwrap(),
                command:row.get(4).unwrap(),
                created_time:row.get(5).unwrap()}
    
            )
    
        }).unwrap();

        for command in commands_iter {
            commands_schedule.push(command.unwrap());
        }

    }
    
    BUFFER_POOL.release_connection(conn);
    
    return commands_schedule;

}


pub fn buffer_up_list_schedule () -> Vec<UpCommand> {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let mut commands_schedule:Vec<UpCommand> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM ClientCommandsTosend").unwrap();

        let commands_iter = smtp.query_map(params![], |row| {
    
            Ok (
    
                UpCommand{command_id:row.get(0).unwrap(), 
                client_id:row.get(1).unwrap(),
                parity_id:row.get(2).unwrap(),
                priority:row.get(3).unwrap(),
                command:row.get(4).unwrap(),
                created_time:row.get(5).unwrap()}
    
            )
    
        }).unwrap();

        for command in commands_iter {
            commands_schedule.push(command.unwrap());
        }

    }
    
    BUFFER_POOL.release_connection(conn);
    
    return commands_schedule;

}


pub fn buffer_up_schedule (client_id:String, parity_id:String, priority:u8, command:String) {

    let registered_ids = get_registred_ids();

    let mut id_generator = UniqueIdGenerator{registered_ids:registered_ids};

    let conn = BUFFER_POOL.get_connection().unwrap();

    let now = Utc::now();
    let timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

    let result = conn.execute(
        "INSERT INTO ClientCommandsTosend (ID, ClientID, ParityId, Priority, Command, CreatedTime) VALUES (?, ?, ?, ?, ?, ?);",
        params![id_generator.gen(), client_id, parity_id, priority, command, timestamp],
    );

    match result {
        Ok(_) => {
            println!("Successfully schedule Command in ClientCommandsTosend");
        }
        Err(e) => {
            eprintln!("An error occurred while scheduling the command in the ClientCommandsTosend table: {}", e);
        }
    }

    BUFFER_POOL.release_connection(conn);

}

pub fn check_if_parity_id_is_registred (parity_id:String) -> bool {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let mut ids:Vec<Result<String, _>> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM ClientCommandsTosend").unwrap();
        let commands_iter = smtp.query_map(params![], |row| { 
            let id:String = row.get(2).unwrap();
            Ok(id)
        }).unwrap();

        for id in commands_iter{
            ids.push(id);
        }

    }

    BUFFER_POOL.release_connection(conn);

    for id in  ids {
        match id {
            Ok(id) => {
                if parity_id == id {
                    return false
                }
            },
            Err (e) => {
                eprintln!("An error occurred while check if parity_id is registred in the ClientCommandsTosend table: {}", e);
            }                                         

        }
        
    }

    return true

} 


pub fn buffer_up_update_schedule (id:i32, client_id:String, parity_id:String, priority:i32, command:String) {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let result = conn.execute(
        "Update ClientCommandsTosend set ClientID = ?, ParityId = ?, Priority = ?, Command = ? where ID = ?",
        params![client_id, parity_id, priority, command, id],
    );

    match result {
        Ok(_) => {
            println!("Successfully update Command in ClientCommandsTosend");
        }
        Err(e) => {
            eprintln!("An error occurred while update the command in the ClientCommandsTosend table: {}", e);
        }
    }

    BUFFER_POOL.release_connection(conn);

}

pub fn buffer_up_clear_old_commands () {

    let now = Utc::now();
    let current_timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

    let schedule = buffer_up_list_schedule();

    if (schedule.is_empty()) {
        return;
    }

    for up_command in schedule {

        let command_timestamp = up_command.created_time;

        let time_difference = (current_timestamp - command_timestamp);

        if time_difference >= 30.0 {

            buffer_up_remove_schedule_by_id(up_command.command_id);
            println!("\nCommand: {} from client: {}, too old, clearing from the buffer up schedule!\n", up_command.parity_id, up_command.client_id);

        }

    }

}

pub fn buffer_up_remove_schedule_by_id (id:i32) {

    let conn = BUFFER_POOL.get_connection().unwrap();
    let result = conn.execute(
        "DELETE from ClientCommandsTosend where ID = ?",
        params![id],
    );

    match result {
        Ok(_) => {
            println!("Successfully removed scheduled Command of id: {} in ClientCommandsTosend", id);
        }
        Err(e) => {
            eprintln!("An error occurred while removing the scheduled the command of id: {} in the ClientCommandsTosend table: {}", id, e);
        }
    }

    BUFFER_POOL.release_connection(conn)

}


pub fn buffer_up_remove_schedule_by_parity_id (client_id:String, parity_id:String) {

    let conn = BUFFER_POOL.get_connection().unwrap();
    let result = conn.execute(
        "DELETE from ClientCommandsTosend where ClientID = ? AND ParityId = ?",
        params![client_id, parity_id],
    );

    match result {
        Ok(_) => {
            println!("Successfully remove schedule Command in ClientCommandsTosend");
        }
        Err(e) => {
            eprintln!("An error occurred while removing scheduled command of parity_id: {} from client: {} in the ClientCommandsTosend table: {}", client_id, parity_id, e);
        }
    }

    BUFFER_POOL.release_connection(conn)

}