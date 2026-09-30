use crate::{
    common::Span,
    diagnostic::Diagnostic,
    parser::{ControlBlock, FragmentNode, Parser, control},
};

impl<'a> Parser<'a> {
    pub fn parse_control_block(&mut self) -> Result<FragmentNode, Diagnostic> {
        println!("Parse Control");

        let control_type = self.parse_identifier()?;
        self.exhaust_whitespace();

        let mut inner_braces = 0;

        let start = self.position;

        while let Some(cur) = self.peek() {
            if cur == b'{' {
                inner_braces += 1;
            }

            if cur == b'}' {
                if inner_braces <= 0 {
                    self.consume();
                    break;
                } else {
                    inner_braces -= 1;
                }
            }

            self.consume();
        }

        let fragment = self.parse_fragment()?;
        self.exhaust_whitespace();

        self.expect_char(b'{')?;

        self.consume();

        self.expect_char(b'/')?;

        self.consume();

        let closing_name = self.parse_identifier()?;

        if self.file.view_span(closing_name)
            != self.file.view_span(control_type)
        {
            return Err(
                self.emit_mismatched_control_block(control_type, closing_name)
            );
        }

        self.expect_char(b'}')?;

        self.consume();

        Ok(FragmentNode::ControlBlock(ControlBlock {
            control_type,
            fragment,
            expression: Span::new(start, self.position),
        }))
    }
}
