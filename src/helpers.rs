use crate::constants::{CARRIAGE_RETURN, NEW_LINE};

pub fn is_line_terminator(c: char) -> bool {
    c == NEW_LINE || c == CARRIAGE_RETURN
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
