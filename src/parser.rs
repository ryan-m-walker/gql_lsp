use crate::ast_types::{
    Argument, BooleanValue, Definition, Directive, Document, EnumValue, Field, FloatValue,
    FragmentDefinition, FragmentSpread, InlineFragment, IntValue, ListType, ListValue, Name,
    NamedType, NonNullType, NullValue, ObjectField, ObjectValue, OperationDefinition,
    OperationType, RootOperationTypeDefinition, ScalarTypeDefinition, SchemaDefinition, Selection,
    SelectionSet, StringValue, Type, Value, Variable, VariableDefinition,
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
        Parser { ptr: 0, tokens }
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

        loop {
            let token = self.peek_safe();

            if token.token_type == LexicalTokenType::EOF {
                return Ok(definitions);
            }

            let position = self.get_current_position();

            match &token.token_type {
                // https://spec.graphql.org/October2021/#sec-Anonymous-Operation-Definitions
                LexicalTokenType::Punctuator(Punctuator::LeftBrace) => {
                    let operation_definition =
                        self.parse_operation_definition(OperationType::Query, true)?;
                    definitions.push(Definition::OperationDefinition(operation_definition));
                    continue;
                }

                // https://spec.graphql.org/October2021/#sec-Named-Operation-Definitions
                LexicalTokenType::Name(name) => {
                    if let Some(operation_type) = OperationType::parse(name) {
                        self.next();
                        let operation_definition =
                            self.parse_operation_definition(operation_type, false)?;
                        definitions.push(Definition::OperationDefinition(operation_definition));
                        continue;
                    }

                    match name.as_str() {
                        "fragment" => {
                            self.next();
                            let fragment_definition = self.parse_fragment_definition()?;
                            definitions.push(Definition::FragmentDefinition(fragment_definition));
                            continue;
                        }
                        "schema" => {
                            let schema_definition = self.parse_schema_definition()?;
                            definitions.push(Definition::SchemaDefinition(schema_definition));
                            continue;
                        }
                        "scalar" => {
                            let scalar_type_definition = self.parse_scalar_type_definition()?;
                            definitions
                                .push(Definition::ScalarTypeDefinition(scalar_type_definition));
                            continue;
                        }
                        _ => {
                            return Err(Diagnostic::new(
                                DiagnosticSeverity::Error,
                                String::from("Expected operation definition"),
                                position,
                            ));
                        }
                    }
                }

                _ => {
                    return Err(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        String::from("Expected operation definition"),
                        position,
                    ));
                }
            };
        }
    }

    fn parse_scalar_type_definition(&mut self) -> Result<ScalarTypeDefinition, Diagnostic> {
        let start_position = self.get_current_position();

        self.expect_next(LexicalTokenType::Name(String::from("scalar")))?;
        let name = self.parse_name()?;
        let directives = self.parse_directives()?;

        Ok(ScalarTypeDefinition {
            name,
            description: None, // TODO
            directives,
            position: Range::new(start_position.start, self.get_current_position().end),
        })
    }

    fn parse_schema_definition(&mut self) -> Result<SchemaDefinition, Diagnostic> {
        let start_position = self.get_current_position();

        self.expect_next(LexicalTokenType::Name(String::from("schema")))?;
        let directives = self.parse_directives()?;
        self.expect_next(LexicalTokenType::Punctuator(Punctuator::LeftBrace))?;

        let mut operation_types: Vec<RootOperationTypeDefinition> = Vec::new();

        loop {
            let token = self.peek()?;

            if token.token_type == LexicalTokenType::Punctuator(Punctuator::RightBrace) {
                self.next();
                break;
            }

            let start_position = self.get_current_position().clone();

            let operation_type_name = self.parse_name()?;
            let operation_type = match OperationType::parse(&operation_type_name.value) {
                Some(operation_type) => operation_type,
                None => {
                    return Err(Diagnostic::new(
                        DiagnosticSeverity::Error,
                        String::from("Expected operation type"),
                        operation_type_name.position,
                    ));
                }
            };

            self.expect_next(LexicalTokenType::Punctuator(Punctuator::Colon))?;
            let named_type = self.parse_named_type()?;

            operation_types.push(RootOperationTypeDefinition {
                operation_type,
                named_type,
                position: Range::new(start_position.start, self.get_current_position().end),
            });
        }

        Ok(SchemaDefinition {
            description: None, // TODO
            operation_types,
            directives,
            position: Range::new(start_position.start, self.get_current_position().end),
        })
    }

    fn parse_type_condition(&mut self) -> Result<NamedType, Diagnostic> {
        let start_position = self.get_current_position();

        self.expect_next(LexicalTokenType::Name(String::from("on")))?;
        let name = self.parse_name()?;

        Ok(NamedType {
            name,
            position: Range::new(start_position.start, self.get_current_position().end),
        })
    }

    fn parse_fragment_definition(&mut self) -> Result<FragmentDefinition, Diagnostic> {
        let start_position = self.get_current_position();

        let name = self.parse_name()?;
        let type_condition = self.parse_type_condition()?;
        let directives = self.parse_directives()?;
        let selection_set = self.parse_selection_set()?;

        Ok(FragmentDefinition {
            name,
            type_condition,
            directives,
            selection_set,
            position: Range::new(start_position.start, self.get_current_position().end),
        })
    }

    fn parse_operation_definition(
        &mut self,
        operation_type: OperationType,
        anonymous: bool,
    ) -> Result<OperationDefinition, Diagnostic> {
        let start_position = self.get_current_position();

        let name = self.parse_name_maybe()?;
        let variable_definitions = self.parse_variable_definitions()?;
        let directives = self.parse_directives()?;
        let selection_set = self.parse_selection_set()?;

        Ok(OperationDefinition {
            name,
            operation: operation_type,
            variable_definitions,
            directives,
            selection_set,
            anonymous,
            position: Range::new(start_position.start, self.get_current_position().end),
        })
    }

    fn parse_directives(&mut self) -> Result<Vec<Directive>, Diagnostic> {
        let mut directives: Vec<Directive> = Vec::new();

        loop {
            let token = self.peek()?;

            if token.token_type != LexicalTokenType::Punctuator(Punctuator::AtSign) {
                return Ok(directives);
            }

            let start_position = self.get_current_position().clone();

            self.next();

            let name = self.parse_name()?;
            let arguments = self.parse_arguments()?;

            directives.push(Directive {
                name,
                arguments,
                position: Range::new(start_position.start, self.get_current_position().end),
            });
        }
    }

    fn parse_arguments(&mut self) -> Result<Vec<Argument>, Diagnostic> {
        let mut arguments: Vec<Argument> = Vec::new();

        let token = self.peek()?.clone();

        if token.token_type != LexicalTokenType::Punctuator(Punctuator::LeftParenthesis) {
            return Ok(arguments);
        }

        self.next();

        loop {
            let token = self.peek()?.clone();

            if token.token_type == LexicalTokenType::Punctuator(Punctuator::RightParenthesis) {
                self.next();
                return Ok(arguments);
            }

            let argument = self.parse_argument()?;
            arguments.push(argument);
        }
    }

    fn parse_argument(&mut self) -> Result<Argument, Diagnostic> {
        let start_position = self.get_current_position().clone();

        let name = self.parse_name()?;
        self.expect_next(LexicalTokenType::Punctuator(Punctuator::Colon))?;
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

        self.expect_next(LexicalTokenType::Punctuator(Punctuator::LeftBrace))?;

        let mut selections: Vec<Selection> = Vec::new();

        loop {
            let token = self.peek()?;

            if token.token_type == LexicalTokenType::Punctuator(Punctuator::RightBrace) {
                self.next();

                return Ok(SelectionSet {
                    selections,
                    position: Range::new(position.start, self.get_current_position().end),
                });
            }

            selections.push(self.parse_selection()?);
        }
    }

    fn parse_fragment_spread(&mut self) -> Result<FragmentSpread, Diagnostic> {
        let position = self.get_current_position().clone();

        let name = self.parse_name()?;
        let directives = self.parse_directives()?;

        Ok(FragmentSpread {
            name,
            directives,
            position: Range::new(position.start, self.get_current_position().end),
        })
    }

    fn parse_inline_fragment(&mut self) -> Result<InlineFragment, Diagnostic> {
        let position = self.get_current_position().clone();

        let mut type_condition: Option<NamedType> = None;

        let token = self.peek()?;
        if token.token_type == LexicalTokenType::Name(String::from("on")) {
            type_condition = Some(self.parse_type_condition()?);
        }

        let directives = self.parse_directives()?;
        let selection_set = self.parse_selection_set()?;

        Ok(InlineFragment {
            type_condition,
            directives,
            selection_set,
            position: Range::new(position.start, self.get_current_position().end),
        })
    }

    /// https://spec.graphql.org/October2021/#Selection
    fn parse_selection(&mut self) -> Result<Selection, Diagnostic> {
        let position = self.get_current_position().clone();
        let token = self.peek()?;

        match &token.token_type {
            LexicalTokenType::Punctuator(Punctuator::Ellipsis) => {
                // TODO: start position needs to account for the `...`
                self.next();

                let token = self.peek()?;
                match &token.token_type {
                    LexicalTokenType::Name(name) if name == "on" => {
                        return Ok(Selection::InlineFragment(self.parse_inline_fragment()?));
                    }
                    LexicalTokenType::Name(_) => {
                        return Ok(Selection::FragmentSpread(self.parse_fragment_spread()?));
                    }
                    _ => {
                        return Err(Diagnostic::new(
                            DiagnosticSeverity::Error,
                            String::from("Expected Fragment Spread or Inline Fragment"),
                            self.get_current_position(),
                        ));
                    }
                }
            }
            LexicalTokenType::Name(_) => {
                let mut name = self.parse_name_maybe()?;
                let mut alias: Option<Name> = None;

                let token = self.peek()?;
                if token.token_type == LexicalTokenType::Punctuator(Punctuator::Colon) {
                    self.next();
                    alias = name;
                    name = self.parse_name_maybe()?;
                }

                let arguments = self.parse_arguments()?;
                let directives = self.parse_directives()?;

                let mut selection_set: Option<SelectionSet> = None;

                let token = self.peek()?;
                if token.token_type == LexicalTokenType::Punctuator(Punctuator::LeftBrace) {
                    selection_set = Some(self.parse_selection_set()?);
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
            _ => {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    String::from("Expected Selection"),
                    self.get_current_position(),
                ))
            }
        }
    }

    fn parse_name(&mut self) -> Result<Name, Diagnostic> {
        let maybe_name = self.parse_name_maybe()?;

        match maybe_name {
            Some(name) => Ok(name),
            None => Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from("Expected Name"),
                self.get_current_position(),
            )),
        }
    }

    fn parse_name_maybe(&mut self) -> Result<Option<Name>, Diagnostic> {
        let position = self.get_current_position();
        let token = self.peek()?.clone();

        if let LexicalTokenType::Name(name) = &token.token_type {
            if is_valid_name(&name) {
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

        Ok(None)
    }

    fn parse_variable_definitions(&mut self) -> Result<Vec<VariableDefinition>, Diagnostic> {
        let mut variable_definitions: Vec<VariableDefinition> = Vec::new();

        let token = self.peek()?;
        if token.token_type != LexicalTokenType::Punctuator(Punctuator::LeftParenthesis) {
            return Ok(variable_definitions);
        }

        // skip over the `(`
        self.next();

        loop {
            let token = self.peek()?;

            if token.token_type == LexicalTokenType::Punctuator(Punctuator::RightParenthesis) {
                self.next();
                return Ok(variable_definitions);
            }

            let variable_definition = self.parse_variable_definition()?;
            variable_definitions.push(variable_definition);
        }
    }

    fn parse_variable_definition(&mut self) -> Result<VariableDefinition, Diagnostic> {
        let token = self.peek()?;

        let position = token.position.clone();

        if token.token_type != LexicalTokenType::Punctuator(Punctuator::DollarSign) {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from("Expected \"$\""),
                token.position.clone(),
            ));
        }
        self.next();

        let name = self.parse_name()?;

        let token = self.peek()?;
        if token.token_type != LexicalTokenType::Punctuator(Punctuator::Colon) {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from("Expected \":\""),
                token.position.clone(),
            ));
        }
        self.next();

        let variable_type = self.parse_type()?;

        let mut default_value: Option<Value> = None;

        let token = self.peek()?;
        if token.token_type == LexicalTokenType::Punctuator(Punctuator::EqualSign) {
            self.next();
            default_value = Some(self.parse_value()?);
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

    fn parse_type(&mut self) -> Result<Type, Diagnostic> {
        let token = self.peek()?;
        let start_position = token.position.clone();

        if token.token_type == LexicalTokenType::Punctuator(Punctuator::LeftBracket) {
            let list_type = self.parse_list_type()?;
            return Ok(self.wrap_if_non_null(list_type)?);
        }

        let name_type = self.parse_name()?;

        return Ok(self.wrap_if_non_null(Type::NamedType(NamedType {
            name: name_type,
            position: Range::new(start_position.start, self.get_current_position().end),
        }))?);
    }

    fn parse_named_type(&mut self) -> Result<NamedType, Diagnostic> {
        let start_position = self.get_current_position().clone();
        let name = self.parse_name()?;

        Ok(NamedType {
            name,
            position: Range::new(start_position.start, self.get_current_position().end),
        })
    }

    fn wrap_if_non_null(&mut self, wrapped_type: Type) -> Result<Type, Diagnostic> {
        let start_position = self.get_current_position().clone();

        let token = self.peek()?;
        if token.token_type != LexicalTokenType::Punctuator(Punctuator::ExclamationMark) {
            return Ok(wrapped_type);
        }

        self.next();

        let end_position = self.get_current_position().clone();

        Ok(Type::NonNullType(NonNullType {
            wrapped_type: Box::new(wrapped_type),
            position: Range::new(start_position.start, end_position.end),
        }))
    }

    fn parse_list_type(&mut self) -> Result<Type, Diagnostic> {
        let start_position = self.get_current_position().clone();

        self.next();
        let wrapped_type = self.parse_type()?;

        let token = self.peek()?;
        if token.token_type != LexicalTokenType::Punctuator(Punctuator::RightBracket) {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from("Expected \"]\""),
                token.position.clone(),
            ));
        }

        self.next();

        let end_position = self.get_current_position().clone();

        Ok(Type::ListType(ListType {
            wrapped_type: Box::new(wrapped_type),
            position: Range::new(start_position.start, end_position.end),
        }))
    }

    // TODO: when to allow variables?
    fn parse_value(&mut self) -> Result<Value, Diagnostic> {
        let token = self.peek()?;
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
            LexicalTokenType::Punctuator(Punctuator::DollarSign) => {
                self.next();
                let name = self.parse_name()?;
                return Ok(Value::Variable(Variable { name, position }));
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

    fn parse_list_value(&mut self) -> Result<Value, Diagnostic> {
        let start_position = self.get_current_position().clone();

        self.next();

        let mut values: Vec<Value> = Vec::new();

        loop {
            let token = self.peek()?;

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

        while self.peek()?.token_type != LexicalTokenType::Punctuator(Punctuator::RightBrace) {
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

        let name = self.parse_name()?;
        self.expect_next(LexicalTokenType::Punctuator(Punctuator::Colon))?;
        let value = self.parse_value()?;

        Ok(ObjectField {
            name,
            value,
            position: Range::new(start_position.start, self.get_current_position().end),
        })
    }

    fn peek(&self) -> Result<&LexicalToken, Diagnostic> {
        let token = self.tokens.get(self.ptr);

        match token {
            Some(token) => Ok(token),
            None => Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from("Unexpected EOF"),
                self.get_current_position(),
            )),
        }
    }

    fn peek_safe(&self) -> LexicalToken {
        let token = self.tokens.get(self.ptr);

        match token {
            Some(token) => token.clone(),
            None => LexicalToken {
                token_type: LexicalTokenType::EOF,
                position: self.get_current_position().clone(),
            },
        }
    }

    fn next(&mut self) {
        self.ptr += 1;
    }

    fn expect_next(&mut self, token_type: LexicalTokenType) -> Result<bool, Diagnostic> {
        let token = self.peek()?;

        if token.token_type == token_type {
            self.next();
            return Ok(true);
        }

        Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            String::from(format!(
                "Unexpected token. Expected {:?}, found {:?}",
                token_type, token
            )),
            self.get_current_position(),
        ))
    }

    fn get_current_position(&self) -> Range {
        let token = self.peek();

        match token {
            Ok(token) => token.position.clone(),
            Err(_) => Range::new(Position::new(0, 0), Position::new(0, 0)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_unnamed_queries() {
        let source = r#"
            query {
                test
            }"#;

        let document = parse(source.to_string()).unwrap();

        match document.definitions.get(0) {
            Some(Definition::OperationDefinition(operation_definition)) => {
                assert_eq!(operation_definition.operation, OperationType::Query);
                assert_eq!(operation_definition.name, None);
                assert_eq!(operation_definition.variable_definitions.len(), 0);
                assert_eq!(operation_definition.directives.len(), 0);
                assert_eq!(operation_definition.anonymous, false);
                assert_eq!(operation_definition.selection_set.selections.len(), 1);
            }
            _ => panic!("Expected OperationDefinition"),
        }
    }

    #[test]
    fn it_parses_named_queries() {
        let source = r#"
            query Test {
                test
            }"#;

        let document = parse(source.to_string()).unwrap();

        match document.definitions.get(0) {
            Some(Definition::OperationDefinition(operation_definition)) => {
                if let Some(name) = &operation_definition.name {
                    assert_eq!(name.value, "Test");
                } else {
                    panic!("Expected name");
                }
            }
            _ => panic!("Expected OperationDefinition"),
        }
    }

    #[test]
    fn it_parses_anonymous_queries() {
        let source = r#"
            {
                test
            }"#;

        let document = parse(source.to_string()).unwrap();

        match document.definitions.get(0) {
            Some(Definition::OperationDefinition(operation_definition)) => {
                assert_eq!(operation_definition.operation, OperationType::Query);
                assert_eq!(operation_definition.name, None);
                assert_eq!(operation_definition.anonymous, true);
            }
            _ => panic!("Expected OperationDefinition"),
        }
    }

    #[test]
    fn it_parses_queries_with_variables() {
        let source = r#"
            query Test($id: ID!, $name: String) {
                test(id: $id, name: $name)
            }"#;

        let document = parse(source.to_string()).unwrap();

        match document.definitions.get(0) {
            Some(Definition::OperationDefinition(operation_definition)) => {
                assert_eq!(operation_definition.variable_definitions.len(), 2);

                let var_1 = operation_definition.variable_definitions.get(0).unwrap();
                assert_eq!(var_1.variable.name.value, "id");
                // TODO assert values

                let var_2 = operation_definition.variable_definitions.get(1).unwrap();
                assert_eq!(var_2.variable.name.value, "name");
                // TODO assert values
            }
            _ => panic!("Expected OperationDefinition"),
        }
    }

    #[test]
    fn it_successfully_parses_a_complex_query() {
        let source = r#"
            query Test($id: ID!, $name: String) @foo @bar(param: "value") {
                test(id: $id, name: $name) {
                    id
                    name
                    age
                    friends {
                        id
                        name
                    }

                    ... on User {
                        email
                    }

                    ... UserFields @test_directive
                }
            }"#;

        let document = parse(source.to_string());
        assert!(document.is_ok());
    }

    #[test]
    fn it_parses_fragment_definitions() {
        let source = r#"
            fragment UserFields on User {
                id
                name
                age
                friends {
                    id
                    name
                }
            }"#;

        let document = parse(source.to_string()).unwrap();

        match document.definitions.get(0) {
            Some(Definition::FragmentDefinition(fragment_definition)) => {
                assert_eq!(fragment_definition.name.value, "UserFields");
                assert_eq!(fragment_definition.type_condition.name.value, "User");
                assert_eq!(fragment_definition.directives.len(), 0);
                assert_eq!(fragment_definition.selection_set.selections.len(), 4);
            }
            _ => panic!("Expected FragmentDefinition"),
        }
    }

    #[test]
    fn it_can_parse_fragment_spreads() {
        let source = r#"
            {
                ...TestFields
                ...TestDirective @test
            }"#;

        let document = parse(source.to_string()).unwrap();

        match document.definitions.get(0) {
            Some(Definition::OperationDefinition(operation_definition)) => {
                let selection_set = &operation_definition.selection_set.selections;
                let fragment_spread_1 = selection_set.get(0).unwrap();
                let fragment_spread_2 = selection_set.get(1).unwrap();

                match fragment_spread_1 {
                    Selection::FragmentSpread(fragment_spread) => {
                        assert_eq!(fragment_spread.name.value, "TestFields");
                        assert_eq!(fragment_spread.directives.len(), 0);
                    }
                    _ => panic!("Expected FragmentSpread"),
                }

                match fragment_spread_2 {
                    Selection::FragmentSpread(fragment_spread) => {
                        assert_eq!(fragment_spread.name.value, "TestDirective");
                        assert_eq!(fragment_spread.directives.len(), 1);
                    }
                    _ => panic!("Expected FragmentSpread"),
                }
            }
            _ => panic!("Expected OperationDefinition"),
        }
    }

    #[test]
    fn it_can_parse_inline_fragments() {
        let source = r#"
            {
                ... on User {
                    id
                    name
                }
            }"#;

        let document = parse(source.to_string()).unwrap();

        match document.definitions.get(0) {
            Some(Definition::OperationDefinition(operation_definition)) => {
                let selection_set = &operation_definition.selection_set.selections;
                let inline_fragment = selection_set.get(0).unwrap();

                match inline_fragment {
                    Selection::InlineFragment(inline_fragment) => {
                        assert_eq!(
                            inline_fragment.type_condition.as_ref().unwrap().name.value,
                            "User"
                        );
                        assert_eq!(inline_fragment.directives.len(), 0);
                        assert_eq!(inline_fragment.selection_set.selections.len(), 2);
                    }
                    _ => panic!("Expected InlineFragment"),
                }
            }
            _ => panic!("Expected OperationDefinition"),
        }
    }

    #[test]
    fn it_can_parse_schema_definitions() {
        let source = r#"
            schema {
                query: Query
                mutation: Mutation
                subscription: Subscription
            }"#;

        let document = parse(source.to_string()).unwrap();

        match document.definitions.get(0) {
            Some(Definition::SchemaDefinition(schema_definition)) => {
                let query = schema_definition.operation_types.get(0).unwrap();
                assert_eq!(query.operation_type, OperationType::Query);
                assert_eq!(query.named_type.name.value, "Query");

                let mutation = schema_definition.operation_types.get(1).unwrap();
                assert_eq!(mutation.operation_type, OperationType::Mutation);
                assert_eq!(mutation.named_type.name.value, "Mutation");

                let subscription = schema_definition.operation_types.get(2).unwrap();
                assert_eq!(subscription.operation_type, OperationType::Subscription);
                assert_eq!(subscription.named_type.name.value, "Subscription");
            }
            _ => panic!("Expected SchemaDefinition"),
        }
    }

    #[test]
    fn it_errors_for_invalid_schema_definitions() {
        let source = r#"
            schema {
                query: Query
                mutation: Mutation
                subscription: Subscription
                foo: Foo
            }"#;

        let document = parse(source.to_string());
        assert!(document.is_err());
    }

    #[test]
    fn it_can_parse_scalar_type_definitions() {
        let source = r#"
            scalar Date
            scalar Time
            scalar DateTime
        "#;

        let document = parse(source.to_string());
        let document = document.unwrap();

        match document.definitions.get(0) {
            Some(Definition::ScalarTypeDefinition(scalar_type_definition)) => {
                assert_eq!(scalar_type_definition.name.value, "Date");
            }
            _ => panic!("Expected ScalarTypeDefinition"),
        }
    }
}
