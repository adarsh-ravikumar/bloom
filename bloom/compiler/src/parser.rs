use crate::{
    err::ParserError,
    io::IOFile,
    node::{
        Attribute, AttributeValue, Fragment, FragmentNode, RegularElement,
        Root, Text,
    },
    span::Span,
};

pub struct Parser {
    pub file: IOFile,
    pub root: Option<Root>,
    position: usize,
}

impl Parser {
    pub fn new(file: IOFile) -> Self {
        Self {
            file,
            position: 0,
            root: None,
        }
    }

    fn advance(&mut self) -> Option<u8> {
        self.advance_by(1)
    }

    fn advance_by(&mut self, by: usize) -> Option<u8> {
        let next = self.peek();
        self.position += by;
        next
    }

    fn revert(&mut self) {
        self.position -= 1;
    }

    fn peek(&self) -> Option<u8> {
        self.peek_by(0)
    }

    fn peek_by(&self, by: usize) -> Option<u8> {
        self.file.get(self.position + by)
    }

    fn exhaust_whitespace(&mut self) {
        while let Some(cur) = self.advance()
            && (cur == b' ' || cur == b'\n' || cur == b'\t')
        {}

        self.revert()
    }

    fn expect_char(&mut self, ch: u8) -> Result<(), ParserError> {
        if let Some(cur) = self.peek()
            && cur == ch
        {
            return Ok(());
        }

        Err(ParserError::UnexpectedCharacter)
    }

    fn parse_identifier(&mut self) -> Result<Span, ParserError> {
        // IDENT = LETTER (LETTER | DIGIT | HYPEHN | UNDERSCORE)*

        let start = self.position;

        // LETTER
        if let Some(cur) = self.advance() {
            if !((cur >= b'a' && cur <= b'z') || (cur >= b'A' && cur <= b'Z')) {
                return Err(ParserError::InvalidIdentifier);
            }
        }

        let mut end = self.position;

        // (LETTER | DIGIT | HYPHEN | UNDERSCORE)*
        while let Some(cur) = self.advance()
            && ((cur >= b'a' && cur <= b'z')
                || (cur >= b'A' && cur <= b'Z')
                || (cur >= b'0' && cur <= b'9')
                || (cur == b'_')
                || (cur == b'-'))
        {
            end += 1
        }

        self.revert();

        Ok(Span::new(start, end))
    }

    fn parse_attribute(&mut self) -> Result<Attribute, ParserError> {
        // ATTRIBUTE = IDENT (SPACE? EQ SPACE? VALUE)?
        // VALUE =  QUOTE (TEXT | SPACE)* QUOTE
        //         | TEXT*
        let name = self.parse_identifier()?;

        self.exhaust_whitespace();

        let is_boolean = self.expect_char(b'=').is_err();

        if is_boolean {
            return Ok(Attribute {
                name,
                value: AttributeValue::True,
            });
        }

        let _ = self.advance();

        self.exhaust_whitespace();

        let value: Span;

        let Some(cur) = self.peek() else {
            return Err(ParserError::UnexpectedEOF);
        };

        let start;
        let mut end;

        if cur == b'"' || cur == b'\'' {
            let quote = cur;
            let _ = self.advance();

            start = self.position;
            end = self.position;

            while let Some(cur) = self.advance()
                && cur != quote
            {
                end += 1;
            }
        } else {
            start = self.position;
            end = self.position;

            while let Some(cur) = self.advance()
                && !(cur == b' ' || cur == b'\n' || cur == b'>' || cur == b'/')
            {
                end += 1;
            }
        }

        value = Span::new(start, end);

        Ok(Attribute {
            name,
            value: AttributeValue::Text(Text { data: value }),
        })
    }

    fn parse_element(&mut self) -> Result<RegularElement, ParserError> {
        self.expect_char(b'<')?;
        let _ = self.advance();

        let name = self.parse_identifier()?;

        let mut attributes = Vec::new();

        self.exhaust_whitespace();

        loop {
            let Some(cur) = self.peek() else {
                return Err(ParserError::UnexpectedEOF);
            };

            if cur == b'/' || cur == b'>' {
                break;
            }

            // parse attributes
            attributes.push(self.parse_attribute()?);

            self.exhaust_whitespace();
        }

        let is_self_closing = self.expect_char(b'/').is_ok();

        if is_self_closing {
            let _ = self.advance();
            self.expect_char(b'>')?;
            let _ = self.advance();

            return Ok(RegularElement {
                name,
                attributes,
                fragment: None,
            });
        }

        self.expect_char(b'>')?;
        let _ = self.advance();

        // parse fragment
        let fragment = self.parse_fragment()?;

        // tag close
        self.expect_char(b'<')?;
        let _ = self.advance();
        self.expect_char(b'/')?;
        let _ = self.advance();

        self.exhaust_whitespace();

        let closing_name = self.parse_identifier()?;

        if self.file.view_span(closing_name) != self.file.view_span(name) {
            return Err(ParserError::InvalidTagClose);
        }

        self.exhaust_whitespace();

        self.expect_char(b'>')?;
        let _ = self.advance();

        Ok(RegularElement {
            name,
            attributes,
            fragment: Some(fragment),
        })
    }

    fn parse_fragment(&mut self) -> Result<Fragment, ParserError> {
        let mut nodes = Vec::new();

        let mut text_start = self.position;
        let mut text_end = self.position;
        let mut is_building_text = false;

        while let Some(cur) = self.peek() {
            if cur == b'<' {
                if is_building_text {
                    nodes.push(FragmentNode::Text(Text {
                        data: Span::new(text_start, text_end),
                    }));

                    is_building_text = false;
                }

                if self.peek_by(1) == Some(b'/') {
                    break;
                }

                nodes.push(FragmentNode::RegularElement(self.parse_element()?));
            } else {
                if is_building_text == false {
                    is_building_text = true;
                    text_start = self.position;
                    text_end = self.position;
                }

                text_end += 1;
                let _ = self.advance();
            }
        }

        Ok(Fragment { nodes })
    }

    pub fn parse(&mut self) -> Result<(), ParserError> {
        self.exhaust_whitespace();

        let fragment = self.parse_fragment()?;

        self.root = Some(Root {
            fragment: Some(fragment),
        });

        Ok(())
    }

    fn display_fragment(&mut self, frag: &Fragment, level: usize) {
        let base_indent = "|  ".repeat(level);

        for node in &frag.nodes {
            match node {
                FragmentNode::RegularElement(elem) => {
                    print!(
                        "{base_indent}name: {} | attribs: [ ",
                        self.file.view_span(elem.name)
                    );

                    for attrib in &elem.attributes {
                        let val = match &attrib.value {
                            AttributeValue::Text(text) => {
                                format!(
                                    "\"{}\"",
                                    self.file.view_span(text.data)
                                )
                            }
                            AttributeValue::True => "true".into(),
                        };

                        print!("{}={} ", self.file.view_span(attrib.name), val)
                    }

                    println!("]");

                    if let Some(frag) = &elem.fragment {
                        self.display_fragment(frag, level + 1);
                    }
                }

                FragmentNode::Text(elem) => {
                    println!(
                        "{base_indent}text: {:?}",
                        self.file.view_span(elem.data)
                    );
                }
            }
        }
    }

    pub fn display(&mut self) {
        let Some(root) = self.root.clone() else {
            return;
        };

        let Some(fragment) = &root.fragment else {
            return;
        };

        self.display_fragment(fragment, 0);
    }
}
