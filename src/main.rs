use std::fs;

use parser::parse;

mod constants;
mod errors;
mod helpers;
mod json;
mod lexer;
mod lsp;
mod parser;

fn main() {
    let file = fs::read_to_string("test_document.graphql").expect("Unable to read file");
    let document = parse(file);
    dbg!(&document);
}
