use lazy_static::lazy_static;

#[macro_use]
use crate::{with_connection, set_new_path_to_buffer_db};
use crate::commom::sql_pool::pool::{SQLiteConnectionPool, UniqueIdGenerator, UniqueParityIdGenerator};

use rusqlite::params;

use serde::{Deserialize, Serialize};

use pyo3::prelude::*;
use pyo3::types::PyDict;

use std::clone;
use std::sync::Arc;

use parking_lot::Mutex;

use serde_json::{from_str, Value};
use std::collections::HashMap;

use chrono::Utc;

use crate::commom::enhanced_buffer::utilities::Command;

use std::sync::RwLock;

use rusqlite::{Connection, Result};

use std::thread;
use std::time::Duration;

lazy_static! {
    static ref BUFFER_NAME: Arc<Mutex<String>> = Arc::new(Mutex::new("buffer.db".to_string()));
    static ref BUFFER_PATH: Arc<Mutex<String>> = Arc::new(Mutex::new("buffer.db".to_string()));
    static ref NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));
    static ref BUFFER_POOL: Mutex<SQLiteConnectionPool> = Mutex::new(SQLiteConnectionPool::empty());
}

pub fn set_workers_num(n_workers: u32) {
    let mut default_num_of_workers = NUM_WORKERS.lock();

    *default_num_of_workers = n_workers;
}

pub fn client_channel_mananger_initialize_table(buffer_path: String) {
    // Create a global Mutex for demonstration
    let mutex1 = Mutex::new(0);
    let mutex2 = Mutex::new(0);

    // Spawn a thread to periodically check for deadlocks
    thread::spawn(|| {
        loop {
            thread::sleep(Duration::from_secs(5)); // Check every 5 seconds
            let deadlocks = parking_lot::deadlock::check_deadlock();
            if deadlocks.is_empty() {
                continue;
            }

            println!("{} deadlocks detected", deadlocks.len());
            for (i, threads) in deadlocks.iter().enumerate() {
                println!("Deadlock #{}", i);
                for t in threads {
                    println!("Thread Id {:?}", t.thread_id());
                    println!("{:?}", t.backtrace());
                }
            }
        }
    });

    set_new_path_to_buffer_db!(BUFFER_POOL, NUM_WORKERS, buffer_path, BUFFER_NAME);

    with_connection!(BUFFER_POOL, |conn: &rusqlite::Connection| {
        let result = conn.execute(
            "CREATE TABLE IF NOT EXISTS UserGroups (ID INT PRIMARY KEY, GroupName TEXT, AllowFileTransfer BOOL, MaxSubChannelsPerClient NUMBER, FunctionsAllowedAreBlackList BOOL, FunctionsAllowed TEXT,  FileTransferFunctionsAllowedAreBlackList BOOL, FileTransferFunctionsAllowed TEXT, AllowRedirectAreBlackList BOOL, AllowRedirectTo TEXT)",
            params![],
        );

        match result {
            Ok(_) => {
                println!("Successfully initialize ClientCommandsTosend table!");
            },
            Err(e) => {
                eprintln!("An error occurred while scheduling the command in the ClientCommandsTosend table: {}", e);
            },
        };
    });
}

#[derive(Debug, Clone)]
struct PermissionRule {
    allowed_callbacks: Vec<String>,
    allow_create_new_clients: bool,
    allow_create_sub_channels: bool,

    max_sub_channels_allowed: bool,

    allow_redirect: bool,
    allowed_to_redirect_are_blacklist: bool,
    allow_to_redirect: Vec<Client>,

    allow_file_transfer: bool,
    allow_transfer_to_are_blacklist: bool,
    allow_transfer_to: Vec<Client>,
}

#[derive(Debug, Clone)]
struct PermissionGroup {
    group_id: u32,
    group_name: String,
    clients_allowed: Vec<Client>,
    permissions: Vec<PermissionRule>,
}
