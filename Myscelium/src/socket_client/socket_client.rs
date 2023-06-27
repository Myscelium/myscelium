
use std::io::prelude::*;
use std::net::TcpStream;

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use serde_json::json;

// #[derive(Serialize, Deserializer, Debug)] is an attribute that automatically
// derives the Serialize and Deserialize traits from the serde crate, witch allow 
// the struct to be converted to and from JSON.

// The Debug Trait, is also derived, which allows the structure to be printed fro debugging purposes

#[derive(Serialize, Deserialize, Debug)]
struct Command {
    id: i32,
    client_id: String,
    parity_id: String,
    priority: i32,
    command: HashMap<String, String>
}

use serde_json::to_string;

fn main() {

    let command = Command {
        id: 1,
        client_id: "client1".to_string(),
        parity_id: "parity1".to_string(),
        priority: 2,
        command: {
            let mut m = HashMap::new();
            m.insert("key".to_string(), "value".to_string());
            m
        },
    };

    let json = to_string(&command).unwrap();
    println!("{}", json);

    // -> Socket Client:

    let mut stream = TcpStream::connect("127.0.0.1:7878").unwrap();


    let command = json!({
        "function": "get_symbols_data",
        "parameters": {
            "symbols_data": {
                "data-type": "AAPL",
                "symbols": "AAPL",
                "start-ts": 162.34,
                "end-ts": 163.34
            }
        }
    });

    let command_string = command.to_string();

    // let msg = b"hello, world!";
    // stream.write(msg).unwrap();

    stream.write_all(command_string.as_bytes()).unwrap();

    // write is used to send a message to the server.

    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    // Then we read the response from the server into a buffer and print it out.

    println!("Received: {}", String::from_utf8_lossy(&buffer[..]))

}
