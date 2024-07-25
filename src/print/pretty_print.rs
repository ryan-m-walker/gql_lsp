use crate::parser::types::{
    Argument, Definition, Directive, Document, Field, Name, OperationDefinition, OperationType,
    Selection, Value,
};

macro_rules! indent {
    ($n:expr, $s:expr) => {{
        let padding = " ".repeat($n * 2);
        format!("{}{}", padding, $s)
    }};
}

pub fn print(document: &Document) -> String {
    return document.pretty_print(0);
}

trait PrettyPrint {
    fn pretty_print(&self, depth: usize) -> String;
}

impl PrettyPrint for Document {
    fn pretty_print(&self, depth: usize) -> String {
        let mut output = String::new();

        for definition in &self.definitions {
            output.push_str(&definition.pretty_print(depth));
            output.push('\n');
        }

        output
    }
}

impl PrettyPrint for Definition {
    fn pretty_print(&self, depth: usize) -> String {
        match self {
            Definition::OperationDefinition(operation_definition) => {
                operation_definition.pretty_print(depth)
            }
            _ => "".to_string(),
        }
    }
}

impl PrettyPrint for Name {
    fn pretty_print(&self, _depth: usize) -> String {
        self.value.to_string()
    }
}

impl PrettyPrint for OperationDefinition {
    fn pretty_print(&self, depth: usize) -> String {
        let mut output: Vec<String> = vec![];

        match &self.operation {
            OperationType::Query => output.push(String::from("query")),
            OperationType::Mutation => output.push(String::from("mutation")),
            OperationType::Subscription => output.push(String::from("subscription")),
        }

        // TODO - args, etc...

        if let Some(name) = &self.name {
            output.push(name.pretty_print(depth));
        }

        if &self.directives.len() > &0 {
            for directive in &self.directives {
                output.push(directive.pretty_print(depth));
            }
        }

        let mut selections = vec![];

        selections.push(String::from("{"));

        for selection in &self.selection_set.selections {
            selections.push(selection.pretty_print(depth + 1));
        }

        selections.push(String::from("}"));
        output.push(selections.join("\n"));

        output.join(" ")
    }
}

impl PrettyPrint for Selection {
    fn pretty_print(&self, depth: usize) -> String {
        match self {
            Selection::Field(field) => field.pretty_print(depth),
            _ => "TODO".to_string(),
            // Selection::FragmentSpread(fragment_spread) => fragment_spread.pretty_print(depth),
            // Selection::InlineFragment(inline_fragment) => inline_fragment.pretty_print(depth),
        }
    }
}

impl PrettyPrint for Field {
    fn pretty_print(&self, depth: usize) -> String {
        let mut output: Vec<String> = vec![];

        if let Some(alias) = &self.alias {
            output.push(alias.pretty_print(depth));
            output.push(String::from(": "));
        }

        output.push(self.name.pretty_print(depth));

        if &self.arguments.len() > &0 {
            output.push(String::from("("));

            let arguments = &self
                .arguments
                .iter()
                .map(|argument| argument.pretty_print(depth))
                .collect::<Vec<String>>()
                .join(", ");

            output.push(arguments.to_string());
            output.push(String::from(")"));
        }

        if &self.directives.len() > &0 {
            output.push(String::from(" "));
            let directives = &self
                .directives
                .iter()
                .map(|directive| directive.pretty_print(depth))
                .collect::<Vec<String>>()
                .join(" ");
            output.push(directives.to_string());
            output.push(String::from(" "));
        }

        if let Some(selection_set) = &self.selection_set {
            if &selection_set.selections.len() > &0 {
                let mut selections = vec![];

                selections.push(String::from("{"));

                for selection in &selection_set.selections {
                    selections.push(selection.pretty_print(depth + 1));
                }

                selections.push(indent!(depth, String::from("}")));
                output.push(selections.join("\n"));
            }
        }

        indent!(depth, output.join(""))
    }
}

impl PrettyPrint for Directive {
    fn pretty_print(&self, depth: usize) -> String {
        let mut output: Vec<String> = vec![];

        output.push(String::from("@"));
        output.push(self.name.pretty_print(depth));

        if &self.arguments.len() > &0 {
            output.push(String::from("("));

            let arguments = &self
                .arguments
                .iter()
                .map(|argument| argument.pretty_print(depth))
                .collect::<Vec<String>>()
                .join(", ");

            output.push(arguments.to_string());
            output.push(String::from(")"));
        }

        output.join("")
    }
}

impl PrettyPrint for Argument {
    fn pretty_print(&self, depth: usize) -> String {
        let name = self.name.pretty_print(depth);
        let value = self.value.pretty_print(depth);
        format!("{}: {}", name, value)
    }
}

impl PrettyPrint for Value {
    fn pretty_print(&self, depth: usize) -> String {
        match self {
            Value::IntValue(node) => node.value.to_string(),
            Value::FloatValue(node) => node.value.to_string(),
            Value::StringValue(node) => format!("\"{}\"", node.value),
            Value::BooleanValue(node) => node.value.to_string(),
            Value::NullValue(_) => "null".to_string(),
            Value::EnumValue(node) => node.value.to_string(),
            Value::ListValue(node) => {
                let values = node
                    .values
                    .iter()
                    .map(|value| value.pretty_print(depth))
                    .collect::<Vec<String>>();
                format!("[{}]", values.join(", "))
            }
            Value::Variable(node) => node.name.pretty_print(depth),
            // TODO - Object Value
            _ => "".to_string(),
        }
    }
}
