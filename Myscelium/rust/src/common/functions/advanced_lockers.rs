use rand::Rng;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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

    loop {
        match mutex.try_lock() {
            Ok(mut guard) => {
                f(&mut guard);
                return;
            },
            Err(_) => {
                let sleep_duration = Duration::from_millis(10) + Duration::from_millis(rng.gen_range(0..10));
                thread::sleep(sleep_duration);
            },
        }
    }
}
