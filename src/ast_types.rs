use crate::lsp_types::{Position, Range};

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub definitions: Vec<Definition>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    OperationDefinition(OperationDefinition),
    TypeSystemDefinitionOrExtension(TypeSystemDefinitionOrExtension),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutableDefinition {
    OperationDefinition,
    FragmentDefinition,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSystemDefinitionOrExtension {
    //
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExecutableDocument {
    definitions: Vec<ExecutableDefinition>,
    position: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OperationDefinition {
    pub name: Option<Name>,
    pub operation: OperationType,
    pub variable_definitions: Vec<VariableDefinition>,

    // pub directives: Option<i8>,           // TODO
    // pub selection_set: i8,                // TODO
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FragmentDefinition {
    position: Position,
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
    // default_value: Option<Value>, TODO
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
