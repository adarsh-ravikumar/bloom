use crate::{common::Span, parser::Parser};

impl<'a> Parser<'a> {
    pub fn handle_stray_closing_tag(&mut self) {
        // if when we encounter this stray closing tag and there is a possilbe
        // candidate in the open_tags stack that might consume it, then we
        // conclude that this is NOT a stray, and in fact belongs to the candidate.
        // this case occurs only when there is an unclosed control block as a child of
        // an widget
        // `
        // <div>
        //   {#if a == b}
        // </div>
        // `
        // recovery will simply be to restore to where the closing starts and
        // continue parsing. this will allow for the parent to parse it correctly
        //
        // as for the unclosed control block, we can simply report it's status as unclosed,
        // and pop it from the open stack. recursion will take naturally handle
        // multiple such unclosed control blocks.
        //
        // the stary case is always guaranteed to occur before we get to the
        // mismatched control block end case. both the cases do the exact same
        // pop-the-stack and push-the-match algorithm. when processing the stray,
        // the error-free case is the desired block is found on the top of the stack.

        // cursor sits at '<' expecting to read '</...>'
        let restore_pos = self.pos;

        self.eat("</");

        self.consume_whitespace();

        let closing_name_start = self.pos;

        // we will synchronize to either '>' or a whitespace
        self.synchronize(&[">", " ", "\n", "\t"]);

        let closing_name = Span::new(closing_name_start, self.pos);

        // find a candidate in the open_tags stack
        let mut candidate_exists = false;

        while !self.open_tags.is_empty() {
            let open = self.open_tags.pop().unwrap();

            if self.src.view_span(closing_name) == self.src.view_span(open) {
                candidate_exists = true;
                self.last_closed_tag = closing_name;

                // we push it back to the open_controls stack as the actual control
                // block to which the tag belongs to will consume it
                self.open_tags.push(open);
                break;
            }
        }

        if candidate_exists {
            // we have found a match. we can now report the unclosed control block,
            // pop it, and restore the cursor
            let open = self.open_controls.pop().unwrap();
            self.emit_unclosed_control(open, closing_name);
            self.restore(restore_pos);
        } else {
            // no match. actual stray.
            self.emit_stray_closing_tag();
            self.synchronize(&["\n", ">"]);
            self.eat(">");
        }
    }

    pub fn handle_stray_closing_control_block(&mut self) {
        // if when we encounter this stray closing control block and there is a possilbe
        // candidate in the open_controls stack that might consume it, then we
        // conclude that this is NOT a stray, and in fact belongs to the candidate.
        // this case occurs only when there is an unclosed tag as a child of
        // a control block
        // `
        // {#if a == b}
        //      <div>
        // {/if}
        // `
        // recovery will simply be to restore to where the closing starts and
        // continue parsing. this will allow for the parent to parse it correctly
        //
        // as for the unclosed tag, we can simply report it's status as unclosed,
        // and pop it from the open stack. recursion will take naturally handle
        // multiple such unclosed tags.
        //
        // the stary case is always guaranteed to occur before we get to the
        // mismatched control block end case. both the cases do the exact same
        // pop-the-stack and push-the-match algorithm. when processing the stray,
        // the error-free case is the desired block is found on the top of the stack.

        // cursor sits at '{' expecting to read '{/...}'
        let restore_pos = self.pos;

        self.eat("{/");

        self.consume_whitespace();

        let closing_name_start = self.pos;

        // we will synchronize to either '}' or a whitespace
        self.synchronize(&["}", " ", "\n", "\t"]);

        let closing_name = Span::new(closing_name_start, self.pos);

        // find a candidate in the open_controls stack
        let mut candidate_exists = false;

        while !self.open_controls.is_empty() {
            let open = self.open_controls.pop().unwrap();

            if self.src.view_span(closing_name) == self.src.view_span(open) {
                candidate_exists = true;
                self.last_closed_control = closing_name;

                // we push it back to the open_controls stack as the actual control
                // block to which the tag belongs to will consume it
                self.open_controls.push(open);
                break;
            }
        }

        if candidate_exists {
            // we have found a match. we can now report the unclosed tag, pop it,
            // and restore the cursor
            let open = self.open_tags.pop().unwrap();
            self.emit_unclosed_tag(open, closing_name);
            self.restore(restore_pos);
        } else {
            // no match. actual stray.
            self.emit_stray_closing_block();
            self.synchronize(&["\n", "}"]);
            self.eat("}");
        }
    }
}
