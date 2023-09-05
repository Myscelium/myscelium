use crate::commom::enhanced_buffer::utilities::Command;
use crate::commom::structs::results_structs::ResultType;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::MutexGuard;

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

pub fn convert_to_value_map(dict: &HashMap<String, ResultType>) -> HashMap<String, Value> {
    dict.iter().map(|(k, v)| (k.clone(), resulttype_to_value(v))).collect()
}

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
