use lazy_static::lazy_static;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

use serde::{Deserialize, Serialize};

use crate::socket_host::client_mananger::mananger::check_if_client_key_exists;

use crate::commom::enhanced_buffer;
use crate::commom::enhanced_buffer::buffer_down_mananger::DownCommand;
use crate::commom::enhanced_buffer::buffer_up_mananger::UpCommand;
use crate::commom::enhanced_buffer::utilities::{Command, CommandType};
use crate::commom::functions::converters::{convert_to_resulttype_map, convert_to_value_map};
use crate::commom::functions::python_functions::{call_callback, dict_to_kwargs, extract_pyobject};
use crate::commom::structs::results_structs::ResultType;

use crate::socket_host::client_mananger::mananger::Client;

use crate::commom::structs::results_structs::ExpectationError;

use std::any::Any;

#[macro_use]
use crate::{init_thread_pool, terminate_pool, run_in_thread_pool, wait_all_threads};
use crate::commom::custom_thread_pool::thread_pool::UnifiedThreadPool;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Condvar,
};

use pyo3::exceptions::PyException;
use pyo3::types::{IntoPyDict, PyAny, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};
use pyo3::IntoPy;
use pyo3::Py;
use pyo3::ToPyObject;
use pyo3::{PyErr, PyObject, PyResult, Python};

use std::error::Error;
use std::time::{Duration, Instant};

use crate::HOST_IS_RUNNING;

use std::fmt;

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

macro_rules! create_error_response_and_return {
    ($error_msg:expr, $converted_m:expr, $to_send:expr) => {{
        let mut resp: HashMap<String, ResultType> = HashMap::new();
        resp.insert("Error".to_string(), ResultType::Str($error_msg.to_string()));

        $to_send.insert("response".to_string(), ResultType::Map(resp));
        $to_send.insert("response_activation_function".to_string(), ResultType::Str($converted_m.get("response_activation_function").unwrap().to_string()));
        $to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));

        return $to_send;
    }};
}

pub enum KwargError {
    MissingKwarg(String),
}

lazy_static! {
    static ref COMMAND_PATTERNS: Arc<Mutex<HashMap<String, Value>>> = {
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
    static ref CALLBACK_PATTERNS: Arc<Mutex<HashMap<String, (Py<PyFunction>, Value)>>> = {
        let command_patterns: HashMap<String, (Py<PyFunction>, Value)> = HashMap::new();
        Arc::new(Mutex::new(command_patterns))
    };
    static ref NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5));
}

pub fn set_socket_host_transposer_workers_num(n_workers: u32) {
    host_logger::register::register_mananger::set_workers_num(n_workers.clone() * 7); // 7 * n because we need 7 for each
    let mut default_num_of_workers = NUM_WORKERS.lock().unwrap();

    *default_num_of_workers = n_workers;

    enhanced_buffer::buffer_down_mananger::set_workers_num(n_workers);
    enhanced_buffer::buffer_up_mananger::set_workers_num(n_workers);
}

