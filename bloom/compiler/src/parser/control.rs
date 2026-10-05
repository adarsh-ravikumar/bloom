use crate::{
    common::{ERRONEOUS_SPAN, Span},
    parser::{
        Parser,
        nodes::{
            Control, EachControl, ElifControl, Fragment, FragmentNode,
            IfControl,
        },
        parser::FragmentEnd,
    },
};

impl<'a> Parser<'a> {
    pub fn parse_control(&mut self) -> FragmentNode {
        let start = self.pos;

        self.eat("{#");

        self.consume_whitespace();

        // the control block only cares about whitespace.
        // parse_ident handles whitespace internally.
        // b' ' is passed here to keep the API simple
        // rather than introducing an Option<u8>
        let name = self.parse_ident(&[b' ']);
        if name == ERRONEOUS_SPAN {
            // error
        }

        let control_type = self.src.view_span(name);

        self.open_controls.push(name); // TODO: push only if valid

        let control_block = match control_type {
            "if" => self.parse_control_if(start, name),
            "each" => self.parse_control_each(start),
            _ => {
                // unknown control block
                unreachable!()
            }
        };

        // if we have hit EOF before trying to parse the close block,
        // the current block was never closed
        if self.peek(0) == 0 {
            if self.last_closed_control.start > name.start {
                self.emit_unclosed_control(name, self.last_closed_control);
            } else {
                self.emit_unclosed_control_eof(name);
            }

            return FragmentNode::Control(control_block);
        }

        // we will look for a closing of the block only if the open_controls stack is non-empty
        // and the current top of the stack is equal to control_block.name (compare spans' start and end)
        // this is done because IF the stary-close-tag handler deems us closed in order to recover,
        // we needn't look for a closing block.
        if !self.open_controls.is_empty()
            && self.open_controls.last().unwrap() == &name
        {
            self.parse_control_block_close(name);
        }

        FragmentNode::Control(control_block)
    }

    pub fn is_valid_control_terminator<S: AsRef<str>>(
        &mut self,
        name: S,
    ) -> bool {
        let name = name.as_ref();

        let is_valid = self.eat(format!("{name}}}"))
            || self.eat(format!("{name} "))
            || self.eat(format!("{name}\t"))
            || self.eat(format!("{name}\n"));

        if is_valid {
            self.restore(self.pos - 1);
        }

        is_valid
    }

    pub fn parse_control_block_close(&mut self, name: Span) {
        let close_start = self.pos;

        // this is a defensive grammar assertion.
        // this case should ideally NEVER occur. but if it does, that means the parser has failed
        // to validate the closing conditions, and hence we must expect garbage.
        if !self.eat("{/") {
            self.emit_expected_close_tag();
            self.synchronize(&["}", "\n", "{"]);
            return;
        }

        self.consume_whitespace();

        let closing_name_start = self.pos;
        let is_valid_closing =
            self.is_valid_control_terminator(self.src.view_span(name));

        if is_valid_closing {
            self.eat("}");
            self.open_controls.pop();

            if self.last_closed_control
                == Span::new(closing_name_start, self.pos)
            {
                self.last_closed_control = ERRONEOUS_SPAN;
            }

            return;
        }

        // ending does NOT match.
        self.synchronize(&["}", " ", "\n"]);
        let closing_name = Span::new(closing_name_start, self.pos);

        self.consume_whitespace();

        // look for candidate
        let mut candidate_exists = false;

        while !self.open_controls.is_empty() {
            let open = self.open_controls.pop().unwrap();

            if self.src.view_span(closing_name) == self.src.view_span(open) {
                candidate_exists = true;
                self.last_closed_control = closing_name;

                // we push it back to the open_tags stack as the actual widget
                // to which the tag belongs to will consume it
                self.open_controls.push(open);
                break;
            }
        }

        if candidate_exists {
            self.emit_unclosed_control(name, closing_name);
            self.restore(close_start);
            return;
        } else {
            self.emit_mismatched_control(name, closing_name);
        }

        // pop the currently open tag
        self.open_controls.pop();

        if self.last_closed_control == closing_name {
            self.last_closed_control = ERRONEOUS_SPAN;
        }

        if !self.eat("}") {
            // TODO: error
        }
    }

