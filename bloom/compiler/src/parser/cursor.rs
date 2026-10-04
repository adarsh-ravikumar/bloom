use crate::parser::Parser;

impl<'a> Parser<'a> {
    pub fn consume(&mut self, by: usize) {
        self.pos += by;
    }

    pub fn peek(&self, by: usize) -> u8 {
        self.src.get(self.pos + by)
    }

    pub fn eat<S: AsRef<str>>(&mut self, seq: S) -> bool {
        let seq = seq.as_ref().as_bytes();

        for (i, &ch) in seq.iter().enumerate() {
            if self.peek(i) != ch {
                return false;
            }
        }

        self.consume(seq.len());
        true
    }

    pub fn restore(&mut self, to: usize) {
        assert!(to <= self.pos);
        self.pos = to;
    }

    pub fn consume_whitespace(&mut self) {
        while matches!(self.peek(0), b' ' | b'\t' | b'\n') {
            self.consume(1);
        }
    }
}
