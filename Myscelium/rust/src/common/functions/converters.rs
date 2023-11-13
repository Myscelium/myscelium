use crate::common::enhanced_buffer::utilities::Command;
use crate::common::structs::results_structs::ResultType;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::MutexGuard;

/// Converts a `HashMap<String, ResultType>` into another `HashMap<String, ResultType>`
/// with specific transformations.
///
/// This function goes through each entry in the map and based on the type of the value,
/// it performs specific transformations. Currently, it supports cloning strings and
/// recursively converting nested maps.
///
/// # Arguments
///
/// * `m` - The input map to be converted.
///
/// # Returns
///
/// * A transformed `HashMap<String, ResultType>`.
pub fn convert_to_resulttype_map(m: &HashMap<String, ResultType>) -> HashMap<String, ResultType> {
    m.iter()
        .filter_map(|(k, v)| {
            match v {
                ResultType::Str(s) => Some((k.clone(), ResultType::Str(s.clone()))),
                ResultType::Map(inner_map) => {
                    // Convert the inner map recursively
                    let inner_resulttype_map = convert_to_resulttype_map(inner_map);
                    Some((k.clone(), ResultType::Map(inner_resulttype_map)))
                },
                // Add other cases for different ResultType variants if needed
                _ => None,
            }
        })
        .collect()
}

/// Converts a `serde_json::Value` into a `ResultType`.
///
/// This function takes a reference to a `serde_json::Value` and attempts to convert it into a
/// corresponding `ResultType`. The function returns a `Result` wrapping the `ResultType` to handle
/// potential conversion errors.
///
/// # Parameters
/// - `value`: A reference to the `serde_json::Value` to be converted.
///
/// # Returns
/// - `Ok(ResultType)`: If the conversion is successful.
/// - `Err(String)`: If the conversion fails, returning an error message as a `String`.
///
/// # Examples
/// ```
/// use serde_json::json;
/// use your_module::ResultType;
/// use your_module::value_to_resulttype;
///
/// let value = json!("Hello, World!");
/// let result_type = value_to_resulttype(&value);
/// assert!(matches!(result_type, Ok(ResultType::Str(_))));
/// ```
pub fn value_to_resulttype(value: &Value) -> Result<ResultType, String> {
    match value {
        Value::String(s) => Ok(ResultType::Str(s.clone())),
        Value::Number(n) => {
            if n.is_i64() {
                Ok(ResultType::Int(n.as_i64().unwrap()))
            } else if n.is_f64() {
                Ok(ResultType::Float(n.as_f64().unwrap()))
            } else {
                Err("Number is neither i64 nor f64".to_string())
            }
        },
        Value::Bool(b) => Ok(ResultType::Bool(*b)),
        Value::Object(map) => {
            let mut result_map = HashMap::new();
            for (k, v) in map.iter() {
                result_map.insert(k.clone(), value_to_resulttype(v)?);
            }
            Ok(ResultType::Map(result_map))
        },
        Value::Array(list) => {
            let results: Result<Vec<_>, _> = list.iter().map(value_to_resulttype).collect();
            results.map(ResultType::List)
        },
        Value::Null => Ok(ResultType::Empty),
        // Handling of other Value variants if needed
        _ => Err("Unsupported Value variant".to_string()),
    }
}

/// Converts a `ResultType` into its corresponding `serde_json::Value` representation.
///
/// This function is useful when you need to serialize a `ResultType` into JSON.
///
/// # Arguments
///
/// * `result` - The `ResultType` instance to be converted.
///
/// # Returns
///
/// * The corresponding `serde_json::Value` representation of the input `ResultType`.
pub fn resulttype_to_value(result: &ResultType) -> Value {
    match result {
        ResultType::Str(s) => Value::String(s.clone()),
        ResultType::Int(i) => Value::Number(serde_json::Number::from(*i)),
        ResultType::Float(f) => {
            // Assuming that the f64 can be safely converted to a serde_json::Number
            Value::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| serde_json::Number::from(0)))
        },
        ResultType::Bool(b) => Value::Bool(*b),
        ResultType::Map(map) => {
            let mut result_obj = serde_json::Map::new();
            for (k, v) in map.iter() {
                result_obj.insert(k.clone(), resulttype_to_value(v));
            }
            Value::Object(result_obj)
        },
        ResultType::List(list) => {
            let values: Vec<Value> = list.iter().map(resulttype_to_value).collect();
            Value::Array(values)
        },
        ResultType::Empty => Value::Null,
        ResultType::Error(err) => Value::String(format!("Error: {}", err)),
        // Add transformations for other ResultType variants if needed
    }
}

/// Converts a `HashMap<String, ResultType>` into a `HashMap<String, Value>`.
///
/// This function is a utility to transform a map of `ResultType` values into their
/// corresponding JSON representations.
///
/// # Arguments
///
/// * `dict` - The input map to be converted.
///
/// # Returns
///
/// * A `HashMap<String, Value>` with values converted into their `serde_json::Value` representations.
pub fn convert_to_value_map(dict: &HashMap<String, ResultType>) -> HashMap<String, Value> {
    dict.iter().map(|(k, v)| (k.clone(), resulttype_to_value(v))).collect()
}

/// Recursively deserializes a JSON string into a `Command` structure.
///
/// This function is useful when the "response" field of a `Command` contains another serialized `Command` as a string.
/// It will deserialize the outer command, and if a nested serialized command is found, it will recursively deserialize it as well.
///
/// # Arguments
///
/// * `json_str` - The JSON string representation of a `Command`.
///
/// # Returns
///
/// * A `Command` structure with any nested serialized commands also deserialized.
pub fn recursive_deserialize_command(json_str: &str) -> Command {
    let mut command: Command = serde_json::from_str(json_str).unwrap();

    if let Some(response) = &command.command.get("response") {
        if let Value::String(inner_json) = response {
            let inner_command = recursive_deserialize_command(inner_json);
            command.command.insert("response".to_string(), serde_json::to_value(inner_command).unwrap());
        }
    }

    command
}
