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

impl ResultType {
    // Extract methods for each variant

    pub fn to_map(&self) -> Option<HashMap<String, ResultType>> {
        if let ResultType::Map(ref map) = &self {
            Some(map.clone())
        } else {
            None
        }
    }

    pub fn to_list(&self) -> Option<Vec<ResultType>> {
        if let ResultType::List(ref list) = &self {
            Some(list.clone())
        } else {
            None
        }
    }

    pub fn to_str(&self) -> Option<String> {
        if let ResultType::Str(ref s) = &self {
            Some(s.clone())
        } else {
            None
        }
    }

    pub fn to_int(&self) -> Option<i32> {
        if let ResultType::Int(i) = &self {
            Some(*i)
        } else {
            None
        }
    }

    pub fn to_float(&self) -> Option<f64> {
        if let ResultType::Float(f) = &self {
            Some(*f)
        } else {
            None
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        if let ResultType::Bool(b) = &self {
            Some(*b)
        } else {
            None
        }
    }

    pub fn to_error(&self) -> Option<String> {
        if let ResultType::Error(ref err) = &self {
            Some(err.clone())
        } else {
            None
        }
    }
}

pub enum ExpectationError {
    Missingkwarg(String),
    MismatchType(String),
    TargetIsEmpty,
    MismatchRelativeLength,
}

/// To an given case:
///
/// ```ignore
/// ResultType::List(Vec(ResultType::List(Vec(ResultType::Float, ResultType::Int)), ResultType::Int))
/// ```
///
/// We may want to try to iterate recursivelly into the List checking its types and if some of the types dont match we can return a bool
/// The structure here will be like a tree, we will ramificate the tests and we can also add some multithreading to help with large data
/// Checking.
impl ResultType {
    fn type_of(&self) -> &'static str {
        match self {
            ResultType::Map(_) => "Map",
            ResultType::List(_) => "List",
            ResultType::Str(_) => "Str",
            ResultType::Int(_) => "Int",
            ResultType::Float(_) => "Float",
            ResultType::Bool(_) => "Bool",
            ResultType::Empty => "Empty",
            ResultType::Error(_) => "Error",
        }
    }

    pub fn fast_verify_kwargs_and_types(&self, target: &ResultType) -> Result<(), ExpectationError> {
        match (self, target) {
            // Recursively check map types
            (ResultType::Map(map), ResultType::Map(target_map)) => {
                // For simplicity, we'll assume that if the target_map is empty, we just want to return false
                if target_map.is_empty() {
                    return Err(ExpectationError::TargetIsEmpty);
                }

                for (tk, tv) in target_map {
                    // Case where the map doesn't contain the expected key
                    if !&map.contains_key(tk) {
                        return Err(ExpectationError::Missingkwarg(tk.clone()));
                    }

                    // Check if inner ResultsTypes are correct
                    map.get(tk).unwrap().fast_verify_kwargs_and_types(tv)?;
                }

                return Ok(());
            },

            // Recursively check list types
            (ResultType::List(list), ResultType::List(target_list)) => {
                if target_list.is_empty() {
                    return Err(ExpectationError::TargetIsEmpty);
                }

                if list.len() != target_list.len() {
                    return Err(ExpectationError::MismatchRelativeLength);
                }

                // Otherwise, we'll use the first entry in target_map as the example structure for all values in list.
                for (i, element) in list.iter().enumerate() {
                    element.fast_verify_kwargs_and_types(&target_list[i])?;
                }

                return Ok(());
            },

            _ => {
                if std::mem::discriminant(self) == std::mem::discriminant(target) {
                    return Ok(());
                } else {
                    return Err(ExpectationError::MismatchType(self.type_of().to_string()));
                }
            },
        }
    }
}
