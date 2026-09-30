use crate::{
    common::Span,
    diagnostic::{
        Diagnostic, DiagnosticClass, DiagnosticSeverity, Label, LabelKind,
    },
    parser::Parser,
};

impl<'a> Parser<'a> {
    pub fn emit_unexpected_character(
        &mut self,
        expected: u8,
        got: u8,
        pos: usize,
    ) -> Diagnostic {
        let got = if got == 0 {
            "EOF".into()
        } else {
            format!("{}", got as char)
        };

        Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnexpectedChar,
            msg: format!("unexpected char '{}'", got),
            location: Span::new(pos, pos),
            labels: vec![Label {
                span: Span::new(pos, pos),
                msg: format!("expected '{}', got '{}'", expected as char, got),
                kind: LabelKind::Primary,
                paranthesise: false,
            }],
            notes: vec![],
        }
    }

    pub fn emit_unexpected_eof(
        &mut self,
        pos: usize,
        notes: Vec<String>,
    ) -> Diagnostic {
        Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnexpectedChar,
            msg: format!("unexpected end of file"),
            location: Span::new(pos, pos),
            labels: vec![Label {
                span: Span::new(pos, pos),
                msg: format!("unexpected character"),
                kind: LabelKind::Primary,
                paranthesise: false,
            }],
            notes,
        }
    }

    pub fn emit_invalid_identifier(
        &mut self,
        ch: u8,
        pos: usize,
    ) -> Diagnostic {
        Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnexpectedChar,
            msg: format!("invalid identifier"),
            location: Span::new(pos, pos),
            labels: vec![
                Label {
                    span: Span::new(pos, pos),
                    msg: format!("got '{}'", ch as char),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: Span::new(pos, pos),
                    msg: format!("identifiers may only start with letters"),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        }
    }

    pub fn emit_mismatched_tag(
        &mut self,
        expected: Span,
        got: Span,
    ) -> Diagnostic {
        let expected_txt = self.file.view_span(expected);

        Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnmatchedTag,
            msg: format!(
                "mismatched tag: expected '{}', found '{}'",
                expected_txt,
                self.file.view_span(got),
            ),
            location: got,
            labels: vec![
                Label {
                    span: got,
                    msg: format!("expected '</{}>'", expected_txt),
                    paranthesise: false,
                    kind: LabelKind::Primary,
                },
                Label {
                    span: expected,
                    msg: format!("'{}' opened here", expected_txt),
                    paranthesise: false,
                    kind: LabelKind::Secondary,
                },
            ],
            notes: vec![],
        }
    }

    pub fn emit_unclosed_tag(
        &mut self,
        eof_pos: usize,
        expected: Span,
    ) -> Diagnostic {
        let expected_txt = self.file.view_span(expected);

        Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnmatchedTag,
            msg: format!(
                "unclosed tag: expected '{}', reached EOF",
                expected_txt,
            ),
            location: Span::new(eof_pos, eof_pos),
            labels: vec![
                Label {
                    span: Span::new(eof_pos, eof_pos),
                    msg: format!(
                        "expected '</{}>' before end of file",
                        expected_txt
                    ),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: expected,
                    msg: format!("'{}' opened here", expected_txt),
                    paranthesise: false,
                    kind: LabelKind::Secondary,
                },
            ],
            notes: vec![],
        }
    }

    pub fn emit_mismatched_control_block(
        &mut self,
        expected: Span,
        got: Span,
    ) -> Diagnostic {
        let expected_txt = self.file.view_span(expected);

        Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnmatchedTag,
            msg: format!(
                "mismatched control block: expected '{}', found '{}'",
                expected_txt,
                self.file.view_span(got),
            ),
            location: got,
            labels: vec![
                Label {
                    span: got,
                    msg: format!("expected '{}'", expected_txt),
                    paranthesise: false,
                    kind: LabelKind::Primary,
                },
                Label {
                    span: expected,
                    msg: format!("'{}' opened here", expected_txt),
                    paranthesise: false,
                    kind: LabelKind::Secondary,
                },
            ],
            notes: vec![],
        }
    }

    pub fn emit_unclosed_control_block(
        &mut self,
        eof_pos: usize,
        expected: Span,
    ) -> Diagnostic {
        let expected_txt = self.file.view_span(expected);

        Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnmatchedTag,
            msg: format!(
                "unclosed control block: expected '{}', reached EOF",
                expected_txt,
            ),
            location: Span::new(eof_pos, eof_pos),
            labels: vec![
                Label {
                    span: Span::new(eof_pos, eof_pos),
                    msg: format!(
                        "expected '{}' before end of file",
                        expected_txt
                    ),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: expected,
                    msg: format!("'{}' opened here", expected_txt),
                    paranthesise: false,
                    kind: LabelKind::Secondary,
                },
            ],
            notes: vec![],
        }
    }
}
