fn parse_input(input: &str) -> jsonparser::Result<jsonparser::JsonValue> {
    let tokens = jsonparser::tokenise(input)?;
    jsonparser::parse(tokens)
}

// ── Valid JSON: tokenisation ─────────────────────────────────────

#[test]
fn when_given_valid_json_will_tokenise() {
    let input = r#"{"name": "Winny", "age": 29, "active": true }"#;

    let tokens = jsonparser::tokenise(input).unwrap();

    assert!(tokens.len().gt(&1))
}

#[test]
fn when_given_only_whitespace_will_tokenise_to_empty() {
    assert!(jsonparser::tokenise("").unwrap().is_empty());
    assert!(jsonparser::tokenise("  \t\n\r  ").unwrap().is_empty());
}

// ── Valid JSON: objects ──────────────────────────────────────────

#[test]
fn when_given_valid_json_will_parse() {
    let input = r#"{"name": "Soraya", "age": 30, "active": true }"#;

    let value = parse_input(input).unwrap();

    assert_eq!(
        format!("{}", value),
        r#"{"name": "Soraya", "age": 30, "active": true}"#
    );
}

#[test]
fn when_given_nested_json_object_will_parse() {
    let input = r#"{"user": {"name": "Soraya", "meta": {"active": true}}, "age": 30}"#;

    let value = parse_input(input).unwrap();

    assert_eq!(
        format!("{}", value),
        r#"{"user": {"name": "Soraya", "meta": {"active": true}}, "age": 30}"#
    );
}

