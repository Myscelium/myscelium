use serde_json::Value;
use std::collections::HashMap;

use crate::common::enhanced_buffer::utilities::{CommandInstructions, CommandType};
use crate::socket_host::client_manager::manager::check_if_client_key_exists;

use crate::common::enhanced_buffer;
use crate::common::enhanced_buffer::buffer_down_manager::DownCommand;
use crate::common::enhanced_buffer::buffer_up_manager::UpCommand;

use crate::common::functions::converters::convert_to_value_map;

use crate::common::structs::results_structs::ResultType;

macro_rules! create_error_response_and_return {
    ($error_msg:expr, $converted_m:expr, $to_send:expr) => {{
        $to_send.insert("command_type".to_string(), ResultType::Str("response".to_string()));
        $to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
        $to_send.insert("status".to_string(), ResultType::Str("error".to_string()));
        $to_send.insert("response_activation_function".to_string(), ResultType::Str($converted_m.get("response_activation_function").unwrap().to_string()));
        $to_send.insert("message".to_string(), ResultType::Str($error_msg.to_string()));
        $to_send
    }};
}

use crate::socket_host::host_logger::log_handler::Logger;
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

//> ------------------------------------------------------------------------------------------------------------------------------------------------
//> Handle Redirect

/// Handles redirection logic for incoming commands.
///
/// This function processes an incoming command, checks if it contains
/// the necessary keys for redirection, and updates the client ID to which
/// future commands will be sent. It also schedules an `UpCommand` based on
/// the provided `DownCommand`.
///
/// # Parameters
///
/// - `m`: A `HashMap` representing the incoming command. It should contain
///   keys and values represented as `ResultType` variants.
/// - `client_id`: A mutable reference to the client ID. This ID will be updated
///   if the redirection is successful.
/// - `down_command`: The `DownCommand` based on which an `UpCommand` will be scheduled.
///
/// # Returns
///
/// A `HashMap` containing the response. If there's an error during the processing,
/// an error response will be returned with a corresponding message.
///
/// # Errors
///
/// The function can return error responses in the following scenarios:
///
/// - The incoming command does not contain the "redirect_to" key.
/// - The specified client to redirect to does not exist.
/// - The incoming command does not contain the "kwargs" key.
///
/// # Panics
///
/// This function can panic in the following scenarios (due to `unwrap` calls):
///
/// - The `redirect_to` or `response_activation_function` values are not present in `converted_m`.
/// - The `redirect_to` or `response_activation_function` values cannot be deserialized to a `String`.
///
/// # Examples
///
/// ```rust
/// let mut client_id = "client123".to_string();
/// let down_command = DownCommand::new(..=); // Initialize a DownCommand
/// let m = ...; // Initialize the HashMap command
///
/// let response = handle_redirect(m, &mut client_id, down_command);
/// ```
///
pub fn handle_redirect(m: CommandInstructions, client_id: &mut String, parity_id: String, priority: u8) -> CommandInstructions {
    let logger = acquire_logger!("[Process][Handle Redirect]");

    let mut to_send = HashMap::new();

    println!("Try to redirect: {:?}", m);

    let converted_m = convert_to_value_map(&m);

    if m.command_type != CommandType::Redirect {
        logger.warn("Error! Callback response args don't have redirect_to client_id field!".to_string());
        return create_error_response_and_return!("Error! Callback response args don't have redirect_to client_id field!", converted_m, to_send);
        // error_response!("Error! Callback response args don't have redirect_to client_id field!");
    }

    let redirect_to_value = converted_m.get("redirect_to").unwrap().clone();
    let redirect_to: String = serde_json::from_value(redirect_to_value).unwrap();

    if !check_if_client_key_exists(redirect_to.to_string()) {
        logger.warn(format!("Error! request to redirect to client_id: {} failed, client doesn't exist!", redirect_to.to_string()));
        return create_error_response_and_return!(format!("Error! request to redirect to client_id: {} failed, client doesn't exist!", redirect_to.to_string()), converted_m, to_send);
        // return error_response!(format!("Error! request to redirect to client_id: {} failed, client doesn't exist!", redirect_to.to_string()));
    }

    //> This was remove because in the cases that sends a lot of redirect this makes a spamming into the client that sends the list to retransmit:
    // let mut command_map = HashMap::new();
    // command_map.insert("command_type".to_string(), Value::String("special_function".to_string()));
    // command_map.insert("function".to_string(), Value::String("C210".to_string()));
    // let response = serde_json::to_string(&command_map).unwrap();

    // let up_command = UpCommand::new(client_id.clone(), parity_id.clone(), priority.clone(), response);
    // enhanced_buffer::buffer_up_manager::buffer_up_schedule(up_command);

    logger.debug(format!("Converted redirect command: {:?}", converted_m));

    if !converted_m.contains_key("kwargs") {
        logger.warn("Error! Callback response args don't have response kwarg!".to_string());
        return create_error_response_and_return!("Error! Callback response args don't have response kwarg!", converted_m, to_send);
        // return error_response!("Error! Callback response args don't have response kwarg!");
    }

    let response_act_fn_value = converted_m.get("response_activation_function").unwrap().clone();
    let function: String = serde_json::from_value(response_act_fn_value).unwrap();

    // TODO >>> Add a logic here to see when the redirect is to redirect a `update_available_host_commands` command and use this as a function and set response mode to to host

    to_send.insert("command_type".to_string(), ResultType::Str("function".to_string()));
    to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
    to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
    to_send.insert("function".to_string(), ResultType::Str(function.to_string()));
    to_send.insert("kwargs".to_string(), m.get("kwargs").unwrap().clone());
    to_send.insert("origin".to_string(), ResultType::Str(client_id.clone())); // -> This will be an identifier, to know the origin of the retransmited command

    // to_send.insert("message".to_string(), ResultType::Str($error_msg.to_string()));

    *client_id = redirect_to.to_string(); // > Update the client id that it will send to

    // {'response_mode':'to_origin', 'response_activation_function':response_activation_function, 'response':response}
    // {"response": Map({"data": Str("hello!")}), "response_activation_function": Str("test_handler"), "response_mode": Str("to_origin")}

    return to_send;
}
