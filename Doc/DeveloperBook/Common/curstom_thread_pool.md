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

This is a representation of the mpsc sender job:

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

```rust
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

Stop first check if the pool is running cause you can't stop a pool that isn't because it is already stoped. Then it will iterate in the workers and send None to all of them, this will send a terminate message to the worker;

After sending tis terminating message to the workers we iterate in all workers taking it's treads heads and joining all of them, this finish all the threads preventing zoombie threads.

After this our work is easy from that one, is just store a stopped in the pool status as stopped true and it's done!

---

### How a Worker Really works?

A worker of this `UnifiedThreadPool` is a very complex codeblock that allows functions and preatty much any code to be executed inside it, it uses `mpsc` channels to receive Jobs and has complex system to sync it's state in relation to the pool signilizing if it's available, if it is busy, etc...

This code bellow creates a new worker, lets analize it:

```rust
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

    Worker { id, thread: Some(thread), busy }
}
```

This top section:

```rust
let busy = Arc::new(AtomicBool::new(false));
let busy_clone = Arc::clone(&busy);
let free_condvar_clone = Arc::clone(&free_condvar);
let task_count_clone = Arc::clone(&task_count);
```

Represents the states of this work, and we have to update them accordinly to the states of the worker itself so that it don't confuses the pool in relation to the worker status.

Then we have the thread isolation section of our code that is respnsible for execute the Jobs that we pass using the mpsc channel:

```rust
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
```

This is basically a loop that, the way that it works is relativelly simple, the client receives the work in it's relative tread that is detached of the sync related to the rest of the pool, then this thread receives this job, lock in it and change the thread states to busy, then executes this job, the response of the job return to the place that called it automatically using the Job custom context, then, when it finish processing the job it change the states to not busy and wait for new jobs.

In resume the pool looks something like that:

<img src="../Resources/HowThreadPollWorks.png" alt="How the thread pool works" width="850" height="550">

Yes this is a little complicated, however we have macros that makes it all more simple, bellow we can see some important macros in the macros section that we ca use to use this thread pool with easy:

## Macros:

### Initialize Thread Pool

```rust
#[macro_export]
macro_rules! init_thread_pool {
    ($size:expr) => {{
        use std::sync::{mpsc, Arc, Mutex};
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let pool = Arc::new(Mutex::new(crate::common::custom_thread_pool::thread_pool::UnifiedThreadPool::new($size)));
            if let Err(err) = tx.send(pool) {
                println!("Error initializing thread pool: {:?}", err);
            }
        });
        match rx.recv() {
            Ok(pool) => pool,
            Err(err) => {
                println!("Error receiving thread pool: {:?}", err);
                panic!("Failed to initialize thread pool!"); // or handle the error as appropriate
            },
        }
    }};
}
```

This macro simplifies the process of setting up a thread pool by encapsulating the necessary boilerplate code into a reusable component. Let's break down how it works:

##### Macro Definition

- `#[macro_export]`: This attribute makes the macro available for use in other modules that import this module. It's necessary for reusability of the macro outside the module where it's defined.
- `macro_rules! init_thread_pool`: This defines a new macro named `init_thread_pool`.

##### Macro Body

The macro takes a single expression `$size` as input. This expression specifies the size of the thread pool, i.e., the number of threads it should contain.

##### Inside the Macro

1. **Imports**:

   It starts by importing necessary items from the `std::sync` module - `mpsc` (multi-producer, single-consumer) for creating a communication channel, `Arc` (atomic reference counted) for thread-safe reference counting, and `Mutex` for mutual exclusion.

2. **Creating a Channel**:

   `let (tx, rx) = mpsc::channel();` creates a channel for sending (`tx`) and receiving (`rx`) messages. This channel is used to communicate the newly created thread pool from the spawned thread to the calling thread.

3. **Spawning a Thread**:

   `std::thread::spawn(move || {...});` is used to spawn a new thread. Inside this thread:

   - A new `UnifiedThreadPool` is created with the specified size (`$size`).
   - The thread pool is wrapped in an `Arc<Mutex<...>>` to ensure safe concurrent access.
   - The newly created thread pool is sent back to the calling thread via the `tx` channel.
   - If sending fails, an error is printed to the console.

4. **Receiving the Thread Pool**:

   The `match rx.recv()` statement waits for the new thread pool to be sent from the spawned thread. It handles two cases:

   - `Ok(pool)`: The thread pool is successfully received, and it's returned.
   - `Err(err)`: An error occurred while receiving the thread pool. This error is printed, and then the macro causes a panic, indicating that the thread pool initialization failed.

##### Usage

The macro is used to initialize a thread pool with a specified number of threads. For instance:

```rust
let pool = init_thread_pool!(4);
```

This will create a thread pool with 4 threads and return a handle to this pool.

##### Advantages

