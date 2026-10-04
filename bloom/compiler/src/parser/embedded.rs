use crate::{
    common::{ERRONEOUS_SPAN, Span},
    parser::Parser,
};

impl<'a> Parser<'a> {
    pub fn parse_embedded_expression(&mut self, delim: &'a str) -> Span {
        // goal of the method is to correctly find the end of a JS expression
        // expects that current pointer is at the start of the expr
        let mut num_braces = 0usize;
        let mut quote: Option<u8> = None;

        let start = self.pos;

        loop {
            let cur = self.peek(0);

            let may_restore_to = self.pos;
            if num_braces == 0 && quote.is_none() && self.eat(delim) {
                self.restore(may_restore_to);
                return if start == self.pos {
                    ERRONEOUS_SPAN
                } else {
                    Span::new(start, self.pos)
                };
            }

            if cur == 0 {
                // we have exhausted the expression until EOF
                // this may or may not be intended but we do know that we are yet to find the delimiter.
                // this error is NOT a concern of the embedded scanner. the caller is expected to resolve it.
                return ERRONEOUS_SPAN;
            }

            let cur_is_quote = matches!(cur, b'"' | b'\'' | b'`');
            // we don't handle the template literal case separately.
            // the JS/TS parser can later take over for that.
            // the string handling guarantees that all the braces within the literal are skipped
            // and that is the only case we genuienly care about

            if cur_is_quote {
                if quote.is_none() {
                    quote = Some(cur);
                } else if quote == Some(cur) {
                    quote = None;
                }

                self.consume(1);
                continue;
            }

            if quote.is_some() && cur == b'\\' {
                self.consume(2); // skip whatever character comes after the escape
                // this is naive, but it handles \', \" and \`, which is more than sufficient
                continue;
            }

            if cur == b'{' && quote.is_none() {
                num_braces += 1;
            }

            if cur == b'}' && num_braces > 0 && quote.is_none() {
                num_braces -= 1;
            }

            self.consume(1);
        }
    }
}
