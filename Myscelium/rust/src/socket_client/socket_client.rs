use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::buffer_down_manager::DownCommand;

use crate::common::enhanced_buffer::utilities::{Command, CommandType};

use lazy_static::lazy_static;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::sync::{mpsc, Arc};
use std::thread;

use crate::common::functions::converters::value_to_resulttype;

use std::sync::atomic::Ordering;

use std::time::Duration;

use crate::CLIENT_IS_RUNNING;

use std::net::TcpStream;

use std::io::Read;
use std::io::Write;

use serde_json::json;

use super::client_logger::log_handler::Logger;
use crate::CLIENT_LOG_LEVEL;

use crate::CLIENT_NODE_NAME;

use parking_lot::Mutex;

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let client_log_level;
        {
            client_log_level = CLIENT_LOG_LEVEL.lock().clone();
        }
        Logger::new(client_log_level, $section_name)
    }};
}

use crate::common::structs::avaliable_commands::CommandPatterns;

lazy_static! {
    pub static ref COMMAND_PATTERNS: Mutex<CommandPatterns> = Mutex::new(CommandPatterns::new());
    static ref HOST_ALLOWED_COMMANDS: Arc<Mutex<HashMap<String, Value>>> = {
        let json_str = r#"{
            "get_symbols_data": {
                "symbols_data": {
                    "data-type": "str",
                    "symbols": "str",
                    "start-ts": "float",
                    "end-ts": "float"
                }
            },
            "get_other_symbols_data": {
                "symbols_data": {
                    "data-type": "str",
                    "symbols": "str",
                    "start-ts": "float",
                    "end-ts": "float"
                }
            }
        }"#;

        let command_patterns: HashMap<String, Value> = from_str(json_str).unwrap();
        Arc::new(Mutex::new(command_patterns))
    };
    static ref CLIENT_ID: Arc<Mutex<String>> = Arc::new(Mutex::new(' '.to_string()));
}

// >-------------------------------------------------------------------------------------------------------------------------------------------

// -> Socket Interactive Functions:

/// Sets the callback patterns for the socket client.
///
/// This function allows the user to define the command patterns that dictate
/// how commands are recognized and processed by the socket client.
///
/// # Arguments
/// - `callbacks_patterns`: A `HashMap` containing the desired command patterns.
pub fn set_socket_client_callbacks_patterns(callbacks_patterns: HashMap<String, Value>) {
    let client_name = CLIENT_NODE_NAME.lock().clone();

    let mut global_command_patterns = COMMAND_PATTERNS.lock();
    global_command_patterns.add_commands_from_map(client_name.as_str(), callbacks_patterns);
}

/// Initializes the client buffer by setting up the necessary tables.
///
/// This function is responsible for initializing the buffer tables for both
/// up and down commands. If the tables aren't already initialized, they will be
/// created at the specified `buffer_location`.
///
/// # Arguments
/// - `buffer_location`: The location where the buffer database will be initialized.
///
/// # Side Effects
/// - If not already initialized, the function will create and initialize the buffer database
///   at the specified location.
pub fn initialize_client_buffer(buffer_location: String) {
    println!("initializing the buffer database into: {}buffer.db, if not initialized!", buffer_location);

    enhanced_buffer::buffer_down_manager::buffer_down_initialize_table(buffer_location.clone());

    enhanced_buffer::buffer_up_manager::buffer_up_initialize_table(buffer_location.clone());

    println!("All buffer initialized successfully!");

    return;
}

// Keep the thread alive until HOST_IS_RUNNING is set to false
// if !CLIENT_IS_RUNNING.load(Ordering::SeqCst) {
//     print!("running is set to false, skipping");
//     break;
// }

// The incoming method is called on the listener, which returns an iterator that gives us a sequence of
// TCP streams (representing a series of connections). The server will then handle each connection in a loop.

// handle_connection is a function that handles each TCP stream. It reads from the stream into a buffer,
// then writes the contents of the buffer back to the stream.

/// Retrieves the available command patterns registered for the socket client.
///
/// This function provides access to the current command patterns registered in the socket client.
/// These patterns dictate how commands are recognized and processed.
///
/// # Returns
/// - A `HashMap` containing the available command patterns.
pub fn get_available_handlers_registered() -> HashMap<String, Value> {
    let global_command_patterns = COMMAND_PATTERNS.lock().clone();
    return global_command_patterns.extract_all_commands();
}

// > --------------------------------------------------------------------------------------------------------------------------------------

// -> Socket client functionality structures:

// #[derive(Serialize, Deserializer, Debug)] is an attribute that automatically
// derives the Serialize and Deserialize traits from the serde crate, witch allow
// the struct to be converted to and from JSON.

