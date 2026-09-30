use crate::{
    diagnostic::Diagnostic,
    parser::{
        Parser,
        node::{Fragment, FragmentNode},
    },
};

impl<'a> Parser<'a> {
    pub fn parse_fragment(&mut self) -> Result<Fragment, Diagnostic> {
        println!("Parse Fragment");

        let mut nodes = Vec::new();

        while let Some(_) = self.peek() {
            let parts = self.parse_parts(&[b'<'])?;
            nodes.extend(parts.nodes);

            self.exhaust_whitespace();

            // EOF
            // NOT tag
            if self.peek().is_none() || self.peek() != Some(b'<') {
                break;
            }

            // Tag close
            if self.peek_by(1) == Some(b'/') {
                break;
            }

            if let Some(elem) = self.parse_element()? {
                nodes.push(FragmentNode::Element(elem));
            }
        }

        Ok(Fragment { nodes })
    }
}
