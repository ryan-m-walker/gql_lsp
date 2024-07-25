use crate::constants::{BOM, CARRIAGE_RETURN, NEW_LINE, SPACE, TAB};
use crate::helpers::is_line_terminator;
use crate::lexer::types::{char_to_punctuator, LexicalToken, LexicalTokenType, Punctuator};
use crate::lsp::types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub mod types;

mod tests;

pub fn lex(source: String) -> Result<Vec<LexicalToken>, Diagnostic> {
    let mut lexer = Lexer::new(source);
    lexer.lex()
}

struct Lexer {
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

    pub fn lex(&mut self) -> Result<Vec<LexicalToken>, Diagnostic> {
        let mut tokens: Vec<LexicalToken> = Vec::new();

        while let Some(c) = self.peek() {
            match c {
                // Ignored tokens
                // https://spec.graphql.org/October2021/#sec-Language.Source-Text.Ignored-Tokens
                SPACE | TAB | BOM | ',' => {
                    self.next();
                }

                NEW_LINE | CARRIAGE_RETURN => {
                    self.line += 1;
                    self.next();
                    self.character = 0;
                }

                // Comments
                // https://spec.graphql.org/October2021/#sec-Comments
                '#' => self.ignore_while(|c| !is_line_terminator(c)),

                // Punctuators
                // https://spec.graphql.org/October2021/#sec-Punctuators
                '!' | '$' | '&' | '(' | ')' | ':' | '=' | '@' | '[' | ']' | '{' | '}' | '|' => {
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
                '.' => {
                    let start_position = Position::new(self.line, self.character);

                    self.next();
                    self.expect_peek('.')?;
                    self.next();
                    self.expect_peek('.')?;
                    self.next();

                    tokens.push(LexicalToken::new(
                        LexicalTokenType::Punctuator(Punctuator::Ellipsis),
                        Range::new(start_position, Position::new(self.line, self.character)),
                    ));
                }
                '"' => {
                    // if self.peek_at(1) == Some('"') && self.peek_at(2) == Some('"') {
                    //     tokens.push(self.tokenize_block_string()?);
                    // } else {
                    tokens.push(self.tokenize_string()?);
                    // }
                }

                '-' => {
                    tokens.push(self.tokenize_number()?);
                }

                _ => {
                    let character = self.character;
                    let line = self.line;

                    if c.is_ascii_digit() {
                        tokens.push(self.tokenize_number()?);
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
                        return Err(Diagnostic::new(
                            DiagnosticSeverity::Error,
                            String::from(format!("Unexpected character: {}", c)),
                            Range::new(
                                Position::new(line, character),
                                Position::new(self.line, self.character),
                            ),
                        ));
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

        Ok(tokens)
    }

    fn tokenize_string(&mut self) -> Result<LexicalToken, Diagnostic> {
        let start_position = Position::new(self.line, self.character);
        self.expect_next('"')?;

        let mut result = String::new();

        while let Some(c) = self.peek() {
            if c == '"' {
                self.next();
                return Ok(LexicalToken::new(
                    LexicalTokenType::StringValue(result),
                    Range::new(start_position, Position::new(self.line, self.character)),
                ));
            }

            if c == '\\' {
                self.next();
                let escaped = self.peek();

                match escaped {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    // TODO maybe
                    // Some('u')
                    _ => {
                        return Err(Diagnostic::new(
                            DiagnosticSeverity::Error,
                            String::from("Invalid character escape sequence."),
                            Range::new(
                                Position::new(self.line, self.character),
                                Position::new(self.line, self.character + 1),
                            ),
                        ));
                    }
                }

                self.next();
                continue;
            }

            if c == '\n' || c == '\r' {
                break;
            }

            result.push(c);
            self.next();
        }

        Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            String::from("Unterminated string."),
            Range::new(
                Position::new(self.line, self.character),
                Position::new(self.line, self.character + 1),
            ),
        ))
    }

    fn tokenize_block_string(&mut self) -> Result<LexicalToken, Diagnostic> {
        unimplemented!()
    }

    fn tokenize_number(&mut self) -> Result<LexicalToken, Diagnostic> {
        let sign = if let Some('-') = self.peek() {
            self.next();
            "-"
        } else {
            ""
        };

        let number_value = self.consume_while(|c| c.is_ascii_digit());

        if number_value.is_empty() {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from("Invalid number, expected digit"),
                Range::new(
                    Position::new(self.line, self.character),
                    Position::new(self.line, self.character + 1),
                ),
            ));
        }

        let next = self.peek();

        if let Some('.') = next {
            self.next();
            let decimal_value = self.consume_while(|c| c.is_ascii_digit());

            if decimal_value.is_empty() {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    String::from("Invalid number, expected digit"),
                    Range::new(
                        Position::new(self.line, self.character),
                        Position::new(self.line, self.character + 1),
                    ),
                ));
            }

            let parsed_float = format!("{}{}.{}", sign, number_value, decimal_value).parse::<f32>();

            match parsed_float {
                Ok(value) => {
                    return Ok(LexicalToken::new(
                        LexicalTokenType::FloatValue(value),
                        Range::new(
                            Position::new(self.line, self.character),
                            Position::new(
                                self.line,
                                self.character + number_value.len() + decimal_value.len(),
                            ),
                        ),
                    ));
                }
                Err(_) => {
                    return Err(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        String::from("Invalid number"),
                        Range::new(
                            Position::new(self.line, self.character),
                            Position::new(self.line, self.character + 1),
                        ),
                    ));
                }
            }
        }

        let parsed_int = format!("{}{}", sign, number_value).parse::<i32>();

        match parsed_int {
            Ok(value) => {
                return Ok(LexicalToken::new(
                    LexicalTokenType::IntValue(value),
                    Range::new(
                        Position::new(self.line, self.character),
                        Position::new(self.line, self.character + number_value.len()),
                    ),
                ));
            }
            Err(_) => {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    String::from("Invalid number"),
                    Range::new(
                        Position::new(self.line, self.character),
                        Position::new(self.line, self.character + 1),
                    ),
                ));
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.source.chars().nth(self.ptr)
    }

    fn expect_next(&mut self, expected: char) -> Result<char, Diagnostic> {
        let next = self.next();

        match next {
            Some(c) if c == expected => Ok(c),
            Some(c) => Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from(format!("Expected \"{}\", found \"{}\"", expected, c)),
                Range::new(
                    Position::new(self.line, self.character),
                    Position::new(self.line, self.character + 1),
                ),
            )),
            None => Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from(format!("Expected \"{}\", found EOF", expected)),
                Range::new(
                    Position::new(self.line, self.character),
                    Position::new(self.line, self.character + 1),
                ),
            )),
        }
    }

    fn expect_peek(&self, expected: char) -> Result<char, Diagnostic> {
        let next = self.peek();

        match next {
            Some(c) if c == expected => Ok(c),
            Some(c) => Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from(format!("Expected \"{}\", found \"{}\"", expected, c)),
                Range::new(
                    Position::new(self.line, self.character),
                    Position::new(self.line, self.character + 1),
                ),
            )),
            None => Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from(format!("Expected \"{}\", found EOF", expected)),
                Range::new(
                    Position::new(self.line, self.character),
                    Position::new(self.line, self.character + 1),
                ),
            )),
        }
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