// The Debug Trait, is also derived, which allows the structure to be printed fro debugging purposes

/// Represents possible responses from the server.
///
/// This enum is used to capture the different types of responses that the server can send.
///
/// Variants:
/// - `Command`: Represents a valid command response from the server.
/// - `None`: Represents an absence of response or an invalid response.
enum Response {
    Command(Command),
    None,
}

macro_rules! create_special_command {
    ($code:expr) => {{
        use std::collections::HashMap;

        let mut command_map = HashMap::new();
        command_map.insert("command_type".to_string(), Value::String("special_function".to_string()));
        command_map.insert("function".to_string(), Value::String($code.to_string()));

        Command {
            client_key: "some_client_id".to_string(),
            parity_id: "itisaspecialcase".to_string(),
            priority: 11,
            command: command_map,
        }
    }};
}

/// Verifies the connection to the server by sending a special command and checking the response.
///
/// This function sends a special command `"C202"` to the server and expects a response with the function `"C200"`.
/// If the response matches the expectation, it means the connection is verified.
///
/// # Arguments
/// - `stream`: A mutable reference to the active TcpStream.
///
/// # Returns
/// - `true` if the connection is verified successfully.
/// - `false` otherwise.
///
/// # Behavior
/// - The function logs any unexpected responses or errors.
fn verify_connection(stream: &mut TcpStream) -> bool {
    let logger = acquire_logger!("Core");

    let command = create_special_command!("C202");

    let command_json = json!(command).to_string();

    stream.write_all(command_json.as_bytes()).unwrap();

    let mut buffer = [0; 4096];
    stream.read(&mut buffer).unwrap();

    let buffer_string = String::from_utf8_lossy(&buffer).trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0').to_string();

    let command: Command = serde_json::from_str(&buffer_string).unwrap();

    logger.debug(format!("{:?}", command));

    match command.command.get("function") {
        Some(Value::String(function)) => {
            if function == "C200" {
                return true;
            } else {
                return false;
            }
        },
        _ => {
            logger.warn(format!("The function name is not found or not a string."));
            return false;
        },
    }
}

/// Sends a command to the server and waits for a response.
///
/// Before sending the command, the function verifies the connection using the `verify_connection` function.
/// If the connection is not verified, the function returns `Response::None`.
///
/// # Arguments
/// - `stream`: A mutable reference to the active TcpStream.
/// - `command`: The `Command` object to be sent to the server.
///
/// # Returns
/// - A `Response` variant containing the server's response.
///
/// # Behavior
/// - If the connection is not verified, the function logs the event and returns `Response::None`.
fn send(stream: &mut TcpStream, command: Command) -> Response {
    let logger = acquire_logger!("Core");

    let conn: bool = verify_connection(stream);

    if !conn {
        logger.info(format!("Not connected!"));
        return Response::None;
    }

    let command_json = json!(command).to_string();

    stream.write_all(command_json.as_bytes()).unwrap();

    let mut buffer = [0; 4096];
    stream.read(&mut buffer).unwrap();

    let buffer_string = String::from_utf8_lossy(&buffer).trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0').to_string();

    let command: Command = serde_json::from_str(&buffer_string).unwrap();

    logger.debug(format!("Received: {:?}", command));

    return Response::Command(command);
}

/// Sends a ping request to the server.
///
/// This function sends a special command `"C206"` to ping the server. It utilizes the `send` function
/// to send the ping request and then processes the received response using the `handle_response` function.
///
/// # Arguments
/// - `stream`: A mutable reference to the active TcpStream.
///
/// # Returns
/// - An `Option<DownCommand>` containing the processed command if there's any, or `None` otherwise.
///
/// # Behavior
/// - If the `CLIENT_IS_RUNNING` global flag is set to false, the function will immediately return `None`.
pub fn send_ping(mut stream: &mut TcpStream) -> Option<DownCommand> {
    if !CLIENT_IS_RUNNING.load(Ordering::SeqCst) {
        return None;
    }

    let command_to_request = create_special_command!("C206");
    let received = send(&mut stream, command_to_request.clone());
    if let Some(down_command) = handle_response(received) {
        return Some(down_command);
    } else {
        return None;
    }
}

