use crate::{
    common::Span,
    diagnostic::{
        Diagnostic, DiagnosticClass, DiagnosticSeverity, IdentType, Label,
        LabelKind,
    },
    parser::Parser,
};

impl<'a> Parser<'a> {
    pub fn synchronize(&mut self, set: &[&'static str]) {
        loop {
            if self.peek(0) == 0 {
                return;
            }

            let pos = self.pos;

            for &item in set {
                if self.eat(item) {
                    self.restore(pos); // we restore, as we do not intend to eat the sync token
                    return;
                }
            }

            self.consume(1);
        }
    }

    fn reached_fatal_eof(&self) -> bool {
        // an unclosed expression or quoted attribute value is guaranteed to
        // exhaust the source and reach EOF. any diagnostics produced after
        // this point are cascading errors and hence should be discarded
        // this check is run every time a diagnostic gets emitted (see parser/diagnostics.rs)
        if let Some(last) = self.diagnostics.last() {
            return matches!(
                last.class,
                DiagnosticClass::UnclosedExpression
                    | DiagnosticClass::UnclosedQuotedValue
                    | DiagnosticClass::UnclosedComment
                    | DiagnosticClass::UnclosedControlHeader
            );
        }

        false
    }
    pub fn emit_invalid_tag_name(&mut self, start: usize, end: usize) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(
        Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::InvalidIdentifier(IdentType::Tag),
            msg: "invalid tag name".into(),
            location: Span::new(start, end),
            labels: vec![Label {
                span: Span::new(start, end),
                msg: "invalid tag name".into(),
                kind: LabelKind::Primary,
                paranthesise: false,
            }],
            notes: vec![
                "tag names must start with a letter or _ and can contain letters, numbers, $, _, and - only"
                    .into(),
            ],
        })
    }

    pub fn emit_invalid_closing_tag_name(&mut self, start: usize, end: usize) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::InvalidIdentifier(IdentType::Tag),
            msg: "invalid closing tag name".into(),
            location: Span::new(start, end),
            labels: vec![Label {
                span: Span::new(start, end),
                msg: "invalid closing tag name".into(),
                kind: LabelKind::Primary,
                paranthesise: false,
            }],
            notes: vec![
                "tag names must start with a letter or _ and can contain letters, numbers, $, _, and - only"
                    .into(),
            ],
        })
    }

    pub fn emit_invalid_attribute_name(&mut self, start: usize, end: usize) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::InvalidIdentifier(IdentType::Attribute),
            msg: "invalid attribute name".into(),
            location: Span::new(start, end),
            labels: vec![Label {
                span: Span::new(start, end),
                msg: "invalid attribute name".into(),
                kind: LabelKind::Primary,
                paranthesise: false,
            }],
            notes: vec![
                "attribute names must start with a letter or _ and can contain letters, numbers, $, _, and - only"
                    .into(),
            ],
        })
    }

    pub fn emit_invalid_each_context_name(&mut self, start: usize, end: usize) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::InvalidIdentifier(IdentType::EachContext),
            msg: "invalid each context name".into(),
            location: Span::new(start, end),
            labels: vec![Label {
                span: Span::new(start, end),
                msg: "invalid each context name".into(),
                kind: LabelKind::Primary,
                paranthesise: false,
            }],
            notes: vec![
                "'each' context name must start with a letter or _ and can contain letters, numbers, $, _, and - only"
                    .into(),
            ],
        })
    }

    pub fn emit_expect_open_tag_end(&mut self, open: usize) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::ExpectedToken,
            msg: "expected end of opening tag".into(),
            location: Span::new(self.pos, self.pos),
            labels: vec![
                Label {
                    span: Span::new(self.pos, self.pos),
                    msg: "expected '/>' or '>'".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: Span::new(open, open),
                    msg: "tag opened here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_expect_close_tag_end(&mut self, open: Span) {
        if self.reached_fatal_eof() {
            return;
        }

        let close = Span::new(self.pos, self.pos);

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::ExpectedToken,
            msg: "expected end of closing tag".into(),
            location: close,
            labels: vec![
                Label {
                    span: close,
                    msg: "expected '>'".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: open,
                    msg: "closing tag starts here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_mismatched_tag(&mut self, open: Span, close: Span) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::MismatchedTag,
            msg: "mismatched closing tag".into(),
            location: close,
            labels: vec![
                Label {
                    span: close,
                    msg: "closing tag does not match".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: open,
                    msg: "tag opened here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_unclosed_tag_eof(&mut self, open: Span) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnclosedTag,
            msg: "this tag was never closed".into(),
            location: open,
            labels: vec![
                Label {
                    span: open,
                    msg: "this tag was never closed".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: self.src.eof(),
                    msg: "reached end of file here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_unclosed_tag(&mut self, open: Span, close: Span) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnclosedTag,
            msg: "this tag was never closed".into(),
            location: open,
            labels: vec![
                Label {
                    span: open,
                    msg: "this tag was never closed".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: close,
                    msg: format!("{} closed here", self.src.view_span(close)),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_stray_closing_tag(&mut self) {
        if self.reached_fatal_eof() {
            return;
        }

        let span = Span::new(self.pos, self.pos + 2);

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::StrayClosing,
            msg: "stray closing tag".into(),
            location: span,
            labels: vec![Label {
                span,
                msg: "closing tag is not expected here".into(),
                kind: LabelKind::Primary,
                paranthesise: false,
            }],
            notes: vec![],
        });
    }

    pub fn emit_stray_closing_block(&mut self) {
        if self.reached_fatal_eof() {
            return;
        }

        let span = Span::new(self.pos, self.pos + 2);

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::StrayClosing,
            msg: "stray closing block".into(),
            location: span,
            labels: vec![Label {
                span,
                msg: "closing block is not expected here".into(),
                kind: LabelKind::Primary,
                paranthesise: false,
            }],
            notes: vec![],
        });
    }

    pub fn emit_stray_control_case(&mut self) {
        if self.reached_fatal_eof() {
            return;
        }

        let span = Span::new(self.pos, self.pos + 2);

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::StrayClosing,
            msg: "stray control case".into(),
            location: span,
            labels: vec![Label {
                span,
                msg: "control case is not expected here".into(),
                kind: LabelKind::Primary,
                paranthesise: false,
            }],
            notes: vec![],
        });
    }

    pub fn emit_unclosed_expression(&mut self, expr_start: usize) {
        if self.reached_fatal_eof() {
            return;
        }

        let open = Span::new(expr_start, expr_start + 1);
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnclosedExpression,
            msg: "this expression was never closed".into(),
            location: open,
            labels: vec![
                Label {
                    span: open,
                    msg: "expression starts here".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: self.src.eof(),
                    msg: "reached end of file here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_unclosed_attribute_value(&mut self, start: usize) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnclosedQuotedValue,
            msg: "this attribute value was never closed".into(),
            location: Span::new(start, start + 1),
            labels: vec![
                Label {
                    span: Span::new(start, start + 1),
                    msg: "attribute value starts here".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: self.src.eof(),
                    msg: "reached end of file here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_expected_close_tag(&mut self) {
        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::ExpectedToken,
            msg: "expected closing tag".into(),
            location: Span::new(self.pos, self.pos),
            labels: vec![Label {
                span: Span::new(self.pos, self.pos),
                msg: "expected `</` here".into(),
                kind: LabelKind::Primary,
                paranthesise: false,
            }],
            notes: vec![],
        });
    }

    pub fn emit_unclosed_comment(&mut self, comment_start: Span) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnclosedComment,
            msg: "this comment was never closed".into(),
            location: comment_start,
            labels: vec![
                Label {
                    span: comment_start,
                    msg: "comment starts here".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: self.src.eof(),
                    msg: "reached end of file here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_unclosed_control_header(
        &mut self,
        control_start: Span,
        control: &'static str,
    ) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnclosedControlHeader,
            msg: format!("expected end of `{control}` block header"),
            location: control_start,
            labels: vec![
                Label {
                    span: control_start,
                    msg: format!("`{control}` block starts here"),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: self.src.eof(),
                    msg: "reached end of file here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_mismatched_control(&mut self, open: Span, close: Span) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::MismatchedControl,
            msg: "mismatched closing block".into(),
            location: close,
            labels: vec![
                Label {
                    span: close,
                    msg: "closing block does not match".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: open,
                    msg: format!("block starts here"),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_case_after_else(
        &mut self,
        else_start: Span,
        case_start: usize,
    ) {
        if self.reached_fatal_eof() {
            return;
        }

        let case = Span::new(case_start, self.pos);

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::ExpectedControlEnd,
            msg: "expected end of control block after `else`".into(),
            location: case,
            labels: vec![
                Label {
                    span: case,
                    msg: "unexpected control case".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: else_start,
                    msg: "`else` block starts here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_unknown_control_case(
        &mut self,
        case_start: usize,
        expected: &[&'static str],
    ) {
        if self.reached_fatal_eof() {
            return;
        }

        let case = Span::new(case_start, self.pos);

        let mut expected_string = String::new();

        for (i, &item) in expected.iter().enumerate() {
            expected_string += &format!("`{item}`");
            if i == expected.len() - 2 {
                expected_string += " or ";
            } else if i == expected.len() - 1 {
                expected_string += " ";
            } else {
                expected_string += ", "
            }
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnknownControlCase,
            msg: "unknown control case".into(),
            location: case,
            labels: vec![Label {
                span: case,
                msg: "unknown control case".into(),
                kind: LabelKind::Primary,
                paranthesise: false,
            }],
            notes: vec![format!("expected {expected_string}")],
        });
    }

    pub fn emit_unexpected_control_expression(
        &mut self,
        case_start: Span,
        expr: Span,
        close: Span,
    ) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnexpectedControlExpression,
            msg: "unexpected expression in `else` case".into(),
            location: expr,
            labels: vec![
                Label {
                    span: expr,
                    msg: "unexpected expression".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: case_start,
                    msg: "`else` case starts here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
                Label {
                    span: close,
                    msg: "found `}` here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_unclosed_control_eof(&mut self, open: Span) {
        if self.reached_fatal_eof() {
            return;
        }

        let name = self.src.view_span(open);

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnclosedControl,
            msg: format!("this `{name}` control block was never closed"),
            location: open,
            labels: vec![
                Label {
                    span: open,
                    msg: format!(
                        "this `{name}` control block was never closed"
                    ),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: self.src.eof(),
                    msg: "reached end of file here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_unclosed_control(&mut self, open: Span, close: Span) {
        if self.reached_fatal_eof() {
            return;
        }

        let name = self.src.view_span(open);

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::UnclosedControl,
            msg: format!("this `{name}` control block was never closed"),
            location: open,
            labels: vec![
                Label {
                    span: open,
                    msg: format!(
                        "this `{name}` control block was never closed"
                    ),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: close,
                    msg: format!("{} closed here", self.src.view_span(close)),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_expected_each_in(&mut self, ident: Span) {
        if self.reached_fatal_eof() {
            return;
        }

        let expected = Span::new(ident.end, ident.end);

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::ExpectedToken,
            msg: format!("expected `in` after `{}`", self.src.view_span(ident)),
            location: expected,
            labels: vec![
                Label {
                    span: expected,
                    msg: "expected `in` here".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: ident,
                    msg: "iteration context starts here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_expected_control_expression(
        &mut self,
        open: Span,
        block_name: &'static str,
    ) {
        if self.reached_fatal_eof() {
            return;
        }

        let expected = Span::new(self.pos, self.pos);

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::ExpectedToken,
            msg: "expected expression".into(),
            location: expected,
            labels: vec![
                Label {
                    span: expected,
                    msg: "expected expression here".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
                Label {
                    span: open,
                    msg: format!("`{block_name}` block starts here"),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        });
    }

    pub fn emit_multiple_script_tags(
        &mut self,
        first_script_open: Span,
        second_script_open: Span,
    ) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::MultipleScriptTags,
            msg: "widget can only have one <script> tag".into(),
            location: second_script_open,
            labels: vec![
                Label {
                    span: first_script_open,
                    msg: "first <script> tag declared here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
                Label {
                    span: second_script_open,
                    msg: "another <script> tag declared here".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
            ],
            notes: vec![],
        })
    }

    pub fn emit_nested_script_tag(
        &mut self,
        script_span: Span,
        outer_span: Span,
    ) {
        if self.reached_fatal_eof() {
            return;
        }

        self.diagnostics.push(Diagnostic {
            severity: DiagnosticSeverity::Error,
            class: DiagnosticClass::NestedScriptTag,
            msg: "<script> tags cannot be nested".into(),
            location: script_span,
            labels: vec![
                Label {
                    span: outer_span,
                    msg: "outer tag here".into(),
                    kind: LabelKind::Secondary,
                    paranthesise: false,
                },
                Label {
                    span: script_span,
                    msg: "nested <script> tag declared here".into(),
                    kind: LabelKind::Primary,
                    paranthesise: false,
                },
            ],
            notes: vec![
                "<script> tags must be declared at the top level".into(),
            ],
        });
    }
}
