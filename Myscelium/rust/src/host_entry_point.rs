// use socket_client;

use std::collections::HashMap;

use crate::common::functions::extract_arg_types;
use crate::common::functions::translate_value_to_py;
use crate::{HOST_COMMAND_PATTERNS, HOST_IS_RUNNING};
use OxidizedMyscelium::{ClientStatusPoolError, Clients};

use lazy_static::lazy_static;
use parking_lot::Mutex;

use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyBool, PyDict, PyFloat, PyFunction, PyInt, PyList, PyString, PyTuple};

use serde_json::Value;

use std::sync;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::MutexGuard;
use std::thread;
use std::time::Duration;

use OxidizedMyscelium::CommandType;
use OxidizedMyscelium::{Client, ClientError};
use OxidizedMyscelium::{HandlerStatus, Node, NodeHandler, NodeStatus, NodeVersion, VersionIndentifier};

lazy_static! {
    pub static ref CLIENTS_SYNC_CONTROLLER: Arc<Mutex<Clients>> = Arc::new(Mutex::new(Clients::new()));
}

// #[pyfunction]
// fn registry_socket_host_callbacks (py: Python, commands: &PyList) -> PyResult<()> {

//     for command in commands.iter() {
//         let command_dict: &PyDict = command.downcast().unwrap();
//         let function: &PyAny = command_dict.get_item("function").unwrap();
//         let args_dict: &PyDict = command_dict.get_item("args").unwrap().downcast().unwrap();

//         // Extract the Python function name
//         let function_name: &str = function.getattr("__name__")?.extract()?;

//         let mut command_patterns = HashMap::new();
//         command_patterns.insert(function_name.to_string(), Value::String(args_dict.to_string()));

//         set_socket_host_callbacks (command_patterns);

//         // Convert the args dict to a Vec and then to a tuple
//         let args_vec: Vec<&PyAny> = args_dict.values().extract::<Vec<&PyAny>>()?;
//         let args_tuple: &PyTuple = PyTuple::new(py, args_vec);

//         // Call the Python function with the args
//         let _result = function.call1(args_tuple)?;
//     }

//     Ok(())
// }

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

// #[pyfunction]
// pub fn set_socket_host_transposer_num_of_workers(n_workers: &PyInt) {
//     let workers_num: u32 = n_workers.extract().unwrap();

//     OxidizedMyscelium::set_socket

//     set_socket_host_transposer_workers_num(workers_num);

//     return;
// }
// #[pyfunction]
// pub fn set_socket_host_max_connections(n_max_conns: &PyInt) {
//     let max_conns: u32 = n_max_conns.extract().unwrap();

//     set_host_clients_manager__pool_workers_num(max_conns.clone());
//     set_max_conns(max_conns);

//     return;
// }

// #[pyfunction]
// pub fn initialize_host_buffer_tables(path: &PyString) {
//     let buffer_path: String = path.extract().unwrap();

//     initialize_host_logs_database_dir(buffer_path.clone());
//     initialize_host_buffer(buffer_path.clone());
//     clients_manager_initialize_table(buffer_path.clone());

//     return;
// }

// #[pyfunction]
// pub fn set_socket_host_log_level(log_level: &PyString) {
//     let log_level: String = log_level.extract().unwrap();

//     set_host_log_level(log_level);

//     return;
// }

// -> --------------------------------------------------------------------------------------------------------------

// #[pyfunction]
// fn registry_host_logs_handler(py: Python, commands: &PyList) -> PyResult<()> {
//     let mut callback_pattern = HashMap::new();

//     process_commands!(py, commands, callback_pattern);

//     set_host_logs_handler_callback(callback_pattern);

//     println!("set the log callback");

//     Ok(())
// }

/// Registers a callback function for the socket host to trigger when a client sends a heartbeat message.
///
/// This function updates the global callback that will be called each time the socket host receives a heartbeat
/// message from a client.
///
/// # Parameters
///
/// - `py`: The Python interpreter.
/// - `commands`: A Python list of dictionaries containing the callback function and its expected arguments.
///
/// # Returns
///
/// Returns an empty result if successful, or a Python error if there's a problem with the provided list.
///
/// # Python Binding
///
/// This function is exposed to Python and can be called from a Python script.
#[pyfunction]
pub fn registry_socket_host_client_heartbeat_contact_callback(py: Python, commands: &PyList) -> PyResult<()> {
    let mut callback_pattern = HashMap::new();

    process_commands!(py, commands, callback_pattern);

    set_heartbeat_callback(callback_pattern);

    Ok(())
}

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
pub fn registry_socket_host_callbacks(py: Python, commands: &PyList) -> PyResult<()> {
    let mut host_node_handlers: Vec<NodeHandler> = Vec::new();

    let mut callbacks_patterns = HashMap::new();

    for command in commands.iter() {
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

        // Store the function name and argument types in the command patterns
        let host_handler: NodeHandler = NodeHandler::new(function_name.to_string(), args_types_value.clone(), CommandType::ExternalFunction, HandlerStatus::NotTested, HashMap::new(), "".to_string());

        host_node_handlers.push(host_handler);

        let function = function.downcast::<PyFunction>()?.clone();

        let function: Py<PyFunction> = function.into_py(py); // convert &PyAny to Py<PyFunction>
        callbacks_patterns.insert(function_name.to_string(), (function, args_types_value));
    }

    // Now you can use the command_patterns
    set_socket_host_transposer_callbacks(callbacks_patterns);

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

    let address = format!("{}:{}", ip, port);

    thread::spawn(|| {
        ctrlc::set_handler(move || {
            if HOST_IS_RUNNING.load(Ordering::SeqCst) {
                println!("\nreceived Ctrl+C!\n");
                stop_socket_host();
            }
        })
        .expect("Error setting Ctrl-C handler");

        initialize_host(address, client_id);
        println!("Socket host exited successfully!");
    });

    loop {
        initialize_socket_host_transposer(py);

        if !HOST_IS_RUNNING.load(Ordering::SeqCst) {
            println!("Stop the core!");
            thread::sleep(Duration::from_secs(7));
            break;
        }
    }

    println!("Socket transposer exited successfully!");
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
    let commands = get_available_commands_registered();

    // Convert the HashMap values to PyObjects
    let py_dict: &PyDict = PyDict::new(py);
    for (key, value) in commands {
        let py_value = translate_value_to_py(py, value)?;
        py_dict.set_item(key, py_value)?;
    }

    Ok(py_dict.into())
}