/// Handles the received response from the server and processes it accordingly.
///
/// This function processes the response from the server based on its type. Depending on the
/// type and content of the received response, the function might remove a command from the
/// schedule, log an error or warning, shut down the client, or produce a new down command for
/// further processing.
///
/// # Arguments
/// - `received`: A `Response` variant containing the server's response.
///
/// # Returns
/// - An `Option<DownCommand>`: If a new down command is produced based on the received response,
///   this function will return `Some(DownCommand)`. Otherwise, it will return `None`.
///
/// # Behavior
/// - If the received response indicates a confirmation (`C210`), the corresponding command is
///   removed from the schedule and no further action is taken.
/// - If an error is received, the error is logged and the client might be shut down.
/// - If the received command is of type `Response`, a new down command may be produced based on
///   the received content.
/// - If the received command is of an unknown type, a warning is logged.
///
/// # Notes
/// - This function uses the `COMMAND_PATTERNS` global lock to access and modify the command patterns.
/// - The function also accesses the `CLIENT_IS_RUNNING` global flag to control the client's running state.
fn handle_response(received: Response) -> Option<DownCommand> {
    let logger = acquire_logger!("Core");

    let command_received;

    match received {
        Response::None => {
            logger.warn(format!("Received invalid data!"));
            return None;
        },
        Response::Command(c) => {
            logger.debug(format!("\nReceived command: {:?}", c));
            command_received = c;
        },
    }

    match command_received.command_type() {
        CommandType::Function(f) => {
            // TODO >>> Need to actualize this to the new patter like Response handler to redirect works as intended!
            // > Also we can use a similar system to sync multiple hosts

            logger.info(format!("[Socket Client] - Received a function!:\n {:?}", f));

            // match serde_json::from_value::<String>(f) {
            //     Ok(function) => {
            //         if function == "Error" {
            //             // TODO >>> See if this is necessary to maintain since the errors now are pretended to be redirected
            //             let val = Value::String("Unknown error".to_string());
            //             let error_msg = command_received.command.get("Error").unwrap_or(&val);
            //             logger.exception(format!("\nAn error occurred in host, the error was: {}\n", error_msg));
            //             enhanced_buffer::buffer_up_manager::buffer_up_remove_schedule_by_parity_id(command_received.client_key, command_received.parity_id);
            //             CLIENT_IS_RUNNING.store(false, Ordering::SeqCst);
            //             return None;
            //         } // Optionally handle other string cases here...
            //     },
            //     Err(_) => {
            //         // This block will execute if the JSON is not a string.
            //         // Just continue, without doing anything.
            //     },
            // }

            let down_command = DownCommand::from_command(command_received.clone());

            return Some(down_command);
        },

        CommandType::DirectFunction(df) => {
            // TODO >>> Need to actualize this to the new patter like Response handler to redirect works as intended!
            // > Also we can use a similar system to sync multiple hosts

            logger.info(format!("[Socket Client] - Received a direct function!:\n {:?}", df));

            // match serde_json::from_value::<String>(f) {
            //     Ok(function) => {
            //         if function == "Error" {
            //             // TODO >>> See if this is necessary to maintain since the errors now are pretended to be redirected
            //             let val = Value::String("Unknown error".to_string());
            //             let error_msg = command_received.command.get("Error").unwrap_or(&val);
            //             logger.exception(format!("\nAn error occurred in host, the error was: {}\n", error_msg));
            //             enhanced_buffer::buffer_up_manager::buffer_up_remove_schedule_by_parity_id(command_received.client_key, command_received.parity_id);
            //             CLIENT_IS_RUNNING.store(false, Ordering::SeqCst);
            //             return None;
            //         } // Optionally handle other string cases here...
            //     },
            //     Err(_) => {
            //         // This block will execute if the JSON is not a string.
            //         // Just continue, without doing anything.
            //     },
            // }

            let down_command = DownCommand::from_command(command_received.clone());

            return Some(down_command);
        },

        CommandType::SpecialFunction(f) => {
            let function: String = serde_json::from_value(f.clone()).unwrap();

            if command_received.parity_id != "itisaspecialcase" {
                if function == "C210".to_string() {
                    enhanced_buffer::buffer_up_manager::buffer_up_remove_schedule_by_parity_id(command_received.client_key, command_received.parity_id);
                    logger.info(format!("Received Confirmation!"));
                    return None;
                } else if function == "Error".to_string() {
                    logger.exception(format!("\nAn error occurred in host, the error was: {}\n", command_received.command.get("Error").unwrap()));
                    enhanced_buffer::buffer_up_manager::buffer_up_remove_schedule_by_parity_id(command_received.client_key, command_received.parity_id);
                    CLIENT_IS_RUNNING.store(false, Ordering::SeqCst);
                    return None;
                }
            }

            logger.debug(format!("Receive a special function: {:?}", f));
            return None;
        },

        CommandType::Response(r) => {
            //* From now this is basically equal to response
            logger.info(format!("[Socket Client] - Received a response!: \n{:?}", r));

            let status = serde_json::from_value::<String>(r.get("status").unwrap().clone()).unwrap();

            // if status == "error".to_string() {
            //     let val = Value::String("Unknown error".to_string());
            //     let error_msg = command_received.command.get("message").unwrap_or(&val);
            //     logger.exception(format!("\nAn error occurred in host, the error was: {}\n", serde_json::from_value::<String>(error_msg.clone()).unwrap()));
            //     enhanced_buffer::buffer_up_manager::buffer_up_remove_schedule_by_parity_id(command_received.client_key, command_received.parity_id);
            //     CLIENT_IS_RUNNING.store(false, Ordering::SeqCst);
            //     return None;
            // }

            let down_command = DownCommand::from_command(command_received.clone());

            enhanced_buffer::buffer_up_manager::buffer_up_remove_schedule_by_parity_id(command_received.client_key, command_received.parity_id);

            return Some(down_command);
        },

        CommandType::Error(_) => {
            logger.exception(format!("\nAn error occurred in host, the error was: {}\n", command_received.command.get("message").unwrap()));
            // CLIENT_IS_RUNNING.store(false, Ordering::SeqCst);

            // return None;

            let down_command = DownCommand::from_command(command_received.clone());

            enhanced_buffer::buffer_up_manager::buffer_up_remove_schedule_by_parity_id(command_received.client_key, command_received.parity_id);

            return Some(down_command);
        },

        CommandType::Redirect(_) => {
            logger.warn(format!("Received an Unknown command!"));
            return None;
        },

        CommandType::Unknown => {
            logger.warn(format!("Received an Unknown command!"));
            return None;
        },
    }
}

