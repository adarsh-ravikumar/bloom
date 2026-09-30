use crate::{common::IOFile, diagnostic::Diagnostic, parser::node::Root};

pub struct Parser<'a> {
    pub file: &'a IOFile,
    pub root: Option<Root>,
    pub position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(file: &'a IOFile) -> Self {
        Self {
            file,
            position: 0,
            root: None,
        }
    }

    pub fn parse(&mut self) -> Result<(), Diagnostic> {
        self.exhaust_whitespace();

        let fragment = self.parse_fragment()?;

        self.root = Some(Root {
            fragment: Some(fragment),
        });

        Ok(())
    }

    pub fn advance(&mut self) -> Option<u8> {
        self.advance_by(1)
    }

    pub fn advance_by(&mut self, by: usize) -> Option<u8> {
        let next = self.peek();
        self.consume_by(by);
        next
    }

    pub fn consume(&mut self) {
        self.consume_by(1);
    }

    pub fn consume_by(&mut self, by: usize) {
        self.position += by;
    }

    pub fn revert(&mut self) {
        self.position -= 1;
    }

    pub fn peek(&self) -> Option<u8> {
        self.peek_by(0)
    }

    pub fn peek_by(&self, by: usize) -> Option<u8> {
        self.file.get(self.position + by)
    }

    pub fn exhaust_whitespace(&mut self) {
        while let Some(cur) = self.advance()
            && (cur == b' ' || cur == b'\n' || cur == b'\t')
        {}

        self.revert()
    }

    pub fn expect_char(&mut self, ch: u8) -> Result<(), Diagnostic> {
        let cur = self.peek();

        if cur.is_none() {
            Err(self.emit_unexpected_character(ch, 0, self.position))
        } else if cur != Some(ch) {
            Err(self.emit_unexpected_character(ch, cur.unwrap(), self.position))
        } else {
            Ok(())
        }
    }
}
