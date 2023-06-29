// use std::sync::Mutex;
// use std::collections::HashMap;

use lazy_static::lazy_static;

use pyo3::prelude::*;
use pyo3::types::PyDict;

use std::sync::{Arc, Mutex};

// use pyo3::wrap_pyfunction;
// use pyo3::types::IntoPyDict;

use super::buffer_functions;

use buffer_functions::UniqueIdGenerator;
use buffer_functions::SQLiteConnectionPool;

// use chrono::NaiveDateTime;
// use std::result::Result;

use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::params;

// mod buffer_functions;
// use buffer_functions::UniqueIdGenerator;
// use buffer_functions::SQLiteConnectionPool;

pub struct ClientCommand {
    pub client_id:i32,
    pub client_name:String,
    pub client_key:String,
    pub client_group:String,
    pub client_las_contact: f64,
}

impl IntoPy <PyObject> for ClientCommand {
    fn into_py (self, py:Python) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item("client_id", self.client_id).unwrap();
        dict.set_item("client_name", self.client_name).unwrap();
        dict.set_item("client_key", self.client_key).unwrap();
        dict.set_item("client_group", self.client_group).unwrap();
        dict.set_item("client_las_contact", self.client_las_contact).unwrap();
        dict.into()
    }
}


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

 pub fn client_buffer_initialize_table (buffer_path:String) {

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
        "CREATE TABLE IF NOT EXISTS Clients (ID INT PRIMARY KEY, NAME TEXT, KEY TEXT, \"GROUP\" TEXT, LastContact DATETIME)",
        params![],
    );
    
    match result {
        Ok(_) => {
            println!("Successfully initialize Clients table!");
        }
        Err(e) => {
            eprintln!("An error occurred while scheduling the command in the Clients table: {}", e);
        }
    }

    buffer_pool.release_connection(conn);

    return;

}



 fn get_registred_ids () -> Vec<i32> {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let mut ids:Vec<i32> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM Clients").unwrap();
        let ids_iter = smtp.query_map(params![], |row| {
            let id:i32 = row.get(0).unwrap();
            Ok(id)
        }).unwrap();

        for id in ids_iter {
            ids.push(id.unwrap());
        }

    }

    BUFFER_POOL.release_connection(conn);

    return ids;
}


pub fn client_buffer_list_clients () -> Vec<ClientCommand> {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let mut data:Vec<ClientCommand> = Vec::new();

    {
        let mut smtp = conn.prepare("SELECT * FROM Clients").unwrap();

        let commands_iter = smtp.query_map(params![], |row| {
            Ok(ClientCommand { 
                client_id: row.get(0)?, 
                client_name: row.get(1)?, 
                client_key: row.get(2)?,
                client_group:  row.get(3)?,
                client_las_contact: row.get(4)?
            })
        }).unwrap();

        for command in commands_iter {
            data.push(command.unwrap())
        }

    }

    BUFFER_POOL.release_connection(conn);

    return data;

}


pub fn client_buffer_registry_new_client (name:String, key:String, group:String) {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let registered_ids = get_registred_ids();
    let mut id_generator = UniqueIdGenerator{registered_ids:registered_ids};

    let system_time = SystemTime::now();
    let since_the_epoch = system_time.duration_since(UNIX_EPOCH).expect("Time went backwards");

    let last_contact = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

    let result = conn.execute(
        "INSERT INTO Clients (ID, NAME, KEY, GROUP, LastContact) VALUES (?, ?, ?, ?, ?);",
        params![id_generator.gen(), name, key, group, last_contact],
    );

    match result {
        Ok(_) => {
            println!("Successfully registred a new client in Clients table!");
        }
        Err(e) => {
            eprintln!("An error occurred while registring a new client: {}", e);
        }
    }

    BUFFER_POOL.release_connection(conn);

}


pub fn client_buffer_update_client (client_id:i32, client_name:String, client_key:String, client_group:String) {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let result = conn.execute(
        "Update Clients set NAME = ?, KEY = ?, TYPE = ?, LastContact = ? WHERE ID = ?",
        params![client_name, client_key, client_group, client_id],
    );

    match result {
        Ok(_) => {
            println!("Successfully update client in Clients table!");
        }
        Err(e) => {
            eprintln!("An error occurred while update client in Clients table: {}", e);
        }
    }

    BUFFER_POOL.release_connection(conn);

}


pub fn client_buffer_update_client_ts (client_key:String) {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let system_time = SystemTime::now();
    let since_the_epoch = system_time.duration_since(UNIX_EPOCH).expect("Time went backwards");

    let last_contact = since_the_epoch.as_secs() * 1000 + since_the_epoch.subsec_nanos() as u64 / 1_000_000;

    let result = conn.execute(
        "Update Clients set LastContact = ? WHERE KEY = ?",
        params![last_contact, client_key],
    );

    match result {
        Ok(_) => {
            println!("Successfully update client alive ts  signal in Clients table!");
        }
        Err(e) => {
            eprintln!("An error occurred while update client alive ts signal in Clients table: {}", e);
        }
    }

    BUFFER_POOL.release_connection(conn);

}


pub fn client_buffer_remove_client_by_id (client_id:i32) {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let result = conn.execute(
        "DELETE from Clients WHERE ID = ?",
        params![client_id],
    );

    match result {
        Ok(_) => {
            println!("Successfully remove client, by client id: {} of Clients table!", client_id);
        }
        Err(e) => {
            eprintln!("An error occurred while removing client, by client id: {} of Clients table: {}", client_id, e);
        }
    }

    BUFFER_POOL.release_connection(conn); 

}


pub fn client_buffer_remove_client_by_key (client_key:String) {

    let conn = BUFFER_POOL.get_connection().unwrap();

    let result = conn.execute(
        "DELETE from Clients where KEY = ?",
        params![client_key],
    );

    match result {
        Ok(_) => {
            println!("Successfully remove client: {} of Clients table!", client_key);
        }
        Err(e) => {
            eprintln!("An error occurred while removing client: {} of Clients table: {}", client_key, e);
        }
    }

    BUFFER_POOL.release_connection(conn); 

}