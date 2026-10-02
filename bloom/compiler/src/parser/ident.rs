use crate::common::{ERRONEOUS_SPAN, Span};
use crate::parser::Parser;

impl<'a> Parser<'a> {
    pub fn byte_to_str(byte: u8) -> &'static str {
        match byte {
            b' ' => " ",
            b'\t' => "\t",
            b'\n' => "\n",
            b'}' => "}",
            b'{' => "{",
            b'<' => "<",
            b'>' => ">",
            b'/' => "/",
            _ => panic!("byte_to_str called with unsupported byte"),
        }
    }

    pub fn parse_ident(&mut self, delim: u8) -> Span {
        let start = self.pos;

        if !matches!(self.peek(0), b'a'..b'z' | b'A'..b'Z' | b'_') {
            self.synchronize(&[" ", "\t", "\n", Self::byte_to_str(delim)]);
            return ERRONEOUS_SPAN;
        }

        self.consume(1);

        loop {
            let cur = self.peek(0);

            if matches!(cur, b' ' | b'\t' | b'\n' | b'\0') || cur == delim {
                break;
            }

            let is_valid_char = matches!(cur, b'a'..b'z' | b'A'..b'Z' | b'0'..b'9' | b'$' | b'-' | b'_');

            if !is_valid_char {
                self.synchronize(&[" ", "\t", "\n", Self::byte_to_str(delim)]);
                return ERRONEOUS_SPAN;
            }

            self.consume(1);
        }

        Span::new(start, self.pos)
    }
}
