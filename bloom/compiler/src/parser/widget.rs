use crate::{
    common::{ERRONEOUS_SPAN, Span},
    parser::{
        Parser,
        nodes::{Attribute, AttributeValue, FragmentNode, Part, Widget},
        parser::FragmentEnd,
    },
};

impl<'a> Parser<'a> {
    pub fn parse_open_tag(&mut self) -> (Widget, bool) {
        let tag_open_pos = self.pos;
        self.eat("<");

        let name_start = self.pos; // record for diagnostics
        let is_platform = self.eat(":");
        let mut name = self.parse_ident(&[b'>']);
        if name == ERRONEOUS_SPAN {
            self.emit_invalid_tag_name(name_start, self.pos);
        }

        name.start = name_start; // we keep the ":" in the name

        self.consume_whitespace();

        // attribute list
        let diag_pos_before_attribs = self.diagnostics.len();
        let attributes = self.parse_attributes();

        // self closing
        let self_closing = self.eat("/>");

        if !self_closing {
            self.open_tags.push(name);
        }

        if !self_closing && !self.eat(">") {
            // we failed to establish the opening-tag boundary.
            // attribute diagnostics were hence producced speculatively and can no longer be
            // trusted, because we don't know that we were actually parsing a well-delimitied
            // attribute list. remove all errors emitted by `parse_attrbiutes`.

            while self.diagnostics.len() > diag_pos_before_attribs {
                self.diagnostics.pop();
            }

            self.emit_expect_open_tag_end(tag_open_pos);
        }

        (
            Widget {
                name,
                attributes,
                fragment: None,
                is_platform,
            },
            self_closing,
        )
    }

    pub fn parse_attribute_value(&mut self) -> Option<AttributeValue> {
        let expr_start = self.pos;
        if self.eat("{") {
            let expr = self.parse_embedded_expression("}");

            if !self.eat("}") {
                self.emit_unclosed_expression(expr_start);
            }

            return Some(AttributeValue::Expression(expr));
        }

        let quote = self.peek(0);
        let quote_start = self.pos;

        if matches!(quote, b'\'' | b'"') {
            self.consume(1);

            // value must either be a quoted string or an expression
            let mut parts = Vec::new();

            while self.peek(0) != quote && self.peek(0) != 0 {
                let text = self.parse_text(&[quote, b'{']);
                if let Some(text) = text {
                    parts.push(Part::Text(text));
                }

                let expr_start = self.pos;

                if !self.eat("{") {
                    break;
                }

                let expr = self.parse_embedded_expression("}");

                if !self.eat("}") {
                    self.emit_unclosed_expression(expr_start);
                }

                parts.push(Part::Expression(expr));
            }

            let quote = if quote == b'"' { "\"" } else { "'" };

            if !self.eat(quote) {
                self.emit_unclosed_attribute_value(quote_start);
            }

            return Some(AttributeValue::Parts(parts));
        }

        return None;
    }

    pub fn parse_attributes(&mut self) -> Vec<Attribute> {
        let mut attributes = Vec::new();

        // attributes can be of one of the following kinds
        // name
        // name={}
        // name="..."
        // name='...'

        while !matches!(self.peek(0), b'>' | b'<' | b'/') && self.peek(0) != 0 {
            // ident
            let start = self.pos; // record for diagnostics
            let name = self.parse_ident(&[b'=', b'{', b'>', b'"', b'\'', b':']);

            if name == ERRONEOUS_SPAN {
                self.emit_invalid_attribute_name(start, self.pos);
            }

            self.consume_whitespace();

            // binding
            if self.eat(":") {
                self.consume_whitespace();

                let bind_to =
                    self.parse_ident(&[b'=', b'{', b'>', b'"', b'\'', b':']);

                self.consume_whitespace();

                let bind_from = name;
                let name = Span {
                    start: name.start,
                    end: bind_to.end,
                };

                let value = if self.eat("=") {
                    // if there is a binding value, we consume it.
                    if let Some(v) = self.parse_attribute_value() {
                        Some(Box::new(v))
                    } else {
                        // invalid value
                        // diagnostics would have already been pushed. do nothing here.
                        continue;
                    }
                } else {
                    None
                };

                attributes.push(Attribute {
                    name,
                    value: AttributeValue::BindingAttribute {
                        bind_from,
                        bind_to,
                        value,
                    },
                });

                self.consume_whitespace();
                continue;
            }

            // boolean
            if !self.eat("=") {
                attributes.push(Attribute {
                    name,
                    value: AttributeValue::True,
                });

                self.consume_whitespace();
                continue;
            }

            // value
            let value = self.parse_attribute_value();
            if let Some(value) = value {
                attributes.push(Attribute { name, value });
            } else {
                // invalid value
                // diagnostics would have already been pushed. do nothing here.
                continue;
            }

            self.consume_whitespace();
        }

        attributes
    }

