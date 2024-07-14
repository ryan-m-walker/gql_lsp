use crate::{
    ast_types::OperationType,
    constants::{CARRIAGE_RETURN, MUTATION_KEYWORD, NEW_LINE, QUERY_KEYWORD, SUBSCRIPTION_KEYWORD},
};

pub fn is_line_terminator(c: char) -> bool {
    c == NEW_LINE || c == CARRIAGE_RETURN
}

pub fn to_operation_type(value: &String) -> Option<OperationType> {
    match value.as_str() {
        QUERY_KEYWORD => Some(OperationType::Query),
        MUTATION_KEYWORD => Some(OperationType::Mutation),
        SUBSCRIPTION_KEYWORD => Some(OperationType::Subscription),
        _ => None,
    }
}

pub fn is_valid_name(value: &String) -> bool {
    let mut chars = value.chars();

    let first_char = match chars.next() {
        Some(c) => c,
        None => return false,
    };

    if !first_char.is_alphabetic() && first_char != '_' {
        return false;
    }

    for c in chars {
        if !c.is_alphabetic() && !c.is_digit(10) && c != '_' {
            return false;
        }
    }

    true
}
