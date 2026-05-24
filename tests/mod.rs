#[test]
fn when_given_valid_json_will_tokenise() {
    let input = r#"{"name": "Winny", "age": 29, "active": true }"#;

    let tokens = jsonparser::tokenise(input);

    assert!(tokens.len().gt(&1))
}

#[test]
fn when_given_valid_json_will_parse() {
    let input = r#"{"name": "Soraya", "age": 30, "active": true }"#;

    let tokens = jsonparser::tokenise(input);

    let value = jsonparser::parse(tokens);

    assert_eq!(
        format!("{}", value),
        r#"{"name": "Soraya", "age": 30, "active": true}"#
    );
}

#[test]
fn when_given_nested_json_object_will_parse() {
    let input = r#"{"user": {"name": "Soraya", "meta": {"active": true}}, "age": 30}"#;

    let tokens = jsonparser::tokenise(input);

    let value = jsonparser::parse(tokens);

    assert_eq!(
        format!("{}", value),
        r#"{"user": {"name": "Soraya", "meta": {"active": true}}, "age": 30}"#
    );
}

#[test]
fn when_given_empty_object_will_parse() {
    let input = r#"{}"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(format!("{}", value), r#"{}"#);
}

#[test]
fn when_given_empty_array_will_parse() {
    let input = r#"[]"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(format!("{}", value), r#"[]"#);
}

#[test]
fn when_given_array_with_mixed_primitives_will_parse() {
    let input = r#"[null, true, false, -12.5, "text"]"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(
        format!("{}", value),
        r#"[null, true, false, -12.5, "text"]"#
    );
}

#[test]
fn when_given_escaped_and_unicode_string_will_parse() {
    let input = r#"{"text": "line\nquote: \" and snowman: \u2603"}"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(
        format!("{}", value),
        "{\"text\": \"line\nquote: \" and snowman: ☃\"}"
    );
}

#[test]
#[should_panic(expected = "Expected ':' after object key")]
fn when_given_object_without_colon_will_panic() {
    let input = r#"{"name" "Soraya"}"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Expected String key")]
fn when_given_object_with_non_string_key_will_panic() {
    let input = r#"{1: "value"}"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Expected ',' or ']' in array")]
fn when_given_array_with_missing_comma_will_panic() {
    let input = r#"[1 2]"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}
