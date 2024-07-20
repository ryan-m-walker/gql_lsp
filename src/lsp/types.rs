#[derive(Debug, Clone)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    pub line: usize,
    pub character: usize,
}

impl Position {
    pub fn new(line: usize, character: usize) -> Position {
        Position { line, character }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    pub fn new(start: Position, end: Position) -> Range {
        Range { start, end }
    }
}

/// Represents a diagnostic, such as a compiler error or warning.
/// Diagnostic objects are only valid in the scope of a resource
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// The range at which the message applies
    pub range: Range,

    /// The diagnostic's severity. To avoid interpretation mismatches when a
    /// server is used with different clients it is highly recommended that
    /// servers always provide a severity value. If omitted, it’s recommended
    /// for the client to interpret it as an Error severity.
    pub severity: DiagnosticSeverity,

    // The diagnostic's message.
    pub message: String,
    // The diagnostic's code, which might appear in the user interface.
    // code: Option<i32>,

    // An optional property to describe the error code.
    // codeDescription?: CodeDescription;

    // A human-readable string describing the source of this
    // diagnostic, e.g. 'typescript' or 'super lint'.
    // source: String,

    // Additional metadata about the diagnostic.
    // tags?: DiagnosticTag[];

    // An array of related diagnostic information, e.g. when symbol-names within
    // a scope collide all definitions can be marked via this property.
    // relatedInformation?: DiagnosticRelatedInformation[];
}

impl Diagnostic {
    pub fn new(severity: DiagnosticSeverity, message: String, range: Range) -> Diagnostic {
        Diagnostic {
            severity,
            message,
            range,
        }
    }

    pub fn print(&self, source: &str) {
        println!("{:?}: {:?}", self.severity, self.message);

        let lines = source.lines().collect::<Vec<&str>>();
        let error_line = lines.get(self.range.start.line);

        if let Some(error_line) = error_line {
            println!("{}", error_line);
            let mut caret = String::new();
            for _ in 0..self.range.start.character {
                caret.push(' ');
            }
            for _ in self.range.start.character..self.range.end.character {
                caret.push('^');
            }
            println!("{}", caret);
        }
    }
}
