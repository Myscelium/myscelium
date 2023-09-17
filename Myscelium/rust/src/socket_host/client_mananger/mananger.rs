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

use rusqlite::Row;
use rusqlite::Statement;

use std::time::SystemTime;
use std::time::UNIX_EPOCH;

#[macro_export]
macro_rules! handle_client_error {
    ($client_result:expr) => {
        match $client_result {
            Ok(c) => c, // Return the unwrapped client directly
            Err(e) => {
                match e {
                    ClientError::ClientAlwreadyExist(c) => {
                        println!("Error client: {} already exist", c);
                    },
                    ClientError::ClientDoesNotExist(c) => {
                        println!("Error client: {} doesn't exist", c);
                    },
                    ClientError::UnexpectedError(e) => {
                        println!("Get a unexpected error: {}", e);
                    },
                    _ => {
                        println!("Get a unexpected error!");
                    },
                }
                panic!("Client error encountered!"); // Panic after printing the error
            },
        }
    };
}

lazy_static! {
    static ref SQL_NAME: Arc<Mutex<String>> = Arc::new(Mutex::new("Data.db".to_string()));
    static ref SQL_PATH: Arc<Mutex<String>> = Arc::new(Mutex::new("Data.db".to_string()));
    static ref NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));
    static ref SQL_POOL: Mutex<SQLiteConnectionPool> = Mutex::new(SQLiteConnectionPool::empty());
}

pub fn set_host_clients_mananger__pool_workers_num(n_workers: u32) {
    let mut default_num_of_workers = NUM_WORKERS.lock();

    *default_num_of_workers = n_workers;
}

pub fn clients_mananger_initialize_table(sql_path: String) {
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

    set_new_path_to_buffer_db!(SQL_POOL, NUM_WORKERS, sql_path, SQL_NAME);

    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        let result = conn.execute(
            "CREATE TABLE IF NOT EXISTS Clients (ID INT PRIMARY KEY, ClientName TEXT, ClientKey TEXT, ClientType TEXT, PermissionGroup TEXT, SuperUser BOOL, LastContact NUMBER, MaxSubChannels NUMBER, OwnedSubChannelsKeys TEXT, SubChannelsInUse NUMBER)",
            params![],
        );

        match result {
            Ok(_) => {
                println!("Successfully initialize Clients table!");
            },
            Err(e) => {
                eprintln!("An error occurred while scheduling the command in the Clients table: {}", e);
            },
        };
    });
}

#[derive(Debug, Clone)]
pub enum ClientError {
    ClientDoesNotExist(String),
    ClientAlwreadyExist(String),
    UnexpectedError(String),
}

#[derive(Debug, Clone)]
pub struct Client {
    pub client_id: u32,
    client_name: String,
    client_key: String,
    client_type: String,
    permission_group: String,
    is_super_user: bool,
    last_contact: f64,
    max_sub_channels: u32,
    owned_sub_channels_keys: Vec<String>,
    sub_channels_in_use: u32,
}

// > Get client by key
// > Get client by name
// > edit client
// > change key
// > new client
// > client from
// > delet client

impl Client {
    pub fn new(client_name: String, client_key: String, client_type: String, permission_group: String, is_super_user: bool, max_sub_channels: u32, owned_sub_channels_keys: Vec<String>) -> Result<Self, ClientError> {
        let mut client_id;

        {
            let registered_ids = get_registred_ids();

            let mut id_generator = UniqueIdGenerator { registered_ids: registered_ids };

            client_id = id_generator.gen();
        }

        Ok(Self {
            client_id,
            client_name,
            client_key,
            client_type,
            permission_group,
            is_super_user,
            last_contact: -1.0,
            max_sub_channels,
            owned_sub_channels_keys,
            sub_channels_in_use: 0u32,
        })
    }

    pub fn save_into_db(&self) {
        let serialzied_owned_sub_channels_keys = serde_json::to_string(&self.owned_sub_channels_keys).expect("Failed to serialize to JSON");

        with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
            // let now = Utc::now();
            // let timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

            let result = conn.execute(
                "INSERT INTO Clients (ID, ClientName, ClientKey, ClientType, PermissionGroup, SuperUser, LastContact, MaxSubChannels, OwnedSubChannelsKeys, SubChannelsInUse) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);",
                params![
                    self.client_id,
                    self.client_name,
                    self.client_key,
                    self.client_type,
                    self.permission_group,
                    self.is_super_user,
                    self.last_contact,
                    self.max_sub_channels,
                    serialzied_owned_sub_channels_keys,
                    self.sub_channels_in_use
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
        });
    }