    pub fn parse_close_tag(&mut self, name: Span) {
        let tag_start = self.pos;

        // this is a defensive grammar assertion.
        // this case should ideally NEVER occur. but if it does, that means the parser has failed
        // to validate the closing conditions, and hence we must expect garbage.
        if !self.eat("</") {
            self.emit_expected_close_tag();
            self.synchronize(&[">", "\n", "<"]);
            return;
        }

        let closing_name_start = self.pos; // record for diagnostics
        self.eat(":"); // if this is a closing tag for a platform, the ":" must be consumed
        // before parsing the identifier. but the name string should still
        // include the ":", as that's what's expected.
        let mut closing_name = self.parse_ident(&[b'>']);
        if closing_name == ERRONEOUS_SPAN {
            self.emit_invalid_closing_tag_name(closing_name_start, self.pos);
        }

        closing_name.start = closing_name_start;

        // we check to ensure that the widget's name itself is non-erroneous,
        // as it is nonsensical to report that the tags don't match when the name
        // itself is illegal
        if self.src.view_span(name) != self.src.view_span(closing_name) {
            // look for candidate
            let mut candidate_exists = false;

            for (mut i, open) in self.open_tags.iter().rev().enumerate() {
                if self.src.view_span(closing_name) == self.src.view_span(open)
                {
                    candidate_exists = true;
                    self.last_closed_tag = closing_name;

                    // we have found a match. now, we can safely pop all the tags
                    // we pop until we hit i = 1, which means the tag at the top
                    // is the currently matched tag
                    while i > 1 {
                        self.open_tags.pop();
                        i -= 1;
                    }

                    break;
                }
            }

            if candidate_exists {
                self.emit_unclosed_tag(name, closing_name);
                self.restore(tag_start);
                return;
            } else {
                self.emit_mismatched_tag(name, closing_name);
            }
        }

        // pop the currently open tag
        self.open_tags.pop();

        if self.last_closed_tag == closing_name {
            self.last_closed_tag = ERRONEOUS_SPAN;
        }

        if !self.eat(">") {
            self.emit_expect_close_tag_end(Span::new(tag_start, self.pos));
        }
    }

    pub fn parse_widget(
        &mut self,
        mut elem: Widget,
        is_self_closing: bool,
    ) -> FragmentNode {
        if is_self_closing {
            return FragmentNode::Widget(elem);
        }

        let fragment = self.parse_fragment(&[FragmentEnd::Widget]);

        elem.fragment = Some(fragment);

        // if we have hit EOF before trying to parse the close tag,
        // the current tag was never closed
        if self.peek(0) == 0 {
            if self.last_closed_tag.start > elem.name.start {
                self.emit_unclosed_tag(elem.name, self.last_closed_tag);
            } else {
                self.emit_unclosed_tag_eof(elem.name);
            }

            return FragmentNode::Widget(elem);
        }

        // we will look for a closing tag only if the open_tags stack is non-empty
        // and the current top of the stack is equal to elem.name (compare spans' start and end)
        // this is done because IF the stary-close-block handler deems us closed in order to recover,
        // we needn't look for a closing tag.
        if !self.open_tags.is_empty()
            && self.open_tags.last().unwrap() == &elem.name
        {
            self.parse_close_tag(elem.name);
        }

        FragmentNode::Widget(elem)
    }
}
