use std::fs;

use lexer::Lexer;
use parser::Parser;
use pretty_print::PrettyPrint;

mod ast_types;
mod constants;
mod errors;
mod json;
mod helpers;
mod lexer;
mod lsp_types;
mod parser;
mod pretty_print;
mod tokens;

fn main() {
    let file = fs::read_to_string("test_document.graphql").expect("Unable to read file");

    let mut lexer = Lexer::new(file);
    let tokens = lexer.lex();

    let mut parser = Parser::new(&tokens);
    let ast = parser.parse();

    dbg!(&ast);

    // if let Ok(ast) = ast {
    //     println!("{}", ast.pretty_print());
    // }
}
