// ── Valid JSON: tokenisation ─────────────────────────────────────

#[test]
fn when_given_valid_json_will_tokenise() {
    let input = r#"{"name": "Winny", "age": 29, "active": true }"#;

    let tokens = jsonparser::tokenise(input);

    assert!(tokens.len().gt(&1))
}

#[test]
fn when_given_only_whitespace_will_tokenise_to_empty() {
    assert!(jsonparser::tokenise("").is_empty());
    assert!(jsonparser::tokenise("  \t\n\r  ").is_empty());
}

// ── Valid JSON: objects ──────────────────────────────────────────

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
fn when_given_object_with_multiple_types_will_parse() {
    let input = r#"{"str": "hello", "num": 42, "dec": -3.14, "yes": true, "no": false, "nothing": null}"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(
        format!("{}", value),
        r#"{"str": "hello", "num": 42, "dec": -3.14, "yes": true, "no": false, "nothing": null}"#
    );
}

#[test]
fn when_given_object_with_various_whitespace_will_parse() {
    let input = "{\n\t\"key\"\r:\n\"val\"\n}";

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(format!("{}", value), r#"{"key": "val"}"#);
}

#[test]
fn when_given_deeply_nested_objects_will_parse() {
    let input = r#"{"a": {"b": {"c": {"d": {"e": true}}}}}"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(
        format!("{}", value),
        r#"{"a": {"b": {"c": {"d": {"e": true}}}}}"#
    );
}

// ── Valid JSON: arrays ───────────────────────────────────────────

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
fn when_given_nested_arrays_will_parse() {
    let input = r#"[[1, 2], [3, [4, 5]]]"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(format!("{}", value), r#"[[1, 2], [3, [4, 5]]]"#);
}

#[test]
fn when_given_array_of_objects_will_parse() {
    let input = r#"[{"x": 1}, {"y": 2}, {"z": 3}]"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(
        format!("{}", value),
        r#"[{"x": 1}, {"y": 2}, {"z": 3}]"#
    );
}

#[test]
fn when_given_object_with_array_values_will_parse() {
    let input = r#"{"name": "nums", "values": [1, 2, 3], "meta": {"count": 3}}"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(
        format!("{}", value),
        r#"{"name": "nums", "values": [1, 2, 3], "meta": {"count": 3}}"#
    );
}

// ── Valid JSON: numbers ──────────────────────────────────────────

#[test]
fn when_given_various_numbers_will_parse() {
    let input = r#"[0, -0, 42, -42, 3.14, -3.14, 0.5, -0.5]"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(
        format!("{}", value),
        r#"[0, -0, 42, -42, 3.14, -3.14, 0.5, -0.5]"#
    );
}

// ── Valid JSON: strings with escape sequences ────────────────────

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
fn when_given_all_escape_sequences_will_parse() {
    let input = r#"{"escapes": "\" \\ \/ \n \t \r \b \f"}"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(
        format!("{}", value),
        "{\"escapes\": \"\" \\ / \n \t \r \u{8} \u{c}\"}"
    );
}

#[test]
fn when_given_unicode_escapes_will_parse() {
    let input = r#"{"uni": "\u0048\u0065\u006c\u006c\u006f \u0057\u006f\u0072\u006c\u0064"}"#;

    let tokens = jsonparser::tokenise(input);
    let value = jsonparser::parse(tokens);

    assert_eq!(
        format!("{}", value),
        "{\"uni\": \"Hello World\"}"
    );
}

// ── Invalid JSON: panic on bad syntax ────────────────────────────

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

#[test]
#[should_panic(expected = "Expected ',' or '}' in Object")]
fn when_given_object_with_missing_comma_will_panic() {
    let input = r#"{"a": 1 "b": 2}"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Expected String key")]
fn when_given_object_with_trailing_comma_will_panic() {
    let input = r#"{"a": 1,}"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Unexpected ']'")]
fn when_given_array_with_trailing_comma_will_panic() {
    let input = r#"[1,]"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Unexpected end of token stream")]
fn when_given_unclosed_object_will_panic() {
    let input = r#"{"a": 1"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Unexpected end")]
fn when_given_unclosed_array_will_panic() {
    let input = r#"[1, 2"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Unexpected end of token stream")]
fn when_given_empty_input_to_parse_will_panic() {
    let tokens = jsonparser::tokenise("");
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Unexpected '}'")]
fn when_given_unopened_object_close_will_panic() {
    let input = r#"}"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Unexpected ']'")]
fn when_given_unopened_array_close_will_panic() {
    let input = r#"]"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Unexpected ':'")]
fn when_given_stray_colon_will_panic() {
    let input = r#":"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Unexpected ','")]
fn when_given_stray_comma_will_panic() {
    let input = r#","#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

// ── Invalid JSON: tokeniser panics ───────────────────────────────

#[test]
#[should_panic(expected = "Unterminated String")]
fn when_given_unterminated_string_will_panic() {
    let input = r#"{"key": "unterminated"#;

    let tokens = jsonparser::tokenise(input);
    let _ = jsonparser::parse(tokens);
}

#[test]
#[should_panic(expected = "Unexpected character")]
fn when_given_invalid_character_will_panic() {
    let input = r#"{"key": @}"#;

    jsonparser::tokenise(input);
}