#[test]
fn when_given_empty_object_will_parse() {
    let input = r#"{}"#;

    let value = parse_input(input).unwrap();

    assert_eq!(format!("{}", value), r#"{}"#);
}

#[test]
fn when_given_object_with_multiple_types_will_parse() {
    let input =
        r#"{"str": "hello", "num": 42, "dec": -3.14, "yes": true, "no": false, "nothing": null}"#;

    let value = parse_input(input).unwrap();

    assert_eq!(
        format!("{}", value),
        r#"{"str": "hello", "num": 42, "dec": -3.14, "yes": true, "no": false, "nothing": null}"#
    );
}

#[test]
fn when_given_object_with_various_whitespace_will_parse() {
    let input = "{\n\t\"key\"\r:\n\"val\"\n}";

    let value = parse_input(input).unwrap();

    assert_eq!(format!("{}", value), r#"{"key": "val"}"#);
}

#[test]
fn when_given_deeply_nested_objects_will_parse() {
    let input = r#"{"a": {"b": {"c": {"d": {"e": true}}}}}"#;

    let value = parse_input(input).unwrap();

    assert_eq!(
        format!("{}", value),
        r#"{"a": {"b": {"c": {"d": {"e": true}}}}}"#
    );
}

// ── Valid JSON: arrays ───────────────────────────────────────────

#[test]
fn when_given_empty_array_will_parse() {
    let input = r#"[]"#;

    let value = parse_input(input).unwrap();

    assert_eq!(format!("{}", value), r#"[]"#);
}

#[test]
fn when_given_array_with_mixed_primitives_will_parse() {
    let input = r#"[null, true, false, -12.5, "text"]"#;

    let value = parse_input(input).unwrap();

    assert_eq!(
        format!("{}", value),
        r#"[null, true, false, -12.5, "text"]"#
    );
}

#[test]
fn when_given_nested_arrays_will_parse() {
    let input = r#"[[1, 2], [3, [4, 5]]]"#;

    let value = parse_input(input).unwrap();

    assert_eq!(format!("{}", value), r#"[[1, 2], [3, [4, 5]]]"#);
}

#[test]
fn when_given_array_of_objects_will_parse() {
    let input = r#"[{"x": 1}, {"y": 2}, {"z": 3}]"#;

    let value = parse_input(input).unwrap();

    assert_eq!(format!("{}", value), r#"[{"x": 1}, {"y": 2}, {"z": 3}]"#);
}

#[test]
fn when_given_object_with_array_values_will_parse() {
    let input = r#"{"name": "nums", "values": [1, 2, 3], "meta": {"count": 3}}"#;

    let value = parse_input(input).unwrap();

    assert_eq!(
        format!("{}", value),
        r#"{"name": "nums", "values": [1, 2, 3], "meta": {"count": 3}}"#
    );
}

// ── Valid JSON: numbers ──────────────────────────────────────────

#[test]
fn when_given_various_numbers_will_parse() {
    let input = r#"[0, -0, 42, -42, 3.14, -3.14, 0.5, -0.5]"#;

    let value = parse_input(input).unwrap();

    assert_eq!(
        format!("{}", value),
        r#"[0, -0, 42, -42, 3.14, -3.14, 0.5, -0.5]"#
    );
}

// ── Valid JSON: strings with escape sequences ────────────────────

#[test]
fn when_given_escaped_and_unicode_string_will_parse() {
    let input = r#"{"text": "line\nquote: \" and snowman: \u2603"}"#;

    let value = parse_input(input).unwrap();

    assert_eq!(
        format!("{}", value),
        "{\"text\": \"line\nquote: \" and snowman: ☃\"}"
    );
}

#[test]
fn when_given_all_escape_sequences_will_parse() {
    let input = r#"{"escapes": "\" \\ \/ \n \t \r \b \f"}"#;

    let value = parse_input(input).unwrap();

    assert_eq!(
        format!("{}", value),
        "{\"escapes\": \"\" \\ / \n \t \r \u{8} \u{c}\"}"
    );
}

#[test]
fn when_given_unicode_escapes_will_parse() {
    let input = r#"{"uni": "\u0048\u0065\u006c\u006c\u006f \u0057\u006f\u0072\u006c\u0064"}"#;

    let value = parse_input(input).unwrap();

    assert_eq!(format!("{}", value), "{\"uni\": \"Hello World\"}");
}

// ── Invalid JSON: parser errors ──────────────────────────────────

#[test]
fn when_given_object_without_colon_will_error() {
    assert!(parse_input(r#"{"name" "Soraya"}"#).is_err());
}

#[test]
fn when_given_object_with_non_string_key_will_error() {
    assert!(parse_input(r#"{1: "value"}"#).is_err());
}

#[test]
fn when_given_array_with_missing_comma_will_error() {
    assert!(parse_input(r#"[1 2]"#).is_err());
}

#[test]
fn when_given_object_with_missing_comma_will_error() {
    assert!(parse_input(r#"{"a": 1 "b": 2}"#).is_err());
}

#[test]
fn when_given_object_with_trailing_comma_will_error() {
    assert!(parse_input(r#"{"a": 1,}"#).is_err());
}

#[test]
fn when_given_array_with_trailing_comma_will_error() {
    assert!(parse_input(r#"[1,]"#).is_err());
}

#[test]
fn when_given_unclosed_object_will_error() {
    assert!(parse_input(r#"{"a": 1"#).is_err());
}

#[test]
fn when_given_unclosed_array_will_error() {
    assert!(parse_input(r#"[1, 2"#).is_err());
}

#[test]
fn when_given_empty_input_to_parse_will_error() {
    let tokens = jsonparser::tokenise("").unwrap();

    assert!(jsonparser::parse(tokens).is_err());
}

#[test]
fn when_given_unopened_object_close_will_error() {
    assert!(parse_input(r#"}"#).is_err());
}

#[test]
fn when_given_unopened_array_close_will_error() {
    assert!(parse_input(r#"]"#).is_err());
}

#[test]
fn when_given_stray_colon_will_error() {
    assert!(parse_input(r#":"#).is_err());
}

#[test]
fn when_given_stray_comma_will_error() {
    assert!(parse_input(r#","#).is_err());
}

// ── Invalid JSON: tokeniser errors ───────────────────────────────

#[test]
fn when_given_unterminated_string_will_error() {
    assert!(jsonparser::tokenise(r#"{"key": "unterminated"#).is_err());
}

#[test]
fn when_given_invalid_character_will_error() {
    assert!(jsonparser::tokenise(r#"{"key": @}"#).is_err());
}
