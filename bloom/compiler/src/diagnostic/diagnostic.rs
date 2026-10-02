use crate::{common::Span, utils::Style};

pub enum DiagnosticSeverity {
    Warn,
    Error,
}

impl DiagnosticSeverity {
    pub fn color(&self) -> &str {
        match self {
            Self::Warn => Style::BRIGHT_YELLOW,
            Self::Error => Style::BRIGHT_RED,
        }
    }
}

pub enum IdentType {
    Attribute,
    Tag,
    EachContext,
}

pub enum DiagnosticClass {
    InvalidIdentifier(IdentType),
    MismatchedTag,
    ExpectedToken,
    UnclosedTag,
    StrayClosing,
    UnclosedExpression,
    UnclosedQuotedValue,
    UnclosedComment,
    UnclosedControlHeader,
    UnclosedControl,
    ExpectedControlEnd,
    MismatchedControl,
    UnknownControlCase,
    UnexpectedControlExpression,
}

impl DiagnosticClass {
    pub fn code(&self) -> &'static str {
        match self {
            // 0xx => Parser Errors
            Self::InvalidIdentifier(_) => "E001",
            Self::ExpectedToken => "E002",
            Self::MismatchedTag => "E003",
            Self::UnclosedTag => "E004",
            Self::StrayClosing => "E005",
            Self::UnclosedExpression => "E006",
            Self::UnclosedQuotedValue => "E007",
            Self::UnclosedComment => "E008",
            Self::UnclosedControlHeader => "E009",
            Self::UnclosedControl => "E010",
            Self::MismatchedControl => "E011",
            Self::ExpectedControlEnd => "E012",
            Self::UnknownControlCase => "E013",
            Self::UnexpectedControlExpression => "E014",
        }
    }
}

pub enum LabelKind {
    Primary,
    Secondary,
}

pub struct Label {
    pub span: Span,
    pub msg: String,
    pub kind: LabelKind,
    pub paranthesise: bool,
}

pub struct Diagnostic {
    pub severity: DiagnosticSeverity,
    pub class: DiagnosticClass,

    pub msg: String,

    pub location: Span,

    pub labels: Vec<Label>,
    pub notes: Vec<String>,
}
