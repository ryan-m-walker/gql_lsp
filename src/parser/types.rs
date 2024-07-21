use crate::lsp::types::Range;

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub definitions: Vec<Definition>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    OperationDefinition(OperationDefinition),
    FragmentDefinition(FragmentDefinition),
    SchemaDefinition(SchemaDefinition),
    ScalarTypeDefinition(ScalarTypeDefinition),
    ObjectTypeDefinition(ObjectTypeDefinition),
    InterfaceTypeDefinition(InterfaceTypeDefinition),
    UnionTypeDefinition(UnionTypeDefinition),
    EnumTypeDefinition(EnumTypeDefinition),
    InputObjectTypeDefinition(InputObjectTypeDefinition),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputObjectTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub directives: Vec<Directive>,
    pub fields: Vec<InputValueDefinition>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumValueDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub directives: Vec<Directive>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub directives: Vec<Directive>,
    pub values: Vec<EnumValueDefinition>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnionTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub directives: Vec<Directive>,
    pub member_types: Vec<NamedType>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub interfaces: Vec<NamedType>,
    pub directives: Vec<Directive>,
    pub fields: Vec<FieldDefinition>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub interfaces: Vec<NamedType>,
    pub directives: Vec<Directive>,
    pub fields: Vec<FieldDefinition>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FieldDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub arguments: Vec<InputValueDefinition>,
    pub field_type: Type,
    pub directives: Vec<Directive>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputValueDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub input_type: Type,
    pub default_value: Option<Value>,
    pub directives: Vec<Directive>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScalarTypeDefinition {
    pub description: Option<StringValue>,
    pub name: Name,
    pub directives: Vec<Directive>,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RootOperationTypeDefinition {
    pub operation_type: OperationType,
    pub named_type: NamedType,
    pub position: Range,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaDefinition {
    pub description: Option<StringValue>,
    pub operation_types: Vec<RootOperationTypeDefinition>,
    pub directives: Vec<Directive>,
    pub position: Range,
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

impl OperationType {
    pub fn parse(value: &str) -> Option<OperationType> {
        match value {
            "query" => Some(OperationType::Query),
            "mutation" => Some(OperationType::Mutation),
            "subscription" => Some(OperationType::Subscription),
            _ => None,
        }
    }
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

#[derive(Debug, Clone, PartialEq)]
pub enum ExecutableDirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition,
}

impl ExecutableDirectiveLocation {
    pub fn parse(value: &str) -> Option<ExecutableDirectiveLocation> {
        match value {
            "QUERY" => Some(ExecutableDirectiveLocation::Query),
            "MUTATION" => Some(ExecutableDirectiveLocation::Mutation),
            "SUBSCRIPTION" => Some(ExecutableDirectiveLocation::Subscription),
            "FIELD" => Some(ExecutableDirectiveLocation::Field),
            "FRAGMENT_DEFINITION" => Some(ExecutableDirectiveLocation::FragmentDefinition),
            "FRAGMENT_SPREAD" => Some(ExecutableDirectiveLocation::FragmentSpread),
            "INLINE_FRAGMENT" => Some(ExecutableDirectiveLocation::InlineFragment),
            "VARIABLE_DEFINITION" => Some(ExecutableDirectiveLocation::VariableDefinition),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSystemDirectiveLocation {
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}

impl TypeSystemDirectiveLocation {
    pub fn parse(value: &str) -> Option<TypeSystemDirectiveLocation> {
        match value {
            "SCHEMA" => Some(TypeSystemDirectiveLocation::Schema),
            "SCALAR" => Some(TypeSystemDirectiveLocation::Scalar),
            "OBJECT" => Some(TypeSystemDirectiveLocation::Object),
            "FIELD_DEFINITION" => Some(TypeSystemDirectiveLocation::FieldDefinition),
            "ARGUMENT_DEFINITION" => Some(TypeSystemDirectiveLocation::ArgumentDefinition),
            "INTERFACE" => Some(TypeSystemDirectiveLocation::Interface),
            "UNION" => Some(TypeSystemDirectiveLocation::Union),
            "ENUM" => Some(TypeSystemDirectiveLocation::Enum),
            "ENUM_VALUE" => Some(TypeSystemDirectiveLocation::EnumValue),
            "INPUT_OBJECT" => Some(TypeSystemDirectiveLocation::InputObject),
            "INPUT_FIELD_DEFINITION" => Some(TypeSystemDirectiveLocation::InputFieldDefinition),
            _ => None,
        }
    }
}