- **Encapsulation**: The macro hides the complexity of setting up a thread pool, making the code cleaner and easier to read.
- **Reusability**: Since it's exported, this macro can be used across different parts of your application or even in different applications.
- **Error Handling**: The macro includes basic error handling for thread pool creation and communication, ensuring that failures are noticed and handled appropriately.

This macro is a convenient way to abstract the details of initializing a thread pool, making your Rust code more modular and maintainable.

---

### Wait All Threads:

This macro allows to join all threads for some purpose, it waits all threads finish wath they are doing and then continues to next line, bellow is better explained how it works:

```rust
#[macro_export]
macro_rules! wait_all_threads {
    ($receivers:expr) => {{
        let mut results = Vec::new();
        for rx in $receivers {
            match rx.recv() {
                Ok(result) => results.push(result),
                Err(err) => {
                    println!("Error receiving result from thread: {:?}", err);
                },
            }
        }
        results
    }};
}
```

The `wait_all_threads` macro is designed to collect results from multiple threads. It's particularly useful in scenarios where you have spawned multiple threads and need to gather their results or outputs. Let's break down how this macro works:

##### Macro Definition

- `#[macro_export]`: This attribute makes the macro available for use outside the module in which it is defined, allowing it to be imported and used in other parts of your codebase.

- `macro_rules! wait_all_threads`: This declares a new macro named `wait_all_threads`.

##### Macro Body

The macro takes a single input, `$receivers`, which is expected to be an iterable collection of receiver endpoints of channels (usually `mpsc::Receiver`).

#### Inside the Macro

1. **Initializing a Vector**:

   `let mut results = Vec::new();` initializes an empty vector to store the results collected from the threads.

2. **Iterating Over Receivers**:

   The macro iterates over each receiver in `$receivers`. For each receiver, it attempts to receive a message (or result) from the corresponding thread.

3. **Receiving and Handling Messages**:

   - `match rx.recv() {...}`: This match statement is used to receive a message from the current receiver (`rx`). The `recv()` method blocks execution until a message is received or the channel is closed.
   - `Ok(result) => results.push(result)`: If a message is successfully received, it's added to the `results` vector.
   - `Err(err) => {...}`: If there's an error in receiving a message (e.g., if the sending end of the channel has been dropped), the error is printed to the console.

4. **Returning Results**:

   Finally, the macro returns the `results` vector, which contains all the successfully received messages from the threads.

##### Usage

This macro is typically used in situations where you have multiple threads performing tasks, and you need to collect their results. For example:

```rust
let receivers = vec![rx1, rx2, rx3]; // rx1, rx2, rx3 are Receivers from threads
let all_results = wait_all_threads!(receivers);
```

In this example, `rx1`, `rx2`, and `rx3` are the receiver ends of channels connected to different threads. The macro waits for each thread to send its result and collects these results into `all_results`.

##### Advantages

- **Synchronization**: It provides a simple way to synchronize with multiple threads, ensuring that you wait for all of them to complete their execution.
- **Collection of Results**: It collects the results of thread executions in an orderly and manageable way.
- **Error Handling**: Basic error handling is included, which helps in debugging issues related to thread communication.
- **Code Clarity and Reusability**: By encapsulating this common pattern into a macro, your code becomes cleaner and more reusable, reducing the likelihood of repetitive code.

Overall, the `wait_all_threads` macro is a handy tool in concurrent Rust programming, simplifying the pattern of waiting for and collecting results from multiple threads.

---

### Run In Thread Pool:

This ine is the most important one because it simplifyes schedule work for the thread_pool workers, the structure for it is that:

```rust
#[macro_export]
macro_rules! run_in_thread_pool {
    ($pool:expr, $code:block) => {{
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();
        let mut locked_pool = $pool.lock().unwrap();
        locked_pool.execute(move || {
            let result = $code;
            if let Err(err) = tx.send(result) {
                println!("Error sending result from thread: {:?}", err);
            }
        });
        rx
    }};
}
```

The `run_in_thread_pool` macro is designed to execute a block of code within a thread from a thread pool and return a receiver for the result. This macro is useful for delegating tasks to a pool of worker threads and asynchronously retrieving their results. Let's break down its functionality:

##### Macro Definition

- `#[macro_export]`: This attribute exports the macro, making it available for use in other modules outside of where it's defined.

- `macro_rules! run_in_thread_pool`: This starts the definition of a new macro named `run_in_thread_pool`.

##### Macro Body

The macro takes two inputs:

1. `$pool`: This is the thread pool where the code block will be executed. The pool is typically a shared resource, managed by an `Arc<Mutex<...>>` for thread safety.

2. `$code`: This is a block of code to be executed in the thread pool. This code block is expected to produce a result which will be sent back to the calling context.

##### Inside the Macro

1. **Creating a Channel**:

   `let (tx, rx) = mpsc::channel();` creates a new channel. `tx` is the transmitter used to send the result from the worker thread, and `rx` is the receiver used in the calling context to retrieve the result.

