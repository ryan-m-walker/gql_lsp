use crate::lsp::types::Range;

#[derive(Debug, Clone, PartialEq)]
pub enum Punctuator {
    ExclamationMark,
    DollarSign,
    Ampersand,
    LeftParenthesis,
    RightParenthesis,
    Ellipsis,
    Colon,
    EqualSign,
    AtSign,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    VerticalBar,
}

pub fn char_to_punctuator(c: char) -> Punctuator {
    match c {
        '!' => Punctuator::ExclamationMark,
        '$' => Punctuator::DollarSign,
        '&' => Punctuator::Ampersand,
        '(' => Punctuator::LeftParenthesis,
        ')' => Punctuator::RightParenthesis,
        '.' => Punctuator::Ellipsis,
        ':' => Punctuator::Colon,
        '=' => Punctuator::EqualSign,
        '@' => Punctuator::AtSign,
        '[' => Punctuator::LeftBracket,
        ']' => Punctuator::RightBracket,
        '{' => Punctuator::LeftBrace,
        '}' => Punctuator::RightBrace,
        '|' => Punctuator::VerticalBar,
        _ => panic!("Invalid punctuator"),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LexicalToken {
    pub token_type: LexicalTokenType,
    pub position: Range,
}

impl LexicalToken {
    pub fn new(token_type: LexicalTokenType, position: Range) -> LexicalToken {
        LexicalToken {
            token_type,
            position,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexicalTokenType {
    Punctuator(Punctuator),
    Name(String),
    IntValue(i32),
    FloatValue(f32),
    StringValue(String),
    EOF,
}
