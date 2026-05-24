fn main() {
    let input = r#"{"name": "Soraya", "age": 30, "active": true }"#;

    let tokens = jsonparser::tokenise(input);

    let value = jsonparser::parse(tokens);

    println!("{value}");
}
