# Custom Thread Pool

How it works? Well, imagine a tool that allows you to have multiple containers inside it running in parallel, this is the purpose of the thread pool, here is a custom one that allows you to run this inside a special type of box that encapsulates code, any rust code, this pool has a structure like that:

```rust
pub struct UnifiedThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
    free_condvar: Arc<Condvar>,
    stopped: Arc<AtomicBool>,
    task_count: Arc<AtomicUsize>,
}
```

There is several fields inside the struct of the tread pool, they are essential to make it works, bellow is a explanation of each one and how they works:

##### Workers:

This represents the workers in fact, each worker is a thread reference that allows to run rust code inside it, workers have a unique id, a thread pointer and a status if they are busy or not, they have this structure:

```rust
struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
    busy: Arc<AtomicBool>,
}
```

They are designed to resume when they crash and send the error back and also the status allows we to see if the work is available or not what helps a lot actuall

They are designed to resume when they crash and send the error back and also the status allows we to see if the work is available or not what helps a lot when comes to find a free worker in a easy way automatically.

##### Sender:

Sender is a mpsc channel that has a job parameter that ensure safety and ensure that this will be droped when finish, the sender job is a option of a box with boundaries to ensure that will only run one unique time plus ensure that it is safe to send throught threads and ensure that it is safe by adding a lifetime to it

This is arepresentation of the mpsc sender job:

```Rust
type Job = Option<Box<dyn FnOnce() + Send + 'static>>;
```

##### Free Condvar:

A Condition Variable

Condition variables represent the ability to block a thread such that it consumes no CPU time while waiting for an event to occur. Condition variables are typically associated with a boolean predicate (a condition) and a mutex. The predicate is always verified inside of the mutex before determining that a thread must block.

Functions in this module will block the current thread of execution. Note that any attempt to use multiple mutexes on the same condition variable may result in a runtime panic.

##### Stopped:

This allows the entire pool to resume if something goes wrong and activates a global break, this is necessary to ensure that it will resume completelly and not only partially, avoiding random detached threads that can cause issues in the system. Since this is very low level control it can happen sometimes with very specific conditions making a thing that i called zoombie threads.

##### Task Count:

Task count does exactly it it say it does, it counts the tasks that the pool received allowing to know when is some task pendent and then assign it to a worker and when isn't any new tasks to process making it reactive to the times when are tasks to process instead of keep high refreshing rate in the loops.

---

### The implementation and its methods

#### New:

It does what it say it does, it creates a new `UnifiedThreadPool`, and it's very interesting how it does that. It takes a `size` argument used to determine the amount of workers this pool will have, this size initialize the workers and push then into the workers vec, this is the code for the new mehtod:

```Rust
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
```

#### Execute:

The execute method is the one that is responsible to execute things inthe pool, this is required to make the pool able to execute the tasks, in this case `f` is the task to be executed, the arg is called f becuase the task in the end is a function wrapping some codeblock, the code for the execute is demonstrated bellow:

```rust
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
```

The execute method is blinded with boundaries too to it execute once and be able to be send safelly between threads, also have a lifetime derivate that ensures the that the function will live time enougt to be processed before it drops.

The task has the same time of the Job wrapped inside the sender and this is on purpose since the job is injected inside the execute method at some point in run time.

The `execute` method will add one to the task count using a fetch add with the `Ordering::SeqCst` that is a Atomic memory ordering

> Atomic Memory Orderings specify the way atomic operations synchronize memory. In its weakest Ordering::Relaxed, only the memory directly touched by the operation is synchronized. On the other hand, a store-load pair of Ordering::SeqCst operations synchronize other memory while additionally preserving a total order of such operations across all threads.

Then if loads the state of the pool to see if it isn't stoped, because we don't want to execute things inside a pool that is stoped nor add something to execute in this case, so if it isn't executing we return, in cases that we are in debug mode we will print it for debug purposes.

However if the pool is running then we encapsulate the function (Taks or Job) inside a Box<>, then we use the mpsc channel to send it throught the memory link, a free worker will catch it and execut it. Then this will be imediatly remove from jobs list since is a `FnOnce`.

> Obs: isn't common to use the execute directly, the ideal is to use the `wait_for_free_worker` method described bellow for security reasons related to memory safety and worker lifetime.

#### Wait For Free Worker:

Wait for free workers do wat it says, it wait for a free worker to assign a task to it, the logic of the wait for free worker is described bellow:

```rust
pub fn wait_for_free_worker(&self, f: Job) {
    let lock = Mutex::new(());
    let mut guard = lock.lock().unwrap();
    while self.free_workers().is_empty() {
      guard = self.free_condvar
                  .wait_timeout(guard, std::time::Duration::from_secs(1)).unwrap().0;
    }
    if let Some(func) = f {
        self.execute(func);
    }
}
```

Above we can see that what wait for free work does is create a mutext guard, lock in it, then wait for a timeout in the `free_condvar`, this makes we enter on a heap queue inside the memory to lock in a worker, when some worker becomes free, then we verify if the function is some, if the function has a real function inside it and then execute. This aproach ensures that we will wait for a free worker, and this reduces errors when working with more threads that we have in proportion to workers since we will have some queue at some point.

It uses a function that collects the `free_workers`

#### Free workers:

This is basically a filter that filters the free workers by their busy status and creates a Vec of workers available.

```rust
pub fn free_workers(&self) -> Vec<usize> {
    self.workers.iter().filter(|worker| !worker.busy.load(Ordering::SeqCst)).map(|worker| worker.id).collect()
}
```

The Vec as you see is a usize, this is on purpose since we don't collect the workers in essence, but the number of workers free in each section of the pool, then we send it back to use in cases like `wait_for_free_worker` for example.

#### All Workers Free:

Helper method to check if all worker threads are free. Simillar to the `free_workers` method but this instead of returning a Vec of usize it returns a boolean saying if all workers are free or not, this is the code for it:

```
fn all_workers_free(&self) -> bool {
    self.workers.iter().all(|worker| !worker.busy.load(Ordering::SeqCst))
}
```

It is used in cases like the pool global stop, that uses this to ensure that all workers are with no tasks being executed.

#### Join:

The join is a blocking operation that waits until all workers have finished their tasks. The code bellow demonstrates how it works:

```rust
pub fn join(&self) {
    let lock = Mutex::new(());
    let mut guard = lock.lock().unwrap();
    while !self.all_workers_free() {
      guard = self.free_condvar.wait_timeout(guard, std::time::Duration::from_millis(10)).unwrap().0;
    }
}
```

As you can see above it uses the `all_workers_free` to check when all works finish its respective tasks, it also uses the `free_condvar` trick to wait by the timeout and lock it to join inside the while loop, when all loops lock it return, meaning that the worker is free to to wathever you want, this is used in the stop to join all workers an the stop all them.

#### Stop:

This method sends a termination message to all workers and waits for them to finish their current tasks. The process is a little complicated but it ensures that the pool stops nicelly by finishing all tasks before it doesn't allow any new task to be scheduled.

The code for it is bellow:

```rust
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
```