pub fn set_socket_host_transposer_callbacks(commands_patterns: HashMap<String, Value>, callbacks_patterns: HashMap<String, (Py<PyFunction>, Value)>) {
    let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();
    *command_patterns = commands_patterns;

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

fn handle_redirect(m: HashMap<String, ResultType>, client_id: &mut String, down_command: DownCommand) -> HashMap<String, ResultType> {
    let mut to_send = HashMap::new();

    let converted_m = convert_to_value_map(&m);

    if !m.contains_key("redirect_to") {
        create_error_response_and_return!("Error! Callback response args don't have redirect_to client_id field!", converted_m, to_send);
        // error_response!("Error! Callback response args don't have redirect_to client_id field!");
    }

    let redirect_to_value = converted_m.get("redirect_to").unwrap().clone();
    let redirect_to: String = serde_json::from_value(redirect_to_value).unwrap();

    if !check_if_client_key_exists(redirect_to.to_string()) {
        create_error_response_and_return!(format!("Error! request to redirect to client_id: {} failed, client doesn't exist!", redirect_to.to_string()), converted_m, to_send);
        // return error_response!(format!("Error! request to redirect to client_id: {} failed, client doesn't exist!", redirect_to.to_string()));
    }

    let up_command = UpCommand::new(client_id.clone(), down_command.parity_id.clone(), down_command.priority.clone(), "C210".to_string());
    enhanced_buffer::buffer_up_mananger::buffer_up_schedule(up_command);

    *client_id = redirect_to.to_string(); // > Update the client id that it will send to

    println!("Converted redirect command: {:?}", converted_m);

    if !converted_m.contains_key("kwargs") {
        create_error_response_and_return!("Error! Callback response args don't have response kwarg!", converted_m, to_send);
        // return error_response!("Error! Callback response args don't have response kwarg!");
    }

    let mut resp: HashMap<String, ResultType> = HashMap::new();

    let response_act_fn_value = converted_m.get("response_activation_function").unwrap().clone();
    let response_act_fn: String = serde_json::from_value(response_act_fn_value).unwrap();

    to_send.insert("kwargs".to_string(), m.get("kwargs").unwrap().clone());
    to_send.insert("response_activation_function".to_string(), ResultType::Str(response_act_fn.to_string()));
    to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));

    // {'response_mode':'to_origin', 'response_activation_function':response_activation_function, 'response':response}
    // {"response": Map({"data": Str("hello!")}), "response_activation_function": Str("test_handler"), "response_mode": Str("to_origin")}

    resp.insert("response".to_string(), ResultType::Map(to_send));

    return resp;
}

