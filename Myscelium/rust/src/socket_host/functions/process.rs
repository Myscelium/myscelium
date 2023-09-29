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
        let mut resp: HashMap<String, ResultType> = HashMap::new();
        resp.insert("Error".to_string(), ResultType::Str($error_msg.to_string()));

        $to_send.insert("response".to_string(), ResultType::Map(resp));
        $to_send.insert("response_activation_function".to_string(), ResultType::Str($converted_m.get("response_activation_function").unwrap().to_string()));
        $to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));

        $to_send
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
    let mut to_send = HashMap::new();

    let converted_m = convert_to_value_map(&m);

    if !m.contains_key("redirect_to") {
        return create_error_response_and_return!("Error! Callback response args don't have redirect_to client_id field!", converted_m, to_send);
        // error_response!("Error! Callback response args don't have redirect_to client_id field!");
    }

    let redirect_to_value = converted_m.get("redirect_to").unwrap().clone();
    let redirect_to: String = serde_json::from_value(redirect_to_value).unwrap();

    if !check_if_client_key_exists(redirect_to.to_string()) {
        return create_error_response_and_return!(format!("Error! request to redirect to client_id: {} failed, client doesn't exist!", redirect_to.to_string()), converted_m, to_send);
        // return error_response!(format!("Error! request to redirect to client_id: {} failed, client doesn't exist!", redirect_to.to_string()));
    }

    let up_command = UpCommand::new(client_id.clone(), down_command.parity_id.clone(), down_command.priority.clone(), "C210".to_string());
    enhanced_buffer::buffer_up_mananger::buffer_up_schedule(up_command);

    *client_id = redirect_to.to_string(); // > Update the client id that it will send to

    println!("Converted redirect command: {:?}", converted_m);

    if !converted_m.contains_key("kwargs") {
        return create_error_response_and_return!("Error! Callback response args don't have response kwarg!", converted_m, to_send);
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

// > --------------------------------------------------------------------------------------------------------------------------------------------
// > Internal Mannangement Handler

pub fn handle_internal_mannangment(m: HashMap<String, ResultType>, client_id: &mut String) -> HashMap<String, ResultType> {
    let mut to_send = HashMap::new();

    let converted_m = convert_to_value_map(&m);

    if !m.contains_key("response_mode") {
        return create_error_response_and_return!("Error! Callback response args don't have response_mode kwarg!", converted_m, to_send);
    } else if !m.contains_key("activation_function") {
        return create_error_response_and_return!("Error! Callback response args don't have activation_function kwarg!", converted_m, to_send);
    } else if !m.contains_key("kwargs") {
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
                    return create_error_response_and_return!("Error! Callback response kwargs don't have new_client kwarg!", converted_m, to_send);
                }

                // TODO >>> Add the case where need to add the client

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
                expected.insert("owned_sub_channels_keys".to_string(), ResultType::List(Vec::new()));

                let expectation_result: Result<(), ExpectationError> = new_client.fast_verify_kwargs_and_types(&ResultType::Map(expected));

                match expectation_result {
                    Err(e) => match e {
                        ExpectationError::MismatchType(tp) => {
                            println!("ERROR, Client kwargs have mismatch type {} kwarg!", tp);
                            return create_error_response_and_return!(format!("Error! Client kwargs have mismatch type {} kwarg!", tp), converted_m, to_send);
                        },
                        ExpectationError::MismatchRelativeLength => {
                            println!("ERROR, Client kwargs have mismatch relative length kwargs!");
                            return create_error_response_and_return!("Error! Client kwargs have mismatch relative length kwargs!", converted_m, to_send);
                        },
                        ExpectationError::Missingkwarg(k) => {
                            println!("ERROR, Client kwargs have a missing kwarg: {}!", k);
                            return create_error_response_and_return!(format!("Error! Client kwargs have a missing kwarg: {}!", k), converted_m, to_send);
                        },
                        ExpectationError::TargetIsEmpty => {
                            println!("ERROR, Client target pattern is empty!");
                            return create_error_response_and_return!("Error! Client target pattern is empty!", converted_m, to_send);
                        },
                    },

                    Ok(_) => {
                        println!("Command add_client received matches the expectations!");
                    },
                }

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

                println!("New client: {:?}", new_client);

                new_client.save_into_db(); //> It Alwready create the new client

                println!("New client saved into the database!");

                let mut resp: HashMap<String, ResultType> = HashMap::new();

                resp.insert("Success".to_string(), ResultType::Str(format!("Sussefuly add a client: {}!", client_key).to_string()));

                to_send.insert("response".to_string(), ResultType::Map(resp));
                to_send.insert("response_activation_function".to_string(), ResultType::Str(converted_m.get("add_client_handler").unwrap().to_string()));
                to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));

                return to_send;
            } else {
                return create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            }
        },

        "update_client" => {
            // > update client
            // {'response_mode':'InternalMannangement', 'activation_function':'update_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'actual_client_key':String, 'updated_client':client} // Client have to have the same client key
            // 'client': {"client_name":str, "client_key":str, "client_type":str, "permission_group":str, "is_super_user":bool, "max_sub_channels":int, "owned_sub_channels_keys":list}

            println!("Receive a update client inner command!");

            if let ResultType::Map(inner_map) = &kwargs {
                if !inner_map.contains_key("actual_client_key") {
                    return create_error_response_and_return!("Error! Callback response kwargs don't have actual_client_key kwarg!", converted_m, to_send);
                }

                if !inner_map.contains_key("updated_client") {
                    println!("ERROR, Error! Callback response kwargs don't have update_client kwarg!");
                    return create_error_response_and_return!("Error! Callback response kwargs don't have update_client kwarg!", converted_m, to_send);
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
                            println!("ERROR, Client kwargs have mismatch type {} kwarg!", tp);
                            return create_error_response_and_return!(format!("Error! Client kwargs have mismatch type {} kwarg!", tp), converted_m, to_send);
                        },
                        ExpectationError::MismatchRelativeLength => {
                            println!("ERROR, Client kwargs have mismatch relative length kwargs!");
                            return create_error_response_and_return!("Error! Client kwargs have mismatch relative length kwargs!", converted_m, to_send);
                        },
                        ExpectationError::Missingkwarg(k) => {
                            println!("ERROR, Client kwargs have a missing kwarg: {}!", k);
                            return create_error_response_and_return!(format!("Error! Client kwargs have a missing kwarg: {}!", k), converted_m, to_send);
                        },
                        ExpectationError::TargetIsEmpty => {
                            println!("ERROR, Client target pattern is empty!");
                            return create_error_response_and_return!("Error! Client target pattern is empty!", converted_m, to_send);
                        },
                    },

                    Ok(_) => {},
                }

                let updated_client_unwraped: HashMap<String, ResultType> = updated_client.to_map().unwrap();

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

                let old_client = handle_client_error!(Client::get_by_key(&actual_client_key));

                let result = old_client.update_to(&new_client); //> It alwready saves into the database

                // TODO >>> Maybe implement a fast resultype to client if needed

                match result {
                    Ok(c) => {
                        let mut resp: HashMap<String, ResultType> = HashMap::new();

                        resp.insert(
                            "Success".to_string(),
                            ResultType::Str(format!("Sussefuly executed the function: {} and remove client: {}!", activation_function, client_key).to_string()),
                        );

                        to_send.insert("response".to_string(), ResultType::Map(resp));
                        to_send.insert("response_activation_function".to_string(), ResultType::Str(converted_m.get("update_client_handler").unwrap().to_string()));
                        to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));

                        return to_send;
                    },

                    Err(e) => match e {
                        ClientError::ClientDoesNotExist(e) => return create_error_response_and_return!(format!("Error! Can't Update client because client {} Don't exist!", e), converted_m, to_send.clone()),
                        _ => return create_error_response_and_return!("Error! Can Update client because a unexpected error!", converted_m, to_send.clone()),
                    },
                }
            } else {
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

                let client_key = inner_map.get("client_key").unwrap().to_string();

                let client = handle_client_error!(Client::get_by_key(&client_key));

                let result = client.delete();

                match result {
                    Err(e) => match e {
                        ClientError::ClientDoesNotExist(e) => return create_error_response_and_return!(format!("Error! Can't Remove client because client {} Don't exist!", e), converted_m, to_send.clone()),
                        _ => return create_error_response_and_return!("Error! Can Remove client because a unexpected error!", converted_m, to_send.clone()),
                    },
                    Ok(_) => {
                        let mut resp: HashMap<String, ResultType> = HashMap::new();

                        resp.insert(
                            "Success".to_string(),
                            ResultType::Str(format!("Sussefuly executed the function: {} and remove client: {}!", activation_function, client_key).to_string()),
                        );

                        to_send.insert("response".to_string(), ResultType::Map(resp));
                        to_send.insert("response_activation_function".to_string(), ResultType::Str(converted_m.get("remove_client_handler").unwrap().to_string()));
                        to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));

                        return to_send;
                    },
                }
            } else {
                return create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            }
        },

        _ => {
            let mut resp: HashMap<String, ResultType> = HashMap::new();

            resp.insert("Success".to_string(), ResultType::Str(format!("Response Activation Function: {} doesn't exists!!", activation_function).to_string()));

            to_send.insert("response".to_string(), ResultType::Map(resp));
            to_send.insert("response_activation_function".to_string(), ResultType::Str(converted_m.get("response_activation_function").unwrap().to_string()));
            to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));

            return to_send;
        },
    }

    // TODO >>> Add the cases to handle the following internal mannangement things:

    //* Need to implement the 'response_activation_function' in the wrapper

    let mut resp: HashMap<String, ResultType> = HashMap::new();

    resp.insert("Success".to_string(), ResultType::Str(format!("Sussefuly executed the function: {}!", activation_function).to_string()));

    to_send.insert("response".to_string(), ResultType::Map(resp));
    to_send.insert("response_activation_function".to_string(), ResultType::Str(converted_m.get("response_activation_function").unwrap().to_string()));
    to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));

    return to_send;
}
