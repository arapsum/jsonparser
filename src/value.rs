use std::fmt::{self, Display, Write};

/// A parsed JSON value.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    /// A JSON object.
    Object(
        /// Object members stored in source order as key/value pairs.
        Vec<(String, JsonValue)>,
    ),
    /// A JSON array.
    Array(
        /// Array elements stored in source order.
        Vec<JsonValue>,
    ),
    /// A JSON string.
    String(
        /// String contents after escape decoding.
        String,
    ),
    /// A JSON number.
    Number(
        /// Numeric value stored as `f64`.
        f64,
    ),
    /// A JSON boolean.
    Boolean(
        /// Boolean value.
        bool,
    ),
    /// The JSON `null` value.
    Null,
}

impl Display for JsonValue {
    /// Formats a JSON value in normalized JSON-like form.
    ///
    /// # Errors
    ///
    /// Returns any formatting error reported by the formatter.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_null() {
        assert_eq!(format!("{}", JsonValue::Null), "null");
    }

    #[test]
    fn display_true() {
        assert_eq!(format!("{}", JsonValue::Boolean(true)), "true");
    }

    #[test]
    fn display_false() {
        assert_eq!(format!("{}", JsonValue::Boolean(false)), "false");
    }

    #[test]
    fn display_number_integer() {
        assert_eq!(format!("{}", JsonValue::Number(42.0)), "42");
    }

    #[test]
    fn display_number_negative() {
        assert_eq!(format!("{}", JsonValue::Number(-1.0)), "-1");
    }

    #[test]
    fn display_number_decimal() {
        assert_eq!(format!("{}", JsonValue::Number(3.14)), "3.14");
    }

    #[test]
    fn display_string() {
        assert_eq!(format!("{}", JsonValue::String("hello".to_string())), "\"hello\"");
    }

    #[test]
    fn display_string_empty() {
        assert_eq!(format!("{}", JsonValue::String("".to_string())), "\"\"");
    }

    #[test]
    fn display_string_with_escapes() {
        assert_eq!(format!("{}", JsonValue::String("line1\nline2".to_string())), "\"line1\nline2\"");
    }

    #[test]
    fn display_empty_array() {
        assert_eq!(format!("{}", JsonValue::Array(vec![])), "[]");
    }

    #[test]
    fn display_array_one_element() {
        assert_eq!(
            format!("{}", JsonValue::Array(vec![JsonValue::Number(1.0)])),
            "[1]"
        );
    }

    #[test]
    fn display_array_multiple() {
        assert_eq!(
            format!("{}", JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::String("two".to_string()),
                JsonValue::Boolean(true),
                JsonValue::Null,
            ])),
            r#"[1, "two", true, null]"#
        );
    }

    #[test]
    fn display_empty_object() {
        assert_eq!(format!("{}", JsonValue::Object(vec![])), "{}");
    }

    #[test]
    fn display_object_single_pair() {
        assert_eq!(
            format!("{}", JsonValue::Object(vec![
                ("key".to_string(), JsonValue::String("value".to_string())),
            ])),
            r#"{"key": "value"}"#
        );
    }

    #[test]
    fn display_object_multiple_pairs() {
        assert_eq!(
            format!("{}", JsonValue::Object(vec![
                ("a".to_string(), JsonValue::Number(1.0)),
                ("b".to_string(), JsonValue::Boolean(true)),
                ("c".to_string(), JsonValue::Null),
            ])),
            r#"{"a": 1, "b": true, "c": null}"#
        );
    }

    #[test]
    fn display_nested_array() {
        let inner = JsonValue::Array(vec![JsonValue::Number(1.0), JsonValue::Number(2.0)]);
        let outer = JsonValue::Array(vec![inner, JsonValue::Number(3.0)]);
        assert_eq!(format!("{}", outer), "[[1, 2], 3]");
    }

    #[test]
    fn display_nested_object() {
        let inner = JsonValue::Object(vec![("x".to_string(), JsonValue::Number(1.0))]);
        let outer = JsonValue::Object(vec![("outer".to_string(), inner)]);
        assert_eq!(format!("{}", outer), r#"{"outer": {"x": 1}}"#);
    }

    #[test]
    fn display_mixed_nesting() {
        let obj = JsonValue::Object(vec![
            ("name".to_string(), JsonValue::String("test".to_string())),
            ("nums".to_string(), JsonValue::Array(vec![
                JsonValue::Number(1.0),
                JsonValue::Number(2.0),
            ])),
        ]);
        assert_eq!(
            format!("{}", obj),
            r#"{"name": "test", "nums": [1, 2]}"#
        );
    }
}