2. **Accessing the Thread Pool**:

   `let mut locked_pool = $pool.lock().unwrap();` locks the thread pool mutex to gain access to the pool. This is necessary because the thread pool might be shared across multiple threads.

3. **Executing the Code in the Thread Pool**:

   `locked_pool.execute(move || {...});`: This line schedules the execution of the provided code block on one of the threads in the pool.

   - The `move` keyword is used to transfer ownership of the captured variables (including the transmitter `tx`) into the closure.
   - Inside the closure, the code block `$code` is executed, and its result is captured.
   - The result is then sent back to the calling context using the transmitter `tx`.
   - If sending the result fails (e.g., if the receiver has been dropped), an error is printed to the console.

4. **Returning the Receiver**:

   The macro returns `rx`, the receiver part of the channel. This allows the calling context to receive the result asynchronously once the worker thread completes its execution.

##### Usage Example

Here's a basic example of how you might use this macro:

```rust
let pool = Arc::new(Mutex::new(MyThreadPool::new(4))); // assuming MyThreadPool implements a thread pool

let rx = run_in_thread_pool!(pool, {
    // Code block to execute in the thread
    let computation = some_computation();
    computation // This is the result sent back
});

// Later, in the calling context
match rx.recv() {
    Ok(result) => println!("Received: {:?}", result),
    Err(e) => println!("Failed to receive: {:?}", e),
}
```

In this example, `some_computation()` is executed in one of the threads from `MyThreadPool`, and its result is sent back to the main thread.

##### Advantages

- **Concurrency Management**: The macro simplifies the use of a thread pool for executing concurrent tasks.
- **Asynchronous Execution**: It allows for asynchronous execution of code blocks, with results communicated back via channels.
- **Error Handling**: Basic error handling is included, which helps in dealing with issues related to inter-thread communication.
- **Code Reusability and Clarity**: By encapsulating the pattern of executing tasks in a thread pool, the macro enhances code clarity and reusability.

The `run_in_thread_pool` macro is a powerful tool for efficiently managing tasks that need to be run concurrently in Rust, particularly in applications where workload distribution among multiple threads is essential.

---

### Terminate Pool:

This is auto sugestive but has a key role of terminate the entire pool safelly, the code for it is the bellow one:

```rust
#[macro_export]
macro_rules! terminate_pool {
    ($pool:expr) => {{
        let mut locked_pool = $pool.lock().unwrap();
        locked_pool.stop();
    }};
}
```

The `terminate_pool` macro is a straightforward yet important piece of code used in the context of managing thread pools in Rust. Its purpose is to safely terminate all threads within a given thread pool. Let's dissect it for a clearer understanding:

##### Macro Definition

- `#[macro_export]`: This attribute makes the macro available for use in other modules outside of where it's defined, facilitating reuse across your codebase.

- `macro_rules! terminate_pool`: This line defines a new macro called `terminate_pool`.

##### Macro Body

The macro accepts a single input:

- `$pool`: This represents the thread pool that you want to terminate. It is expected to be a shared, mutable resource, likely wrapped in a thread-safe construct like `Arc<Mutex<...>>`.

##### Inside the Macro

1. **Accessing the Thread Pool**:

   `let mut locked_pool = $pool.lock().unwrap();` acquires a lock on the thread pool. The `lock()` method is called on the `$pool` variable (which should be a `Mutex`), and `unwrap()` is used to handle the `Result` returned by `lock()`. In a production environment, better error handling instead of `unwrap()` might be advisable to prevent potential panics.

2. **Terminating the Pool**:

   `locked_pool.stop();` calls the `stop` method on the thread pool. This method is assumed to be part of the thread pool's interface and is responsible for gracefully shutting down all threads in the pool. The actual implementation of `stop` would depend on how the thread pool is designed. Typically, it involves signaling all threads to complete their current work and then exit.

##### Usage

This macro is used to terminate a thread pool, typically when the program is done with executing parallel tasks, or when it's cleaning up resources before exiting. For example:

```rust
let pool = Arc::new(Mutex::new(MyThreadPool::new(4))); // assuming MyThreadPool is a custom thread pool implementation

// ... use the pool for various tasks ...

terminate_pool!(pool); // gracefully shut down the pool
```

In this example, `MyThreadPool` is an assumed implementation of a thread pool, and `terminate_pool!(pool)` is called when the pool is no longer needed.

##### Advantages

- **Simplicity and Clarity**: The macro provides a clear and concise way to terminate a thread pool.
- **Safe Resource Management**: It promotes the safe and orderly shutdown of threads, which is crucial for avoiding issues like resource leaks or unfinished tasks.
- **Reusability**: Being a macro, it can be easily reused across different parts of an application where thread pool management is necessary.

The `terminate_pool` macro is an essential tool in concurrent Rust programming, especially in applications that heavily rely on thread pools for managing parallel tasks. It encapsulates the best practice of gracefully shutting down thread pools, thus contributing to robust and maintainable code.
