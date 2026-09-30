use crate::{
    err::ParserError,
    io::IOFile,
    node::{
        Attribute, AttributeValue, ControlBlock, Element, Fragment,
        FragmentNode, Root,
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
        self.consume_by(by);
        next
    }

    fn consume(&mut self) {
        self.consume_by(1);
    }

    fn consume_by(&mut self, by: usize) {
        self.position += by;
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

    fn is_letter(cur: u8) -> bool {
        cur.is_ascii_alphabetic()
    }

    fn is_identifier_part(cur: u8) -> bool {
        cur.is_ascii_alphanumeric() || cur == b'_' || cur == b'-'
    }

    fn parse_identifier(&mut self) -> Result<Span, ParserError> {
        // IDENT = LETTER (LETTER | DIGIT | HYPEHN | UNDERSCORE)*

        let start = self.position;

        // LETTER
        let Some(cur) = self.advance() else {
            return Err(ParserError::UnexpectedEOF);
        };

        if !Self::is_letter(cur) {
            return Err(ParserError::InvalidIdentifier);
        }

        let mut end = self.position;

        // (LETTER | DIGIT | HYPHEN | UNDERSCORE)*
        while let Some(cur) = self.advance()
            && Self::is_identifier_part(cur)
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

        self.consume();

        self.exhaust_whitespace();

        let Some(cur) = self.peek() else {
            return Err(ParserError::UnexpectedEOF);
        };

        let parts;

        if cur == b'"' || cur == b'\'' {
            self.consume();
            parts = self.parse_parts(&[cur])?;
        } else {
            parts = self.parse_parts(&[b' ', b'\n', b'>', b'/'])?;
            self.revert();
        }

        Ok(Attribute {
            name,
            value: AttributeValue::Parts(parts),
        })
    }

    fn parse_control_block(&mut self) -> Result<FragmentNode, ParserError> {
        let control_type = self.parse_identifier()?;
        self.exhaust_whitespace();

        let mut inner_braces = 0;

        let start = self.position;
        let mut end = self.position;

        while let Some(cur) = self.peek() {
            if cur == b'{' {
                inner_braces += 1;
            }

            if cur == b'}' {
                if inner_braces <= 0 {
                    self.consume();
                    break;
                } else {
                    inner_braces -= 1;
                }
            }

            end += 1;
            self.consume();
        }

        let fragment = self.parse_fragment()?;
        self.exhaust_whitespace();

        self.expect_char(b'{')?;

        self.consume();

        self.expect_char(b'/')?;

        self.consume();

        let closing_name = self.parse_identifier()?;

        if self.file.view_span(closing_name)
            != self.file.view_span(control_type)
        {
            return Err(ParserError::InvalidTagClose);
        }

        self.expect_char(b'}')?;

        Ok(FragmentNode::ControlBlock(ControlBlock {
            control_type,
            fragment,
            expression: Span::new(start, end),
        }))
    }

    fn parse_parts(&mut self, delim: &[u8]) -> Result<Fragment, ParserError> {
        let mut nodes = Vec::new();

        let mut start = self.position;
        let mut end = self.position;
        let mut is_building_expr = false;

        let mut inner_braces = 0;

        while let Some(cur) = self.peek() {
            if delim.contains(&cur) {
                self.consume();
                break;
            }

            if cur == b'{' {
                if is_building_expr {
                    inner_braces += 1;
                } else {
                    if start < end {
                        nodes.push(FragmentNode::Text(Span::new(start, end)));
                    }

                    let Some(next_char) = self.peek_by(1) else {
                        break;
                    };

                    if next_char == b'#' {
                        // consume '{' and '#'
                        let _ = self.advance_by(2);
                        nodes.push(self.parse_control_block()?);
                    }

                    start = self.position + 1;
                    is_building_expr = true;
                }

                match self.peek_by(1) {
                    Some(b'#') => nodes.push(self.parse_control_block()?),
                    Some(b'/') => self.revert(),
                    _ => continue,
                }

                break;
            }

            if cur == b'}' {
                if inner_braces <= 0 {
                    nodes.push(FragmentNode::Expression(Span::new(start, end)));
                    start = self.position + 1;
                    is_building_expr = false;
                } else {
                    inner_braces -= 1;
                }
            }

            end += 1;
            self.consume();
        }

        if start < end {
            nodes.push(FragmentNode::Text(Span::new(start, end)));
        }

        Ok(Fragment { nodes })
    }

    fn parse_element(&mut self) -> Result<Element, ParserError> {
        self.expect_char(b'<')?;
        self.consume();

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

            attributes.push(self.parse_attribute()?);

            self.exhaust_whitespace();
        }

        let is_self_closing = self.expect_char(b'/').is_ok();

        if is_self_closing {
            self.consume();
            self.expect_char(b'>')?;
            self.consume();

            return Ok(Element {
                name,
                attributes,
                fragment: None,
            });
        }

        self.expect_char(b'>')?;
        self.consume();

        let fragment = self.parse_fragment()?;

        // tag close
        self.expect_char(b'<')?;
        self.consume();
        self.expect_char(b'/')?;
        self.consume();

        self.exhaust_whitespace();

        let closing_name = self.parse_identifier()?;

        if self.file.view_span(closing_name) != self.file.view_span(name) {
            return Err(ParserError::InvalidTagClose);
        }

        self.exhaust_whitespace();

        self.expect_char(b'>')?;
        self.consume();

        Ok(Element {
            name,
            attributes,
            fragment: Some(fragment),
        })
    }

    fn parse_fragment(&mut self) -> Result<Fragment, ParserError> {
        let mut nodes = Vec::new();

        while let Some(_) = self.peek() {
            let parts = self.parse_parts(&[b'<'])?;
            nodes.extend(parts.nodes);
            self.revert();

            self.exhaust_whitespace();

            // EOF
            if self.peek().is_none() {
                break;
            }

            // Tag close
            if self.peek_by(1) == Some(b'/') {
                break;
            }

            nodes.push(FragmentNode::Element(self.parse_element()?));
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

    fn display_fragment(&self, frag: &Fragment, level: usize, ch: char) {
        let base_indent = format!("{ch}  ").repeat(level);

        for node in &frag.nodes {
            match node {
                FragmentNode::Element(elem) => {
                    println!("{base_indent}tag");
                    print!(
                        "{base_indent}{ch}  name: {}\n{base_indent}{ch}  attribs: [",
                        self.file.view_span(elem.name)
                    );

                    if elem.attributes.is_empty() {
                        println!("]");
                    } else {
                        println!("");
                        for attrib in &elem.attributes {
                            match &attrib.value {
                                AttributeValue::True => {
                                    print!(
                                        "{base_indent}   {},\n",
                                        self.file.view_span(attrib.name)
                                    )
                                }

                                AttributeValue::Parts(parts) => {
                                    print!(
                                        "{base_indent}   {}:\n",
                                        self.file.view_span(attrib.name)
                                    );
                                    self.display_fragment(
                                        parts,
                                        level + 3,
                                        ' ',
                                    );
                                }
                            }
                        }

                        println!("{base_indent}]");
                    }

                    if let Some(frag) = &elem.fragment {
                        self.display_fragment(frag, level + 2, '|');
                    }
                }

                FragmentNode::Text(text) => {
                    println!(
                        "{base_indent}text: {:?}",
                        self.file.view_span(text)
                    );
                }

                FragmentNode::Expression(expr) => {
                    println!(
                        "{base_indent}expr: {:?}",
                        self.file.view_span(expr)
                    );
                }

                FragmentNode::ControlBlock(block) => {
                    println!(
                        "{base_indent}{}: {:?}",
                        self.file.view_span(block.control_type),
                        self.file.view_span(block.expression),
                    );
                    self.display_fragment(&block.fragment, level + 1, '|');
                }
            }
        }
    }

    pub fn display(&self) {
        let Some(root) = &self.root else {
            return;
        };

        let Some(fragment) = &root.fragment else {
            return;
        };

        self.display_fragment(fragment, 0, '|');
    }
}
