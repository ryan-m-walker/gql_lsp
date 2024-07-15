use crate::ast_types::{
    BooleanValue, Definition, Document, EnumValue, Field, FloatValue, IntValue, ListType,
    ListValue, Name, NamedType, NonNullType, NullValue, ObjectField, ObjectValue,
    OperationDefinition, OperationType, Selection, SelectionSet, StringValue, Type, Value,
    Variable, VariableDefinition, Directive, Argument,
};
use crate::helpers::{is_valid_name, to_operation_type};
use crate::lexer::lex;
use crate::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use crate::tokens::{LexicalToken, LexicalTokenType, Punctuator};

pub fn parse(source: String) -> Result<Document, Diagnostic> {
    let tokens = lex(source)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[derive(Debug, Clone)]
struct Parser {
    tokens: Vec<LexicalToken>,
    ptr: usize,
}

impl Parser {
    pub fn new(tokens: Vec<LexicalToken>) -> Parser {
        Parser {
            ptr: 0,
            tokens
        }
    }

    pub fn parse(&mut self) -> Result<Document, Diagnostic> {
        self.parse_document()
    }

    fn parse_document(&mut self) -> Result<Document, Diagnostic> {
        let start_position = self.get_current_position();
        let definitions = self.parse_definitions()?;
        let end_position = self.get_current_position();

        Ok(Document {
            definitions,
            position: Range::new(start_position.start, end_position.end),
        })
    }

    fn parse_definitions(&mut self) -> Result<Vec<Definition>, Diagnostic> {
        let mut definitions: Vec<Definition> = Vec::new();

        while self.ptr < self.tokens.len() {
            let token = self.peek();

            if let Some(token) = token {
                let position = token.position.clone();

                match &token.token_type {
                    // https://spec.graphql.org/October2021/#sec-Anonymous-Operation-Definitions
                    LexicalTokenType::Punctuator(Punctuator::LeftBrace) => {
                        let operation_definition =
                            self.parse_operation_definition(OperationType::Query)?;
                        definitions.push(Definition::OperationDefinition(operation_definition));
                        continue;
                    }

                    // https://spec.graphql.org/October2021/#sec-Named-Operation-Definitions
                    LexicalTokenType::Name(name) => {
                        if let Some(operation_type) = to_operation_type(name) {
                            self.next();
                            let operation_definition =
                                self.parse_operation_definition(operation_type)?;
                            definitions.push(Definition::OperationDefinition(operation_definition));
                            continue;
                        }
                    }

                    _ => {
                        // return Err(Diagnostic::new(
                        //     DiagnosticSeverity::Error,
                        //     String::from("Expected operation definition"),
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
        let directives = self.parse_directives()?;
        let selection_set = self.parse_selection_set()?;

        Ok(OperationDefinition {
            name,
            operation: operation_type,
            variable_definitions,
            directives,
            selection_set,
            position: Range::new(start_position.start, self.get_current_position().end),
        })
    }

    fn parse_directives(&mut self) -> Result<Vec<Directive>, Diagnostic> {
        let mut directives: Vec<Directive> = Vec::new();

        loop {
            if let Some(token) = self.peek() {
                if token.token_type != LexicalTokenType::Punctuator(Punctuator::AtSign) {
                    break;
                }

                let start_position = self.get_current_position().clone();

                self.next();

                let name = self.parse_name()?;
                let arguments = self.parse_arguments()?;

                directives.push(Directive {
                    name: name.unwrap(),
                    arguments,
                    position: Range::new(start_position.start, self.get_current_position().end),
                });
            }
        }


        Ok(directives)
    }

    fn parse_arguments(&mut self) -> Result<Vec<Argument>, Diagnostic> {
        let mut arguments: Vec<Argument> = Vec::new();

        if let Some(token) = self.peek() {
            if token.token_type != LexicalTokenType::Punctuator(Punctuator::LeftParenthesis) {
                return Ok(arguments);
            }

            self.next();

            while let Some(token) = self.peek() {
                if token.token_type == LexicalTokenType::Punctuator(Punctuator::RightParenthesis) {
                    self.next();
                    return Ok(arguments);
                }

                let argument = self.parse_argument()?;
                arguments.push(argument);
            }
        }

        Ok(arguments)
    }

    fn parse_argument(&mut self) -> Result<Argument, Diagnostic> {
        let start_position = self.get_current_position().clone();

        let name = self.parse_name()?;

        if name.is_none() {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from("Expected Name"),
                start_position,
            ));
        }

        let name = name.unwrap();

        if let Some(token) = self.peek() {
            dbg!(token);
            if token.token_type != LexicalTokenType::Punctuator(Punctuator::Colon) {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    String::from("Expected \":\""),
                    token.position.clone(),
                ));
            }
            self.next();
        }

        let value = self.parse_value()?;

        Ok(Argument {
            name,
            value,
            position: Range::new(start_position.start, self.get_current_position().end),
        })
    }

    /// https://spec.graphql.org/October2021/#sec-Selection-Sets
    fn parse_selection_set(&mut self) -> Result<SelectionSet, Diagnostic> {

        let position = self.get_current_position().clone();
        let token = self.peek();

        let mut selections: Vec<Selection> = Vec::new();

        if let Some(token) = token {
            if token.token_type == LexicalTokenType::Punctuator(Punctuator::LeftBrace) {
                self.next();

                while let Some(token) = self.peek() {
                    if token.token_type == LexicalTokenType::Punctuator(Punctuator::RightBrace) {
                        self.next();

                        return Ok(SelectionSet {
                            selections,
                            position: Range::new(position.start, self.get_current_position().end),
                        });
                    }


                    let selection = self.parse_selection()?;
                    selections.push(selection);
                    continue;
                }
            }
        }

        Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            String::from("Expected Selection Set"),
            self.get_current_position(),
        ))
    }

    /// https://spec.graphql.org/October2021/#Selection
    fn parse_selection(&mut self) -> Result<Selection, Diagnostic> {
        let position = self.get_current_position().clone();
        let token = self.peek();

        if let Some(token) = token {
            match &token.token_type {
                LexicalTokenType::Name(_) => {
                    let mut name = self.parse_name()?;
                    let mut alias: Option<Name> = None;

                    if let Some(token) = self.peek() {
                        if token.token_type == LexicalTokenType::Punctuator(Punctuator::Colon) {
                            self.next();
                            alias = name;
                            name = self.parse_name()?;
                        }
                    }

                    let arguments = self.parse_arguments()?;
                    let directives = self.parse_directives()?;

                    let mut selection_set: Option<SelectionSet> = None;

                    if let Some(token) = self.peek() {
                        if token.token_type == LexicalTokenType::Punctuator(Punctuator::LeftBrace) {
                            selection_set = Some(self.parse_selection_set()?);
                        }
                    }


                    return Ok(Selection::Field(Field {
                        alias,
                        name: name.unwrap(),
                        selection_set,
                        arguments,
                        directives,
                        position: Range::new(position.start, self.get_current_position().end),
                    }));
                }
                _ => {}
            }
        }

        Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            String::from("Expected Selection"),
            self.get_current_position(),
        ))
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

        if let Some(token) = self.peek() {
            // check to make sure we're starting with `(`
            if token.token_type != LexicalTokenType::Punctuator(Punctuator::LeftParenthesis) {
                return Ok(variable_definitions);
            }

            // skip over the `(`
            self.next();

            while let Some(token) = self.peek() {
                // we found `)` so we're done
                if token.token_type == LexicalTokenType::Punctuator(Punctuator::RightParenthesis) {
                    self.next();
                    return Ok(variable_definitions);
                }

                let variable_definition = self.parse_variable_definition()?;
                variable_definitions.push(variable_definition);
            }
        }

        Ok(variable_definitions)
    }

    fn parse_variable_definition(&mut self) -> Result<VariableDefinition, Diagnostic> {
        let token = self.peek();

        if let Some(token) = token {
            let position = token.position.clone();

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

            let mut default_value: Option<Value> = None;

            if let Some(token) = self.peek() {
                if token.token_type == LexicalTokenType::Punctuator(Punctuator::EqualSign) {
                    self.next();
                    default_value = Some(self.parse_value()?);
                }
            }

            return Ok(VariableDefinition {
                variable: Variable {
                    name,
                    position: Range::new(position.start.clone(), self.get_current_position().end),
                },
                variable_type,
                default_value,
                position: Range::new(position.start, self.get_current_position().end),
            });
        }

        Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            String::from("Expected Variable Definition"),
            self.get_current_position(),
        ))
    }

    fn parse_type(&mut self) -> Result<Type, Diagnostic> {
        if let Some(token) = self.peek() {
            let start_position = token.position.clone();

            if token.token_type == LexicalTokenType::Punctuator(Punctuator::LeftBracket) {
                let list_type = self.parse_list_type()?;
                return Ok(self.wrap_if_non_null(list_type));
            }

            let name_type = self.parse_name()?;

            if let Some(name_type) = name_type {
                return Ok(self.wrap_if_non_null(Type::NamedType(NamedType {
                    name: name_type,
                    position: Range::new(start_position.start, self.get_current_position().end),
                })));
            }
        }

        Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            String::from("Expected Type"),
            self.get_current_position(),
        ))
    }

    fn wrap_if_non_null(&mut self, wrapped_type: Type) -> Type {
        let start_position = self.get_current_position().clone();

        if let Some(token) = self.peek() {
            if token.token_type != LexicalTokenType::Punctuator(Punctuator::ExclamationMark) {
                return wrapped_type;
            }

            self.next();
        }

        let end_position = self.get_current_position().clone();

        Type::NonNullType(NonNullType {
            wrapped_type: Box::new(wrapped_type),
            position: Range::new(start_position.start, end_position.end),
        })
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
            position: Range::new(start_position.start, end_position.end),
        }))
    }

    fn parse_value(&mut self) -> Result<Value, Diagnostic> {
        if let Some(token) = self.peek() {
            let position = token.position.clone();

            match &token.token_type {
                LexicalTokenType::IntValue(value) => {
                    let value = value.clone();
                    self.next();
                    return Ok(Value::IntValue(IntValue { value, position }));
                }
                LexicalTokenType::FloatValue(value) => {
                    let value = value.clone();
                    self.next();
                    return Ok(Value::FloatValue(FloatValue { value, position }));
                }
                LexicalTokenType::StringValue(value) => {
                    let value = value.clone();
                    self.next();
                    return Ok(Value::StringValue(StringValue {
                        value,
                        block: false,
                        position,
                    }));
                }
                LexicalTokenType::Name(name) if name == "true" => {
                    self.next();
                    return Ok(Value::BooleanValue(BooleanValue {
                        value: true,
                        position,
                    }));
                }
                LexicalTokenType::Name(name) if name == "false" => {
                    self.next();
                    return Ok(Value::BooleanValue(BooleanValue {
                        value: false,
                        position,
                    }));
                }
                LexicalTokenType::Name(name) if name == "null" => {
                    self.next();
                    return Ok(Value::NullValue(NullValue { position }));
                }
                LexicalTokenType::Punctuator(Punctuator::LeftBracket) => {
                    return Ok(self.parse_list_value()?);
                }
                LexicalTokenType::Punctuator(Punctuator::LeftBrace) => {
                    return Ok(self.parse_object_value()?);
                }
                LexicalTokenType::Name(name) => {
                    return Ok(Value::EnumValue(EnumValue {
                        value: name.to_string(),
                        position,
                    }));
                }
                _ => {
                    return Err(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        String::from("Expected Value"),
                        position,
                    ));
                }
            }
        }

        Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            String::from("Expected Value"),
            self.get_current_position(),
        ))
    }

    fn parse_list_value(&mut self) -> Result<Value, Diagnostic> {
        let start_position = self.get_current_position().clone();

        self.next();

        let mut values: Vec<Value> = Vec::new();

        while let Some(token) = self.peek() {
            if token.token_type == LexicalTokenType::Punctuator(Punctuator::RightBracket) {
                self.next();
                break;
            }

            let value = self.parse_value()?;
            values.push(value);
        }

        let end_position = self.get_current_position().clone();

        Ok(Value::ListValue(ListValue {
            values,
            position: Range::new(start_position.start, end_position.end),
        }))
    }

    fn parse_object_value(&mut self) -> Result<Value, Diagnostic> {
        let start_position = self.get_current_position().clone();

        self.next();

        let mut object_fields: Vec<ObjectField> = Vec::new();

        while let Some(token) = self.peek() {
            if token.token_type == LexicalTokenType::Punctuator(Punctuator::RightBrace) {
                self.next();
                break;
            }

            let object_field = self.parse_object_field()?;
            object_fields.push(object_field);
        }

        let end_position = self.get_current_position().clone();

        Ok(Value::ObjectValue(ObjectValue {
            fields: object_fields,
            position: Range::new(start_position.start, end_position.end),
        }))
    }

    fn parse_object_field(&mut self) -> Result<ObjectField, Diagnostic> {
        let start_position = self.get_current_position().clone();

        let name = match self.parse_name()? {
            Some(name) => name,
            None => {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    String::from("Expected Name"),
                    start_position,
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

        let value = self.parse_value()?;

        Ok(ObjectField {
            name,
            value,
            position: Range::new(start_position.start, self.get_current_position().end),
        })
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

        Range::new(Position::new(0, 0), Position::new(0, 0))
    }
}
