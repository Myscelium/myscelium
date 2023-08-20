pub fn client_channel_mananger_initialize_table(buffer_path: String) {
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

    set_new_path_to_buffer_db!(BUFFER_POOL, NUM_WORKERS, buffer_path, BUFFER_NAME);

    with_connection!(BUFFER_POOL, |conn: &rusqlite::Connection| {
        let result = conn.execute(
            "CREATE TABLE IF NOT EXISTS UserGroups (ID INT PRIMARY KEY, GroupName TEXT, AllowFileTransfer BOOL, MaxSubChannelsPerClient NUMBER, FunctionsAllowedAreBlackList BOOL, FunctionsAllowed TEXT,  FileTransferFunctionsAllowedAreBlackList BOOL, FileTransferFunctionsAllowed TEXT, AllowRedirectAreBlackList BOOL, AllowRedirectTo TEXT)",
            params![],
        );

        match result {
            Ok(_) => {
                println!("Successfully initialize ClientCommandsTosend table!");
            },
            Err(e) => {
                eprintln!("An error occurred while scheduling the command in the ClientCommandsTosend table: {}", e);
            },
        };
    });
}
