use crate::constants::{BOM, CARRIAGE_RETURN, NEW_LINE, SPACE, TAB};
use crate::helpers::is_line_terminator;
use crate::lsp_types::{Position, Range};
use crate::tokens::{char_to_punctuator, LexicalToken, LexicalTokenType};

pub struct Lexer {
    source: String,
    ptr: usize,
    character: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: String) -> Lexer {
        Lexer {
            source,
            ptr: 0,
            character: 0,
            line: 0,
        }
    }

    pub fn lex(&mut self) -> Vec<LexicalToken> {
        let mut tokens: Vec<LexicalToken> = Vec::new();

        while let Some(c) = self.peek() {
            match c {
                // Ignored tokens
                // https://spec.graphql.org/October2021/#sec-Language.Source-Text.Ignored-Tokens
                SPACE | TAB | BOM | ',' => {
                    self.next();
                }

                NEW_LINE | CARRIAGE_RETURN => {
                    self.character = 0;
                    self.line += 1;
                    self.next();
                }

                // Comments
                // https://spec.graphql.org/October2021/#sec-Comments
                '#' => self.ignore_while(|c| !is_line_terminator(c)),

                // Punctuators
                // https://spec.graphql.org/October2021/#sec-Punctuators
                '!' | '$' | '&' | '(' | ')' | '.' | ':' | '=' | '@' | '[' | ']' | '{' | '}'
                | '|' => {
                    // TODO: handle ellipsis
                    let punctuator = char_to_punctuator(c);
                    let character = self.character;

                    self.next();

                    tokens.push(LexicalToken::new(
                        LexicalTokenType::Punctuator(punctuator),
                        Range::new(
                            Position::new(self.line, character),
                            Position::new(self.line, self.character),
                        ),
                    ));
                }

                '"' => {
                    let character = self.character;
                    let line = self.line;

                    self.next();
                    let value = self.consume_while(|c| c != '"'); // TODO: handle unexpected EOF
                    self.next();

                    tokens.push(LexicalToken::new(
                        LexicalTokenType::StringValue(value.clone()),
                        Range::new(
                            Position::new(line, character),
                            Position::new(self.line, self.character),
                        ),
                    ));
                }

                _ => {
                    let character = self.character;
                    let line = self.line;

                    // TODO: float value
                    if c.is_ascii_digit() {
                        let value = self.consume_while(|c| c.is_ascii_digit());

                        tokens.push(LexicalToken::new(
                            LexicalTokenType::IntValue(value.parse().unwrap()),
                            Range::new(
                                Position::new(line, character),
                                Position::new(self.line, self.character),
                            ),
                        ));
                    } else if c.is_ascii_alphabetic() || c == '_' {
                        let value = self.consume_while(|c| c.is_ascii_alphanumeric() || c == '_');

                        tokens.push(LexicalToken::new(
                            LexicalTokenType::Name(value.clone()),
                            Range::new(
                                Position::new(line, character),
                                Position::new(self.line, self.character),
                            ),
                        ));
                    } else {
                        panic!("Unexpected character: '{}'", c);
                    }
                }
            }
        }

        tokens.push(LexicalToken::new(
            LexicalTokenType::EOF,
            Range::new(
                Position::new(self.line, self.character),
                Position::new(self.line, self.character),
            ),
        ));

        tokens
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.ptr)
    }

    fn next(&mut self) -> Option<char> {
        let next_char = self.peek();
        self.ptr += 1;
        self.character += 1;
        next_char
    }

    fn consume_while<F>(&mut self, condition: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if condition(c) {
                result.push(c);
                self.next();
            } else {
                break;
            }

            if c == NEW_LINE || c == CARRIAGE_RETURN {
                self.character = 0;
                self.line += 1;
            }
        }
        result
    }

    fn ignore_while<F>(&mut self, condition: F)
    where
        F: Fn(char) -> bool,
    {
        while let Some(c) = self.peek() {
            if condition(c) {
                self.next();
            } else {
                break;
            }

            if c == NEW_LINE || c == CARRIAGE_RETURN {
                self.character = 0;
                self.line += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_tokenizes_string_values() {
        let source = String::from("\"Hello, World!\"");
        let tokens = Lexer::new(source).lex();
        let token = tokens.first().unwrap();

        assert_eq!(
            token.token_type,
            LexicalTokenType::StringValue(String::from("Hello, World!"))
        );
        assert_eq!(token.position.start.line, 0);
        assert_eq!(token.position.start.character, 0);

        assert_eq!(token.position.end.line, 0);
        assert_eq!(token.position.end.character, 15);
    }
}
