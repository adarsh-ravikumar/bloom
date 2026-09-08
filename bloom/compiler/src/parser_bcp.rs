use crate::{
    err::ParserError,
    io::IOFile,
    node::{ASTNode, Attribute, Tag},
    span::Span,
};

pub struct Parser {
    file: IOFile,
    cur_idx: usize,
}

impl Parser {
    pub fn new(file: IOFile) -> Self {
        Self { file, cur_idx: 0 }
    }

    // IDENT = LETTER (LETTER | DIGIT | UNDERSCORE | HYPHEN)* ;
    fn is_ident_start(ch: u8) -> bool {
        (ch >= b'a' && ch <= b'z') || (ch >= b'A' && ch <= b'Z')
    }

    fn is_ident_part(ch: u8) -> bool {
        (ch >= b'a' && ch <= b'z')
            || (ch >= b'A' && ch <= b'Z')
            || (ch >= b'0' && ch <= b'9')
            || (ch == b'_')
            || (ch == b'-')
    }

    fn advance(&mut self) -> Option<u8> {
        let next = self.file.get(self.cur_idx);

        self.cur_idx += 1;

        next
    }

    fn revert(&mut self) {
        self.cur_idx -= 1;
    }

    fn peek(&mut self) -> Option<u8> {
        self.file.get(self.cur_idx)
    }

    fn exhaust_whitespace(&mut self) {
        while let Some(cur) = self.advance() {
            if cur != b' ' {
                self.revert();
                break;
            }
        }
    }

    fn exhaust_ident(&mut self) -> ASTNode {
        let start = self.cur_idx;
        let mut end = self.cur_idx;

        while let Some(cur) = self.advance() {
            if !Self::is_ident_part(cur) {
                self.revert();
                // cur_idx left on character to read next.
                break;
            }

            end += 1;
        }

        return ASTNode::Ident(Span::new(start, end));
    }

    fn exhaust_attribute(&mut self) -> Result<ASTNode, ParserError> {
        let attrib_name = self.exhaust_ident();

        let Some(cur) = self.advance() else {
            // encountered EOF.
            // assume end of attrib and return. error elsewhere
            return Ok(ASTNode::Attribute(Attribute::new(attrib_name, None)));
        };

        if cur != b'=' {
            return Ok(ASTNode::Attribute(Attribute::new(attrib_name, None)));
        }

        let Some(cur) = self.advance() else {
            // unexpected eof
            // expected value
            return Err(ParserError::UnexpectedEOF);
        };

        if cur != b'"' && cur != b'\'' {
            // invalid value
            return Err(ParserError::InvalidValue);
        }

        let quote = cur;

        let start = self.cur_idx;
        let mut end = self.cur_idx;

        // exhaust value
        while let Some(cur) = self.advance() {
            if cur == quote {
                break;
            }

            end += 1;
        }

        Ok(ASTNode::Attribute(Attribute::new(
            attrib_name,
            Some(Span::new(start, end)),
        )))
    }

    pub fn parse(&mut self) -> Result<ASTNode, ParserError> {
        self.cur_idx = 0;

        while let Some(cur) = self.advance() {
            if cur == b'<' {
                let Some(cur) = self.peek() else {
                    return Err(ParserError::UnexpectedEOF);
                };

                // opening tag
                if Self::is_ident_start(cur) {
                    let tag_name = self.exhaust_ident();

                    // attributes
                    let mut attributes = Vec::new();

                    loop {
                        self.exhaust_whitespace();

                        let attribute = self.exhaust_attribute()?;
                        attributes.push(attribute);

                        let Some(cur) = self.peek() else {
                            return Err(ParserError::UnexpectedEOF);
                        };

                        // end of opening tag
                        if cur == b'>' {
                            break;
                        }
                    }

                    // body

                    return Ok(ASTNode::Tag(Tag::new(tag_name, attributes)));
                }
            }
        }

        unreachable!();
    }

    pub fn print_tree(&self, tree: ASTNode, level: usize) {
        let base_indent = "\t".repeat(level);

        match tree {
            ASTNode::Tag(tag) => {
                let ASTNode::Ident(tag_name) = tag.tag_name.as_ref() else {
                    unreachable!()
                };

                print!(
                    "{base_indent}{} [",
                    self.file.view_span(tag_name.clone())
                );

                for attrib in tag.attributes {
                    let ASTNode::Attribute(attrib) = attrib else {
                        unreachable!()
                    };

                    let ASTNode::Ident(attrib_name) =
                        attrib.attrib_name.as_ref()
                    else {
                        unreachable!()
                    };

                    print!(" {}", self.file.view_span(attrib_name.clone()));

                    if let Some(value) = attrib.value {
                        print!("=\"{}\"", self.file.view_span(value));
                    }
                }

                println!(" ]");
            }

            _ => (),
        }
    }
}
