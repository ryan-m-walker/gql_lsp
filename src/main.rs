use std::fs;

use lexer::Lexer;
use parser::Parser;

mod ast_types;
mod constants;
mod errors;
mod helpers;
mod lexer;
mod lsp_types;
mod parser;
mod tokens;

fn main() {
    let file = fs::read_to_string("schema.graphql").expect("Unable to read file");

    let mut lexer = Lexer::new(file);
    let tokens = lexer.lex();

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse();

    dbg!(ast);
}
