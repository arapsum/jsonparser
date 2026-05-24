# JSON Parser (Rust)

A small JSON parser written in Rust with a hand-built lexer and recursive-descent parser.

It can tokenise JSON text, parse it into a `JsonValue` tree, and print it back in normalized JSON form.

## Features

- Tokenises JSON input into typed tokens (`{}`, `[]`, strings, numbers, booleans, `null`)
- Parses nested objects and arrays
- Supports string escapes including Unicode (`\uXXXX`)
- Exposes both library functions and a binary example
- Includes tests for valid and invalid JSON scenarios

## Project Layout

- `src/tokeniser.rs` - Lexer/tokeniser (`tokenise`)
- `src/parser.rs` - Recursive-descent parser (`parse`)
- `src/value.rs` - `JsonValue` enum and display formatting
- `src/lib.rs` - Public library exports
- `src/bin/main.rs` - Example binary entrypoint
- `tests/mod.rs` - Integration tests

## Prerequisites

- Rust (stable) and Cargo

## Build and Run

Build the project:

```bash
cargo build
```

Run the example binary:

```bash
cargo run --bin jsonparser
```

Run tests:

```bash
cargo test
```

## Library Usage

```rust
use jsonparser::{parse, tokenise};

fn main() {
    let input = r#"{"name":"Soraya","age":30,"active":true}"#;
    let tokens = tokenise(input);
    let value = parse(tokens);
    println!("{value}");
}
```

## Supported JSON Values

- Objects
- Arrays
- Strings (with common escapes and `\uXXXX`)
- Numbers (parsed as `f64`)
- Booleans
- `null`

## Current Limitations

- Parsing errors currently panic instead of returning `Result`
- The top-level parse function parses one value and does not explicitly reject trailing tokens
