use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, format};

// Define a custom type that can be either Empty, Map, or Error
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum ResultType {
    Map(HashMap<String, ResultType>),
    List(Vec<ResultType>),
    Str(String),
    Int(i32),
    Float(f64),
    Bool(bool),
    Empty,
    Error(String), // Assuming Error variant holds a String
                   // ... any other variants you might have
}

// Implement Display for ResultType to be able to print it
impl fmt::Display for ResultType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResultType::Empty => write!(f, "Empty"),
            ResultType::Str(s) => write!(f, "\"{}\"", s),
            ResultType::Int(i) => write!(f, "{}", i),
            ResultType::Float(fl) => write!(f, "{}", fl),
            ResultType::Bool(b) => write!(f, "{}", b),
            ResultType::List(list) => {
                write!(f, "[")?;
                for (index, item) in list.iter().enumerate() {
                    if index != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            },
            ResultType::Map(map) => {
                write!(f, "{{")?;
                let mut first = true;
                for (key, value) in map {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "\"{}\": {}", key, value)?;
                    first = false;
                }
                write!(f, "}}")
            },
            ResultType::Error(err) => write!(f, "Error: {}", err),
        }
    }
}
