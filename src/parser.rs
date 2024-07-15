use crate::ast_types::{
    Argument, BooleanValue, Definition, Directive, Document, EnumValue, Field, FloatValue,
    IntValue, ListType, ListValue, Name, NamedType, NonNullType, NullValue, ObjectField,
    ObjectValue, OperationDefinition, OperationType, Selection, SelectionSet, StringValue, Type,
    Value, Variable, VariableDefinition, FragmentSpread, InlineFragment, FragmentDefinition,
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

        while self.ptr < self.tokens.len() {
            let token = self.peek();

            if let Some(token) = token {
                let position = token.position.clone();

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
                        if let Some(operation_type) = to_operation_type(name) {
                            self.next();
                            let operation_definition =
                                self.parse_operation_definition(operation_type, false)?;
                            definitions.push(Definition::OperationDefinition(operation_definition));
                            continue;
                        }

                        if name == "fragment" {
                            self.next();
                            let fragment_definition = self.parse_fragment_definition()?;
                            definitions.push(Definition::FragmentDefinition(fragment_definition));
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

    fn parse_type_condition(&mut self) -> Result<NamedType, Diagnostic> {
        let start_position = self.get_current_position();

        if let Some(token) = self.peek() {
            if token.token_type != LexicalTokenType::Name(String::from("on")) {
                return Err(Diagnostic::new(
                    DiagnosticSeverity::Error,
                    String::from("Expected \"on\""),
                    token.position.clone(),
                ));
            }

            self.next();
        }

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
            if let Some(token) = self.peek() {
                if token.token_type != LexicalTokenType::Punctuator(Punctuator::AtSign) {
                    break;
                }

                let start_position = self.get_current_position().clone();

                self.next();

                let name = self.parse_name_maybe()?;
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

        let name = self.parse_name_maybe()?;

        if name.is_none() {
            return Err(Diagnostic::new(
                DiagnosticSeverity::Error,
                String::from("Expected Name"),
                start_position,
            ));
        }

        let name = name.unwrap();

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
            match &token.token_type {
                LexicalTokenType::Punctuator(Punctuator::LeftBrace) => {
                    self.next();

                    while let Some(token) = self.peek() {
                        if token.token_type == LexicalTokenType::Punctuator(Punctuator::RightBrace)
                        {
                            self.next();

                            return Ok(SelectionSet {
                                selections,
                                position: Range::new(
                                    position.start,
                                    self.get_current_position().end,
                                ),
                            });
                        }

                        let selection = self.parse_selection()?;
                        selections.push(selection);
                        continue;
                    }
                }
                _ => {}
            }
        }

        Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            String::from("Expected Selection Set"),
            self.get_current_position(),
        ))
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

        if let Some(token) = self.peek() {
            if token.token_type == LexicalTokenType::Name(String::from("on")) {
                type_condition = Some(self.parse_type_condition()?);
            }
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
        let token = self.peek();

        if let Some(token) = token {
            match &token.token_type {
                LexicalTokenType::Punctuator(Punctuator::Ellipsis) => {
                    self.next();

                    if let Some(token) = self.peek() {
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
                }
                LexicalTokenType::Name(_) => {
                    let mut name = self.parse_name_maybe()?;
                    let mut alias: Option<Name> = None;

                    if let Some(token) = self.peek() {
                        if token.token_type == LexicalTokenType::Punctuator(Punctuator::Colon) {
                            self.next();
                            alias = name;
                            name = self.parse_name_maybe()?;
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

            let name = match self.parse_name_maybe()? {
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

            let name_type = self.parse_name_maybe()?;

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

    // TODO: when to allow variables?
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
                LexicalTokenType::Punctuator(Punctuator::DollarSign) => {
                    self.next();
                    let name = self.parse_name_maybe()?;
                    match name {
                        Some(name) => {
                            return Ok(Value::Variable(Variable { name, position }));
                        }
                        None => {
                            return Err(Diagnostic::new(
                                DiagnosticSeverity::Error,
                                String::from("Expected Name"),
                                position,
                            ));
                        }
                    }
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

        let name = match self.parse_name_maybe()? {
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

    fn expect_token(&self, token_type: LexicalTokenType) -> Result<(), Diagnostic> {
        if let Some(token) = self.peek() {
            if token.token_type == token_type {
                return Ok(());
            }
        }

        return Err(Diagnostic::new(
            DiagnosticSeverity::Error,
            String::from(format!("Expected {:?}", token_type)),
            self.get_current_position(),
        ));
    }

    fn get_current_position(&self) -> Range {
        if let Some(token) = self.peek() {
            return token.position.clone();
        }

        Range::new(Position::new(0, 0), Position::new(0, 0))
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

        dbg!(&document);

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
                        assert_eq!(inline_fragment.type_condition.as_ref().unwrap().name.value, "User");
                        assert_eq!(inline_fragment.directives.len(), 0);
                        assert_eq!(inline_fragment.selection_set.selections.len(), 2);
                    }
                    _ => panic!("Expected InlineFragment"),
                }
            }
            _ => panic!("Expected OperationDefinition"),
        }
    }
}
