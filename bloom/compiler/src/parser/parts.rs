use crate::{
    common::Span,
    diagnostic::Diagnostic,
    parser::{
        Parser,
        node::{Fragment, FragmentNode},
    },
};

impl<'a> Parser<'a> {
    pub fn parse_parts(
        &mut self,
        delim: &[u8],
    ) -> Result<Fragment, Diagnostic> {
        println!(
            "Parse Parts {} {}",
            self.position,
            self.peek().unwrap_or_default() as char
        );

        let mut nodes = Vec::new();

        let mut start = self.position;
        let mut is_building_expr = false;

        let mut inner_braces = 0;

        while let Some(cur) = self.peek() {
            if delim.contains(&cur) && !is_building_expr {
                break;
            }

            if cur == b'{' {
                // consume the brace
                self.consume();

                if is_building_expr {
                    inner_braces += 1;
                } else {
                    match self.peek() {
                        Some(b'#') => {
                            self.consume();
                            nodes.push(self.parse_control_block()?);
                            start = self.position;
                            continue;
                        }

                        // this state only occurs when inside a
                        Some(b'/') => {
                            self.revert();
                            break;
                        }

                        _ => {
                            if start < self.position {
                                let span = Span::new(start, self.position);
                                let txt = self.file.view_span(span);

                                if !txt.trim().is_empty() {
                                    nodes.push(FragmentNode::Text(span));
                                }
                            }

                            start = self.position;
                            is_building_expr = true;
                            continue;
                        }
                    }
                }
            }

            if cur == b'}' {
                if inner_braces <= 0 {
                    nodes.push(FragmentNode::Expression(Span::new(
                        start,
                        self.position,
                    )));

                    self.consume();

                    start = self.position;
                    is_building_expr = false;

                    continue;
                } else {
                    inner_braces -= 1;
                }
            }

            self.consume();
        }

        if start < self.position {
            let span = Span::new(start, self.position);
            let txt = self.file.view_span(span);

            if !txt.trim().is_empty() {
                nodes.push(FragmentNode::Text(span));
            }
        }

        Ok(Fragment { nodes })
    }
}
