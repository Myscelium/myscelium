use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;

/// Attempts to acquire a lock and execute a closure, retrying with a randomized delay if the lock is not immediately available.
///
/// # Arguments
/// * `mutex` - A reference to the Arc containing the Mutex.
/// * `f` - A closure that will be executed once the lock is acquired.
pub fn smart_lock<T, F>(mutex: &Arc<Mutex<T>>, f: F)
where
    F: FnOnce(&mut T),
{
    let mut rng = rand::thread_rng();
    let start_time = Instant::now();
    let timeout = Duration::from_secs(10); // Example timeout of 10 seconds

    loop {
        match mutex.try_lock() {
            Ok(mut guard) => {
                f(&mut guard);
                return;
            },
            Err(_) => {
                if start_time.elapsed() > timeout {
                    eprintln!("Failed to acquire lock after {:?}, giving up", timeout);
                    return;
                }
                let sleep_duration = Duration::from_millis(10) + Duration::from_millis(rng.gen_range(0..10));
                thread::sleep(sleep_duration);
            },
        }
    }
}
