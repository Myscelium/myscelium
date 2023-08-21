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
            "CREATE TABLE IF NOT EXISTS ClientCommandsTosend (ID INT PRIMARY KEY, OwnerClientKey TEXT, ChanelName TEXT, ChannelPurpose TEXT, Status TEXT, ChannelLifetime NUMBER, LastContact NUMBER, Streaming BOOL)",
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
enum ChannelStatus {
    Waithing,
    Streaming,
    Seleeping,
    Dead,
}

#[derive(Debug, Clone)]
enum ChannelError {
    ChannelDoesNotExists,
    ChannelAlwreadyStreaming,
    IncompatiblePurpose,
}

#[derive(Debug, Clone)]
enum ChannelPurpose {
    BinaryTransfer,
    BinarySignalStream,
}

#[derive(Debug, Clone)]
struct Channel {
    channel_id: Option<u32>,
    owner_key: String,
    channel_name: String,
    channel_purpose: ChannelPurpose,
    channel_status: ChannelStatus,
    channel_lifetime: String,
    last_contact: f64,
    is_streaming: bool,
}

impl Channel {
    fn new(channel_id: u32, owner_key: String, channel_name: String, channel_purpose: String, channel_lifetime: String, last_contact: f64, is_streaming: bool) {}
    fn from() {}
    fn is_streaming() {}
    fn get_last_contact() {}
    fn get_lifetime() {}
    fn get_channel_by_id(channel_id: u32) {}
}
