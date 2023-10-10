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

use crate::socket_host::client_mananger::mananger::{Client, ClientError};

#[macro_use]
use crate::handle_client_error;

use crate::commom::structs::results_structs::ExpectationError;

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
pub fn handle_redirect(m: HashMap<String, ResultType>, client_id: &mut String, down_command: DownCommand) -> HashMap<String, ResultType> {
    let logger = acquire_logger!("[Process][Handle Redirect]");

    let mut to_send = HashMap::new();

    let converted_m = convert_to_value_map(&m);

    if !m.contains_key("redirect_to") {
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

    let mut command_map = HashMap::new();
    command_map.insert("command_type".to_string(), Value::String("special_function".to_string()));
    command_map.insert("function".to_string(), Value::String("C210".to_string()));
    let response = serde_json::to_string(&command_map).unwrap();

    let up_command = UpCommand::new(client_id.clone(), down_command.parity_id.clone(), down_command.priority.clone(), response);
    enhanced_buffer::buffer_up_mananger::buffer_up_schedule(up_command);

    *client_id = redirect_to.to_string(); // > Update the client id that it will send to

    logger.debug(format!("Converted redirect command: {:?}", converted_m));

    if !converted_m.contains_key("kwargs") {
        logger.warn("Error! Callback response args don't have response kwarg!".to_string());
        return create_error_response_and_return!("Error! Callback response args don't have response kwarg!", converted_m, to_send);
        // return error_response!("Error! Callback response args don't have response kwarg!");
    }

    let response_act_fn_value = converted_m.get("response_activation_function").unwrap().clone();
    let function: String = serde_json::from_value(response_act_fn_value).unwrap();

    to_send.insert("command_type".to_string(), ResultType::Str("function".to_string()));
    to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
    to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
    to_send.insert("function".to_string(), ResultType::Str(function.to_string()));
    to_send.insert("kwargs".to_string(), m.get("kwargs").unwrap().clone());
    // to_send.insert("message".to_string(), ResultType::Str($error_msg.to_string()));

    // {'response_mode':'to_origin', 'response_activation_function':response_activation_function, 'response':response}
    // {"response": Map({"data": Str("hello!")}), "response_activation_function": Str("test_handler"), "response_mode": Str("to_origin")}

    return to_send;
}

// > --------------------------------------------------------------------------------------------------------------------------------------------
// > Internal Mannangement Handler

pub fn handle_internal_mannangment(m: HashMap<String, ResultType>, client_id: &mut String) -> HashMap<String, ResultType> {
    let logger = acquire_logger!("[Process][Internal Manangement]");

    let mut to_send = HashMap::new();

    let converted_m = convert_to_value_map(&m);

    logger.debug(format!("Converted m: {:?}", &converted_m));

    if !m.contains_key("response_mode") {
        logger.warn("Error! Callback response args don't have response_mode kwarg!".to_string());
        return create_error_response_and_return!("Error! Callback response args don't have response_mode kwarg!", converted_m, to_send);
    } else if !m.contains_key("activation_function") {
        logger.warn("Error! Callback response args don't have activation_function kwarg!".to_string());
        return create_error_response_and_return!("Error! Callback response args don't have activation_function kwarg!", converted_m, to_send);
    } else if !m.contains_key("kwargs") {
        logger.warn("Error! Callback response args don't have kwargs kwarg!".to_string());
        return create_error_response_and_return!("Error! Callback response args don't have kwargs kwarg!", converted_m, to_send);
    }

    let activation_function: String = serde_json::from_value(converted_m.get("activation_function").unwrap().clone()).unwrap();

    let kwargs: ResultType = m.get("kwargs").unwrap().clone();

    match activation_function.as_str() {
        "add_client" => {
            // > edit client
            // {'response_mode':'InternalMannangement', 'activation_function':'add_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'new_client':clientpattern}

            if let ResultType::Map(inner_map) = &kwargs {
                if !inner_map.contains_key("new_client") {
                    logger.warn("Error! Callback response kwargs don't have new_client kwarg!".to_string());
                    return create_error_response_and_return!("Error! Callback response kwargs don't have new_client kwarg!", converted_m, to_send);
                }

                let result: HashMap<String, ResultType> = kwargs.to_map().unwrap();
                let new_client: ResultType = result.get("new_client").unwrap().clone();

                // from("client_name":"str", "client_key":"str", "client_type":"str", "permission_group":"str", "is_super_user":"bool", "max_sub_channels":"int", "owned_sub_channels_keys":"list")

                let mut expected: HashMap<String, ResultType> = HashMap::new();

                expected.insert("client_name".to_string(), ResultType::Str("".to_string()));
                expected.insert("client_key".to_string(), ResultType::Str("".to_string()));
                expected.insert("client_type".to_string(), ResultType::Str("".to_string()));
                expected.insert("permission_group".to_string(), ResultType::Str("".to_string()));
                expected.insert("is_super_user".to_string(), ResultType::Bool(false));
                expected.insert("max_sub_channels".to_string(), ResultType::Int(0));
                expected.insert("owned_sub_channels_keys".to_string(), ResultType::List(vec![]));

                let parsed_new_client = new_client.fast_parse(&ResultType::Map(expected.clone()));

                let new_client = match parsed_new_client {
                    Err(e) => match e {
                        ExpectationError::MismatchType(tp) => {
                            logger.warn(format!("ERROR, Client kwargs have mismatch type {} kwarg!", tp));
                            return create_error_response_and_return!(format!("Error! Client kwargs have mismatch type {} kwarg!", tp), converted_m, to_send);
                        },
                        ExpectationError::MismatchRelativeLength => {
                            logger.warn("ERROR, Client kwargs have mismatch relative length kwargs!".to_string());
                            return create_error_response_and_return!("Error! Client kwargs have mismatch relative length kwargs!", converted_m, to_send);
                        },
                        ExpectationError::Missingkwarg(k) => {
                            logger.warn(format!("ERROR, Client kwargs have a missing kwarg: {}!", k));
                            return create_error_response_and_return!(format!("Error! Client kwargs have a missing kwarg: {}!", k), converted_m, to_send);
                        },
                        ExpectationError::TargetIsEmpty => {
                            logger.warn("ERROR, Client target pattern is empty!".to_string());
                            return create_error_response_and_return!("Error! Client target pattern is empty!", converted_m, to_send);
                        },
                    },

                    Ok(new_client) => new_client,
                };

                let updated_client_unwraped: HashMap<String, ResultType> = new_client.to_map().unwrap();

                let owned_sub_channels_keys: Vec<String> = updated_client_unwraped.get("owned_sub_channels_keys").unwrap().to_list().unwrap().iter().map(|v| v.to_str().unwrap()).collect();

                let client_key = updated_client_unwraped.get("client_key").unwrap().to_str().unwrap();

                let new_client = handle_client_error!(Client::new(
                    updated_client_unwraped.get("client_name").unwrap().to_str().unwrap(),
                    client_key.clone(),
                    updated_client_unwraped.get("client_type").unwrap().to_str().unwrap(),
                    updated_client_unwraped.get("permission_group").unwrap().to_str().unwrap(),
                    updated_client_unwraped.get("is_super_user").unwrap().to_bool().unwrap(),
                    updated_client_unwraped.get("max_sub_channels").unwrap().to_int().unwrap() as u32,
                    owned_sub_channels_keys,
                ));

                logger.debug(format!("New client: {:?}", new_client));

                new_client.save_into_db(); //> It Alwready create the new client

                logger.debug("New client saved into the database!".to_string());

                let mut resp_kwargs: HashMap<String, ResultType> = HashMap::new();

                resp_kwargs.insert("actual_client_key".to_string(), ResultType::Str(client_key.to_string()));

                to_send.insert("command_type".to_string(), ResultType::Str("response".to_string()));
                to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
                to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
                to_send.insert("message".to_string(), ResultType::Str(format!("Sussefuly add a client: {}!", client_key).to_string()));
                to_send.insert("response_activation_function".to_string(), ResultType::Str("add_client_handler".to_string()));
                to_send.insert("kwargs".to_string(), ResultType::Map(resp_kwargs));

                logger.info(format!("Sussefuly add a client: {}!", client_key));

                // TODO >>> Implement a mecanism to send back the confirmation or a error message originated from the operation

                return to_send;
            } else {
                logger.warn("Error! Callback response kwargs isn't a Map!".to_string());
                return create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            }
        },

        "update_client" => {
            // > update client
            // {'response_mode':'InternalMannangement', 'activation_function':'update_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'actual_client_key':String, 'updated_client':client} // Client have to have the same client key
            // 'client': {"client_name":str, "client_key":str, "client_type":str, "permission_group":str, "is_super_user":bool, "max_sub_channels":int, "owned_sub_channels_keys":list}

            logger.debug("Receive a update client inner command!".to_string());

            if let ResultType::Map(inner_map) = &kwargs {
                if !inner_map.contains_key("actual_client_key") {
                    logger.warn("Error! Callback response kwargs don't have actual_client_key kwarg!".to_string());
                    return create_error_response_and_return!("Error! Callback response kwargs don't have actual_client_key kwarg!", converted_m, to_send);
                }

                if !inner_map.contains_key("updated_client") {
                    logger.warn("ERROR, Error! Callback response kwargs don't have update_client kwarg!".to_string());
                    return create_error_response_and_return!("Error! Callback response kwargs don't have update_client kwarg!", converted_m, to_send);
                }

                let result = kwargs.to_map().unwrap();
                let actual_client_key = result.get("actual_client_key").unwrap().to_str().unwrap();
                let updated_client = result.get("updated_client").unwrap().clone();

                // from("client_name":"str", "client_key":"str", "client_type":"str", "permission_group":"str", "is_super_user":"bool", "max_sub_channels":"int", "owned_sub_channels_keys":"list")

                let mut expected = HashMap::new();

                expected.insert("client_name".to_string(), ResultType::Str("".to_string()));
                expected.insert("client_key".to_string(), ResultType::Str("".to_string()));
                expected.insert("client_type".to_string(), ResultType::Str("".to_string()));
                expected.insert("permission_group".to_string(), ResultType::Str("".to_string()));
                expected.insert("is_super_user".to_string(), ResultType::Bool(false));
                expected.insert("max_sub_channels".to_string(), ResultType::Int(0));
                expected.insert("owned_sub_channels_keys".to_string(), ResultType::List(vec![]));

                let parsed_new_client = updated_client.fast_parse(&ResultType::Map(expected.clone()));

                let new_client = match parsed_new_client {
                    Err(e) => match e {
                        ExpectationError::MismatchType(tp) => {
                            logger.warn(format!("ERROR, Client kwargs have mismatch type {} kwarg!", tp));
                            return create_error_response_and_return!(format!("Error! Client kwargs have mismatch type {} kwarg!", tp), converted_m, to_send);
                        },
                        ExpectationError::MismatchRelativeLength => {
                            logger.warn("ERROR, Client kwargs have mismatch relative length kwargs!".to_string());
                            return create_error_response_and_return!("Error! Client kwargs have mismatch relative length kwargs!", converted_m, to_send);
                        },
                        ExpectationError::Missingkwarg(k) => {
                            logger.warn(format!("ERROR, Client kwargs have a missing kwarg: {}!", k));
                            return create_error_response_and_return!(format!("Error! Client kwargs have a missing kwarg: {}!", k), converted_m, to_send);
                        },
                        ExpectationError::TargetIsEmpty => {
                            logger.warn("ERROR, Client target pattern is empty!".to_string());
                            return create_error_response_and_return!("Error! Client target pattern is empty!", converted_m, to_send);
                        },
                    },

                    Ok(new_client) => new_client,
                };

                let updated_client_unwraped: HashMap<String, ResultType> = new_client.to_map().unwrap();

                let owned_sub_channels_keys: Vec<String> = updated_client_unwraped.get("owned_sub_channels_keys").unwrap().to_list().unwrap().iter().map(|v| v.to_str().unwrap()).collect();

                let client_key = updated_client_unwraped.get("client_key").unwrap().to_str().unwrap();

                logger.debug(format!("Updated client: {:?}", new_client));

                let new_client = handle_client_error!(Client::new(
                    updated_client_unwraped.get("client_name").unwrap().to_str().unwrap(),
                    client_key.clone(),
                    updated_client_unwraped.get("client_type").unwrap().to_str().unwrap(),
                    updated_client_unwraped.get("permission_group").unwrap().to_str().unwrap(),
                    updated_client_unwraped.get("is_super_user").unwrap().to_bool().unwrap(),
                    updated_client_unwraped.get("max_sub_channels").unwrap().to_int().unwrap() as u32,
                    owned_sub_channels_keys,
                ));

                let old_client = handle_client_error!(Client::get_by_key(&actual_client_key));

                let result = old_client.update_to(&new_client); //> It alwready saves into the database

                // TODO >>> Maybe implement a fast resultype to client if needed

                match result {
                    Ok(c) => {
                        to_send.insert("command_type".to_string(), ResultType::Str("response".to_string()));
                        to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
                        to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
                        to_send.insert(
                            "message".to_string(),
                            ResultType::Str(format!("Sussefuly executed the function: {} and remove client: {}!", activation_function, client_key).to_string()),
                        );
                        to_send.insert("response_activation_function".to_string(), ResultType::Str("update_client_handler".to_string()));

                        let mut resp_kwargs: HashMap<String, ResultType> = HashMap::new();

                        resp_kwargs.insert("actual_client_key".to_string(), ResultType::Str(client_key.to_string()));

                        to_send.insert("kwargs".to_string(), ResultType::Map(resp_kwargs));

                        logger.info(format!("Sussefuly executed the function: {} and remove client: {}!", activation_function, client_key));

                        return to_send;
                    },

                    Err(e) => match e {
                        ClientError::ClientDoesNotExist(e) => {
                            logger.warn(format!("Error! Can't Update client because client {} Don't exist!", e));
                            return create_error_response_and_return!(format!("Error! Can't Update client because client {} Don't exist!", e), converted_m, to_send.clone());
                        },
                        _ => {
                            logger.warn("Error! Can Update client because a unexpected error!".to_string());
                            return create_error_response_and_return!("Error! Can Update client because a unexpected error!", converted_m, to_send.clone());
                        },
                    },
                }

                // TODO >>> Implement a mecanism to send back the confirmation or a error message originated from the operation
            } else {
                logger.warn("Error! Callback response kwargs isn't a Map!".to_string());
                return create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            }
        },

        "remove_client" => {
            // > remove client
            // {'response_mode':'InternalMannangement', 'activation_function':'remove_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'client_key':String}

            if let ResultType::Map(inner_map) = &kwargs {
                if !inner_map.contains_key("client_key") {
                    return create_error_response_and_return!("Error! Callback response kwargs don't have client_key kwarg!", converted_m, to_send);
                }

                let client_key = inner_map.get("client_key").unwrap().to_str().unwrap();

                let client = handle_client_error!(Client::get_by_key(&client_key));

                let result = client.delete();

                match result {
                    Err(e) => match e {
                        ClientError::ClientDoesNotExist(e) => {
                            logger.warn(format!("Error! Can't Remove client because client {} Don't exist!", e));
                            return create_error_response_and_return!(format!("Error! Can't Remove client because client {} Don't exist!", e), converted_m, to_send.clone());
                        },
                        _ => {
                            logger.warn("Error! Can Remove client because a unexpected error!".to_string());
                            return create_error_response_and_return!("Error! Can Remove client because a unexpected error!", converted_m, to_send.clone());
                        },
                    },
                    Ok(_) => {
                        to_send.insert("command_type".to_string(), ResultType::Str("response".to_string()));
                        to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
                        to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
                        to_send.insert(
                            "message".to_string(),
                            ResultType::Str(format!("Sussefuly executed the function: {} and remove client: {}!", activation_function, client_key).to_string()),
                        );
                        to_send.insert("response_activation_function".to_string(), ResultType::Str("remove_client_handler".to_string()));

                        let mut resp_kwargs: HashMap<String, ResultType> = HashMap::new();

                        resp_kwargs.insert("actual_client_key".to_string(), ResultType::Str(client_key.to_string()));

                        to_send.insert("kwargs".to_string(), ResultType::Map(resp_kwargs));

                        logger.info(format!("Sussefuly executed the function: {} and remove client: {}!", activation_function, client_key));

                        return to_send;
                    },
                }
            } else {
                logger.warn("Error! Callback response kwargs isn't a Map!".to_string());
                return create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            }

            // TODO >>> Implement a mecanism to send back the confirmation or a error message originated from the operation
        },

        _ => {
            to_send.insert("command_type".to_string(), ResultType::Str("response".to_string()));
            to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
            to_send.insert("status".to_string(), ResultType::Str("error".to_string()));
            to_send.insert("message".to_string(), ResultType::Str(format!("Response Activation Function: {} doesn't exists!!", activation_function).to_string()));
            to_send.insert("response_activation_function".to_string(), ResultType::Str("response_activation_function".to_string()));

            logger.warn(format!("Response Activation Function: {} doesn't exists!!", activation_function));

            return to_send;
        },
    }

    // TODO >>> Add the cases to handle the following internal mannangement things:

    //* Need to implement the 'response_activation_function' in the wrapper
}