    pub fn update_to(&self, new_client: &Client) -> Result<Self, ClientError> {
        let serialzied_owned_sub_channels_keys = serde_json::to_string(&new_client.owned_sub_channels_keys).expect("Failed to serialize to JSON");

        with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
            // let now = Utc::now();
            // let timestamp = now.timestamp() as f64 + (now.timestamp_subsec_millis() as f64 / 1000.0);

            let result = conn.execute(
                "UPDATE Clients SET ClientName = ?, ClientKey = ?, ClientType = ?, PermissionGroup = ?, SuperUser = ?, LastContact = ?, MaxSubChannels = ?, OwnedSubChannelsKeys = ?, SubChannelsInUse = ? WHERE ID = ?",
                params![
                    new_client.client_name,
                    new_client.client_key,
                    new_client.client_type,
                    new_client.permission_group,
                    new_client.is_super_user,
                    new_client.last_contact,
                    new_client.max_sub_channels,
                    serialzied_owned_sub_channels_keys,
                    new_client.sub_channels_in_use,
                    new_client.client_id,
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
        });

        Ok(Self {
            client_id: new_client.client_id,
            client_name: new_client.client_name.clone(),
            client_key: new_client.client_key.clone(),
            client_type: new_client.client_type.clone(),
            permission_group: new_client.permission_group.clone(),
            is_super_user: new_client.is_super_user,
            last_contact: new_client.last_contact,
            max_sub_channels: new_client.max_sub_channels,
            owned_sub_channels_keys: new_client.owned_sub_channels_keys.clone(),
            sub_channels_in_use: new_client.sub_channels_in_use,
        })
    }

