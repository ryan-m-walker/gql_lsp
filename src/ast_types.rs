use std::fmt::Display;

use crate::lsp_types::{Position, Range};

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub definitions: Vec<Definition>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    OperationDefinition(OperationDefinition),
    FragmentDefinition(FragmentDefinition),
    // TypeSystemDefinitionOrExtension(TypeSystemDefinitionOrExtension),
}

// #[derive(Debug, Clone, PartialEq)]
// pub enum ExecutableDefinition {
//     OperationDefinition,
//     FragmentDefinition,
// }

// #[derive(Debug, Clone, PartialEq)]
// pub enum TypeSystemDefinitionOrExtension {
//
// }

// #[derive(Debug, Clone, PartialEq)]
// pub struct ExecutableDocument {
//     definitions: Vec<ExecutableDefinition>,
//     position: Position,
// }

#[derive(Debug, Clone, PartialEq)]
pub struct OperationDefinition {
    pub name: Option<Name>,
    pub operation: OperationType,
    pub variable_definitions: Vec<VariableDefinition>,
    pub selection_set: SelectionSet,
    pub directives: Vec<Directive>,
    pub position: Range,
    pub anonymous: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FragmentDefinition {
    pub name: Name,
    pub type_condition: NamedType,
    pub directives: Vec<Directive>,
    pub selection_set: SelectionSet,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    pub value: String,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: Name,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDefinition {
    pub variable: Variable,
    pub variable_type: Type,
    pub default_value: Option<Value>,
    pub position: Range,
}

// Type
// https://spec.graphql.org/October2021/#sec-Type-References

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    NamedType(NamedType),
    ListType(ListType),
    NonNullType(NonNullType),
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedType {
    pub name: Name,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListType {
    pub wrapped_type: Box<Type>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonNullType {
    pub wrapped_type: Box<Type>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Variable(Variable),
    IntValue(IntValue),
    FloatValue(FloatValue),
    StringValue(StringValue),
    BooleanValue(BooleanValue),
    NullValue(NullValue),
    EnumValue(EnumValue),
    ListValue(ListValue),
    ObjectValue(ObjectValue),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntValue {
    pub value: i32,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatValue {
    pub value: f32,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StringValue {
    pub value: String,
    pub block: bool,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanValue {
    pub value: bool,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NullValue {
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValue {
    pub value: String,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListValue {
    pub values: Vec<Value>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectValue {
    pub fields: Vec<ObjectField>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectField {
    pub name: Name,
    pub value: Value,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    Field(Field),
    FragmentSpread(FragmentSpread),
    InlineFragment(InlineFragment),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<Argument>,
    pub directives: Vec<Directive>,
    pub selection_set: Option<SelectionSet>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FragmentSpread {
    pub name: Name,
    pub directives: Vec<Directive>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InlineFragment {
    pub type_condition: Option<NamedType>,
    pub directives: Vec<Directive>,
    pub selection_set: SelectionSet,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionSet {
    pub selections: Vec<Selection>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    pub name: Name,
    pub position: Range,
    pub arguments: Vec<Argument>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Argument {
    pub name: Name,
    pub value: Value,
    pub position: Range,
}
