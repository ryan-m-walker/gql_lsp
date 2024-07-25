use std::fs;

use parser::parse;

use print::pretty_print::print;

mod constants;
mod errors;
mod helpers;
mod lexer;
mod lsp;
mod parser;
mod print;
mod visitor;

fn main() {
    let file = fs::read_to_string("test_document.graphql").expect("Unable to read file");
    let document = parse(file).unwrap();
    let pretty = print(&document);
    println!("{}", pretty);
}
