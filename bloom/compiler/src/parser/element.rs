use crate::{
    diagnostic::Diagnostic,
    parser::{Parser, node::Element},
};

impl<'a> Parser<'a> {
    pub fn parse_element(&mut self) -> Result<Option<Element>, Diagnostic> {
        println!("Parse Element");
        self.expect_char(b'<')?;
        self.consume();

        if self.peek() == Some(b'%') {
            self.consume();

            while !(self.peek() == Some(b'%') && self.peek_by(1) == Some(b'>'))
            {
                self.consume();
            }

            self.consume_by(2);

            return Ok(None);
        }

        let name = self.parse_identifier()?;

        let mut attributes = Vec::new();

        self.exhaust_whitespace();

        loop {
            let Some(cur) = self.peek() else {
                return Err(self.emit_unexpected_eof(
                    self.position,
                    vec!["Expected attribute list, '>' or '/>'".into()],
                ));
            };

            if cur == b'/' || cur == b'>' {
                break;
            }

            attributes.push(self.parse_attribute()?);

            self.exhaust_whitespace();
        }

        let is_self_closing = self.expect_char(b'/').is_ok();

        if is_self_closing {
            self.consume();
            self.expect_char(b'>')?;
            self.consume();

            return Ok(Some(Element {
                name,
                attributes,
                fragment: None,
            }));
        }

        self.expect_char(b'>')?;
        self.consume();

        let fragment = self.parse_fragment()?;

        // tag close
        self.exhaust_whitespace();

        if self.peek().is_none() {
            return Err(self.emit_unclosed_tag(self.position, name));
        }

        self.expect_char(b'<')?;
        self.consume();
        self.expect_char(b'/')?;
        self.consume();

        self.exhaust_whitespace();

        let closing_name = self.parse_identifier()?;

        if self.file.view_span(closing_name) != self.file.view_span(name) {
            return Err(self.emit_mismatched_tag(name, closing_name));
        }

        self.exhaust_whitespace();

        self.expect_char(b'>')?;
        self.consume();

        Ok(Some(Element {
            name,
            attributes,
            fragment: Some(fragment),
        }))
    }
}