fn handle_internal_mannangment(m: HashMap<String, ResultType>, client_id: &mut String) -> HashMap<String, ResultType> {
    let mut to_send = HashMap::new();

    let converted_m = convert_to_value_map(&m);

    if !m.contains_key("response_mode") {
        create_error_response_and_return!("Error! Callback response args don't have response_mode kwarg!", converted_m, to_send);
    } else if !m.contains_key("activation_function") {
        create_error_response_and_return!("Error! Callback response args don't have activation_function kwarg!", converted_m, to_send);
    } else if !m.contains_key("kwargs") {
        create_error_response_and_return!("Error! Callback response args don't have kwargs kwarg!", converted_m, to_send);
    }

    let activation_function: String = serde_json::from_value(converted_m.get("activation_function").unwrap().clone()).unwrap();

    let kwargs: ResultType = m.get("kwargs").unwrap().clone();

    match activation_function.as_str() {
        "add_client" => {
            // > edit client
            // {'response_mode':'InternalMannangement', 'activation_function':'add_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'new_client':clientpattern}

            if let ResultType::Map(inner_map) = kwargs {
                if !inner_map.contains_key("new_client") {
                    create_error_response_and_return!("Error! Callback response kwargs don't have new_client kwarg!", converted_m, to_send);
                }
                // TODO >>> Add the case where need to add the client
            } else {
                create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            }
        },

        "update_client" => {
            // > update client
            // {'response_mode':'InternalMannangement', 'activation_function':'update_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'actual_client_key':String, 'updated_client':client} // Client have to have the same client key
            // 'client': {"client_name":str, "client_key":str, "client_type":str, "permission_group":str, "is_super_user":bool, "max_sub_channels":int, "owned_sub_channels_keys":list}

            if let ResultType::Map(inner_map) = kwargs {
                if !inner_map.contains_key("actual_client_key") {
                    create_error_response_and_return!("Error! Callback response kwargs don't have actual_client_key kwarg!", converted_m, to_send);
                }

                if !inner_map.contains_key("updated_client") {
                    create_error_response_and_return!("Error! Callback response kwargs don't have update_client kwarg!", converted_m, to_send);
                }

                // TODO >>> Add the case where need to update the client

                let result = kwargs.to_map().unwrap();
                let actual_client_key: String = result.get("actual_client_key").unwrap().clone().to_string();
                let updated_client = result.get("updated_client").unwrap().clone();

                // from("client_name":"str", "client_key":"str", "client_type":"str", "permission_group":"str", "is_super_user":"bool", "max_sub_channels":"int", "owned_sub_channels_keys":"list")

                let mut expected = HashMap::new();

                expected.insert("client_name".to_string(), ResultType::Str("".to_string()));
                expected.insert("client_key".to_string(), ResultType::Str("".to_string()));
                expected.insert("client_type".to_string(), ResultType::Str("".to_string()));
                expected.insert("permission_group".to_string(), ResultType::Str("".to_string()));
                expected.insert("is_super_user".to_string(), ResultType::Bool(false));
                expected.insert("max_sub_channels".to_string(), ResultType::Int(0));
                expected.insert("owned_sub_channels_keys".to_string(), ResultType::List(Vec::new()));

                let expectation_result = updated_client.fast_verify_kwargs_and_types(&ResultType::Map(expected));

                match expectation_result {
                    Err(e) => match e {
                        ExpectationError::MismatchType(tp) => {
                            // TODO >>> Return the error MismatchType to client
                        },
                        ExpectationError::MismatchRelativeLength => {
                            // TODO >>> Return the MismatchRelativeLength error to client
                        },
                        ExpectationError::Missingkwarg(k) => {
                            // TODO >>> Return the error Missingkwarg to client
                        },
                        ExpectationError::TargetIsEmpty => {
                            // TODO >>> Return the error TargetIsEmpty to client
                        },
                        ExpectationError::MismatchRelativeLength => {
                            // TODO >>> Return the error MismatchRelativeLength to client
                        },
                    },

                    Ok(_) => {},
                }

                // let updated_client = Client::new(
                //     client_new_name,
                //     client_new_key,
                //     client_new_type,
                //     client_new_perm_group,
                //     client_is_super_user,
                //     client_max_allowed_sub_channels,
                //     client_owned_sub_hannels_keys,
                // );

                // TODO >>> Maybe implement a fast resultype to client if needed
            } else {
                create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            }
        },

        "remove_client" => {
            // > remove client
            // {'response_mode':'InternalMannangement', 'activation_function':'remove_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'client_key':String}

            if let ResultType::Map(inner_map) = kwargs {
                if !inner_map.contains_key("client_key") {
                    create_error_response_and_return!("Error! Callback response kwargs don't have client_key kwarg!", converted_m, to_send);
                }

                let client_key: String = inner_map.get("client_key");

                // TODO >>> Add the case where need to update the client
            } else {
                create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            }
        },

        _ => {},
    }

    // TODO >>> Add the cases to handle the following internal mannangement things:

    //* Need to implement the 'response_activation_function' in the wrapper

    let mut resp: HashMap<String, ResultType> = HashMap::new();

    resp.insert("Error".to_string(), ResultType::Str(format!("Sussefuly executed the function: {}!", activation_function).to_string()));

    to_send.insert("response".to_string(), ResultType::Map(resp));
    to_send.insert("response_activation_function".to_string(), ResultType::Str(converted_m.get("response_activation_function").unwrap().to_string()));
    to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));

    return to_send;
}

