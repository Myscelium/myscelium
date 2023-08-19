#[macro_export]
macro_rules! init_thread_pool {
    ($size:expr) => {{
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let pool = crate::custom_thread_pool::thread_pool::UnifiedThreadPool::new($size);
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

#[macro_export]
macro_rules! terminate_pool {
    ($pool:expr) => {{
        $pool.stop();
    }};
}

#[macro_export]
macro_rules! run_in_thread_pool {
    ($pool:expr, $code:block) => {{
        use std::sync::mpsc;
        let (tx, rx) = mpsc::channel();
        $pool.execute(move || {
            let result = $code;
            if let Err(err) = tx.send(result) {
                println!("Error sending result from thread: {:?}", err);
            }
        });
        match rx.recv() {
            Ok(result) => result,
            Err(err) => {
                println!("Error receiving result from thread: {:?}", err);
                panic!("Failed to receive result from thread!"); // or handle the error as appropriate
            },
        }
    }};
}

#[macro_export]
macro_rules! wait_all_threads {
    ($pool:expr) => {{
        $pool.join();
    }};
}
