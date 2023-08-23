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

use crate::socket_host::client_mananger::mananger::{Client, ClientError};

use rusqlite::Row;
use rusqlite::Statement;
use std::thread;
use std::time::Duration;

lazy_static! {
    static ref BUFFER_NAME: Arc<Mutex<String>> = Arc::new(Mutex::new("buffer.db".to_string()));
    static ref BUFFER_PATH: Arc<Mutex<String>> = Arc::new(Mutex::new("buffer.db".to_string()));
    static ref NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));
    static ref SQL_POOL: Mutex<SQLiteConnectionPool> = Mutex::new(SQLiteConnectionPool::empty());
}

pub fn set_workers_num(n_workers: u32) {
    let mut default_num_of_workers = NUM_WORKERS.lock();

    *default_num_of_workers = n_workers;
}

// > Permission Rules

// allowed_callbacks: Vec<String>,
// allow_create_new_clients: bool,
// allow_create_sub_channels: bool,

// max_sub_channels_allowed: bool,

// allow_redirect: bool,
// allowed_to_redirect_are_blacklist: bool,
// allow_to_redirect: Vec<Client>,

// allow_file_transfer: bool,
// allow_transfer_to_are_blacklist: bool,
// allow_transfer_to: Vec<Client>,

// > Permission Groups
// group_id: u32,
// group_name: String,
// clients_allowed: Vec<Client>,
// permissions: Vec<PermissionRule>,

pub fn groups_mananger_initialize_table(buffer_path: String) {
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

    set_new_path_to_buffer_db!(SQL_POOL, NUM_WORKERS, buffer_path, BUFFER_NAME);

    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        let result = conn.execute("CREATE TABLE IF NOT EXISTS PermissionGroups (ID INT PRIMARY KEY, GroupName TEXT, AllowedCallbacks TEXT, AllowCreateNewClients BOOL, AllowCreateSubChannels BOOL, MaxSubChannelsAllowed BOOL, AllowRedirect BOOL, AllowedToRedirectAreBlacklist BOOL, AllowToRedirect TEXT, AllowFileTransfer BOOL, AllowFileTransferAreBlackList BOOL, AllowTransferTo TEXT)", params![]);

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

// #[derive(Debug, Clone)]
// struct PermissionRule {
//     allowed_callbacks: Vec<String>,
//     allow_create_new_clients: bool,
//     allow_create_sub_channels: bool,

//     max_sub_channels_allowed: bool,

//     allow_redirect: bool,
//     allowed_to_redirect_are_blacklist: bool,
//     allow_to_redirect: Vec<Client>,

//     allow_file_transfer: bool,
//     allow_transfer_to_are_blacklist: bool,
//     allow_transfer_to: Vec<Client>,
// }

// impl PermissionRule {
//     fn new(
//         allowed_callbacks: Vec<String>,
//         allow_create_new_clients: bool,
//         allow_create_sub_channels: bool,
//         max_sub_channels_allowed: bool,
//         allow_redirect: bool,
//         allowed_to_redirect_are_blacklist: bool,
//         allow_to_redirect: Vec<Client>,
//         allow_file_transfer: bool,
//         allow_transfer_to_are_blacklist: bool,
//         allow_transfer_to: Vec<Client>,
//     ) -> Self {
//         Self {
//             allowed_callbacks,
//             allow_create_new_clients,
//             allow_create_sub_channels,

//             max_sub_channels_allowed,

//             allow_redirect,
//             allowed_to_redirect_are_blacklist,
//             allow_to_redirect,

//             allow_file_transfer,
//             allow_transfer_to_are_blacklist,
//             allow_transfer_to,
//         }
//     }
// }

#[derive(Debug, Clone)]
struct PermissionGroup {
    group_id: u32,
    group_name: String,
    clients_allowed: Vec<String>,
    allowed_callbacks: Vec<String>,
    allow_create_new_clients: bool,
    allow_create_sub_channels: bool,
    max_sub_channels_allowed: bool,
    allow_redirect: bool,
    allowed_to_redirect_are_blacklist: bool,
    allow_to_redirect: Vec<String>,
    allow_file_transfer: bool,
    allow_transfer_to_are_blacklist: bool,
    allow_transfer_to: Vec<String>,
}

impl PermissionGroup {
    fn create(
        group_id: u32,
        group_name: String,
        clients_allowed: Vec<String>,
        allowed_callbacks: Vec<String>,
        allow_create_new_clients: bool,
        allow_create_sub_channels: bool,

        max_sub_channels_allowed: bool,

        allow_redirect: bool,
        allowed_to_redirect_are_blacklist: bool,
        allow_to_redirect: Vec<String>,

        allow_file_transfer: bool,
        allow_transfer_to_are_blacklist: bool,
        allow_transfer_to: Vec<String>,
    ) -> Self {
        let group = Self {
            group_id,
            group_name,
            clients_allowed,
            allowed_callbacks,
            allow_create_new_clients,
            allow_create_sub_channels,

            max_sub_channels_allowed,

            allow_redirect,
            allowed_to_redirect_are_blacklist,
            allow_to_redirect,

            allow_file_transfer,
            allow_transfer_to_are_blacklist,
            allow_transfer_to,
        };
        group
    }
}

pub fn check_if_permission_group_key_exists(client_key: String) -> bool {
    let client_keys: Vec<String> = get_permission_group_keys_registred();

    if client_keys.contains(&client_key) {
        return true;
    } else {
        return false;
    }
}

fn get_permission_group_keys_registred() -> Vec<String> {
    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        let mut keys: Vec<String> = Vec::new();
        {
            let mut smtp: Statement<'_> = conn.prepare("SELECT * FROM PermissionGroups").unwrap();
            let commands_iter = smtp
                .query_map(params![], |row: &Row<'_>| {
                    let key: String = row.get(1)?;
                    Ok(key)
                })
                .unwrap();

            for command in commands_iter {
                keys.push(command.unwrap());
            }
        }

        keys
    })
}

