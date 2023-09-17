use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, format};

/// `ResultType` Enum
///
/// This enum represents a versatile data structure designed to encapsulate multiple types
/// including basic data types, collections, and even error messages.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum ResultType {
    /// Represents a key-value structure, where values can be of `ResultType` itself.
    Map(HashMap<String, ResultType>),
    /// Represents an ordered list of `ResultType` values.
    List(Vec<ResultType>),
    /// Represents a simple string.
    Str(String),
    /// Represents an integer value.
    Int(i32),
    /// Represents a floating-point number.
    Float(f64),
    /// Represents a boolean value.
    Bool(bool),
    /// Represents an absence of a value.
    Empty,
    /// Represents an error message.
    /// Assumption: The error variant holds a String detailing the error.
    Error(String),
}

/// Display Trait Implementation for `ResultType`
///
/// This allows for a user-friendly representation of the `ResultType` enum variants.
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
/// Utility Methods for `ResultType`
///
/// These methods allow for extracting data from the `ResultType` enum by providing
/// type-specific getter methods.
impl ResultType {
    /// Attempts to extract the `Map` variant.
    pub fn to_map(&self) -> Option<HashMap<String, ResultType>> {
        if let ResultType::Map(ref map) = &self {
            Some(map.clone())
        } else {
            None
        }
    }

    /// Attempts to extract the `List` variant.
    pub fn to_list(&self) -> Option<Vec<ResultType>> {
        if let ResultType::List(ref list) = &self {
            Some(list.clone())
        } else {
            None
        }
    }

    /// Attempts to extract the `Str` variant.
    pub fn to_str(&self) -> Option<String> {
        if let ResultType::Str(ref s) = &self {
            Some(s.clone())
        } else {
            None
        }
    }

    /// Attempts to extract the `Int` variant.
    pub fn to_int(&self) -> Option<i32> {
        if let ResultType::Int(i) = &self {
            Some(*i)
        } else {
            None
        }
    }

    /// Attempts to extract the `Float` variant.
    pub fn to_float(&self) -> Option<f64> {
        if let ResultType::Float(f) = &self {
            Some(*f)
        } else {
            None
        }
    }

    /// Attempts to extract the `Bool` variant.
    pub fn to_bool(&self) -> Option<bool> {
        if let ResultType::Bool(b) = &self {
            Some(*b)
        } else {
            None
        }
    }

    /// Attempts to extract the `Error` variant.
    pub fn to_error(&self) -> Option<String> {
        if let ResultType::Error(ref err) = &self {
            Some(err.clone())
        } else {
            None
        }
    }
}

/// `ExpectationError` Enum
///
/// This enum represents the types of errors that can occur when verifying
/// the structure and types of `ResultType` instances.
pub enum ExpectationError {
    /// Occurs when a required keyword argument is missing.
    Missingkwarg(String),
    /// Occurs when there's a type mismatch between the target and the current type.
    MismatchType(String),
    /// Occurs when the target for comparison is empty.
    TargetIsEmpty,
    /// Occurs when the relative lengths between a list and its target are different.
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
    /// Helper function to get the type of the current `ResultType` variant as a string.
    ///
    /// This makes it easier to handle type mismatches by providing a human-readable
    /// type name.
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

    /// Quickly verifies the structure and types of the current `ResultType` against a target.
    ///
    /// This function performs a recursive check of nested structures (like maps and lists)
    /// to ensure that the current instance matches the expected structure of the target.
    /// If the structures match but the types within them differ, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `target` - The `ResultType` instance that represents the expected structure and types.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the current instance matches the target in both structure and types.
    /// * `Err(ExpectationError)` if there's any mismatch.
    pub fn fast_verify_kwargs_and_types(&self, target: &ResultType) -> Result<(), ExpectationError> {
        match (self, target) {
            // Check if the Maps have matching keys and types.
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

            // Check if Lists have matching elements and types.
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

            // For other types, just check if the types match.
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
