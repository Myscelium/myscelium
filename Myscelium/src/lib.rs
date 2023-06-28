

// use socket_client;

mod socket_host;
use socket_host::socket_host as sckt_h;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

use pyo3::types::{PyDict, PyTuple};

#[pymodule]
fn rust_module(py: Python, m: &PyModule) -> PyResult<()> {
    #[pyfn(m, "call_python_function")]
    fn call_python_function(py: Python, command: &PyDict) -> PyResult<()> {
        let function: &PyAny = command.get_item("function").unwrap();
        let args_dict: &PyDict = command.get_item("args").unwrap().downcast().unwrap();

        // Extract the Python function name
        let function_name: &str = function.getattr("__name__")?.extract()?;

        // Convert the args dict to a Vec and then to a tuple
        let args_vec: Vec<&PyAny> = args_dict.values().extract::<Vec<&PyAny>>()?;
        let args_tuple: &PyTuple = PyTuple::new(py, &args_vec);

        // Call the Python function with the args
        let _result = function.call1(args_tuple)?;

        Ok(())
    }

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