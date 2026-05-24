use std::fmt::{self, Display, Write};

#[derive(Debug, Clone)]
pub enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

impl Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", display(self))
    }
}

#[must_use]
pub fn display(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => String::from("null"),
        JsonValue::Boolean(true) => String::from("true"),
        JsonValue::Boolean(false) => String::from("false"),
        JsonValue::Number(n) => format!("{n}"),
        JsonValue::String(s) => format!("\"{s}\""),
        JsonValue::Array(arr) => {
            let mut result = String::from("[");
            let mut first = true;
            for el in arr {
                if !first {
                    result.push_str(", ");
                }
                result.push_str(&display(el));
                first = false;
            }
            result.push(']');
            result
        }
        JsonValue::Object(pairs) => {
            let mut result = String::from("{");
            let mut first = true;
            for (key, value) in pairs {
                if !first {
                    result.push_str(", ");
                }
                let _ = write!(result, "\"{key}\": {}", display(value));
                first = false;
            }
            result.push('}');
            result
        }
    }
}
