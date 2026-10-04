use crate::parser::{
    Parser,
    nodes::{Fragment, FragmentNode},
    parser::FragmentEnd,
};

impl<'a> Parser<'a> {
    pub fn parse_fragment(&mut self, expected_end: &[FragmentEnd]) -> Fragment {
        let mut nodes: Vec<FragmentNode> = Vec::new();

        loop {
            let cur = self.peek(0);

            match cur {
                0 => break,

                b'<' => {
                    if self.eat("<!--") {
                        self.parse_comment();
                        continue;
                    }

                    if self.peek(1) == b'/' {
                        if expected_end.contains(&FragmentEnd::Widget) {
                            // we have spotted an end tag.
                            // we must return whatever we have parsed as the fragment.
                            // the caller is expected to handle the end tag
                            // cursor points to the '<'
                            return Fragment { nodes };
                        } else {
                            self.handle_stray_closing_tag();
                            return Fragment { nodes };
                        }
                    }
                    // parse widget
                    let (elem, is_self_closing) = self.parse_open_tag();

                    match self.src.view_span(elem.name) {
                        "script" => self.parse_script_tag(elem.name),
                        _ => {
                            nodes.push(self.parse_widget(elem, is_self_closing))
                        }
                    }
                }

                b'{' => {
                    if self.peek(1) == b'#' {
                        nodes.push(self.parse_control());
                    } else if self.peek(1) == b'/' {
                        if expected_end.contains(&FragmentEnd::ControlEnd) {
                            // we have spotted an end tag.
                            // we must return whatever we have parsed as the fragment.
                            // the caller is expected to handle the end of control block
                            // cursor points to the '{'
                            return Fragment { nodes };
                        } else {
                            self.handle_stray_closing_control_block();
                            return Fragment { nodes };
                        }
                    } else if self.peek(1) == b':' {
                        if expected_end.contains(&FragmentEnd::ControlCase) {
                            // we have spotted an end tag.
                            // we must return whatever we have parsed as the fragment.
                            // the caller is expected to handle the end of control block
                            // cursor points to the '{'
                            return Fragment { nodes };
                        } else {
                            self.emit_stray_control_case();
                            self.synchronize(&["\n", "}"]);
                            self.eat("}");
                            continue;
                        }
                    } else {
                        let expr_start = self.pos;

                        self.consume(1);
                        nodes.push(FragmentNode::Expression(
                            self.parse_embedded_expression("}"),
                        ));
                        if !self.eat("}") {
                            self.emit_unclosed_expression(expr_start);
                        }
                    }
                }

                _ => {
                    // default to parse text
                    if let Some(text) = self.parse_text(&[b'<', b'{']) {
                        nodes.push(FragmentNode::Text(text))
                    }
                }
            }
        }

        Fragment { nodes }
    }
}
