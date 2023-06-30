
use crate::socket_host::enhanced_buffer;
use lazy_static::lazy_static;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::collections::HashMap;
use serde_json::{Value, from_str};

use serde::{Serialize, Deserialize};

use crate::socket_host::enhanced_buffer::buffer_down_mananger::DownCommand;

use pyo3::types::{IntoPyDict, PyString, PyInt, PyAny, PyDict, PyTuple, PyList};
use pyo3::{Python, PyResult, PyObject, PyErr};

use pyo3::exceptions::PyException;

use std::time::{Duration, Instant};

use crate::RUNNING;
use std::sync::atomic::Ordering;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Command {
    client_id: String,
    parity_id: String,
    priority: i32,
    command: HashMap<String, Value>,
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

    static ref NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5)); // Default
}

pub fn set_workers_num (n_workers:u32) {

    let mut default_num_of_workers = NUM_WORKERS.lock().unwrap();

    *default_num_of_workers = n_workers;

    enhanced_buffer::buffer_down_mananger::set_workers_num(n_workers);
    enhanced_buffer::buffer_up_mananger::set_workers_num(n_workers);
    enhanced_buffer::buffer_client_mananger::set_workers_num(n_workers);

}

pub fn set_transposer_callbacks (callback_patterns:HashMap<String, Value>) {

    let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();
    *command_patterns = callback_patterns;

}

// > thread Manangement:

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(job).unwrap();
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {

        let thread = thread::spawn(move || loop {
            let job = match receiver.lock().unwrap().recv() {
                Ok(job) => job,
                Err(_) => return,
            };

            println!("Worker {} got a job; executing.", id);

            job();
        });

        Worker {
            id,
            thread: Some(thread),
        }

    }
}

// > Transposer:

fn dict_to_tuple(py: Python, dict: &HashMap<String, Value>) -> PyResult<Vec<PyObject>> {

    let function_name = match dict.get("function") {
        Some(Value::String(function_name)) => function_name,
        _ => return Err(PyErr::new::<PyException, _>("The function name is not found or not a string.")),
    };

    let sub_dict = match dict.get(function_name) {
        Some(Value::Object(map)) => map.clone().into_iter().collect::<HashMap<String, Value>>(),
        _ => return Err(PyErr::new::<PyException, _>("The arguments are not found or not an object.")),
    };

    let py_dict = PyDict::new(py);
    for (key, value) in sub_dict {
        let py_key = PyString::new(py, &key);
        let py_value = PyString::new(py, &value.to_string());
        py_dict.set_item(py_key, py_value)?;
    }

    Ok(vec![py_dict.into()])
}

fn handle_command (command:Command) -> PyResult<PyObject> {

    let gil = Python::acquire_gil();
    let py = gil.python();

    let function_name = match command.command.get("function") {
        Some(Value::String(function_name)) => function_name,
        _ => return Err(PyErr::new::<PyException, _>("The function name is not found or not a string.")),
    };

    // The name of the Python function
    let function: &PyAny = py.eval(function_name, None, None).unwrap();

    let args = dict_to_tuple(py, &command.command).map_err(|e| {
        eprintln!("Error converting arguments to tuple: {:?}", e);
        PyErr::new::<PyException, _>(format!("Error converting arguments to tuple: {:?}", e))
    })?;

    // Convert the Vec<Py<PyAny>> to a PyTuple
    let args_tuple = PyTuple::new(py, args);

    // Call the Python function with the converted arguments
    let result = function.call1(args_tuple).map_err(|e| {
        eprintln!("Error calling function: {:?}", e);
        e
    })?;

    println!("Function returned: {:?}", result);
    let result: PyObject = result.extract().unwrap();
    Ok(result)

}


fn process (down_command:DownCommand) {

    let command_patterns = COMMAND_PATTERNS.lock().unwrap().clone();
    let patters = command_patterns;

    let command_id:i32 = down_command.command_id;
    let command:String = down_command.command;

    let hashmap_command:HashMap<String, Value> = serde_json::from_str(&command).unwrap();

    let translated_command:Command = Command{
                                                client_id: down_command.client_id,
                                                parity_id: down_command.parity_id,
                                                priority: down_command.priority,
                                                command: hashmap_command,
                                            };

    let function = match translated_command.command.get("function") {
        Some(Value::String(function)) => function,
        _ => {
            println!("The function name is not found or not a string.");
            return;
        }
    };

    if !patters.contains_key(function) { // -> Remove command from schedule if it isn't on the patterns
        println!("Command isn't registred in the patterns");

        enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id);

        println!("command skipped and remvoed from schedule");
        return;
    }

    let response = handle_command (translated_command.clone());

    // TODO >>> Implement the response handling mecanism

    println!("The response to the callback are: {:?}", response);

    println!("command: {}, processed!", translated_command.parity_id);

    enhanced_buffer::buffer_down_mananger::buffer_down_remove_schedule_by_id(command_id);

}

pub fn initialize_transposer () {

    loop {

        if !RUNNING.load(Ordering::SeqCst) {
            println!("Stop the transposer!");
            break;
        }

        let num_of_workers = NUM_WORKERS.lock().unwrap();

        let pool = ThreadPool::new(*num_of_workers as usize);

        let schedule:Vec<DownCommand> = enhanced_buffer::buffer_down_mananger::buffer_down_list_schedule();

        if !schedule.len() > 0 {
            println!("Nothing in the schedule, skipping >>>");
            thread::sleep(Duration::from_secs(5));
            continue;
        }

        for dow_command in schedule {
            process(dow_command);
        }

        let mut command_patterns = COMMAND_PATTERNS.lock().unwrap();

    }

    return;

    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();

    //     pool.execute(|| {
    //         handle_connection(stream);
    //     });
    // }

}
