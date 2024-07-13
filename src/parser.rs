use crate::{
    ast_types::{Definition, Document, Name, OperationDefinition, OperationType, VariableDefinition, Type, ListType, NamedType, NonNullType, Variable},
    helpers::{is_valid_name, to_operation_type},
    lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range},
    tokens::{LexicalToken, LexicalTokenType, Punctuator},
};

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    tokens: &'a Vec<LexicalToken>,
    ptr: usize,
    pub diagnostics: Vec<Diagnostic>,
}

impl<'a> Parser<'_> {
    pub fn new(tokens: &'a Vec<LexicalToken>) -> Parser {
        Parser {
            ptr: 0,
            tokens,
            diagnostics: Vec::new(),
        }
    }

    pub fn parse(&mut self) -> Result<Document, Vec<Diagnostic>>{
        let result = self.parse_document();

        match result {
            Ok(document) => Ok(document),
            Err(diagnostic) => {
                self.diagnostics.push(diagnostic);
                return Err(self.diagnostics.clone());
            }
        }
    }

    fn parse_document(&mut self) -> Result<Document, Diagnostic> {
        let start_position = self.get_current_position();

        let definitions = self.parse_definitions()?;
        dbg!(&definitions);

        let end_position = self.get_current_position();

        Ok(Document {
            definitions,
            position: Range::new(start_position.start, end_position.end)
        })
    }

    fn parse_definitions(&mut self) -> Result<Vec<Definition>, Diagnostic> {
        let mut definitions: Vec<Definition> = Vec::new();

        while self.ptr < self.tokens.len() {
            let token = self.peek();

            if let Some(token) = token {
                let position = token.position.clone();

                match &token.token_type {
                    LexicalTokenType::Name(name) => {
                        if let Some(operation_type) = to_operation_type(name) {
                            self.next();

                            let operation_definition =
                                self.parse_operation_definition(operation_type)?;

                            definitions.push(Definition::OperationDefinition(
                                operation_definition,
                            ));

                            continue;
                        }

                        // return Err(Diagnostic::new(
                        //     DiagnosticSeverity::Error,
                        //     String::from("Expected definition"),
                        //     position,
                        // ));
                    }
                    _ => {
                        // return Err(Diagnostic::new(
                        //     DiagnosticSeverity::Error,
                        //     String::from("Expected operation definition"), // TODO: better error
                        //     position,
                        // ));
                    }
                };
            }

            self.next();
        }

        Ok(definitions)
    }

    fn parse_operation_definition(
        &mut self,
        operation_type: OperationType,
    ) -> Result<OperationDefinition, Diagnostic> {
        let start_position = self.get_current_position();

        let name = self.parse_name()?;
        let variable_definitions = self.parse_variable_definitions()?;

        Ok(OperationDefinition {
            name,
            operation: operation_type,
            variable_definitions,
            position: Range::new(start_position.start, self.get_current_position().end)
        })
    }

    fn parse_name(&mut self) -> Result<Option<Name>, Diagnostic> {
        let token = self.peek().cloned();

        if let Some(token) = token {
            match &token.token_type {
                LexicalTokenType::Name(name) => {
                    let position = token.position.clone();

                    if is_valid_name(name) {
                        self.next();

                        return Ok(Some(Name {
                            value: name.clone(),
                            position,
                        }));
                    }

                    return Err(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        String::from("Invalid name"),
                        position,
                    ));
                }
                _ => return Ok(None),
            }
        }

