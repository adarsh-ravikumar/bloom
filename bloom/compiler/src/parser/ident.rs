use core::sync;

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
            b'=' => "=",
            _ => panic!(
                "byte_to_str called with unsupported byte '{byte}':{}",
                byte as char
            ),
        }
    }

    pub fn parse_ident(&mut self, delim: &[u8]) -> Span {
        let start = self.pos;

        if !matches!(self.peek(0), b'a'..b'z' | b'A'..b'Z' | b'_') {
            let mut sync_set = delim
                .iter()
                .map(|&b| Self::byte_to_str(b))
                .collect::<Vec<&str>>();

            sync_set.extend(&[" ", "\t", "\n"]);

            self.synchronize(&sync_set);

            return ERRONEOUS_SPAN;
        }

        self.consume(1);

        loop {
            let cur = self.peek(0);

            if matches!(cur, b' ' | b'\t' | b'\n' | b'\0')
                || delim.contains(&cur)
            {
                break;
            }

            let is_valid_char = matches!(cur, b'a'..b'z' | b'A'..b'Z' | b'0'..b'9' | b'$' | b'-' | b'_');

            if !is_valid_char {
                let mut sync_set = delim
                    .iter()
                    .map(|&b| Self::byte_to_str(b))
                    .collect::<Vec<&str>>();

                sync_set.extend(&[" ", "\t", "\n"]);
                self.synchronize(&sync_set);
                return ERRONEOUS_SPAN;
            }

            self.consume(1);
        }

        Span::new(start, self.pos)
    }
}
