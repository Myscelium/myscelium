// use socket_client;

use std::collections::HashMap;

use crate::common::functions::convert_to_pydict;
use crate::common::functions::wrap_py_function;

use OxidizedMyscelium::{HOST_COMMAND_PATTERNS, HOST_IS_RUNNING};

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyFunction, PyList};

use serde_json::Value;

use indexmap::IndexMap;
use std::sync::atomic::Ordering;

use OxidizedMyscelium::registry_new_client;
use OxidizedMyscelium::CommandType;
use OxidizedMyscelium::{Client, ClientError};
use OxidizedMyscelium::{HandlerStatus, Node, NodeHandler, NodeStatus, NodeVersion, VersionIndentifier};

macro_rules! process_commands {
    ($py:expr, $commands:expr, $callback_pattern:expr) => {
        for command in $commands.iter() {
            let command_dict: &PyDict = command.downcast().unwrap();
            let function: &PyAny = command_dict.get_item("function").unwrap();

            let args_item: &PyAny = command_dict.get_item("args").unwrap();

            // Check if args_item is a dict or a string with the value "None"
            let args_dict: Option<&PyDict>;

            if let Ok(args_as_dict) = args_item.downcast::<PyDict>() {
                args_dict = Some(args_as_dict);
            } else if let Ok(args_as_str) = args_item.extract::<String>() {
                if args_as_str == "None" {
                    args_dict = None;
                } else {
                    return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("args must be a dict or the string 'None'"));
                }
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("args must be a dict or the string 'None'"));
            }

            // Extract the Python function name
            let function_name: &str = function.getattr("__name__")?.extract()?;

            // Extract the argument types
            let args_types_value;
            if let Some(args_dict) = args_dict {
                args_types_value = extract_arg_types(args_dict)?;
            } else {
                args_types_value = Value::Array(Vec::new()); // or whatever default value you want to use
            }

            let function = function.downcast::<PyFunction>()?.clone();

            let function: Py<PyFunction> = function.into_py($py); // convert &PyAny to Py<PyFunction>
            $callback_pattern.insert(function_name.to_string(), (function, args_types_value));
        }
    };
}

#[pyfunction]
pub fn setup_socket_host(buffer_path: String, log_level: String, n_workers: u32, n_max_conns: u32) {
    OxidizedMyscelium::setup_socket_host(&buffer_path, &log_level, &n_workers, &n_max_conns);
}

// TODO >>> Chang eset workers num, max conns, buffer initialization, socket host level

/// Stops the socket host.
///
/// This function sets the global `HOST_IS_RUNNING` flag to false, indicating that the socket host should stop running.
fn stop_socket_host() {
    HOST_IS_RUNNING.store(false, Ordering::SeqCst);
}

