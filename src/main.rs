use std::fs;

use parser::parse;

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
    let document = parse(file);
    dbg!(&document);
}
