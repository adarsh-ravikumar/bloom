use crate::{
    diagnostic::Diagnostic,
    parser::{
        Parser,
        node::{Attribute, AttributeValue},
    },
};

impl<'a> Parser<'a> {
    pub fn parse_attribute(&mut self) -> Result<Attribute, Diagnostic> {
        println!("Parse Attribute");
        // ATTRIBUTE = IDENT (SPACE? EQ SPACE? VALUE)?
        // VALUE =  QUOTE (TEXT | SPACE)* QUOTE
        //         | TEXT*
        let name = self.parse_identifier()?;

        self.exhaust_whitespace();

        if self.expect_char(b'=').is_err() {
            return Ok(Attribute {
                name,
                value: AttributeValue::True,
            });
        }

        self.consume();

        self.exhaust_whitespace();

        let cur = self.peek().unwrap();

        let parts;

        if cur == b'"' || cur == b'\'' {
            self.consume();
            parts = self.parse_parts(&[cur])?;
            self.consume();
        } else {
            parts = self.parse_parts(&[b' ', b'\n', b'>', b'/'])?;
        }

        Ok(Attribute {
            name,
            value: AttributeValue::Parts(parts),
        })
    }
}
