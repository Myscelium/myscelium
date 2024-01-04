use serde_json::Value;
use std::collections::HashMap;

use crate::common::functions::converters::convert_to_value_map;

use crate::common::functions::verifiers::{fast_json_comparator, ComparatorError};
use crate::common::structs::results_structs::ResultType;

use crate::common::enhanced_buffer::utilities::{Command, CommandInstructions, CommandMode, CommandOrigin, CommandStatus, CommandTarget, CommandType};
use crate::socket_host::client_manager::manager::{Client, ClientError};
use crate::socket_host::transposer_functions::handle_direct_function::ProcessResult;

use crate::handle_client_error;

use crate::common::structs::results_structs::ExpectationError;

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

macro_rules! create_error_response_and_return {
    ($client_key:expr, $parity_id:expr, $error:expr) => {{
        let mut command_map = HashMap::new();

        let new_command_instructions = CommandInstructions::new(
            CommandMode::Response,
            CommandType::DirectFunction,
            CommandTarget::Origin,
            CommandStatus::Failure,
            CommandOrigin::Host,
            "error_handler".to_string(),
            HashMap::new(),
            $error.to_string(),
        );

        new_command_instructions
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

// > --------------------------------------------------------------------------------------------------------------------------------------------
// > Internal Management Handler

pub fn handle_internal_management(m: CommandInstructions, client_id: &mut String) -> CommandInstructions {
    let logger = acquire_logger!("[Process][Internal Management]");

    let mut to_send = HashMap::new();

    let activation_function: String = m.actf;
    let kwargs: HashMap<String, Value> = m.kwargs;

    match activation_function.as_str() {
        "add_client" => {
            // > edit client
            // {'response_mode':'InternalManagement', 'activation_function':'add_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'new_client':clientpattern}

            if !kwargs.contains_key("new_client") {
                logger.warn("Error! Callback response kwargs don't have new_client kwarg!".to_string());
                return create_error_response_and_return!("Error! Callback response kwargs don't have new_client kwarg!", m, to_send);
            }

            let new_client: Value = kwargs.get("new_client").unwrap().clone();

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

            let client_handlers: Vec<HashMap<String, Value>> = Vec::new();

            let new_client = handle_client_error!(Client::new(
                updated_client_unwraped.get("client_name").unwrap().to_str().unwrap(),
                client_key.clone(),
                updated_client_unwraped.get("client_type").unwrap().to_str().unwrap(),
                updated_client_unwraped.get("permission_group").unwrap().to_str().unwrap(),
                updated_client_unwraped.get("is_super_user").unwrap().to_bool().unwrap(),
                updated_client_unwraped.get("max_sub_channels").unwrap().to_int().unwrap() as u32,
                owned_sub_channels_keys,
                client_handlers,
            ));

            logger.debug(format!("New client: {:?}", new_client));

            new_client.save_into_db(); //> It Already create the new client

            logger.debug("New client saved into the database!".to_string());

            let mut resp_kwargs: HashMap<String, ResultType> = HashMap::new();

            resp_kwargs.insert("actual_client_key".to_string(), ResultType::Str(client_key.to_string()));

            to_send.insert("command_type".to_string(), ResultType::Str("response".to_string()));
            to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
            to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
            to_send.insert("message".to_string(), ResultType::Str(format!("Successfully add a client: {}!", client_key).to_string()));
            to_send.insert("response_activation_function".to_string(), ResultType::Str("add_client_handler".to_string()));
            to_send.insert("kwargs".to_string(), ResultType::Map(resp_kwargs));
            to_send.insert("origin".to_string(), ResultType::Str("host".to_string())); // This is a identifier to know from where the command is

            logger.info(format!("Successfully add a client: {}!", client_key));

            // TODO >>> Implement a mechanism to send back the confirmation or a error message originated from the operation

            return to_send;
            // else {
            //     logger.warn("Error! Callback response kwargs isn't a Map!".to_string());
            //     return create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            // }
        },

        "update_client" => {
            // > update client
            // {'response_mode':'InternalManagement', 'activation_function':'update_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'actual_client_key':String, 'updated_client':client} // Client have to have the same client key
            // 'client': {"client_name":str, "client_key":str, "client_type":str, "permission_group":str, "is_super_user":bool, "max_sub_channels":int, "owned_sub_channels_keys":list}

            logger.debug("Receive a update client inner command!".to_string());

            if !kwargs.contains_key("actual_client_key") {
                logger.warn("Error! Callback response kwargs don't have actual_client_key kwarg!".to_string());
                return create_error_response_and_return!("Error! Callback response kwargs don't have actual_client_key kwarg!", converted_m, to_send);
            }

            if !inner_map.contains_key("updated_client") {
                logger.warn("ERROR, Error! Callback response kwargs don't have update_client kwarg!".to_string());
                return create_error_response_and_return!("Error! Callback response kwargs don't have update_client kwarg!", converted_m, to_send);
            }

            let actual_client_key = result.get("actual_client_key").unwrap().to_str().unwrap();
            let updated_client = kwargs.get("updated_client").unwrap().clone();

            // from("client_name":"str", "client_key":"str", "client_type":"str", "permission_group":"str", "is_super_user":"bool", "max_sub_channels":"int", "owned_sub_channels_keys":"list")

            let mut expected = HashMap::new();

            expected.insert("client_name".to_string(), ResultType::Str("".to_string()));
            expected.insert("client_key".to_string(), ResultType::Str("".to_string()));
            expected.insert("client_type".to_string(), ResultType::Str("".to_string()));
            expected.insert("permission_group".to_string(), ResultType::Str("".to_string()));
            expected.insert("is_super_user".to_string(), ResultType::Bool(false));
            expected.insert("max_sub_channels".to_string(), ResultType::Int(0));
            expected.insert("owned_sub_channels_keys".to_string(), ResultType::List(vec![]));

            
            let parsed_new_client = fast_json_comparator(&updated_client, &ResultType::Map(expected.clone()))

            let new_client = match parsed_new_client {
                Err(e) => match e {
                    ComparatorError::TypeMismatch(tp) => {
                        logger.warn(format!("ERROR, Client kwargs have mismatch type {} kwarg!", tp));
                        return create_error_response_and_return!(format!("Error! Client kwargs have mismatch type {} kwarg!", tp), converted_m, to_send);
                    },
                    ComparatorError::LengthMismatch => {
                        logger.warn("ERROR, Client kwargs have mismatch relative length kwargs!".to_string());
                        return create_error_response_and_return!("Error! Client kwargs have mismatch relative length kwargs!", converted_m, to_send);
                    },
                    ComparatorError::MissingKey(k) => {
                        logger.warn(format!("ERROR, Client kwargs have a missing kwarg: {}!", k));
                        return create_error_response_and_return!(format!("Error! Client kwargs have a missing kwarg: {}!", k), converted_m, to_send);
                    },
                    ComparatorError::TargetIsEmpty=> {
                        logger.warn("ERROR, Client target pattern is empty!".to_string());
                        return create_error_response_and_return!("Error! Client target pattern is empty!", converted_m, to_send);
                    },
                    ComparatorError::ParseError(e) => {
                        logger.warn(format!("ERROR, Can't parse {:?}!", e).to_string());
                        return create_error_response_and_return!("Error! Client target pattern is empty!", converted_m, to_send);
                    }
                },

                Ok(new_client) => new_client,
            };

            let updated_client_unwraped: HashMap<String, ResultType> = new_client.to_map().unwrap();

            let owned_sub_channels_keys: Vec<String> = updated_client_unwraped.get("owned_sub_channels_keys").unwrap().to_list().unwrap().iter().map(|v| v.to_str().unwrap()).collect();

            let client_key = updated_client_unwraped.get("client_key").unwrap().to_str().unwrap();

            logger.debug(format!("Updated client: {:?}", new_client));

            let client_handlers: Vec<HashMap<String, Value>> = Vec::new();

            let new_client = handle_client_error!(Client::new(
                updated_client_unwraped.get("client_name").unwrap().to_str().unwrap(),
                client_key.clone(),
                updated_client_unwraped.get("client_type").unwrap().to_str().unwrap(),
                updated_client_unwraped.get("permission_group").unwrap().to_str().unwrap(),
                updated_client_unwraped.get("is_super_user").unwrap().to_bool().unwrap(),
                updated_client_unwraped.get("max_sub_channels").unwrap().to_int().unwrap() as u32,
                owned_sub_channels_keys,
                client_handlers,
            ));

            let old_client = handle_client_error!(Client::get_by_key(&actual_client_key));

            let result = old_client.update_to(&new_client); //> It already saves into the database

            // TODO >>> Maybe implement a fast result-ype to client if needed

            match result {
                Ok(_) => {
                    to_send.insert("command_type".to_string(), ResultType::Str("response".to_string()));
                    to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
                    to_send.insert("status".to_string(), ResultType::Str("success".to_string()));
                    to_send.insert(
                        "message".to_string(),
                        ResultType::Str(format!("Successfully executed the function: {} and remove client: {}!", activation_function, client_key).to_string()),
                    );
                    to_send.insert("response_activation_function".to_string(), ResultType::Str("update_client_handler".to_string()));

                    let mut resp_kwargs: HashMap<String, ResultType> = HashMap::new();

                    resp_kwargs.insert("actual_client_key".to_string(), ResultType::Str(client_key.to_string()));

                    to_send.insert("kwargs".to_string(), ResultType::Map(resp_kwargs));
                    to_send.insert("origin".to_string(), ResultType::Str("host".to_string())); // This is a identifier to know from where the command is

                    logger.info(format!("Successfully executed the function: {} and remove client: {}!", activation_function, client_key));

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

            // TODO >>> Implement a mechanism to send back the confirmation or a error message originated from the operation
            // else {
            //     logger.warn("Error! Callback response kwargs isn't a Map!".to_string());
            //     return create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            // }
        },

        "remove_client" => {
            // > remove client
            // {'response_mode':'InternalManagement', 'activation_function':'remove_client', 'kwargs':response, 'response_activation_function':'function_name'}
            // 'kwargs':{'client_key':String}

            if !kwargs.contains_key("client_key") {
                return create_error_response_and_return!("Error! Callback response kwargs don't have client_key kwarg!", converted_m, to_send);
            }

            let client_key: &Value = kwargs.get("client_key").unwrap();

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
                        ResultType::Str(format!("Successfully executed the function: {} and remove client: {}!", activation_function, client_key).to_string()),
                    );
                    to_send.insert("response_activation_function".to_string(), ResultType::Str("remove_client_handler".to_string()));

                    let mut resp_kwargs: HashMap<String, ResultType> = HashMap::new();

                    resp_kwargs.insert("actual_client_key".to_string(), ResultType::Str(client_key.to_string()));

                    to_send.insert("kwargs".to_string(), ResultType::Map(resp_kwargs));
                    to_send.insert("origin".to_string(), ResultType::Str("host".to_string())); // This is a identifier to know from where the command is

                    logger.info(format!("Successfully executed the function: {} and remove client: {}!", activation_function, client_key));

                    return to_send;
                },
            }
            // else {
            //     logger.warn("Error! Callback response kwargs isn't a Map!".to_string());
            //     return create_error_response_and_return!("Error! Callback response kwargs isn't a Map!", converted_m, to_send);
            // }

            // TODO >>> Implement a mechanism to send back the confirmation or a error message originated from the operation
        },

        _ => {
            to_send.insert("command_type".to_string(), ResultType::Str("response".to_string()));
            to_send.insert("response_mode".to_string(), ResultType::Str("to_origin".to_string()));
            to_send.insert("status".to_string(), ResultType::Str("error".to_string()));
            to_send.insert("message".to_string(), ResultType::Str(format!("Response Activation Function: {} doesn't exists!!", activation_function).to_string()));
            to_send.insert("response_activation_function".to_string(), ResultType::Str("response_activation_function".to_string()));
            to_send.insert("origin".to_string(), ResultType::Str("host".to_string())); // This is a identifier to know from where the command is

            logger.warn(format!("Response Activation Function: {} doesn't exists!!", activation_function));

            return to_send;
        },
    }

    // TODO >>> Add the cases to handle the following internal management things:

    //* Need to implement the 'response_activation_function' in the wrapper
}
