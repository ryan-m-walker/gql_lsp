use crate::parser::types::{Document, Node};

pub fn visit(document: Document) {
    let visitor = Visitor::new(document);
}

pub struct Visitor {
    document: Document,
}

impl Visitor {
    pub fn new(document: Document) -> Self {
        Self { document }
    }
}
