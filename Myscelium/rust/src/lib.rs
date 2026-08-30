// SPDX-License-Identifier: MPL-2.0
// Copyright © 2021-2026 Cristian Camargo Filho

#[allow(unused_imports)]
#[allow(unused_extern_crates)]
mod common;

mod host_entry_point;
use host_entry_point::*;

mod client_entry_point;
use client_entry_point::*;

use lazy_static::lazy_static;
use parking_lot::Mutex;

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use OxidizedMyscelium::AllowedNetWorkController;
use OxidizedMyscelium::ClientState;
use OxidizedMyscelium::NetworkMap;
use OxidizedMyscelium::Node;

extern crate chrono;
use crate::chrono::TimeZone;

// TODO >>> Add a protocol id in the host to check if the client is outdated compared to the host
// TODO >> Create a configs file that automatically be created by Host to configure, client key, or host ip, credentials, data dir, etc..

// -> Entries:

#[pymodule]
fn myscelium_engine(_py: Python<'_>, m: Bound<PyModule>) -> PyResult<()> {
    // -> Host
    m.add_function(wrap_pyfunction!(registry_socket_host_callbacks, &m)?)?;
    m.add_function(wrap_pyfunction!(initialize_socket_host, &m)?)?;
    m.add_function(wrap_pyfunction!(get_socket_host_available_commands, &m)?)?;
    m.add_function(wrap_pyfunction!(set_socket_host_allowed_clients, &m)?)?;
    m.add_function(wrap_pyfunction!(setup_socket_host, &m)?)?;
    m.add_function(wrap_pyfunction!(registry_new_allowed_clients, &m)?)?;

    // -> Client
    m.add_function(wrap_pyfunction!(registry_socket_client_callbacks, &m)?)?;
    m.add_function(wrap_pyfunction!(initialize_socket_client, &m)?)?;
    m.add_function(wrap_pyfunction!(get_client_state, &m)?)?;

    m.add_function(wrap_pyfunction!(set_socket_client_transposer_num_of_workers, &m)?)?;
    m.add_function(wrap_pyfunction!(client_send, &m)?)?;
    m.add_function(wrap_pyfunction!(wait_client_resp, &m)?)?;
    m.add_function(wrap_pyfunction!(setup_client, &m)?)?;
    m.add_function(wrap_pyfunction!(get_socket_client_available_handlers, &m)?)?;
    m.add_function(wrap_pyfunction!(is_client_ready, &m)?)?;
    m.add_function(wrap_pyfunction!(is_target_ready, &m)?)?;

    Ok(())
}

// To call by the python side:

/*

import rust_module  # This is your Rust module compiled as a Python extension

def python_function(name, age, birth):
    # Your function logic here
    pass

rust_module.call_python_function({
    "function": python_function,
    "args": {
        "name": "John",
        "age": 30,
        "birth": "1990-01-01"
    }
})

 */