fn process(py: Python, down_command: DownCommand) {
    let logger = acquire_logger!("Transposer - Process");

    logger.debug(format!("Initializing prossesing!"));

    let command_is_not_registry: bool = enhanced_buffer::buffer_up_mananger::check_if_parity_id_is_registred(down_command.parity_id.clone());
    let command_id: u32 = down_command.command_id.clone().unwrap();

    if !command_is_not_registry {
        logger.debug(format!("Command {}, alwready have a response!", down_command.parity_id.clone()));
        enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id);
        return;
    }

    // TODO >>> Use the command.command or create a require type field to redirect the command to another client

    // -> One idea is to create a obrigatory key in the command.command and instead of only function create a type kwarg field
    // > Type can be:
    // >    - same as origin
    // >    - redirect

    // > if it is redirect one extra kwarg is necessary that have the client_id to redirect
    // * This will create a need to have a local database in the host to store the clients
    // * and to store when is the last contact of some client, if it is some threshold value
    // * more it will remove the registred client, if it have a contact recent, this will redirect the message
    // * however if the message is becames too old before the client the message is redirected catches it
    // * The system have to remove this old message from the buffer too.

    let translated_command: Command = Command::from_down_command(down_command.clone());

    logger.debug(format!("Translated command: {:?}", translated_command));

    let function = match translated_command.command.get("function") {
        Some(Value::String(function)) => function,
        _ => {
            logger.warn(format!("The function name is not found or not a string."));
            return;
        },
    };

    let command_patterns = COMMAND_PATTERNS.lock().unwrap().clone();
    let patterns = command_patterns;

    // -> Remove command from schedule if it isn't on the patterns
    if !patterns.contains_key(function) {
        logger.warn(format!("Command isn't registred in the patterns"));
        enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id.clone());
        logger.warn(format!("command skipped and remvoed from schedule"));
        return;
    }

    let response;

    {
        let callback_patterns = CALLBACK_PATTERNS.lock().unwrap();
        response = call_callback(py, translated_command.clone(), callback_patterns);
    }

    let result = match response {
        Ok(r) => extract_pyobject(py, r),
        Err(e) => {
            // Handle the error or log it
            eprintln!("Python error: {:?}", e);
            // You can return a default value or propagate the error further
            ResultType::Error(format!("{:?}", e))
        },
    };

    let mut client_id = down_command.client_id.clone();

    let response;

    match result {
        ResultType::Map(m) => {
            if m.contains_key("response_mode") {
                let response_mode = m.get("response_mode").unwrap();

                if *response_mode == ResultType::Str("to_origin".to_string()) {
                    let converted_to_value = convert_to_value_map(&m);
                    println!("Converted to Value: {:?}", &converted_to_value);
                    response = Ok(serde_json::to_string(&converted_to_value).unwrap());
                } else if *response_mode == ResultType::Str("redirect".to_string()) {
                    println!("Response: {:?}", m);
                    let resp = handle_redirect(m, &mut client_id, down_command.clone());
                    let converted_to_value = convert_to_value_map(&resp);
                    response = Ok(serde_json::to_string(&converted_to_value).unwrap());
                } else if *response_mode == ResultType::Str("InternalMannangement".to_string()) {
                    let resp = handle_internal_mannangment(m, &mut client_id);
                    let converted_to_value = convert_to_value_map(&resp);
                    response = Ok(serde_json::to_string(&converted_to_value).unwrap());
                } else {
                    response = error_response!("Error! Response mode doesn't match any known mode. Please use one of: ('to_origin', 'redirect')!");
                }
            } else {
                response = error_response!("Error! Callback doesn't implement response mode!");
            }
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
        ResultType::List(_) => {
            response = error_response!("Error! Received a list, but expected a map!");
        },
        ResultType::Empty => {
            response = Ok(serde_json::to_string(&"C210".to_string()).unwrap());
        },
        ResultType::Error(e) => {
            response = error_response!(format!("An error occurred while converting the Python callback response. The error was: {:?}", e));
        },
    }

    logger.debug(format!("Function returned: {:?}", response));
    logger.info(format!("Command: {:?}, processed!", down_command.parity_id.clone()));

    enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id.clone());

    let up_command = UpCommand::new(client_id, down_command.parity_id.clone(), down_command.priority.clone(), response.unwrap());

    enhanced_buffer::buffer_up_mananger::buffer_up_schedule(up_command);
}

fn clear_old_data() {
    enhanced_buffer::buffer_down_mananger::buffer_down_clear_old_commands();
    enhanced_buffer::buffer_up_mananger::buffer_up_clear_old_commands();
}

pub fn initialize_socket_host_transposer(py: Python<'_>) {
    let logger = acquire_logger!("Transposer");

    let mut schedule: Vec<DownCommand> = enhanced_buffer::buffer_down_mananger::buffer_down_list_schedule();

    schedule.sort_by(|a, b| b.priority.cmp(&a.priority)); // put the schedule in crescent order

    // logger.debug(format!("Schedule to process:\n{:?}\n", schedule));

    if !(schedule.len() > 0) {
        // logger.debug(format!("Nothing in the schedule, skipping >>>"));
        clear_old_data();
        thread::sleep(Duration::from_millis(500));
        return;
    }

    logger.info(format!("Data found in schedule!"));

    for dow_command in schedule {
        let logger = acquire_logger!("Transposer");

        logger.info(format!("get a pool worker in tranposer!"));

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
