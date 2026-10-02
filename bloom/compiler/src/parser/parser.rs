use crate::{
    common::{ERRONEOUS_SPAN, Source, Span},
    diagnostic::Diagnostic,
    parser::nodes::Root,
};

pub struct Parser<'a> {
    // position will always point to the closest unread byte
    pub pos: usize,
    pub src: &'a Source,
    pub root: Root,
    pub open_tags: Vec<Span>,
    pub open_controls: Vec<Span>,
    pub last_closed_tag: Span,
    pub last_closed_control: Span,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum FragmentEnd {
    Root,
    Element,
    ControlEnd,
    ControlCase,
}

impl<'a> Parser<'a> {
    pub fn new(src: &'a Source) -> Self {
        Self {
            pos: 0,
            src,
            root: Root { fragment: None },
            open_tags: Vec::new(),
            open_controls: Vec::new(),
            last_closed_tag: ERRONEOUS_SPAN,
            last_closed_control: ERRONEOUS_SPAN,
            diagnostics: Vec::new(),
        }
    }

    pub fn parse(&mut self) {
        let frag = self.parse_fragment(&[FragmentEnd::Root]);

        self.root.fragment = Some(frag);
    }

    pub fn parse_text(&mut self, delim: &[u8]) -> Option<Span> {
        let start = self.pos;

        while !delim.contains(&self.peek(0)) && self.peek(0) != 0 {
            self.consume(1);
        }

        let is_empty = self.src.view(start, self.pos).trim().is_empty();

        if !is_empty {
            Some(Span::new(start, self.pos))
        } else {
            None
        }
    }

    pub fn parse_comment(&mut self) {
        let comment_start = Span::new(self.pos - 4, self.pos); // <!--
        let mut nested_comments = 0usize;

        loop {
            if self.eat("<!--") {
                nested_comments += 1;
            }

            if self.eat("-->") {
                if nested_comments == 0 {
                    self.consume_whitespace();
                    return;
                } else {
                    nested_comments -= 1;
                }
            }

            if self.peek(0) == 0 {
                // comment was never closed
                self.emit_unclosed_comment(comment_start);
            }

            self.consume(1);
        }
    }
}