    pub fn parse_control_if(
        &mut self,
        control_start: usize,
        name: Span,
    ) -> Control {
        let control_start = Span::new(control_start, self.pos);

        self.consume_whitespace();

        let expr_start = self.pos;

        let expr = self.parse_embedded_expression("}");

        self.consume_whitespace();

        if self.peek(0) == 0 {
            // expression was not closed
            self.emit_unclosed_expression(expr_start);
        }

        if expr == ERRONEOUS_SPAN {
            self.emit_expected_control_expression(control_start, "if");
        }

        if !self.eat("}") {
            self.emit_unclosed_control_header(control_start, "if"); // reaches fatal EOF
        }

        let fragment = self.parse_fragment(&[
            FragmentEnd::ControlCase,
            FragmentEnd::ControlEnd,
        ]);

        // while we have {:, we have to parse elif, else
        let mut elifs = Vec::new();
        let mut else_body = None;
        let mut else_start = ERRONEOUS_SPAN;

        loop {
            let closing_start = self.pos;

            if self.eat("{/") {
                // revert and return
                self.restore(closing_start);
                break;
            }

            // the breaking conditions when we do not terminate through the {/ path
            // is either we have exhausted the source and hit EOF, or we were closed
            // by the stray analyzer, in which case, we will no longer be on the top of
            // the open_controls stack.
            let is_eof = self.peek(0) == 0;
            let is_on_top = !self.open_controls.is_empty()
                && self.open_controls.last().unwrap() == &name;

            if is_eof || !is_on_top {
                break;
            }

            let case_start = self.pos;
            let is_case = self.eat("{:");

            self.consume_whitespace();

            if is_case {
                if self.is_valid_control_terminator("else") {
                    else_start = Span::new(case_start, self.pos);
                    else_body = Some(self.parse_else(else_start));
                    continue;
                }

                if self.eat("elif ") {
                    if else_body.is_some() {
                        self.emit_case_after_else(else_start, case_start);
                        // no sync needed. we will continue parsing
                    }

                    elifs
                        .push(self.parse_elif(Span::new(case_start, self.pos)));
                    continue;
                }

                // invalid case
                self.synchronize(&[" ", "\n"]); // sync to possible end of case name
                self.emit_unknown_control_case(case_start, &["elif", "else"]);

                // sync to the the start of the next case or a closing case
                self.synchronize(&["{:", "{/"]); // sync to possible end of case name
            }
        }

        Control::If(IfControl {
            condition: expr,
            body: fragment,
            elifs,
            else_body,
        })
    }

    pub fn parse_elif(&mut self, case_start: Span) -> ElifControl {
        self.consume_whitespace();

        let expr = self.parse_embedded_expression("}");

        if expr == ERRONEOUS_SPAN {
            self.emit_expected_control_expression(case_start, "elif");
        }

        self.consume_whitespace();

        if !self.eat("}") {
            self.emit_unclosed_control_header(case_start, "elif"); // reaches fatal EOF
        }

        let fragment = self.parse_fragment(&[
            FragmentEnd::ControlCase,
            FragmentEnd::ControlEnd,
        ]);

        ElifControl {
            condition: expr,
            body: fragment,
        }
    }

    pub fn parse_else(&mut self, case_start: Span) -> Fragment {
        self.consume_whitespace();

        // we are to NOT accept any conditions here
        // we expect the } to occur right after {:else
        // what we can try, is naively sync to the nearest '}', and report that
        // if there isn't any, then we can simply

        if !self.eat("}") {
            self.synchronize(&["}", "\n"]);

            // if we DID find a }, then we say "did not expect expression"
            // otherwise, we just report missing '}'
            if self.eat("}") {
                let expr_end = self.pos - 1;
                self.emit_unexpected_control_expression(
                    case_start,
                    Span::new(case_start.end + 1, expr_end),
                    Span::new(self.pos - 1, self.pos - 1),
                );
            } else {
                // TODO: The else could also mean we hit a new line, as that is
                // a valid sync point. But, the error points to EOF
                self.emit_unclosed_control_header(case_start, "else"); // reaches fatal EOF
            }
        }

        self.parse_fragment(&[
            FragmentEnd::ControlEnd,
            FragmentEnd::ControlCase, // this is an error state. but we still track it for recovery
        ])
    }

    pub fn parse_control_each(&mut self, control_start: usize) -> Control {
        let control_start = Span::new(control_start, self.pos);

        self.consume_whitespace();

        let context_start = self.pos; // record for diagnostics
        let context = self.parse_ident(&[b' ']);
        if context == ERRONEOUS_SPAN {
            self.emit_invalid_each_context_name(context_start, self.pos);
        }

        self.consume_whitespace();

        // this will track the case 'in}' as well, helping us catch "missing expression"
        if !self.is_valid_control_terminator("in") {
            self.emit_expected_each_in(context);
        }

        self.consume_whitespace();

        if self.peek(0) == b'}' {
            self.emit_expected_control_expression(control_start, "each");
        }

        let expr_start = self.pos;

        let expression = self.parse_embedded_expression("}");

        self.consume_whitespace();

        if self.peek(0) == 0 {
            // expression was not closed
            self.emit_unclosed_expression(expr_start);
        }

        if !self.eat("}") {
            self.emit_unclosed_control_header(control_start, "each"); // reaches fatal EOF
        }

        let body = self.parse_fragment(&[FragmentEnd::ControlEnd]);

        Control::Each(EachControl {
            expression,
            context,
            body,
        })
    }
}
