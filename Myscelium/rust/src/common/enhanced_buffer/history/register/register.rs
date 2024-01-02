use lazy_static::lazy_static;
use std::fs::OpenOptions;
use std::{
    fs::File,
    sync::{Arc, Mutex},
};

use std::io::Write;

use crate::common::functions::advanced_lockers::smart_lock;

lazy_static! {
    static ref FILE: Arc<Mutex<Option<File>>> = Arc::new(Mutex::new(None));
}

pub fn initialize_buffer_history(file_path: &str) {
    // Open the file in append mode at the given file path
    let file = OpenOptions::new().create(true).append(true).open(file_path).unwrap();

    // Update FILE with the opened file
    let mut file_ref = FILE.lock().unwrap();
    *file_ref = Some(file);
}

pub fn write_to_file(text: String) {
    let file = &FILE;
    smart_lock(file, |file_option: &mut Option<File>| {
        if let Some(f) = file_option {
            writeln!(f, "{}", text).unwrap();
        } else {
            // Handle the case where the file is not initialized
            println!("File is not initialized.");
        }
    });
}
