#![cfg(test)]

use super::*;

#[test]
fn it_tokenizes_string_values() {
    let source = String::from("\"Hello, World!\"");
    let tokens = lex(source).unwrap();
    let token = tokens.first().unwrap();
    assert_eq!(
        token.token_type,
        LexicalTokenType::StringValue(String::from("Hello, World!"))
    );
}

#[test]
fn it_tokenizes_string_values_with_escaped_characters() {
    let source = String::from("\"Hello,\\nWorld!\"");
    let tokens = lex(source).unwrap();
    let token = tokens.first().unwrap();
    assert_eq!(
        token.token_type,
        LexicalTokenType::StringValue(String::from("Hello,\nWorld!"))
    );
}

#[test]
fn it_errs_if_string_is_unterminated() {
    let source = String::from("\"Hello, World!");
    let result = lex(source);
    assert!(result.is_err());
}

#[test]
fn it_errs_if_string_has_line_break() {
    let source = String::from("\"Hello,\nWorld!\"");
    let result = lex(source);
    assert!(result.is_err());
}

#[test]
fn it_tokenizes_ellipsis() {
    let source = String::from("...");
    let tokens = lex(source).unwrap();
    let token = tokens.first().unwrap();
    assert_eq!(
        token.token_type,
        LexicalTokenType::Punctuator(Punctuator::Ellipsis)
    );
}

#[test]
fn it_tokenizes_valid_names() {
    let valid_names = vec![
        "name",
        "_name",
        "__name",
        "test_name",
        "name123",
        "name_123",
    ];

    for name in valid_names {
        let source = String::from(name);
        let tokens = lex(source).unwrap();
        let token = tokens.first().unwrap();
        assert_eq!(token.token_type, LexicalTokenType::Name(String::from(name)));
    }
}

#[test]
fn it_tokenizes_valid_int_values() {
    let valid_int_values = vec!["0", "123", "1234567890", "-123", "-1234567890"];

    for value in valid_int_values {
        let source = String::from(value);
        let tokens = lex(source).unwrap();
        let token = tokens.first().unwrap();
        assert_eq!(
            token.token_type,
            LexicalTokenType::IntValue(value.parse().unwrap())
        );
    }
}

#[test]
fn it_tokenizes_valid_float_values() {
    let valid_float_values = vec![
        "0.0",
        "0.0000",
        "123.456",
        "123.000001",
        "1234567890.1234567890",
        "-123.456",
        "-1234567890.1234567890",
    ];

    for value in valid_float_values {
        let source = String::from(value);
        let tokens = lex(source).unwrap();
        let token = tokens.first().unwrap();
        assert_eq!(
            token.token_type,
            LexicalTokenType::FloatValue(value.parse().unwrap())
        );
    }
}

#[test]
fn it_does_not_tokenize_invalid_number_values() {
    let invalid_int_values = vec![
        // "01", // TODO
        "-", ".0", ".0",
    ];

    for value in invalid_int_values {
        let source = String::from(value);
        let result = lex(source);
        assert!(result.is_err());
    }
}

#[test]
fn it_returns_error_on_invalid_character() {
    let source = String::from("?");
    let result = lex(source);
    assert!(result.is_err());
}