/// Registers callback functions for the socket host.
///
/// This function updates the global list of callback functions that the socket host can call. Each callback is associated
/// with a specific command that the host might receive.
///
/// # Parameters
///
/// - `py`: The Python interpreter.
/// - `commands`: A Python list of dictionaries containing the callback functions and their expected arguments.
///
/// # Returns
///
/// Returns an empty result if successful, or a Python error if there's a problem with the provided list.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn registry_socket_host_callbacks(py: Python, commands: Bound<PyList>) -> PyResult<()> {
    let mut host_node_handlers: Vec<NodeHandler> = Vec::new();

    let mut callbacks_patterns = HashMap::new();

    for command in commands.iter() {
        let command_dict: &Bound<PyDict> = command.downcast::<PyDict>()?;
        let function: Bound<PyAny> = match command_dict.get_item("function")? {
            Some(f) => f,
            None => return Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing key: function")),
        };

        let args_item: Bound<PyAny> = match command_dict.get_item("args")? {
            Some(f) => f,
            None => return Err(PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing key: args")),
        };

        // Check if args_item is a dict or a string with the value "None"
        let args_dict: Option<Bound<PyDict>>;

        if let Ok(args_as_dict) = args_item.downcast::<PyDict>() {
            args_dict = Some((*args_as_dict).clone());
        } else if let Ok(args_as_str) = args_item.extract::<String>() {
            if args_as_str == "None" {
                args_dict = None;
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("args must be a dict or the string 'None'"));
            }
        } else {
            return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("args must be a dict or the string 'None'"));
        }

        // Extract the Python function name
        let function_name_obj = function.getattr("__name__")?; // Store the temporary object
        let function_name: String = function_name_obj.extract()?; // Extract the value safely

        //> Extract the argument types (This are extracted from the function args requirements)
        let mut args_types_value = IndexMap::new();

        if let Some(args_dict) = args_dict {
            for (key, value) in args_dict.into_iter() {
                let key: String = key.extract()?;
                let value: String = value.extract()?;
                args_types_value.insert(key, value);
            }
            // Ok(map)
            // args_types_value = extract_arg_types(args_dict)?;
        }

        // Store the function name and argument types in the command patterns
        let host_handler: NodeHandler = NodeHandler::new(function_name.to_string(), args_types_value.clone(), CommandType::ExternalFunction, HandlerStatus::NotTested, HashMap::new(), "".to_string());

        host_node_handlers.push(host_handler);

        // Inside your loop over commands
        let function: Py<PyFunction> = function.downcast::<PyFunction>()?.extract()?;

        // Wrap the Python function to match CallbackClosure signature
        let wrapped_function = Box::new(wrap_py_function(function, "Host".to_string()));

        // Assuming `my_callbacks` is an instance of MyCallbacks
        callbacks_patterns.insert(function_name.to_string(), wrapped_function);
    }

    // Now you can use the command_patterns
    OxidizedMyscelium::set_host_callbacks(callbacks_patterns);

    // -> REGISTRY THE DIRECT MANAGEMENT FUNCTIONS INTO THE HANDLERS

    // TODO >>> See if is possible to automatically do it later

    {
        //> Add Client
        {
            let mut args_types_value: IndexMap<String, String> = IndexMap::new();

            args_types_value.insert("client_name".to_string(), "str".to_string());
            args_types_value.insert("client_key".to_string(), "str".to_string());
            args_types_value.insert("client_type".to_string(), "str".to_string());
            args_types_value.insert("permission_group".to_string(), "str".to_string());
            args_types_value.insert("is_super_user".to_string(), "bool".to_string());
            args_types_value.insert("max_sub_channels".to_string(), "int".to_string());
            args_types_value.insert("owned_sub_channels_keys".to_string(), "list".to_string());

            let host_add_client_handler: NodeHandler = NodeHandler::new("add_client".to_string(), args_types_value.clone(), CommandType::DirectFunction, HandlerStatus::Working, HashMap::new(), "".to_string());

            host_node_handlers.push(host_add_client_handler);
        }

        //> Update Client
        {
            let mut args_types_value: IndexMap<String, String> = IndexMap::new();

            args_types_value.insert("actual_client_key".to_string(), "str".to_string());
            args_types_value.insert("updated_client".to_string(), "dict".to_string());

            // TODO >>> make be possible to do sub dict explicity definitions of the parameters that is should have, also do the same with lists too

            let host_update_client_handler: NodeHandler = NodeHandler::new("update_client".to_string(), args_types_value.clone(), CommandType::DirectFunction, HandlerStatus::Working, HashMap::new(), "".to_string());

            host_node_handlers.push(host_update_client_handler);
        }

        //> Remove Client
        {
            let mut args_types_value: IndexMap<String, String> = IndexMap::new();

            args_types_value.insert("client_key".to_string(), "str".to_string());

            let host_remove_client_handler: NodeHandler = NodeHandler::new("remove_client".to_string(), args_types_value.clone(), CommandType::DirectFunction, HandlerStatus::Working, HashMap::new(), "".to_string());

            host_node_handlers.push(host_remove_client_handler);
        }
    }

    // TODO >>> Create a mechanism that allows to only update the necessary information to avoid need update all what can cause isues

    // -> UPDATE HOST NODE WITH THE HANDLERS
    let mut global_command_patterns = HOST_COMMAND_PATTERNS.lock();
    let node_version = NodeVersion::cast_version(1, 3, 0, VersionIndentifier::ReleaseCandidate);
    let host_node: Node = Node::new("host".to_string(), "host".to_string(), "".to_string(), node_version, host_node_handlers, NodeStatus::Online);
    global_command_patterns.add_or_update_if_exists(host_node);

    Ok(())
}