// > --------------------------------------------------------------------------------------------------------
// > Client Management

// use crate::handle_client_error;

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
pub fn set_socket_host_allowed_clients(allowed_client_list: &PyList) -> PyResult<()> {
    for client_allowed in allowed_client_list.iter() {
        let allowed_clients_dict: &PyDict = client_allowed.downcast().unwrap();

        let client_name = extract_string!(allowed_clients_dict.get_item("client_name").unwrap(), "Error: client_name must be a String!");
        let client_key = extract_string!(allowed_clients_dict.get_item("client_key").unwrap(), "Error: client_key must be a String with 16 characters!");

        let client_type = extract_string!(allowed_clients_dict.get_item("client_type").unwrap(), "Error: client_type must be a String!");
        let client_permission_group = extract_string!(allowed_clients_dict.get_item("permission_group").unwrap(), "Error: permission_group must be a String!");
        let client_is_super_user = extract_boolean!(allowed_clients_dict.get_item("is_super_user").unwrap(), "Error: is_super_user must be a String!");

        let client_max_sub_channels = extract_unsigned_int!(allowed_clients_dict.get_item("max_sub_channels").unwrap(), "Error: max_sub_channels must be a String!");
        let client_owned_sub_channels_keys = extract_string_vector!(allowed_clients_dict.get_item("owned_sub_channels_keys").unwrap(), "Error: owned_sub_channels_keys must be a String!");

        {
            let mut controller = CLIENTS_SYNC_CONTROLLER.lock();
            let _ = controller.add_new_client(client_key.clone().to_string(), 10);
            println!("\nSet clients sync controler to:\n{:?}\n", controller);
        }

        // {
        //     let mut client_controller_pool: MutexGuard<Clients> = smart_lock(|| CLIENTS_SYNC_CONTROLLER.lock());
        //     let _ = client_controller_pool.add_new_client(client_key.clone(), 10);
        // }

        // {
        //     println!(
        //         "{:?},{:?},{:?},{:?},{:?},{:?},{:?}",
        //         &client_name, &client_key, &client_type, &client_permission_group, &client_is_super_user, &client_max_sub_channels, &client_owned_sub_channels_keys,
        //     )
        // }

        let client_handlers: Vec<HashMap<String, Value>> = Vec::new();

        if !check_if_client_key_exists(client_key.clone()) {
            let client = handle_client_error!(Client::new(
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
pub fn registry_new_allowed_clients(new_allowed_clients_list: &PyList) -> PyResult<()> {
    for client_allowed in new_allowed_clients_list.iter() {
        let allowed_clients_dict: &PyDict = client_allowed.downcast().unwrap();

        let client_name = extract_string!(allowed_clients_dict.get_item("client_name").unwrap(), "Error: client_name must be a String!");
        let client_key = extract_string!(allowed_clients_dict.get_item("client_key").unwrap(), "Error: client_key must be a String with 16 characters!");

        let client_type = extract_string!(allowed_clients_dict.get_item("client_type").unwrap(), "Error: client_type must be a String!");
        let client_permission_group = extract_string!(allowed_clients_dict.get_item("permission_group").unwrap(), "Error: permission_group must be a String!");
        let client_is_super_user = extract_boolean!(allowed_clients_dict.get_item("is_super_user").unwrap(), "Error: is_super_user must be a String!");

        let client_max_sub_channels = extract_unsigned_int!(allowed_clients_dict.get_item("max_sub_channels").unwrap(), "Error: max_sub_channels must be a String!");
        let client_owned_sub_channels_keys = extract_string_vector!(allowed_clients_dict.get_item("owned_sub_channels_keys").unwrap(), "Error: owned_sub_channels_keys must be a String!");

        let client_handlers: Vec<HashMap<String, Value>> = Vec::new();

        if !check_if_client_key_exists(client_key.clone()) {
            let client = handle_client_error!(Client::new(
                client_name.clone(),
                client_key.clone(),
                client_type,
                client_permission_group,
                client_is_super_user,
                client_max_sub_channels,
                client_owned_sub_channels_keys,
                client_handlers,
            ));

            client.save_into_db()
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
fn remove_all_allowed_clients(allowed_client_list: &PyList) {
    let _ = Client::delete_all();
}

// fn set_socket_host_allowed_clients(allowed_clients_list: &PyList) -> PyResult<()> {
//     for client_allowed in allowed_clients_list.iter() {
//         let allowed_clients_dict: &PyDict = client_allowed.downcast().unwrap();

//         let client_type: &PyAny = allowed_clients_dict.get_item("client_type").unwrap();
//         let client_id: &PyAny = allowed_clients_dict.get_item("client_id").unwrap();

//         if let Ok(extracted_client_type) = client_type.extract::<String>() {
//             if let Ok(extracted_client_id) = client_id.extract::<String>() {
//                 register_client(extracted_client_id, extracted_client_type);
//             } else {
//                 return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Error: client_id must be a String with 16 characters!"));
//             }
//         } else {
//             return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>("Error: client_type must be a String!"));
//         }
//     }

//     Ok(())
// }
