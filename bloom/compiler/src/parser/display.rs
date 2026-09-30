use crate::parser::{
    Parser,
    node::{AttributeValue, Fragment, FragmentNode},
};

impl<'a> Parser<'a> {
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