/// Initializes the client and sets up communication with the specified server address.
///
/// This function connects to the provided server address, and then periodically checks
/// for scheduled commands and sends them to the server. The function also spawns a
/// background thread to monitor for potential deadlocks.
///
/// # Arguments
/// - `address`: The server address to connect to, in the format "ip:port".
/// - `client_id`: A unique identifier for the client, used for communication purposes.
///
/// # How it works
/// 1. The function first spawns a background thread that checks for deadlocks every 5 seconds.
///    If a deadlock is detected, the involved threads' IDs and backtraces are printed.
/// 2. The client attempts to establish a TCP connection with the server using the provided address.
/// 3. Once connected, the function enters a loop where it checks the `CLIENT_IS_RUNNING` global flag.
///    If the flag is set to false, the client will shut down.
/// 4. Inside the loop, the function retrieves the list of scheduled commands (up_schedule) to be sent
///    to the server. If there are no commands in the schedule, the client sends a ping to the server
///    and then waits for a short duration before checking again.
/// 5. For each command in the schedule, the client sends the command to the server and waits for a response.
///    The received response is then processed and scheduled for further handling.
///
/// # Notes
/// - The function uses `parking_lot::deadlock::check_deadlock()` to detect potential deadlocks.
/// - The client sends a ping to the server when there are no commands in the schedule.
/// - The client will wait for 200 milliseconds between retries if a command's response is not received.
/// - The client will continue to check for scheduled commands as long as `CLIENT_IS_RUNNING` is true.
pub fn initialize_client(address: String, client_id: String) {
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

    let logger = acquire_logger!("Core");

    let mut stream = TcpStream::connect(address.clone()).unwrap();

    logger.info(format!("Connected to {:?}!", address.clone()).to_string());

    thread::sleep(Duration::from_millis(200));

    loop {
        if !CLIENT_IS_RUNNING.load(Ordering::SeqCst) {
            logger.info(format!("running is set to false, shutdown socket client main process!"));
            break;
        }

        let up_schedule = enhanced_buffer::buffer_up_manager::buffer_up_list_schedule();

        if !(up_schedule.len() > 0) {
            if let Some(down_command) = send_ping(&mut stream) {
                enhanced_buffer::buffer_down_manager::buffer_down_schedule(down_command.clone());
            }
            // println!("[Socket] - Nothing in schedule, skipping..");
            thread::sleep(Duration::from_millis(100));
            continue;
        }

        for up_command in up_schedule {
            let command_to_request = Command::from_up_command(up_command);

            loop {
                let received = send(&mut stream, command_to_request.clone());

                if let Some(down_command) = handle_response(received) {
                    println!("[Socket Client] - Receives Data.. : {:?}", down_command);
                    enhanced_buffer::buffer_down_manager::buffer_down_schedule(down_command.clone());
                    break;
                }

                thread::sleep(Duration::from_millis(200));
            }
        }

        println!("End schedule data, so skipping >>>");
        continue;
    }
}