        Ok(None)
    }

    fn parse_variable_definitions(&mut self) -> Result<Vec<VariableDefinition>, Diagnostic> {
        let mut variable_definitions: Vec<VariableDefinition> = Vec::new();

        dbg!("parsing variable definitions");

        if let Some(token) = self.peek() {
            // check to make sure we're starting with `(`
            if token.token_type != LexicalTokenType::Punctuator(Punctuator::LeftParenthesis) {
                return Ok(variable_definitions);
            }

            // skip over the `(`
            self.next();

            while let Some(token) = self.peek() {
                let position = token.position.clone();

                // we found `)` so we're done
                if token.token_type == LexicalTokenType::Punctuator(Punctuator::RightParenthesis) {
                    dbg!("found right parenthesis");
                    self.next();
                    return Ok(variable_definitions);
                }

                if token.token_type != LexicalTokenType::Punctuator(Punctuator::DollarSign) {
                    return Err(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        String::from("Expected \"$\""),
                        token.position.clone(),
                    ));
                }
                self.next();

                let name = match self.parse_name()? {
                    Some(name) => name,
                    None => {
                        return Err(Diagnostic::new(
                            DiagnosticSeverity::Error,
                            String::from("Expected Name"),
                            position,
                        ));
                    }
                };

                if let Some(token) = self.peek() {
                    if token.token_type != LexicalTokenType::Punctuator(Punctuator::Colon) {
                        return Err(Diagnostic::new(
                            DiagnosticSeverity::Error,
                            String::from("Expected \":\""),
                            token.position.clone(),
                        ));
                    }
                    self.next();
                }

                let variable_type = self.parse_type()?;

                variable_definitions.push(VariableDefinition {
                    variable: Variable {
                        name,
                        position: Range::new(
                            position.start.clone(),
                            self.get_current_position().end,
                        ),
                    },
                    variable_type,
                    position: Range::new(
                        position.start,
                        self.get_current_position().end,
                    ),
                });
            }
        }

        Ok(variable_definitions)
    }

    fn parse_type(&mut self) -> Result<Type, Diagnostic>{

        if let Some(token) = self.peek() {
            let start_position = token.position.clone();

            if token.token_type == LexicalTokenType::Punctuator(Punctuator::LeftBracket) {
                return self.parse_list_type();
            }

            let name = self.parse_name()?;

            if let Some(type_name) = name {
                if let Some(trailing_token) = self.peek() {
                    if trailing_token.token_type == LexicalTokenType::Punctuator(Punctuator::ExclamationMark) {
                        self.next();

                        return Ok(Type::NonNullType(
                            NonNullType {
                                wrapped_type: Box::new(Type::NamedType(
                                    NamedType {
                                        name: type_name,
                                        position: Range::new(
                                            start_position.clone().start,
                                            self.get_current_position().end,
                                        ),
                                    }
                                )),
                                position: Range::new(
                                    start_position.start,
                                    self.get_current_position().end,
                                ),
                            }
                        ));
                    }
                }
                

                let end_position = self.get_current_position().clone();

                return Ok(Type::NamedType(
                    NamedType {
                        name: type_name,
                        position: Range::new(
                            start_position.start,
                            end_position.end,
                        ),
                    }
                ));
            } else {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    String::from("Expected Name"),
                    start_position,
                ));
            }
        }

        Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            String::from("Expected Type"),
            self.get_current_position(),
        ))
    }

    fn parse_list_type(&mut self) -> Result<Type, Diagnostic> {
        let start_position = self.get_current_position().clone();

        self.next();
        let wrapped_type = self.parse_type()?;

        if let Some(token) = self.peek() {
            if token.token_type != LexicalTokenType::Punctuator(Punctuator::RightBracket) {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    String::from("Expected \"]\""),
                    token.position.clone(),
                ));
            }

            self.next();
        }

        let end_position = self.get_current_position().clone();

        Ok(Type::ListType(ListType {
            wrapped_type: Box::new(wrapped_type),
            position: Range::new(
                start_position.start,
                end_position.end,
            ),
        }))
    }

    fn peek(&self) -> Option<&LexicalToken> {
        self.tokens.get(self.ptr)
    }

    fn next(&mut self) {
        self.ptr += 1;
    }

    fn get_current_position(&self) -> Range {
        if let Some(token) = self.peek() {
            return token.position.clone();
        }

        Range::new(
            Position::new(0, 0),
            Position::new(0, 0),
        )
    }
}