    pub fn get_by_name(client_name: &String) -> Result<Self, ClientError> {
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
                            row.get(7).unwrap(),
                            serde_json::from_str::<Vec<String>>(row.get::<_, String>(8)?.as_str()).unwrap(),
                            row.get(9).unwrap(),
                        ))
                    })
                    .unwrap();

                for client in clients_iter {
                    clients.push(client.unwrap()?);
                }
            }

            if clients.len() == 0 {
                return Err(ClientError::ClientDoesNotExist(client_name.clone()));
            } else {
                return Ok(clients[0].clone());
            }
        })
    }

    pub fn get_by_key(client_key: &String) -> Result<Self, ClientError> {
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
                            row.get(7).unwrap(),
                            serde_json::from_str::<Vec<String>>(row.get::<_, String>(8)?.as_str()).unwrap(),
                            row.get(9).unwrap(),
                        ))
                    })
                    .unwrap();

                for client in clients_iter {
                    clients.push(client.unwrap()?);
                }
            }

            if clients.len() == 0 {
                return Err(ClientError::ClientDoesNotExist(client_key.clone()));
            } else {
                return Ok(clients[0].clone());
            }
        })
    }

    pub fn delete_all() -> Result<(), ClientError> {
        with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
            let result = conn.execute("DELETE FROM Clients", params![]);

            match result {
                Ok(rows) => {
                    println!("Successfully deleted all clients from clients table! {} Rows were affected.", rows);
                },
                Err(e) => {
                    eprintln!("An error occurred while deleting all clients from clients table! And the error was: {}", e);
                },
            }
        });

        Ok(())
    }

    pub fn delete(&self) -> Result<(), ClientError> {
        if !check_if_client_key_exists(self.client_key.clone()) {
            return Err(ClientError::ClientDoesNotExist(self.client_key.clone()));
        }

        with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
            let result = conn.execute("DELETE from Clients WHERE ClientKey = ?", params![self.client_key]);

            match result {
                Ok(rows) => {
                    println!("Successfully deleted Client: {} from clients! {} Rows were affected.", self.client_key, rows);
                },
                Err(e) => {
                    eprintln!("An error occurred while deleting Client: {} from clients! And the error was: {}", self.client_key, e);
                },
            }
        });

        Ok(())
    }

    pub fn update_last_contact(&self) -> Result<Self, ClientError> {
        let now = SystemTime::now();
        let duration_since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");

        let seconds = duration_since_epoch.as_secs() as f64;
        let subsec_nanos = duration_since_epoch.subsec_nanos() as f64;

        let total_seconds = seconds + subsec_nanos * 1e-9; // Convert nanoseconds to seconds and add to the total

        let last_contact = total_seconds.clone();

        let new_client = Self {
            client_id: self.client_id.clone(),
            client_name: self.client_name.clone(),
            client_key: self.client_key.clone(),
            client_type: self.client_type.clone(),
            permission_group: self.permission_group.clone(),
            is_super_user: self.is_super_user.clone(),
            last_contact: last_contact,
            max_sub_channels: self.max_sub_channels.clone(),
            owned_sub_channels_keys: self.owned_sub_channels_keys.clone(),
            sub_channels_in_use: self.sub_channels_in_use.clone(),
        };

        edit_client(new_client.clone());

        Ok(new_client)
    }

    pub fn change_key_to(&self, new_client_key: String) -> Result<Self, ClientError> {
        if !check_if_client_key_exists(self.client_key.clone()) {
            return Err(ClientError::ClientDoesNotExist(self.client_key.clone()));
        }

        let new_client = Self {
            client_id: self.client_id,
            client_name: self.client_name.clone(),
            client_key: new_client_key,
            client_type: self.client_type.clone(),
            permission_group: self.permission_group.clone(),
            is_super_user: self.is_super_user.clone(),
            last_contact: self.last_contact,
            max_sub_channels: self.max_sub_channels,
            owned_sub_channels_keys: self.owned_sub_channels_keys.clone(),
            sub_channels_in_use: self.sub_channels_in_use,
        };

        edit_client(new_client.clone());

        Ok(new_client)
    }

    fn from(
        client_id: u32,
        client_name: String,
        client_key: String,
        client_type: String,
        permission_group: String,
        is_super_user: bool,
        last_contact: f64,
        max_sub_channels: u32,
        owned_sub_channels_keys: Vec<String>,
        sub_channels_in_use: u32,
    ) -> Result<Self, ClientError> {
        Ok(Self {
            client_id,
            client_name,
            client_key,
            client_type,
            permission_group,
            is_super_user,
            last_contact,
            max_sub_channels,
            owned_sub_channels_keys,
            sub_channels_in_use,
        })
    }
}

pub fn check_if_client_key_exists(client_key: String) -> bool {
    let client_keys: Vec<String> = get_clients_keys_registred();

    if client_keys.contains(&client_key) {
        return true;
    } else {
        return false;
    }
}

fn get_clients_keys_registred() -> Vec<String> {
    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        let mut keys: Vec<String> = Vec::new();

        {
            let mut smtp: Statement<'_> = conn.prepare("SELECT * FROM Clients").unwrap();
            let keys_iter = smtp
                .query_map(params![], |row: &Row<'_>| {
                    let key: String = row.get(2)?;
                    Ok(key)
                })
                .unwrap();

            for key in keys_iter {
                keys.push(key.unwrap());
            }
        }

        // println!("Client Keys registred: {:?}", keys.clone());

        keys
    })
}

fn get_registred_ids() -> Vec<u32> {
    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        let mut ids: Vec<u32> = Vec::new();

        {
            let mut smtp = conn.prepare("SELECT * FROM Clients").unwrap();
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

pub fn edit_client(client: Client) {
    with_connection!(SQL_POOL, |conn: &rusqlite::Connection| {
        let serialized_owned_sub_channels_keys = serde_json::to_string(&client.owned_sub_channels_keys).expect("Failed to serialize to JSON");

        let result = conn.execute(
            "UPDATE Clients SET ClientName = ?, ClientKey = ?, ClientType = ?, PermissionGroup = ?, SuperUser = ?, LastContact = ?, MaxSubChannels = ?, OwnedSubChannelsKeys = ?, SubChannelsInUse = ? WHERE ID = ?;",
            params![
                client.client_name,
                client.client_key,
                client.client_type,
                client.permission_group,
                client.is_super_user,
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