fn get_registred_ids() -> Vec<u32> {
    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        let mut ids: Vec<u32> = Vec::new();

        {
            let mut smtp = conn.prepare("SELECT * FROM PermissionGroups").unwrap();
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

        ids
    })
}

pub fn registry_permission_group(
    group_id: u32,
    group_name: String,
    clients_allowed: Vec<String>,
    allowed_callbacks: Vec<String>,
    allow_create_new_clients: bool,
    allow_create_sub_channels: bool,

    max_sub_channels_allowed: bool,

    allow_redirect: bool,
    allowed_to_redirect_are_blacklist: bool,
    allow_redirect_to: Vec<String>,

    allow_file_transfer: bool,
    allow_transfer_to_are_blacklist: bool,
    allow_transfer_to: Vec<String>,
) {
    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        // let now = Utc::now();
        // let timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

        let registered_ids = get_registred_ids();

        let mut id_generator = UniqueIdGenerator { registered_ids: registered_ids };

        let result = conn.execute(
            "INSERT INTO PermissionGroups (ID, GroupName, AllowFileTransfer, MaxSubChannelsPerClient, FunctionsAllowedAreBlackList, FunctionsAllowed, FileTransferFunctionsAllowedAreBlackList, FileTransferFunctionsAllowed, AllowRedirectAreBlackList, AllowRedirectTo) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
            params![
                id_generator.gen(),
                group_name,
                serde_json::to_string(&clients_allowed).unwrap(),
                serde_json::to_string(&allowed_callbacks).unwrap(),
                allow_create_new_clients,
                allow_create_sub_channels,

                max_sub_channels_allowed,

                allow_redirect,
                allowed_to_redirect_are_blacklist,
                serde_json::to_string(&allow_redirect_to).unwrap(),

                allow_file_transfer,
                allow_transfer_to_are_blacklist,
                serde_json::to_string(&allow_transfer_to).unwrap(),
            ],
        );

        match result {
            Ok(rows) => {
                if rows > 0 {
                    println!("Successfully inserted Log in the table Clients. {} row(s) were affected.", rows);
                } else {
                    println!("No rows were affected.");
                }
            },
            Err(e) => {
                eprintln!("An error occurred while inserting the Log in the table Clients: {}", e);
            },
        };
    })
}