/// Initializes and starts the socket host.
///
/// This function sets up the socket host and starts it, allowing it to accept incoming connections.
///
/// # Parameters
///
/// - `py`: The Python interpreter.
/// - `ip`: IP address for the socket host.
/// - `port`: Port for the socket host.
/// - `client_id`: ID of the client.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn initialize_socket_host(py: Python<'_>, ip: String, port: i32, client_id: String) {
    OxidizedMyscelium::initialize_socket_host(ip, port, client_id)
}

/// Fetches the list of available commands that the socket host can recognize.
///
/// This function returns a dictionary of the commands that have been registered with the socket host.
/// Each command is associated with its expected arguments and callback function.
///
/// # Parameters
///
/// - `py`: The Python interpreter.
///
/// # Returns
///
/// Returns a Python dictionary containing the available commands.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn get_socket_host_available_commands(py: Python<'_>) -> PyResult<PyObject> {
    let commands = OxidizedMyscelium::get_socket_host_available_commands();
    convert_to_pydict(py, &commands)
}

// > --------------------------------------------------------------------------------------------------------
// > Client Management

// use crate::handle_manager_client_error;

macro_rules! extract_string {
    ($value:expr, $err_msg:expr) => {
        $value.extract::<String>().map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>($err_msg))?
    };
}

macro_rules! extract_float {
    ($value:expr, $err_msg:expr) => {
        $value.extract::<f64>().map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>($err_msg))?
    };
}

macro_rules! extract_unsigned_int {
    ($value:expr, $err_msg:expr) => {
        $value.extract::<u32>().map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>($err_msg))?
    };
}

macro_rules! extract_string_vector {
    ($value:expr, $err_msg:expr) => {
        $value.extract::<Vec<String>>().map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>($err_msg))?
    };
}

macro_rules! extract_boolean {
    ($value:expr, $err_msg:expr) => {
        $value.extract::<bool>().map_err(|_| PyErr::new::<pyo3::exceptions::PyTypeError, _>($err_msg))?
    };
}

use OxidizedMyscelium::handle_manager_client_error;

/// Sets the list of clients allowed to connect to the socket host.
///
/// This function updates the global list of clients that are permitted to connect to the socket host.
/// If a client is not present in this list, they will be denied access.
///
/// # Parameters
///
/// - `allowed_client_list`: A Python list of dictionaries. Each dictionary should contain the following keys:
///   - `client_name`: Name of the client.
///   - `client_key`: Unique key for the client.
///   - `client_type`: Type of the client.
///   - `permission_group`: The permission group the client belongs to.
///   - `is_super_user`: Boolean indicating if the client has superuser privileges.
///   - `max_sub_channels`: Maximum number of sub-channels the client can use.
///   - `owned_sub_channels_keys`: List of keys of sub-channels owned by the client.
///
/// # Returns
///
/// Returns an empty result if successful, or a Python error if there's a problem with the provided list.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn set_socket_host_allowed_clients(allowed_client_list: Bound<PyList>) -> PyResult<()> {
    // Bound<T> ensures that Python objects remain valid and prevents borrowing issues.

    for client_allowed in allowed_client_list.iter() {
        let allowed_clients_dict = client_allowed.downcast::<PyDict>()?.as_ref();

        let client_name = extract_string!(allowed_clients_dict.get_item("client_name").unwrap(), "Error: client_name must be a String!");
        let client_key = extract_string!(allowed_clients_dict.get_item("client_key").unwrap(), "Error: client_key must be a String with 16 characters!");

        let client_type = extract_string!(allowed_clients_dict.get_item("client_type").unwrap(), "Error: client_type must be a String!");
        let client_permission_group = extract_string!(allowed_clients_dict.get_item("permission_group").unwrap(), "Error: permission_group must be a String!");
        let client_is_super_user = extract_boolean!(allowed_clients_dict.get_item("is_super_user").unwrap(), "Error: is_super_user must be a String!");

        let client_max_sub_channels = extract_unsigned_int!(allowed_clients_dict.get_item("max_sub_channels").unwrap(), "Error: max_sub_channels must be a String!");
        let client_owned_sub_channels_keys = extract_string_vector!(allowed_clients_dict.get_item("owned_sub_channels_keys").unwrap(), "Error: owned_sub_channels_keys must be a String!");

        let client_handlers: Vec<HashMap<String, Value>> = Vec::new();

        registry_new_client(
            client_name.clone(),
            client_key.clone(),
            client_type,
            client_permission_group,
            client_is_super_user,
            client_max_sub_channels,
            client_owned_sub_channels_keys,
            client_handlers,
        );

        println!("Successfully created client: {} of key: {}", client_name, client_key)
    }
    Ok(())
}

