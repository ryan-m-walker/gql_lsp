#![cfg(test)]

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
            scalar Time @tz(offset: 0)
            "This is a description"
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

#[test]
fn it_errs_for_operations_with_description() {
    let source = r#"
            "This is a description"
            query {
                test
            }"#;

    let document = parse(source.to_string());
    assert!(document.is_err());
}

#[test]
fn it_can_parse_object_types() {
    let source = r#"
        type User {
            id: ID!
            name: String
            age: Int
            friends: [User]
        }
    "#;

    let document = parse(source.to_string());
    let document = document.unwrap();

    match document.definitions.get(0) {
        Some(Definition::ObjectTypeDefinition(object_type_definition)) => {
            assert_eq!(object_type_definition.name.value, "User");
            assert_eq!(object_type_definition.fields.len(), 4);
        }
        _ => panic!("Expected ObjectTypeDefinition"),
    }
}

#[test]
fn it_can_parse_interface_types() {
    let source = r#"
        interface User {
            id: ID!
            name: String
            age: Int
            friends: [User]
        }
    "#;

    let document = parse(source.to_string());
    let document = document.unwrap();

    match document.definitions.get(0) {
        Some(Definition::InterfaceTypeDefinition(interface_type_definition)) => {
            assert_eq!(interface_type_definition.name.value, "User");
            assert_eq!(interface_type_definition.fields.len(), 4);
        }
        _ => panic!("Expected InterfaceTypeDefinition"),
    }
}

#[test]
fn it_can_parse_union_types() {
    let source = r#"
        union User = Admin | Member
    "#;

    let document = parse(source.to_string());
    let document = document.unwrap();

    match document.definitions.get(0) {
        Some(Definition::UnionTypeDefinition(union_type_definition)) => {
            assert_eq!(union_type_definition.name.value, "User");
            assert_eq!(union_type_definition.member_types.len(), 2);
        }
        _ => panic!("Expected UnionTypeDefinition"),
    }
}

#[test]
fn it_can_parse_enum_types() {
    let source = r#"
        enum Role {
            ADMIN
            MEMBER
            GUEST
        }
    "#;

    let document = parse(source.to_string());
    let document = document.unwrap();

    match document.definitions.get(0) {
        Some(Definition::EnumTypeDefinition(enum_type_definition)) => {
            assert_eq!(enum_type_definition.name.value, "Role");
            assert_eq!(enum_type_definition.values.len(), 3);
        }
        _ => panic!("Expected EnumTypeDefinition"),
    }
}