fn get_permission_group_by_key(client_key: String) -> Result<Client, ClientError> {
    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        let mut clients: Vec<Client> = Vec::new();

        {
            let mut smtp = conn.prepare("SELECT * FROM Clients WHERE ClientKey = ?").unwrap();

            let clients_iter = smtp
                .query_map(params![client_key], |row| {
                    Ok(Client::from(
                        row.get(0).unwrap(),
                        row.get(1).unwrap(),
                        row.get(2).unwrap(),
                        row.get(3).unwrap(),
                        row.get(4).unwrap(),
                        row.get(5).unwrap(),
                        row.get(6).unwrap(),
                        serde_json::from_str::<Vec<String>>(row.get::<_, String>(7)?.as_str()).unwrap(),
                        row.get(8).unwrap(),
                    ))
                })
                .unwrap();

            for client in clients_iter {
                clients.push(client.unwrap());
            }
        }

        if clients.len() == 0 {
            return Err(ClientError::ClientDoesNotExist(client_key));
        } else {
            return Ok(clients[0].clone());
        }
    })
}

fn get_permission_group_by_name(client_name: String) -> Result<Client, ClientError> {
    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        let mut clients: Vec<Client> = Vec::new();

        {
            let mut smtp = conn.prepare("SELECT * FROM Clients WHERE ClientName = ?").unwrap();

            let clients_iter = smtp
                .query_map(params![client_name], |row| {
                    Ok(Client::from(
                        row.get(0).unwrap(),
                        row.get(1).unwrap(),
                        row.get(2).unwrap(),
                        row.get(3).unwrap(),
                        row.get(4).unwrap(),
                        row.get(5).unwrap(),
                        row.get(6).unwrap(),
                        serde_json::from_str::<Vec<String>>(row.get::<_, String>(7)?.as_str()).unwrap(),
                        row.get(8).unwrap(),
                    ))
                })
                .unwrap();

            for client in clients_iter {
                clients.push(client.unwrap());
            }
        }

        if clients.len() == 0 {
            return Err(ClientError::ClientDoesNotExist(client_name));
        } else {
            return Ok(clients[0].clone());
        }
    })
}

pub fn edit_permission_group(client: Client) {
    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        let serialized_owned_sub_channels_keys = serde_json::to_string(&client.owned_sub_channels_keys).expect("Failed to serialize to JSON");

        let result = conn.execute(
            "UPDATE Clients SET ClientName = ?, ClientKey = ?, PermissionGroup = ?, SuperUser = ?, LastContact = ?, MaxSubChannels = ?, OwnedSubChannelsKeys = ?, SubChannelsInUse = ? WHERE ID = ?;",
            params![
                client.client_name,
                client.client_key,
                client.permission_group,
                client.super_user,
                client.last_contact,
                client.max_sub_channels,
                serialized_owned_sub_channels_keys,
                client.sub_channels_in_use,
                client.client_id,
            ],
        );

        match result {
            Ok(rows) => {
                if rows > 0 {
                    println!("Successfully update client: {} in databse", client.client_name);
                }
            },
            Err(e) => {
                eprintln!("Error while update client: {} in the databse, the error is: {}", client.client_name, e);
            },
        }
    });
}

fn remove_permission_group(client: Client) {
    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        let result = conn.execute("DELETE from Clients WHERE ClientKey = ?", params![client.client_key]);

        match result {
            Ok(rows) => {
                println!("Successfully deleted Client: {} from clients! {} Rows were affected.", client.client_key, rows);
            },
            Err(e) => {
                eprintln!("An error occurred while deleting Client: {} from clients! And the error was: {}", client.client_key, e);
            },
        }
    });
}