/// Registers new clients that are allowed to connect to the socket host.
///
/// This function adds new clients to the global list of clients that are permitted to connect to the socket host.
///
/// # Parameters
///
/// Same as `set_socket_host_allowed_clients`.
///
/// # Returns
///
/// Returns an empty result if successful, or a Python error if there's a problem with the provided list.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn registry_new_allowed_clients(new_allowed_clients_list: Bound<PyList>) -> PyResult<()> {
    for client_allowed in new_allowed_clients_list.iter() {
        let allowed_clients_dict = client_allowed.downcast::<PyDict>()?.as_ref();

        let client_name = extract_string!(allowed_clients_dict.get_item("client_name").unwrap(), "Error: client_name must be a String!");
        let client_key = extract_string!(allowed_clients_dict.get_item("client_key").unwrap(), "Error: client_key must be a String with 16 characters!");

        let client_type = extract_string!(allowed_clients_dict.get_item("client_type").unwrap(), "Error: client_type must be a String!");
        let client_permission_group = extract_string!(allowed_clients_dict.get_item("permission_group").unwrap(), "Error: permission_group must be a String!");
        let client_is_super_user = extract_boolean!(allowed_clients_dict.get_item("is_super_user").unwrap(), "Error: is_super_user must be a String!");

        let client_max_sub_channels = extract_unsigned_int!(allowed_clients_dict.get_item("max_sub_channels").unwrap(), "Error: max_sub_channels must be a String!");
        let client_owned_sub_channels_keys = extract_string_vector!(allowed_clients_dict.get_item("owned_sub_channels_keys").unwrap(), "Error: owned_sub_channels_keys must be a String!");

        let client_handlers: Vec<HashMap<String, Value>> = Vec::new();

        if !OxidizedMyscelium::check_if_client_key_exists(client_key.clone()) {
            let client = handle_manager_client_error!(Client::new(
                client_name.clone(),
                client_key.clone(),
                client_type,
                client_permission_group,
                client_is_super_user,
                client_max_sub_channels,
                client_owned_sub_channels_keys,
                client_handlers,
            ));

            client.save_into_db();
        }

        println!("Successfully created client: {} of key: {}", client_name, client_key)
    }
    Ok(())
}

/// Removes all clients from the list of clients allowed to connect to the socket host.
///
/// This function clears the global list of clients that are permitted to connect to the socket host. After calling this function,
/// no client will be able to connect until new clients are added using either `set_socket_host_allowed_clients` or `registry_new_allowed_clients`.
///
/// # Parameters
///
/// - `allowed_client_list`: A Python list of dictionaries, same structure as `set_socket_host_allowed_clients`.
///
/// # Python Binding
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
fn remove_all_allowed_clients(allowed_client_list: Bound<PyList>) {
    let _ = Client::delete_all();
}
