use crate::{
    common::{ERRONEOUS_SPAN, Span},
    diagnostic::Diagnostic,
    parser::Parser,
};

impl<'a> Parser<'a> {
    fn is_letter(cur: u8) -> bool {
        cur.is_ascii_alphabetic()
    }

    fn is_identifier_part(cur: u8) -> bool {
        cur.is_ascii_alphanumeric() || cur == b'_' || cur == b'-'
    }

    pub fn parse_identifier(&mut self) -> Result<Span, Diagnostic> {
        println!("Parse Identifier");
        // IDENT = LETTER (LETTER | DIGIT | HYPEHN | UNDERSCORE)*

        let start = self.position;

        // LETTER
        let Some(cur) = self.advance() else {
            return Err(self.emit_unexpected_eof(
                self.position,
                vec!["Expected identifier".into()],
            ));
        };

        if !Self::is_letter(cur) {
            return Err(self.emit_invalid_identifier(cur, self.position - 1));
        }

        // (LETTER | DIGIT | HYPHEN | UNDERSCORE)*
        while let Some(cur) = self.advance()
            && Self::is_identifier_part(cur)
        {}

        self.revert();

        Ok(Span::new(start, self.position))
    }
}
