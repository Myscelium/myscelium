use lazy_static::lazy_static;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::buffer_down_manager::DownCommand;
use crate::common::enhanced_buffer::buffer_up_manager::UpCommand;
use crate::common::enhanced_buffer::utilities::Command;
use crate::common::functions::converters::convert_to_value_map;
use crate::common::functions::python_functions::{call_callback, extract_pyobject};
use crate::common::structs::results_structs::ResultType;

use crate::common::structs::avaliable_commands::CommandPatterns;

use serde_json::Error;

use pyo3::types::PyFunction;
use pyo3::Py;
use pyo3::Python;

use std::time::Duration;

use super::host_logger;
use super::host_logger::log_handler::Logger;
use crate::HOST_LOG_LEVEL;

macro_rules! acquire_logger {
    ($section_name:expr) => {{
        let host_log_level;
        {
            host_log_level = HOST_LOG_LEVEL.lock().clone();
        }
        Logger::new(host_log_level, $section_name)
    }};
}

lazy_static! {
    pub static ref COMMAND_PATTERNS: Mutex<CommandPatterns> = Mutex::new(CommandPatterns::new());
    static ref CALLBACK_PATTERNS: Arc<Mutex<HashMap<String, (Py<PyFunction>, Value)>>> = {
        let command_patterns: HashMap<String, (Py<PyFunction>, Value)> = HashMap::new();
        Arc::new(Mutex::new(command_patterns))
    };
    static ref NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));
}

/// Sets the number of workers for the socket host transposer and its associated modules.
///
/// This function updates the number of worker threads that the transposer and its associated modules
/// will use for processing. This can be useful to optimize performance based on available resources.
///
/// # Parameters
///
/// - `n_workers`: The desired number of worker threads. The actual number of workers set for the
///   register manager will be 7 times this value, as each worker requires 7 threads for its operations.
///
/// # Behavior
///
/// - The register manager's workers are set to 7 times the `n_workers` value.
/// - The default number of workers is updated to `n_workers`.
/// - The number of workers for both the down buffer manager and the up buffer manager are set to `n_workers`.
///
/// # Usage
///
/// This function is typically called during the initialization phase of the socket host transposer or
/// when there's a need to adjust performance based on changing workloads or available system resources.
///
/// # Examples
///
/// ```rust
/// let desired_num_workers = 5;
/// set_socket_host_transposer_workers_num(desired_num_workers);
/// ```
///
pub fn set_socket_host_transposer_workers_num(n_workers: u32) {
    host_logger::register::register_manager::set_workers_num(n_workers.clone() * 7); // 7 * n because we need 7 for each
    let mut default_num_of_workers = NUM_WORKERS.lock().unwrap();

    *default_num_of_workers = n_workers;

    enhanced_buffer::buffer_down_manager::set_workers_num(n_workers);
    enhanced_buffer::buffer_up_manager::set_workers_num(n_workers);
}

/// Sets the command patterns and callback patterns for the socket host transposer.
///
/// This function updates the global patterns used by the transposer to handle incoming commands
/// and their associated callbacks. By setting these patterns, the behavior of the transposer
/// in response to specific commands can be defined or modified.
///
/// # Parameters
///
/// - `commands_patterns`: A `HashMap` that maps command names (as `String`s) to their associated
///   patterns (as `Value`s). These patterns determine how specific commands are processed.
/// - `callbacks_patterns`: A `HashMap` that maps command names (as `String`s) to their associated
///   Python callbacks and patterns. The callback (of type `Py<PyFunction>`) is executed when the
///   corresponding command is received, and the pattern (as `Value`) determines the expected structure
///   or behavior of the callback.
///
/// # Usage
///
/// This function is typically called during the initialization phase of the socket host transposer or
/// when there's a need to update or modify the behavior of command processing.
///
/// # Examples
///
/// ```rust
/// let commands_patterns = ...; // Initialize command patterns
/// let callbacks_patterns = ...; // Initialize callback patterns
///
/// set_socket_host_transposer_callbacks(commands_patterns, callbacks_patterns);
/// ```
///
pub fn set_socket_host_transposer_callbacks(commands_patterns: HashMap<String, Value>, callbacks_patterns: HashMap<String, (Py<PyFunction>, Value)>) {
    let mut global_command_patterns = COMMAND_PATTERNS.lock().unwrap();
    global_command_patterns.add_commands_from_map("host", commands_patterns);

    let mut callback_patterns = CALLBACK_PATTERNS.lock().unwrap();
    *callback_patterns = callbacks_patterns;
}

