use crate::{
    common::{ERRONEOUS_SPAN, Span},
    parser::{Parser, Preprocess, Script},
};

const COMMAND: &'static str = "command";
const EVENT: &'static str = "event";
const REACTIVE: &'static str = "reactive";

impl<'a> Parser<'a> {
    pub fn parse_script_tag(&mut self, open: Span) {
        if let Some(script) = &self.root.script {
            self.emit_multiple_script_tags(script.open, open);
        }

        if let Some(&tag) = self.open_tags.last()
            && tag != open
        {
            self.emit_nested_script_tag(open, tag);
        }

        if let Some(&control) = self.open_controls.last() {
            self.emit_nested_script_tag(open, control);
        }

        let source = self.parse_embedded_expression("</");

        self.consume_whitespace();

        if self.peek(0) == 0 {
            self.emit_unclosed_tag_eof(open); // reach fatal eof
        }

        self.parse_close_tag(open);

        if source != ERRONEOUS_SPAN {
            self.preprocess_script(open, source)
        }
    }

    pub fn preprocess_script(&mut self, open: Span, source: Span) {
        // "#" ("command" | "event" | "reactive") (space | tab)* <newline>
        // "function" (space | tab | newline)*  <identifier> [<...> | (...)]
        //
        // we do NOT perform error recovery in the preprocessor, as the syntax itself is
        // insufficient to render clear boundaries for synchronization, and it is added
        // complexity for a process that will eitherway end up terminating the compiler
        // without moving on to the next stage.
        //
        // the implementation itself is quite "hacky" but is intentional.

        let mut pos = source.start;

        let mut quote = None;

        let mut commands = Vec::new();
        let mut events = Vec::new();
        let mut reactive = Vec::new();

        let mut cur = self.src.get(pos);

        let mut script = String::new();

        let mut script_consume_pointer = pos;

        // check for EOF = redundancy. this case should NEVER occur
        while cur != 0 && pos < source.end {
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

                pos += 1;
                cur = self.src.get(pos);
                continue;
            }

            if quote.is_some() && cur == b'\\' {
                pos += 2;
                // this is naive, but it handles \', \" and \`, which is more than sufficient
                cur = self.src.get(pos);
                continue;
            }

            if cur == b'#' && quote.is_none() {
                let directive_start = pos;

                if script_consume_pointer < directive_start {
                    script +=
                        self.src.view(script_consume_pointer, directive_start);
                }

                pos += 1;

                // skip whitespace
                while [b' ', b'\t', b'\n'].contains(&self.src.get(pos)) {
                    pos += 1
                }

                // now we try to consume till the next whitespace
                let start = pos;

                while !([b' ', b'\t', b'\n'].contains(&self.src.get(pos))) {
                    pos += 1
                }

                let end = pos;

                let directive = self.src.view(start, end);

                if !matches!(directive, COMMAND | EVENT | REACTIVE) {
                    self.emit_preprocessor_unknown_directive(
                        directive_start,
                        pos,
                    );
                    break;
                }

                script_consume_pointer = pos;

                // skip whitespace but NOT newline
                while [b' ', b'\t'].contains(&self.src.get(pos)) {
                    pos += 1
                }

                // expect newline
                if self.src.get(pos) != b'\n' {
                    self.emit_preprocessor_expected_newline(pos, pos);
                    break;
                }

                pos += 1;

                // try consume "function"
                let start = pos;

                while !([b' ', b'\t', b'\n'].contains(&self.src.get(pos))) {
                    pos += 1
                }

                let end = pos;

                if self.src.view(start, end) != "function" {
                    self.emit_preprocessor_expected_function(start, end);
                    break;
                }

                // skip whitespace
                while [b' ', b'\t', b'\n'].contains(&self.src.get(pos)) {
                    pos += 1
                }

                // try consume function name
                let name_start = pos;

                while !([b'(', b'<', b' ', b'\t', b'\n']
                    .contains(&self.src.get(pos)))
                {
                    pos += 1
                }

                let name_end = pos;

                let fn_name = self.src.view(name_start, name_end).to_string();

                if fn_name.trim().is_empty() {
                    self.emit_preprocessor_expected_function_name(
                        name_start, name_end,
                    );
                }

                match directive {
                    COMMAND => commands.push(fn_name),
                    EVENT => events.push(fn_name),
                    REACTIVE => reactive.push(fn_name),
                    _ => unreachable!(),
                }
            }

            pos += 1;
            cur = self.src.get(pos);
        }

        if script_consume_pointer < source.end {
            script += self.src.view(script_consume_pointer, source.end);
        }

        self.root.script = Some(Script {
            open,
            source: script,
            pre: Preprocess {
                commands,
                events,
                reactive,
            },
        });
    }
}
