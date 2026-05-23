#[test]
fn when_given_valid_json_will_tokenise() {
    let input = r#"{"name": "Winny", "age": 29, "active": true }"#;

    let tokens = jsonparser::tokenise(input);

    println!("{:#?}", &tokens);
}