// > Transposer:

macro_rules! error_response {
    ($msg:expr) => {{
        println!("{:?}", $msg);
        let mut error_map = HashMap::new();
        error_map.insert("Error".to_string(), $msg.to_string());
        serde_json::to_string(&error_map)
    }};
}

use crate::socket_host::transposer_functions::handle_direct_function::handle_direct_function;
use crate::socket_host::transposer_functions::handle_internal_management::handle_internal_management;
use crate::socket_host::transposer_functions::handle_redirect::handle_redirect;

/// Processes a given `DownCommand`, executing the corresponding logic and handling redirection.
///
/// This function serves as a central processing unit for commands that come in. Based on the command's
/// contents, it can:
/// - Execute callbacks
/// - Translate commands
/// - Handle redirects
/// - Schedule `UpCommand`s for execution
///
/// # Parameters
///
/// - `py`: A Python interpreter instance, used for executing Python callbacks.
/// - `down_command`: The command to be processed.
///
/// # Flow
///
/// 1. Checks if the command is already registered. If it is, removes it from the schedule.
/// 2. Translates the `down_command` into a general `Command`.
/// 3. Retrieves the function to be executed from the command.
/// 4. Executes the callback associated with the function.
/// 5. Processes the response from the callback. This can involve:
///    - Handling direct responses
///    - Handling redirects
///    - Handling internal management commands
/// 6. Based on the processed response, schedules an `UpCommand` for execution.
///
/// # Notes
///
/// The function heavily relies on global patterns (`COMMAND_PATTERNS` and `CALLBACK_PATTERNS`)
/// which determine how commands are processed and which callbacks are executed.
///
/// The function can handle various response types including maps, strings, integers, floats, and booleans.
/// It also has error handling capabilities to handle unexpected response types or errors during processing.
///
/// # Panics
///
/// This function can panic in scenarios related to unwrapping values, especially when certain expected
/// keys are not present in command maps or when deserialization from JSON fails.
///
/// # Examples
///
/// ```rust
/// let py = Python::acquire_gil().python();
/// let down_command = DownCommand::new(...); // Initialize a DownCommand
///
/// process(py, down_command);
/// ```
///
fn process(py: Python, down_command: DownCommand) {
    let logger = acquire_logger!("Transposer - Process");

    logger.debug(format!("Initializing processing!"));

    let command_is_not_registry: bool = enhanced_buffer::buffer_up_manager::check_if_parity_id_is_registered(down_command.parity_id.clone(), down_command.client_id.clone());
    let command_id: u32 = down_command.command_id.clone().unwrap();

    if !command_is_not_registry {
        logger.debug(format!("Command {}, already have a response!", down_command.parity_id.clone()));
        enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id);
        return;
    }

    // TODO >>> Use the command.command or create a require type field to redirect the command to another client

    // -> One idea is to create a mandatory key in the command.command and instead of only function create a type kwarg field
    // > Type can be:
    // >    - same as origin
    // >    - redirect

    // > if it is redirect one extra kwarg is necessary that have the client_id to redirect
    // * This will create a need to have a local database in the host to store the clients
    // * and to store when is the last contact of some client, if it is some threshold value
    // * more it will remove the registered client, if it have a contact recent, this will redirect the message
    // * however if the message is becomes too old before the client the message is redirected catches it
    // * The system have to remove this old message from the buffer too.

    let translated_command: Command = Command::from_down_command(down_command.clone());

    logger.debug(format!("Translated command: {:?}", translated_command));

    let function;

    {
        function = match translated_command.command.get("function") {
            Some(Value::String(function)) => function,
            _ => {
                logger.warn(format!("The function name is not found or not a string."));
                return;
            },
        };
    }

    let global_command_patterns = COMMAND_PATTERNS.lock().unwrap().clone();

    // -> Remove command from schedule if it isn't on the patterns
    if !global_command_patterns.command_exists("host", &function) {
        // TODO >>> Add a mecanism to check if the command exist for the target client
        // TODO >>> Also adda mecanism to commands have a target by default, and if target is host then target is host
        logger.warn(format!("Command isn't registered in the patterns"));
        enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());
        logger.warn(format!("command skipped and removed from schedule"));
        return;
    }

    let direct_functions: Vec<String> = vec!["get_registered_commands", "update_client_commands_ref"].into_iter().map(|s| s.to_string()).collect();

    let result;

    if direct_functions.contains(&function) {
        // -> Default Rust direct function
        result = handle_direct_function(&translated_command.client_id, function, translated_command.command.clone(), command_id);
    } else {
        // -> Default Python function
        let response;

        {
            let callback_patterns = CALLBACK_PATTERNS.lock().unwrap();
            response = call_callback(py, translated_command.clone(), callback_patterns);
        }

        result = match response {
            Ok(r) => extract_pyobject(py, r),
            Err(e) => {
                // Handle the error or log it
                logger.exception(format!("Python error: {:?}", e));
                // You can return a default value or propagate the error further
                ResultType::Error(format!("{:?}", e))
            },
        };
    }

    logger.debug(format!("Callback call response converted to rust: {:?}", result));

    let mut client_id = down_command.client_id.clone();

    let response: Result<String, Error>;

    fn process_map_result(m: HashMap<String, ResultType>, client_id: &String, down_command: &DownCommand) -> (Result<String, Error>, String) {
        let logger = acquire_logger!("Transposer - Process");

        let response: Result<String, Error>;

        let mut client_to_send: String = client_id.clone();

        if m.contains_key("response_mode") {
            let response_mode = m.get("response_mode").unwrap();

            if *response_mode == ResultType::Str("to_origin".to_string()) {
                let converted_to_value = convert_to_value_map(&m);
                logger.debug(format!("Converted to Value: {:?}", &converted_to_value));
                response = Ok(serde_json::to_string(&converted_to_value).unwrap());
                // Response at this point is like this: Map({
                //      "command_type":String("function"),
                //      "response_mode":String("to_origin"),
                //      "status": String("success"),
                //      "response_activation_function":String(response_activation_function),
                //      "message":String(_),
                //      "kwargs":Map(response)
                // })
            } else if *response_mode == ResultType::Str("redirect".to_string()) {
                logger.debug(format!("Response: {:?}", m));

                let resp = handle_redirect(m, &mut client_to_send, down_command.clone());
                let converted_to_value = convert_to_value_map(&resp);
                response = Ok(serde_json::to_string(&converted_to_value).unwrap());
                // Response at this point is like this: Map({
                //      "command_type":String("function"),
                //      "response_mode":String("redirect"),
                //      "status": String("success"),
                //      "response_activation_function":String(response_activation_function),
                //      "message":String(_),
                //      "kwargs":Map(response),
                //      "redirect_to":String(redirect_to_client_id)
                //  })
            } else if *response_mode == ResultType::Str("internal_management".to_string()) {
                let resp = handle_internal_management(m, &mut client_to_send);
                let converted_to_value = convert_to_value_map(&resp);
                response = Ok(serde_json::to_string(&converted_to_value).unwrap());
            } else {
                logger.warn("Error! Response mode doesn't match any known mode. Please use one of: ('to_origin', 'redirect')!".to_string());
                response = error_response!("Error! Response mode doesn't match any known mode. Please use one of: ('to_origin', 'redirect')!");
            }
        } else {
            logger.warn("Error! Callback doesn't implement response mode!".to_string());
            response = error_response!("Error! Callback doesn't implement response mode!");
        }

        return (response, client_to_send);
    }

    let mut client_to_send_back: String;

    match result {
        // TODO >>> Implement change of response here
        ResultType::Map(m) => {
            (response, client_to_send_back) = process_map_result(m, &client_id, &down_command);
        },
        ResultType::Str(s) => {
            response = Ok(s.clone());
        },
        ResultType::Int(i) => {
            response = Ok(i.to_string());
        },
        ResultType::Float(fl) => {
            response = Ok(fl.to_string());
        },
        ResultType::Bool(b) => {
            response = Ok(b.to_string());
        },
        ResultType::List(l) => {
            let mut counter: u64 = 0;
            for res in l {
                match res {
                    ResultType::Map(m) => {
                        if counter == 0 {
                            let (processed_resp, client_to_send_back) = process_map_result(m, &client_id, &down_command);
                            let up_command = UpCommand::new(client_to_send_back, down_command.parity_id.clone(), down_command.priority.clone(), processed_resp.unwrap());
                            enhanced_buffer::buffer_up_manager::buffer_up_schedule(up_command);
                        } else {
                            // TODO >>> Add a mechanism to generate new 20 char parity ids for the other resps besides the firs one

                            let (processed_resp, client_to_send_back) = process_map_result(m, &client_id, &down_command);
                            let up_command = UpCommand::new(client_to_send_back, down_command.parity_id.clone(), down_command.priority.clone(), processed_resp.unwrap());
                            enhanced_buffer::buffer_up_manager::buffer_up_schedule(up_command);
                        }

                        counter += 1;
                    },
                    _ => {
                        response = error_response!("Error! Received a list, but expected a map!");
                        let up_command = UpCommand::new(client_id, down_command.parity_id.clone(), down_command.priority.clone(), response.unwrap());
                        enhanced_buffer::buffer_up_manager::buffer_up_schedule(up_command);
                        break;
                    },
                }
            }
            enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());
            return;
        },
        ResultType::Empty => {
            let mut command_map = HashMap::new();
            command_map.insert("command_type".to_string(), Value::String("special_function".to_string()));
            command_map.insert("function".to_string(), Value::String("C210".to_string()));
            response = Ok(serde_json::to_string(&command_map).unwrap());
        },
        ResultType::Error(e) => {
            logger.warn(format!("An error occurred while converting the Python callback response. The error was: {:?}", e));
            response = error_response!(format!("An error occurred while converting the Python callback response. The error was: {:?}", e));
        },
    }

    logger.debug(format!("Function returned: {:?}", response));
    logger.info(format!("Command: {:?}, processed!", down_command.parity_id.clone()));

    enhanced_buffer::buffer_down_manager::buffer_down_remove_schedule_by_id(command_id.clone());

    let up_command = UpCommand::new(client_id, down_command.parity_id.clone(), down_command.priority.clone(), response.unwrap());

    enhanced_buffer::buffer_up_manager::buffer_up_schedule(up_command);
}

