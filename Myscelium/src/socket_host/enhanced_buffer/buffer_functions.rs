extern crate rand;
use rand::Rng;
use rand::distributions::Alphanumeric;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;

use lazy_static::lazy_static;

use rusqlite::{Connection, Result};
use std::result::Result::Ok;
use std::result::Result::Err;


// Similarly `mod inaccessible` and `mod nested` will locate the `nested.rs`
// and `inaccessible.rs` files and insert them here under their respective
// modules

lazy_static! {
    static ref ID_LENGTH:Mutex<i32> = Mutex::new(9999);
}

/*
    However, the rusqlite library in Rust automatically starts a new 
    transaction before each command and commits it after the command 
    is executed, unless you explicitly start a transaction. This is 
    known as "autocommit mode".
    
 */


 pub struct UniqueParityIdGenerator {
    length: usize,
    registered_ids: Vec<String>,
}

impl UniqueParityIdGenerator {
    pub fn new(length: usize, registered_ids: Vec<String>) -> Self {
        Self {
            length,
            registered_ids,
        }
    }

    pub fn update_registered_parity_ids (&mut self, registered_ids: Vec<String>) {
        self.registered_ids = registered_ids;
    }

    pub fn gen (&mut self) -> String {
        loop {
            let buffer_id = self.random_string();
            if self.validate(&buffer_id) {
                return buffer_id;
            }
        }
    }

    fn random_string (&self) -> String {
        let rng = rand::thread_rng();
        let id: String = rng.sample_iter(&Alphanumeric)
            .take(self.length)
            .map(char::from)
            .collect();
        id
    }

    fn validate (&self, buffer_id: &String) -> bool {
        !self.registered_ids.contains(buffer_id)
    }
}


pub struct UniqueIdGenerator {
    pub registered_ids: Vec<i32>
}

impl UniqueIdGenerator {

    pub fn _new(registered_ids: Vec<i32>) -> Self {
        Self {
            registered_ids
        }
    }

    pub fn _update_registered_ids (&mut self, registered_ids:Vec<i32>) {
        self.registered_ids = registered_ids;
    }

    pub fn gen(&mut self) -> i32 {

        loop {
            let buffer_id = self.gen_buffer_id();
            if self.validate (buffer_id) {
                return buffer_id;
            }
        }
    
    }

    fn gen_buffer_id (&self) -> i32 {

        let length =  ID_LENGTH.lock().unwrap();
        let mut rng = rand::thread_rng();
        rng.gen_range(0..*length)
    
    }

    fn validate(&self, buffer_id:i32) -> bool {
        !self.registered_ids.contains (&buffer_id)
    }

}

// -> Sql custom pool:

pub struct SQLiteConnectionPool {
    connections: Arc<Mutex<mpsc::Receiver<Connection>>>,
    sender: Arc<Mutex<mpsc::Sender<Connection>>>,
}

impl SQLiteConnectionPool {
    pub fn new(max_connections: usize, db: &str) -> Result<Self> {
        let (tx, rx) = mpsc::channel();
        for _ in 0..max_connections {
            let connection = Connection::open(db)?;
            tx.send(connection).unwrap();
        }
        Ok ( Self {
            connections: Arc::new(Mutex::new(rx)),
            sender: Arc::new(Mutex::new(tx)),
        })
    }

    pub fn get_connection(&self) -> Result<Connection, rusqlite::Error> {
        let lock = self.connections.lock().unwrap();
        match lock.try_recv() {
            Ok(connection) => {
                Ok(connection)
            },
            Err(_) => {
                Err(rusqlite::Error::QueryReturnedNoRows)
            }
        }
    }

    pub fn release_connection (&self, connection: Connection) {
        let lock = self.sender.lock().unwrap();
        lock.send(connection).unwrap();
    }
}

// In this rust code above:

//-> SQLiteConnectionPool: 
//> is a struct with two fields: max_connections and connections. 
//> Connections is a receiver end of a channel, wrapped in an 
//> Arc<Mutex<_>> fro safety.

//-> new:
//> is a constructor that creates a new instance of SQLiteConnectionPool. it opens max_connections
//> number of SQLite connections and sends them into the channel.

//-> get_connection:
//> Tries to receive a connection from the channel. if the channel is empty (i.e, all connections are in use),
//> it returns an error.

//-> release_connection:
//> Sends a connection back into the cannel

/*

Also, please note that error handling in Rust is different from Python.
In Rust, you typically return a Result from the functons that can fail, and the 
caller is responsible for handling the error. In this code, 'get_connection'
returns a Result<Connection>, wich means it returns either a Connection or an 
Error. If the channel is empty, it returns an error.

Finaly, please note that Rust uses snake_case for function and variable names, not 
camelCase or PascalCase. This is a convention in the Rust community annd is enforced by the
compiler's built in linter, 'rustc'

*/

