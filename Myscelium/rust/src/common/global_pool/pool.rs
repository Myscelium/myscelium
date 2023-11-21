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

/*

-> In Rust, Eq and Hash are traits that define certain behaviors for types:

> Eq: This is a trait that signifies that a type has an equality comparison operation (== and !=) defined.
> It extends the PartialEq trait, which is for types that can be partially compared for equality (not all types can be).
> The Eq trait indicates that the equality operation is reflexive, symmetric, and transitive, which are the conditions needed for
> full equivalence relation.

> Hash: The Hash trait is used for types that can be hashed. This is necessary for types that you want to use as keys in a HashMap
> or HashSet. When a type implements Hash, it means that it can be transformed into a hash value. This hash value is used internally
> by hash maps or sets to quickly look up values based on a key.

> The Eq and Hash traits are commonly required for types that will be used as keys in a HashMap to ensure that the keys support
> both equality comparisons and the generation of hash values for quick retrieval.

*/

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

    fn get_data(&self, key: &K) -> Option<V> {
        let pool_guard = self.pool.lock().unwrap;
        pool_guard.get(key).cloned();
    }
}

// > -------------------------------------------------------------------------------------------------------------------------------

struct GlobalGroup<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    globals: HashMap<K, Arc<Mutex<V>>>,
}

struct GlobalPool {
    groups: HashMap<String, GlobalGroup>,
}

impl GlobalPool {
    // Initialize a new global pool
    fn new() -> Self {
        GlobalPool { groups: HashMap::new() }
    }

    // Add a global group
    fn create_group(&mut self, group_name: String, group_agents: u32, groups_data: T) {
        self.groups.insert(group_name, group);
    }

    // Access a global
    fn access_global(&self, group_name: &str, global_name: &str) -> Option<Arc<Global>> {
        self.groups.get(group_name)?.globals.get(global_name).cloned()
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
