use lazy_static;
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutext};

// Define a struct for the global pool that can hold any type of data
struct GlobalPool<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pool: Mutext<HashMap<K, V>>,
}

impl<K, V> GlobalPool<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    // Initialize a new global pool
    fn new() -> GlobalPool<K, V> {
        GlobalPool { pool: Mutext::new(HashMap::new()) }
    }

    fn add_data(&self, key: K, value: V) {
        let mut pool_guard = self.pool.lock().unwrap();
        pool_guard.insert(key, value);
    }
}

#[derive(Clone)] // Ensure your data type is cloneable.
struct Data {
    // Your data structure here
}

lazy_static! {
    static ref Globals: Arc<Mutex<HashMap<String, Value>>> = {
        let command_patterns: HashMap<String, Value> = from_str(json_str).unwrap();
        Arc::new(Mutex::new(command_patterns))
    };
}

// Store the Global data inside a HashMap of globals