fn clear_old_data() {
    enhanced_buffer::buffer_down_manager::buffer_down_clear_old_commands();
    enhanced_buffer::buffer_up_manager::buffer_up_clear_old_commands();
}

pub fn initialize_socket_host_transposer(py: Python<'_>) {
    let logger = acquire_logger!("Transposer");

    let mut schedule: Vec<DownCommand> = enhanced_buffer::buffer_down_manager::buffer_down_list_schedule();

    if !(schedule.len() > 0) {
        // logger.debug(format!("Nothing in the schedule, skipping >>>"));
        clear_old_data();
        thread::sleep(Duration::from_millis(100));
        return;
    }

    schedule.sort_by(|a, b| b.priority.cmp(&a.priority)); // put the schedule in crescent order

    // logger.debug(format!("Schedule to process:\n{:?}\n", schedule));

    logger.info(format!("Data found in schedule!"));

    for dow_command in schedule {
        let logger = acquire_logger!("Transposer");

        logger.info(format!("get a pool worker in transposer!"));

        let py;

        {
            let getting_py = unsafe { Python::assume_gil_acquired() };

            let gil_pool = unsafe { getting_py.clone().new_pool() };

            py = gil_pool.python();

            logger.debug(format!("Aquired python in a process task!"));
            process(py, dow_command);
            logger.debug(format!("Finalize a process task!"));
        }
    }

    thread::sleep(Duration::from_millis(100));

    // let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();

    return;

    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();

    //     pool.execute(|| {
    //         handle_connection(stream);
    //     });
    // }
}
