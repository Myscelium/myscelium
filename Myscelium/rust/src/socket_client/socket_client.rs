use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::buffer_down_manager::DownCommand;

use crate::common::enhanced_buffer::utilities::{Command, CommandInstructions, CommandMode, CommandOrigin, CommandStatus, CommandTarget, CommandType};
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

// use parking_lot::Mutex;
use std::sync;
use std::sync::Mutex;

use crate::common::functions::advanced_lockers::smart_lock;

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
use crate::CLIENT_ID;

lazy_static! {
    pub static ref COMMAND_PATTERNS: Arc<sync::Mutex<CommandPatterns>> = Arc::new(sync::Mutex::new(CommandPatterns::new()));
    static ref HOST_ALLOWED_COMMANDS: Arc<Mutex<HashMap<String, Value>>> = {
        let json_str = r#"{
            "get_symbols_data": {
                "symbols_data": {
                    "data-type": "str",
                    "symbols": "str",
                    "start-ts": "float",W
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

    let command_patterns = &COMMAND_PATTERNS;
    smart_lock(&*command_patterns, |patterns: &mut CommandPatterns| {
        patterns.add_commands_from_map(client_name.as_str(), callbacks_patterns);
    });
}

use crate::common::enhanced_buffer::history::register::register::initialize_buffer_history;

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

    initialize_buffer_history(&buffer_location);

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
    let mut global_command_patterns: HashMap<String, Value> = HashMap::new();
    let command_patterns = &COMMAND_PATTERNS;
    smart_lock(&*command_patterns, |patterns: &mut CommandPatterns| {
        global_command_patterns = patterns.extract_all_commands();
    });
    return global_command_patterns;
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

// macro_rules! create_error_command {
//     ($client_key:expr, $parity_id:expr, $error:expr) => {{
//         let mut command_map = HashMap::new();

//         let kwargs: HashMap<String, Value> = HashMap::new();

//         command_map.insert("mode".to_string(), Value::String("function".to_string()));
//         command_map.insert("command_type".to_string(), Value::String("direct_function".to_string()));
//         command_map.insert("target".to_string(), Value::String("origin".to_string()));
//         command_map.insert("status".to_string(), Value::String("failure".to_string()));
//         command_map.insert("actf".to_string(), Value::String("error_handler".to_string()));
//         command_map.insert("kwargs".to_string(), serde_json::to_value(&kwargs).unwrap());
//         command_map.insert("message".to_string(), Value::String($error.to_string()));

//         // TODO >>> Change this for the descriptive form!

//         let command_instructions: CommandInstructions = CommandInstructions::from_value_map(command_map).unwrap();

//         let command = Command {
//             client_key: $client_key.to_string(),
//             parity_id: $parity_id.to_string(),
//             priority: 11,
//             command: command_instructions,
//         };
//         command
//     }};
// }

macro_rules! create_special_command {
    ($client_key:expr, $command_mode:expr, $special_command:expr) => {{
        let command_instructions = CommandInstructions::new(
            $command_mode,
            CommandType::SpecialFunction,
            CommandTarget::Host,
            CommandStatus::Success,
            CommandOrigin::ClientKey($client_key.clone()),
            $special_command.to_string(),
            HashMap::new(),
            "".to_string(),
        );

        let command = Command {
            client_key: $client_key.clone().to_string(),
            parity_id: "itisaspecialcase".to_string(),
            priority: 11,
            command: command_instructions,
        };
        command
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

    let mut client_key: String = "".to_string();

    let client_key_storage = &CLIENT_ID;
    smart_lock(&client_key_storage, |key: &mut String| {
        client_key = key.clone();
    });

    let command = create_special_command!(client_key.clone(), CommandMode::Function, "C202");

    let command_json = json!(command).to_string();

    stream.write_all(command_json.as_bytes()).unwrap();

    let mut buffer = [0; 4096];
    stream.read(&mut buffer).unwrap();

    let buffer_string = String::from_utf8_lossy(&buffer).trim_end_matches(|c| c == '\n' || c == '\r' || c == '\0').to_string();

    let command: Command = serde_json::from_str(&buffer_string).unwrap();

    logger.debug(format!("{:?}", command));

    if command.command.actf == "C200" {
        return true;
    } else {
        return false;
    }

    // logger.warn(format!("The function name is not found or not a string."));
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

    let mut client_key: String = "".to_string();

    let client_key_storage = &CLIENT_ID;
    smart_lock(&*client_key_storage, |key: &mut String| {
        client_key = key.clone();
    });

    let command_to_request = create_special_command!(client_key, CommandMode::Function, "C206");
    let received = send(&mut stream, command_to_request.clone());

    match handle_response(received) {
        Received::DownCommand(down_command) => return Some(down_command),
        Received::Confirmation => {
            return None;
        },
        Received::Error(_) => {
            //TODO >>> Add the mechanism to stop the client if received a error
            return None;
        },
        Received::Nothing => {
            return None;
        },
    }
}

pub enum Received {
    DownCommand(DownCommand),
    Confirmation,
    Nothing,
    Error(String),
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
fn handle_response(received: Response) -> Received {
    let logger = acquire_logger!("Core");

    let command_received;

    match received {
        Response::None => {
            logger.warn(format!("Received invalid data!"));
            return Received::Nothing;
        },
        Response::Command(c) => {
            logger.debug(format!("\nReceived command: {:?}", c));
            command_received = c;
        },
    }

    match command_received.command.mode {
        CommandMode::Function => {},
        CommandMode::Response => {
            // Response format:
            //* From now this is basically equal to response
            logger.info(format!("[Socket Client] - Received a response!: \n{:?}", command_received.command));

            let status: String = command_received.command.status.to_string();

            // TODO >>> Add a better ahndler for error cases:
            if status == "error".to_string() {
                let val = Value::String("Unknown error".to_string());
                let error_msg = command_received.command.message;
                logger.exception(format!("\nAn error occurred in host, the error was: {}\n", error_msg.clone()));
                enhanced_buffer::buffer_up_manager::buffer_up_remove_schedule_by_parity_id(&command_received.client_key, &command_received.parity_id);
                CLIENT_IS_RUNNING.store(false, Ordering::SeqCst);
                return Received::Error(error_msg);
            }

            // let down_command = DownCommand::from_command(command_received.clone());

            enhanced_buffer::buffer_up_manager::buffer_up_remove_schedule_by_parity_id(&command_received.client_key, &command_received.parity_id);

            // return Received::DownCommand(down_command);
        },
    }

    match command_received.command.command_type {
        CommandType::Default => {
            // > Also we can use a similar system to sync multiple hosts
            logger.info(format!("[Socket Client] - Received a function!:\n {:?}", command_received.command.actf));
            return Received::DownCommand(DownCommand::from_command(command_received.clone()));
        },

        CommandType::DirectFunction => {
            // TODO >>> Need to actualize this to the new patter like Response handler to redirect works as intended!
            // > Also we can use a similar system to sync multiple hosts
            logger.info(format!("[Socket Client] - Received a direct function!:\n {:?}", command_received.command.actf));
            return Received::DownCommand(DownCommand::from_command(command_received.clone()));
        },

        CommandType::InternalManagement => {
            return Received::Nothing;
        },

        CommandType::SpecialFunction => {
            if command_received.parity_id != "itisaspecialcase" {
                if command_received.command.actf == "C210".to_string() {
                    logger.info(format!("Received Confirmation! Removing command {} of client: {} from buffer up", command_received.parity_id, command_received.client_key));
                    enhanced_buffer::buffer_up_manager::buffer_up_remove_schedule_by_parity_id(&command_received.client_key, &command_received.parity_id);
                    return Received::Confirmation;
                } else if command_received.command.status == "Failure" {
                    logger.exception(format!("\nAn error occurred in host, the error was: {}\n", command_received.command.message));
                    enhanced_buffer::buffer_up_manager::buffer_up_remove_schedule_by_parity_id(&command_received.client_key, &command_received.parity_id);
                    CLIENT_IS_RUNNING.store(false, Ordering::SeqCst);
                    return Received::Error("".to_string());
                }
            }
            logger.debug(format!("Receive a special function: {:?}", command_received.command.actf));
            return Received::Nothing;
        },

        _ => {
            logger.warn(format!("Received an Unknown command!"));
            return Received::Nothing;
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

    // Here need to send the new handlers to host
    // then receive the host handlers

    logger.info(format!("Connected to {:?}!", address.clone()).to_string());

    thread::sleep(Duration::from_millis(200));

    loop {
        if !CLIENT_IS_RUNNING.load(Ordering::SeqCst) {
            logger.info(format!("running is set to false, shutdown socket client main process!"));
            break;
        }

        let mut client_key: String = " ".to_string();

        let client_key_storage = &CLIENT_ID;
        smart_lock(&*client_key_storage, |key: &mut String| {
            client_key = key.clone();
        });

        // TODO >>> Maybe add a mechanism taht when this is seted it don't verify again to reduce complexity, maybe a boolean

        if client_key == " " {
            // Whait untill client id was seted!
            thread::sleep(Duration::from_millis(200));
            continue;
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
            let command_to_request = match Command::from_up_command(&up_command) {
                Ok(c) => c,
                Err(e) => {
                    println!("Command: {:?} gives an exception when converting to command, the error was: \n{:?}", up_command, e);
                    continue;
                },
            };

            loop {
                println!("Sending to host: {:?}", command_to_request.clone());

                let received = send(&mut stream, command_to_request.clone());

                match handle_response(received) {
                    Received::DownCommand(down_command) => {
                        println!("[Socket Client] - Receives Data.. : {:?}", down_command);
                        enhanced_buffer::buffer_down_manager::buffer_down_schedule(down_command.clone());
                        break;
                    },
                    Received::Confirmation => {
                        break;
                    },
                    Received::Error(e) => {
                        //TODO >>> Add the mechanism to stop the client if received a error
                    },
                    Received::Nothing => {},
                }

                thread::sleep(Duration::from_millis(200));
            }
        }

        println!("End schedule data, so skipping >>>");
        continue;
    }
}
