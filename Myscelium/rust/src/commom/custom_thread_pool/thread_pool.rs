use lazy_static::lazy_static;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;

lazy_static! {
    static ref DEBUG_MODE: bool = true; // Set this to true or false based on your needs
}

type Job = Option<Box<dyn FnOnce() + Send + 'static>>;

pub struct UnifiedThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
    free_condvar: Arc<Condvar>,
    stopped: Arc<AtomicBool>,
    task_count: Arc<AtomicUsize>,
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
    busy: Arc<AtomicBool>,
}

impl UnifiedThreadPool {
    pub fn new(size: usize) -> UnifiedThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let free_condvar = Arc::new(Condvar::new());
        let task_count = Arc::new(AtomicUsize::new(0)); // Initialize the task_count

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(
                id,
                Arc::clone(&receiver),
                Arc::clone(&free_condvar),
                Arc::clone(&task_count), // Pass the cloned task_count here
            ));
        }

        UnifiedThreadPool {
            workers,
            sender,
            free_condvar,
            stopped: Arc::new(AtomicBool::new(false)),
            task_count, // Add this line
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.task_count.fetch_add(1, Ordering::SeqCst);
        // Check if the pool has been stopped
        if self.stopped.load(Ordering::SeqCst) {
            if *DEBUG_MODE {
                println!("Pool has been stopped. Not sending job.");
            }
            return;
        }

        let job = Box::new(f);
        if let Err(err) = self.sender.send(Some(job)) {
            if *DEBUG_MODE {
                println!("Error sending job to worker: {:?}", err);
            }
        }
    }

    pub fn wait_for_free_worker(&self, f: Job) {
        let lock = Mutex::new(());
        let mut guard = lock.lock().unwrap();
        while self.free_workers().is_empty() {
            guard = self.free_condvar.wait_timeout(guard, std::time::Duration::from_secs(1)).unwrap().0;
        }
        if let Some(func) = f {
            self.execute(func);
        }
    }

    pub fn free_workers(&self) -> Vec<usize> {
        self.workers
            .iter()
            .filter(|worker| !worker.busy.load(Ordering::SeqCst))
            .map(|worker| worker.id)
            .collect()
    }

    pub fn stop(&mut self) {
        if !self.stopped.load(Ordering::SeqCst) {
            if *DEBUG_MODE {
                println!("Sending terminate message to all workers.");
            }

            for _ in &self.workers {
                if let Err(err) = self.sender.send(None) {
                    if *DEBUG_MODE {
                        println!("Error sending terminate message to worker: {:?}", err);
                    }
                }
            }

            if *DEBUG_MODE {
                println!("Shutting down all workers.");
            }

            for worker in &mut self.workers {
                if *DEBUG_MODE {
                    println!("Shutting down worker {}", worker.id);
                }

                if let Some(thread) = worker.thread.take() {
                    thread.join().unwrap();
                }
            }

            self.stopped.store(true, Ordering::SeqCst);
        }
    }

    pub fn join(&self) {
        let lock = Mutex::new(());
        let mut guard = lock.lock().unwrap();
        while !self.all_workers_free() {
            guard = self.free_condvar.wait_timeout(guard, std::time::Duration::from_millis(10)).unwrap().0;
        }
    }

    fn all_workers_free(&self) -> bool {
        self.workers.iter().all(|worker| !worker.busy.load(Ordering::SeqCst))
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>, free_condvar: Arc<Condvar>, task_count: Arc<AtomicUsize>) -> Worker {
        let busy = Arc::new(AtomicBool::new(false));
        let busy_clone = Arc::clone(&busy);
        let free_condvar_clone = Arc::clone(&free_condvar);
        let task_count_clone = Arc::clone(&task_count);

        let thread = thread::spawn(move || loop {
            let job = match receiver.lock().unwrap().recv() {
                Ok(Some(job)) => {
                    task_count_clone.fetch_sub(1, Ordering::SeqCst);
                    job
                },
                Ok(None) => return,
                Err(_) => return,
            };

            if *DEBUG_MODE {
                println!("Unified Worker {} got a job; executing.", id);
            }
            busy_clone.store(true, Ordering::SeqCst);
            job();
            busy_clone.store(false, Ordering::SeqCst);
            free_condvar_clone.notify_one();
        });

        Worker {
            id,
            thread: Some(thread),
            busy,
        }
    }
}

impl Drop for UnifiedThreadPool {
    fn drop(&mut self) {
        self.stop();
    }
}
