use std::{sync::Arc, thread, time::Duration};

use chrono::Utc;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use rusqlite::params;

use crate::{
    common::structs::available_commands::{NetworkMap, Node},
    set_new_path_to_buffer_db, with_connection,
};

use crate::common::sql_pool::pool::{SQLiteConnectionPool, UniqueIdGenerator};

lazy_static! {
    static ref CLIENT_STATE_MANAGER: Arc<Mutex<ClientState>> = Arc::new(Mutex::new(ClientState::empty()));
    static ref STATES_BUFFER_NAME: Arc<Mutex<String>> = Arc::new(Mutex::new("buffer.db".to_string()));
    static ref STATES_BUFFER_PATH: Arc<Mutex<String>> = Arc::new(Mutex::new("buffer.db".to_string()));
    static ref STATES_NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));
    static ref STATES_BUFFER_POOL: Mutex<SQLiteConnectionPool> = Mutex::new(SQLiteConnectionPool::empty());
}

pub struct ClientState {
    name: Option<String>,
    key: Option<String>,
    network_map: Option<NetworkMap>,
    client_node_configs: Option<Node>,
    is_initialized: Option<bool>,
    is_ready: Option<bool>,
    is_connected: Option<bool>,
    is_sync: Option<bool>,
    last_change: Option<f64>,
}

pub fn initialize_client_status_table_table(status_db_spath: String) {
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

    set_new_path_to_buffer_db!(STATES_BUFFER_POOL, STATES_NUM_WORKERS, status_db_spath, STATES_BUFFER_NAME);

    with_connection!(STATES_BUFFER_POOL, |conn: &rusqlite::Connection| {
        let result = conn.execute(
            "CREATE TABLE IF NOT EXISTS ClientStatusControler (ID INT PRIMARY KEY, Name TEXT, Key TEXT, NetMap TEXT, ClientNodeConfigs TEXT, IsInitialized BOOL, IsReady BOOL, IsConnected BOOL, IsSync BOOL, LastChange NUMBER)",
            params![],
        );

        match result {
            Ok(_) => {
                println!("Successfully initialize ClientCommandsReceived table!");
            },
            Err(e) => {
                eprintln!("An error occurred while scheduling the command in the ClientCommandsReceived table: {}", e);
            },
        };
    });
}

impl ClientState {
    pub fn new(name: String, key: String, network_map: NetworkMap, client_node_configs: Node, is_initialized: bool, is_ready: bool, is_connected: bool, is_sync: bool, last_change: f64) -> Self {
        Self {
            name: Some(name),
            key: Some(key),
            network_map: Some(network_map),
            client_node_configs: Some(client_node_configs),
            is_initialized: Some(is_initialized),
            is_ready: Some(is_ready),
            is_connected: Some(is_connected),
            is_sync: Some(is_sync),
            last_change: Some(last_change),
        }
    }

    pub fn empty() -> Self {
        Self {
            name: None,
            key: None,
            network_map: None,
            client_node_configs: None,
            is_initialized: None,
            is_ready: None,
            is_connected: None,
            is_sync: None,
            last_change: None,
        }
    }

    pub fn save_in_storage(&self) {
        // TODO >>> Finish this method;

        with_connection!(STATES_BUFFER_POOL, |conn: &rusqlite::Connection| {
            //let registered_ids = get_registred_ids(conn);
            // let mut id_generator = UniqueIdGenerator { registered_ids: registered_ids };
            // This on top isn't necessary since here will only have one client per per db in each
            // client states table.

            let now = Utc::now();
            let timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

            let result = conn.execute(
                "INSERT INTO ClientCommandsTosend (ID, Name, Key, NetMap, ClientNodeConfigs, IsInitialized, IsReady, IsConnected, IsSync, LastChange) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
                params![0], // TODO >>> Add the remaining commands that need to be impl here
            );

            match result {
                Ok(_) => {
                    println!("Successfully schedule Command in ClientCommandsTosend");
                },
                Err(e) => {
                    eprintln!("An error occurred while scheduling the command in the ClientCommandsTosend table: {}", e);
                },
            };
        });
    }

    pub fn load_from_storage(&self) -> Self {
        Self {
            name: None,
            key: None,
            network_map: None,
            client_node_configs: None,
            is_initialized: None,
            is_ready: None,
            is_connected: None,
            is_sync: None,
            last_change: None,
        }
    }
}
