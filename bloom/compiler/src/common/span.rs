use std::fmt;

use oxc_span::Span as OxcSpan;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
// SpannedString uses where there is an erroneous span, I will deliberately allocate a new empty string.
// I do NOT want to deal with match! or Options. If you (the reader of this code, if there is any)
// think that I have committed a crime against humanity, feel free to contribute :)
pub struct SpannedString {
    pub span: Span,
    pub string: String,
}

impl Default for SpannedString {
    fn default() -> Self {
        SpannedString {
            span: ERRONEOUS_SPAN,
            string: String::new(),
        }
    }
}

impl AsRef<Span> for Span {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl From<OxcSpan> for Span {
    fn from(value: OxcSpan) -> Self {
        Self {
            start: value.start as usize,
            end: value.end as usize,
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.start, self.end)
    }
}

pub static ERRONEOUS_SPAN: Span = Span { start: 0, end: 0 };

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}
