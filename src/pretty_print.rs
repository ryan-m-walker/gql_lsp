use crate::ast_types::{
    Definition, Document, ListType, Name, NamedType, NonNullType, OperationType, Type, Variable,
    VariableDefinition, SelectionSet, Selection, Field,
};

pub trait PrettyPrint {
    fn pretty_print(&self) -> String;
}

impl PrettyPrint for Document {
    fn pretty_print(&self) -> String {
        let mut lines: Vec<String> = Vec::new();

        for definition in &self.definitions {
            lines.push(definition.pretty_print());
        }

        lines.join("\n")
    }
}

impl PrettyPrint for Definition {
    fn pretty_print(&self) -> String {
        match self {
            Definition::OperationDefinition(operation_definition) => {
                operation_definition.pretty_print()
            }
            Definition::TypeSystemDefinitionOrExtension(_) => {
                unimplemented!()
            }
        }
    }
}

impl PrettyPrint for crate::ast_types::OperationDefinition {
    fn pretty_print(&self) -> String {
        let mut pretty_print = String::new();

        pretty_print.push_str(&format!("{} ", self.operation.pretty_print()));

        if let Some(name) = &self.name {
            pretty_print.push_str(&name.value);
        }

        if !self.variable_definitions.is_empty() {
            pretty_print.push_str("(");
            for (index, variable_definition) in self.variable_definitions.iter().enumerate() {
                pretty_print.push_str(&variable_definition.pretty_print());
                if index < self.variable_definitions.len() - 1 {
                    pretty_print.push_str(", ");
                }
            }
            pretty_print.push_str(")");
        }

        // TODO: Add directives
        
        
        pretty_print.push_str(&self.selection_set.pretty_print());

        pretty_print
    }
}

impl PrettyPrint for SelectionSet {
    fn pretty_print(&self) -> String {
        let mut pretty_print = String::new();

        pretty_print.push_str("{\n");

        for selection in &self.selections {
            pretty_print.push_str(&selection.pretty_print());
            pretty_print.push_str("\n");
        }

        pretty_print.push_str("}");

        pretty_print
    }
}

impl PrettyPrint for Selection {
    fn pretty_print(&self) -> String {
        match self {
            Selection::Field(field) => field.pretty_print(),
            _ => unimplemented!(),
            // Selection::FragmentSpread(fragment_spread) => fragment_spread.pretty_print(),
            // Selection::InlineFragment(inline_fragment) => inline_fragment.pretty_print(),
        }
    }
}

impl PrettyPrint for Field {
    fn pretty_print(&self) -> String {
        let mut pretty_print = String::new();

        pretty_print.push_str(&self.name.value);

        // TODO
        // if !self.arguments.is_empty() {
        //     pretty_print.push_str("(");
        //     for (index, argument) in self.arguments.iter().enumerate() {
        //         pretty_print.push_str(&argument.pretty_print());
        //         if index < self.arguments.len() - 1 {
        //             pretty_print.push_str(", ");
        //         }
        //     }
        //     pretty_print.push_str(")");
        // }

        // if let Some(selections) = self.selection_set.selections {
        //
        // }

        pretty_print
    }
}

impl PrettyPrint for OperationType {
    fn pretty_print(&self) -> String {
        match self {
            OperationType::Query => String::from("query"),
            OperationType::Mutation => String::from("mutation"),
            OperationType::Subscription => String::from("subscription"),
        }
    }
}

impl PrettyPrint for VariableDefinition {
    fn pretty_print(&self) -> String {
        format!(
            "${}: {}",
            self.variable.pretty_print(),
            self.variable_type.pretty_print()
        )
    }
}

impl PrettyPrint for Variable {
    fn pretty_print(&self) -> String {
        self.name.pretty_print()
    }
}

impl PrettyPrint for Type {
    fn pretty_print(&self) -> String {
        match self {
            Type::NamedType(named_type) => named_type.pretty_print(),
            Type::ListType(list_type) => list_type.pretty_print(),
            Type::NonNullType(non_null_type) => non_null_type.pretty_print(),
        }
    }
}

impl PrettyPrint for ListType {
    fn pretty_print(&self) -> String {
        format!("[{}]", self.wrapped_type.pretty_print())
    }
}

impl PrettyPrint for NonNullType {
    fn pretty_print(&self) -> String {
        format!("{}!", self.wrapped_type.pretty_print())
    }
}

impl PrettyPrint for NamedType {
    fn pretty_print(&self) -> String {
        self.name.pretty_print()
    }
}

impl PrettyPrint for Name {
    fn pretty_print(&self) -> String {
        self.value.clone()
    }
}
