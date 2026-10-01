use crate::{
    common::{ERRONEOUS_SPAN, Source, Span},
    diagnostic::Diagnostic,
    parser::nodes::{
        Attribute, AttributeValue, EachControl, Element, ElifControl, Fragment,
        FragmentNode, IfControl, Part, Root,
    },
};

pub struct Parser<'a> {
    // position will always point to the closest unread byte
    pub pos: usize,
    pub src: &'a Source,
    pub root: Root,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, PartialEq, Eq)]
enum FragmentEnd {
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
            diagnostics: Vec::new(),
        }
    }

    pub fn consume(&mut self, by: usize) {
        self.pos += by;
    }

    pub fn peek(&self, by: usize) -> u8 {
        self.src.get(self.pos + by)
    }

    pub fn expect(&self, ch: u8) -> bool {
        self.peek(0) == ch
    }

    pub fn eat(&mut self, seq: &'a str) -> bool {
        let seq = seq.as_bytes();

        for (i, &ch) in seq.iter().enumerate() {
            if self.peek(i) != ch {
                return false;
            }
        }

        self.consume(seq.len());
        true
    }

    pub fn synchronize(&mut self, set: &[u8]) {
        loop {
            if set.contains(&self.peek(0)) {
                break;
            }

            self.consume(1);
        }
    }

    pub fn skip_whitespace(&mut self) {
        while matches!(self.peek(0), b' ' | b'\t' | b'\n') {
            self.consume(1);
        }
    }

    pub fn parse(&mut self) {
        let frag = self.parse_fragment(&[FragmentEnd::Root]);

        self.root.fragment = Some(frag);
    }

    fn parse_text(&mut self, delim: &[u8]) -> Option<Span> {
        // println!("Parse Text");
        let start = self.pos;

        while !delim.contains(&self.peek(0)) && self.peek(0) != 0 {
            // // println!("{}", self.peek(0) as char);
            self.consume(1);
        }

        let is_empty = self.src.view(start, self.pos).trim().is_empty();

        // // println!("done parsing text");
        if !is_empty {
            Some(Span::new(start, self.pos))
        } else {
            None
        }
    }

    fn parse_embedded_expression(&mut self, delim: u8) -> Span {
        // println!("Parse Expression");
        // goal of the method is to correctly find the end of a JS expression
        // expects that current pointer is at the start of the expr

        let mut stack = Vec::new() as Vec<u8>;
        let mut bounds = Vec::new() as Vec<usize>;

        let start = self.pos;

        let mut quote: Option<u8> = None;

        let mut is_template = false;

        loop {
            let cur = self.peek(0);

            if is_template {
                if &stack.len() == bounds.last().unwrap_or(&0usize) {
                    bounds.pop();
                }

                if bounds.is_empty() && self.eat("}") {
                    is_template = false;
                    continue;
                }
            }

            if cur == delim && stack.is_empty() && quote.is_none() {
                return Span::new(start, self.pos);
            }

            if cur == 0 {
                // handle unclosed string, and brackets unmatched / unclosed
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
                    self.consume(1);
                    continue;
                }

                if quote == Some(cur) {
                    quote = None;
                    self.consume(1);
                    continue;
                }
            }

            if quote.is_some() {
                if cur == b'\\' {
                    self.consume(1); // skip whatever character comes after the escape
                    // this is naive, but it handles \', \" and \`, which is more than sufficient
                }
                self.consume(1);
                continue;
            }

            if quote == Some(b'`') && self.eat("${") {
                is_template = true;
                bounds.push(stack.len());
                continue;
            }

            if matches!(cur, b'(' | b'{' | b'[') {
                // println!("encountered {}, {:?}", cur as char, stack);
                stack.push(cur);
            }

            if matches!(cur, b')' | b'}' | b']') {
                // println!("encountered {}, {:?}", cur as char, stack);
                if stack.is_empty() {
                    // println!("end of expression: {}:{}", start, self.pos);
                    return Span::new(start, self.pos);
                }

                let open = stack.pop().unwrap();
                if !matches!(
                    (open, cur),
                    (b'(', b')') | (b'[', b']') | (b'{', b'}')
                ) {
                    // unmatched bracket pair detected
                }
            }

            self.consume(1);
        }
    }

    fn parse_ident(&mut self, delim: u8) -> Span {
        // println!("Parse Ident");
        let start = self.pos;

        if !matches!(self.peek(0), b'a'..b'z' | b'A'..b'Z') {
            // illegal identifier
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
                self.synchronize(&[b' ', b'\t', b'\n', delim]);
                return ERRONEOUS_SPAN;
            }

            self.consume(1);
        }

        Span::new(start, self.pos)
    }

    fn parse_open_tag(&mut self) -> (Element, bool) {
        // println!("Parse Open Tag");
        self.eat("<");

        let name = self.parse_ident(b'>');
        if name == ERRONEOUS_SPAN {
            // invalid element name error
        }

        self.skip_whitespace();

        // attribute list
        let attributes = self.parse_attributes();

        // self closing
        let self_closing = self.eat("/>");

        if !self.eat("/>") {
            self.eat(">");
        }

        (
            Element {
                name,
                attributes,
                fragment: None,
            },
            self_closing,
        )
    }

    fn parse_attribute_value(&mut self) -> Option<AttributeValue> {
        // println!("Parse Attribute Value");
        if self.eat("{") {
            let expr = self.parse_embedded_expression(b'}');
            if !self.eat("}") {
                // unclosed expression
            }
            return Some(AttributeValue::Expression(expr));
        }

        let quote = self.peek(0);
        if matches!(quote, b'\'' | b'"') {
            self.consume(1);

            // value must either be a quoted string or an expression
            let mut parts = Vec::new();

            while self.peek(0) != quote && self.peek(0) != 0 {
                let text = self.parse_text(&[quote, b'{']);
                if let Some(text) = text {
                    parts.push(Part::Text(text));
                }

                if !self.eat("{") {
                    break;
                }

                let expr = self.parse_embedded_expression(b'}');
                if !self.eat("}") {
                    // unclosed expression
                }

                parts.push(Part::Expression(expr));
            }

            self.consume(1); // consume end quote

            return Some(AttributeValue::Parts(parts));
        }

        return None;
    }

    fn parse_attributes(&mut self) -> Vec<Attribute> {
        // println!("Parse Attributes");
        let mut attributes = Vec::new();

        // name
        // name={}

        while !matches!(self.peek(0), b'>' | b'/') && self.peek(0) != 0 {
            // ident
            let name = self.parse_ident(b'=');
            if name == ERRONEOUS_SPAN {
                // invalid attribute name
            }

            self.skip_whitespace();

            // boolean
            if !self.eat("=") {
                attributes.push(Attribute {
                    name,
                    value: AttributeValue::True,
                });

                continue;
            }

            // value
            let value = self.parse_attribute_value();
            if let Some(value) = value {
                attributes.push(Attribute { name, value });
            } else {
                // invalid value
            }
        }

        attributes
    }

    fn parse_close_tag(&mut self, elem: &Element) {
        // println!("Parse Close Tag");

        self.eat("</");
        let name = self.parse_ident(b'>');
        if name == ERRONEOUS_SPAN {
            // invalid identifier
        }

        if self.src.view_span(name) != self.src.view_span(elem.name) {
            // mismatched tags
        }

        if !self.eat(">") {
            // expected '>' to close tag
        }
    }

    fn parse_element(&mut self) -> FragmentNode {
        // println!("Parse Element");

        let (mut elem, is_self_closing) = self.parse_open_tag();

        if is_self_closing {
            return FragmentNode::Element(elem);
        }

        let fragment = self.parse_fragment(&[FragmentEnd::Element]);

        elem.fragment = Some(fragment);

        self.parse_close_tag(&elem);

        FragmentNode::Element(elem)
    }

    fn parse_control(&mut self) -> FragmentNode {
        self.eat("{#");
        // the control block only cares about whitespace.
        // parse_ident handles whitespace internally.
        // b' ' is passed here to keep the API simple
        // rather than introducing an Option<u8>
        let name = self.parse_ident(b' ');
        if name == ERRONEOUS_SPAN {
            // invalid identifier
        }

        let control_type = self.src.view_span(name);

        match control_type {
            "if" => self.parse_control_if(),
            "each" => self.parse_control_each(),
            _ => {
                // unknown control block
                unreachable!()
            }
        }
    }

    fn parse_control_if(&mut self) -> FragmentNode {
        self.skip_whitespace();
        let expr = self.parse_embedded_expression(b'}');

        self.skip_whitespace();

        if !self.eat("}") {
            // expected '}'
        }

        let fragment = self.parse_fragment(&[
            FragmentEnd::ControlCase,
            FragmentEnd::ControlEnd,
        ]);

        // while we have {:, we have to parse elif, else
        let mut elifs = Vec::new();
        let mut else_body = None;

        loop {
            if self.eat("{/") {
                if !self.eat("if}") {
                    // invalid close tag
                }

                break;
            }

            let is_case = self.eat("{:");

            if is_case && else_body.is_some() {
                // expected {/if} after else block
            }

            if is_case {
                if self.eat("elif") {
                    self.skip_whitespace();
                    let expr = self.parse_embedded_expression(b'}');

                    self.skip_whitespace();

                    if !self.eat("}") {
                        // expected '}'
                    }

                    let fragment = self.parse_fragment(&[
                        FragmentEnd::ControlCase,
                        FragmentEnd::ControlEnd,
                    ]);

                    elifs.push(ElifControl {
                        condition: expr,
                        body: fragment,
                    });

                    continue;
                } else if self.eat("else") {
                    self.skip_whitespace();

                    if !self.eat("}") {
                        // expected '}'
                    }

                    let fragment =
                        self.parse_fragment(&[FragmentEnd::ControlEnd]);

                    else_body = Some(fragment);
                    continue;
                } else {
                    // invalid case
                }
            }
        }

        FragmentNode::If(IfControl {
            condition: expr,
            body: fragment,
            elifs,
            else_body,
        })
    }

    fn parse_control_each(&mut self) -> FragmentNode {
        self.skip_whitespace();

        let start = self.pos;
        let mut end;

        // keep parsing JS expressions until we hit a ' '.
        // we will inspect to see if we have ' as'
        // if we don't, then we just keep parsing until
        // we hit either ' as' -> succeeding case
        // OR '}' -> failing case, expected 'as'.
        loop {
            end = self.parse_embedded_expression(b' ').end;
            self.skip_whitespace();

            // whitespace is the token boundary for 'as'
            if self.eat("as ") || self.eat("as\t") || self.eat("as\n") {
                break;
            }

            if self.peek(0) == b'}' {
                // expected 'as'
            }
        }

        self.skip_whitespace();

        let context = self.parse_ident(b'}');

        if context == ERRONEOUS_SPAN {
            // invalid context name
        }

        self.skip_whitespace();

        if !self.eat("}") {
            // expected '}'
        }

        let body = self.parse_fragment(&[FragmentEnd::ControlEnd]);

        if !self.eat("{/each") {
            // expected {/each
        }

        self.skip_whitespace();

        if !self.eat("}") {
            // expected '}'
        }

        FragmentNode::Each(EachControl {
            expression: Span::new(start, end),
            context,
            body,
        })
    }

    fn parse_fragment(&mut self, expected_end: &[FragmentEnd]) -> Fragment {
        // println!("Parse Fragment");

        let mut nodes: Vec<FragmentNode> = Vec::new();

        loop {
            let cur = self.peek(0);

            match cur {
                0 => break,
                b'<' => {
                    if self.peek(1) == b'/'
                        && expected_end.contains(&FragmentEnd::Element)
                    {
                        // we have spotted an end tag.
                        // we must return whatever we have parsed as the fragment.
                        // the caller is expected to handle the end tag
                        // cursor points to the '<'
                        return Fragment { nodes };
                    } else {
                        // stray closing tag
                    }
                    // parse element
                    nodes.push(self.parse_element());
                }
                b'{' => {
                    if self.peek(1) == b'#' {
                        nodes.push(self.parse_control());
                    } else if self.peek(1) == b'/' {
                        if expected_end.contains(&FragmentEnd::ControlEnd) {
                            // we have spotted an end tag.
                            // we must return whatever we have parsed as the fragment.
                            // the caller is expected to handle the end of control block
                            // cursor points to the '{'
                            return Fragment { nodes };
                        } else {
                            // stray closing block
                        }
                    } else if self.peek(1) == b':' {
                        if expected_end.contains(&FragmentEnd::ControlCase) {
                            // we have spotted an end tag.
                            // we must return whatever we have parsed as the fragment.
                            // the caller is expected to handle the end of control block
                            // cursor points to the '{'
                            return Fragment { nodes };
                        } else {
                            // stary control case
                        }
                    } else {
                        self.consume(1);
                        nodes.push(FragmentNode::Expression(
                            self.parse_embedded_expression(b'}'),
                        ));
                        if !self.eat("}") {
                            // unclosed expression
                        }
                    }
                }
                _ => {
                    if let Some(text) = self.parse_text(&[b'<', b'{']) {
                        nodes.push(FragmentNode::Text(text))
                    }
                } // text
            }
        }

        Fragment { nodes }
    }
}
