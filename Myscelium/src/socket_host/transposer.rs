
use crate::socket_host::enhanced_buffer;
use lazy_static::lazy_static;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;


use crate::socket_host::enhanced_buffer::buffer_down_mananger::DownCommand;

lazy_static! {
    static ref NUM_WORKERS: Arc<Mutex<u32>> = Arc::new(Mutex::new(5)); // Default
}

pub fn set_workers_num (n_workers:u32) {

    let mut default_num_of_workers = NUM_WORKERS.lock().unwrap();

    *default_num_of_workers = n_workers;

    enhanced_buffer::buffer_down_mananger::set_workers_num(n_workers);
    enhanced_buffer::buffer_up_mananger::set_workers_num(n_workers);
    enhanced_buffer::buffer_client_mananger::set_workers_num(n_workers);

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

pub fn initialize_transposer () {

    let num_of_workers = NUM_WORKERS.lock().unwrap();

    let pool = ThreadPool::new(*num_of_workers as usize);

    let schedule:Vec<DownCommand> = enhanced_buffer::buffer_down_mananger::buffer_down_list_schedule();


    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();

    //     pool.execute(|| {
    //         handle_connection(stream);
    //     });
    // }

}
