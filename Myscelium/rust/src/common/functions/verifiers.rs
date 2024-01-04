use serde_json::{Error as JsonError, Value};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ComparatorError {
    MissingKey(String),
    LengthMismatch,
    TargetIsEmpty,
    TypeMismatch(Value),
    ParseError(String),
}

/// Compares two JSON values, `val` and `target`, and attempts to convert `val` to match the structure
/// and types of `target`. This function handles basic conversions like integer to boolean or
/// string to integer, and ensures structural similarity for objects and arrays.
///
/// # Arguments
///
/// * `val` - The JSON value to be compared and converted.
/// * `target` - The target JSON structure to compare against.
///
/// # Returns
///
/// A `Result` containing either the converted `Value` or a `ComparatorError`.
///
/// # Examples
///
/// ```
/// let val = serde_json::json!({"key1": "42", "key2": ["1", "0"]});
/// let target = serde_json::json!({"key1": 42, "key2": [true, false]});
/// let result = fast_json_comparator(&val, &target);
/// assert!(result.is_ok());
/// ```
pub fn fast_json_comparator(val: &Value, target: &Value) -> Result<Value, ComparatorError> {
    match (val, target) {
        (Value::Object(obj), Value::Object(pattern_obj)) => {
            if pattern_obj.is_empty() {
                return Err(ComparatorError::TargetIsEmpty);
            }

            let mut new_obj: HashMap<String, Value> = HashMap::new();
            for (k, pv) in pattern_obj {
                match obj.get(k) {
                    Some(v) => new_obj.insert(k.clone(), fast_json_comparator(v, pv)?),
                    None => return Err(ComparatorError::MissingKey(k.clone())),
                };
            }
            Ok(Value::Object(serde_json::Map::from_iter(new_obj.into_iter())))
        },

        (Value::Array(arr), Value::Array(pattern_arr)) => {
            if pattern_arr.is_empty() {
                return Err(ComparatorError::TargetIsEmpty);
            }

            if arr.len() != pattern_arr.len() {
                return Err(ComparatorError::LengthMismatch);
            }

            let new_arr: Result<Vec<_>, _> = arr.iter().zip(pattern_arr.iter()).map(|(elem, pattern_elem)| fast_json_comparator(elem, pattern_elem)).collect();

            Ok(Value::Array(new_arr?))
        },

        // Convert integer-like strings to numbers
        (Value::String(s), Value::Number(_)) => match s.parse::<serde_json::Number>() {
            Ok(num) => Ok(Value::Number(num)),
            Err(_) => Err(ComparatorError::ParseError(format!("Failed to parse '{}' as number", s))),
        },

        // Convert string "true" or "false" to boolean
        (Value::String(s), Value::Bool(_)) => match s.as_str() {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            _ => Err(ComparatorError::ParseError(format!("Failed to parse '{}' as bool", s))),
        },

        // Convert integer to boolean (0 -> false, otherwise -> true)
        (Value::Number(n), Value::Bool(_)) => Ok(Value::Bool(n.as_i64().unwrap_or(0) != 0)),

        // For other types, check if they are the same
        _ => {
            if val == target {
                Ok(val.clone())
            } else {
                Err(ComparatorError::TypeMismatch(*val))
            }
        },
    }
}
