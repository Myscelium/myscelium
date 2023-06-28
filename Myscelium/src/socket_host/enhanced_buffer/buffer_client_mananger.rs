// use std::sync::Mutex;
// use std::collections::HashMap;

use lazy_static::lazy_static;

use pyo3::prelude::*;
use pyo3::types::PyDict;

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
    client_id:i32,
    client_name:String,
    client_key:String,
    client_group:String,
    client_las_contact: f64,
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
    static ref BUFFER_POOL: SQLiteConnectionPool = SQLiteConnectionPool::new(10, "data.db").unwrap();
}

/*
    However, the rusqlite library in Rust automatically starts a new 
    transaction before each command and commits it after the command 
    is executed, unless you explicitly start a transaction. This is 
    known as "autocommit mode".
    
 */

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