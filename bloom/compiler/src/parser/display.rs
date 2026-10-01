use crate::parser::{
    Parser,
    nodes::{AttributeValue, Fragment, FragmentNode, Part},
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
                        self.src.view_span(elem.name)
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
                                        self.src.view_span(attrib.name)
                                    )
                                }

                                AttributeValue::Parts(parts) => {
                                    print!(
                                        "{base_indent}   {}:\n",
                                        self.src.view_span(attrib.name)
                                    );

                                    for part in parts {
                                        match part {
                                            Part::Text(text) => {
                                                println!(
                                                    "{base_indent}      text: {:?}",
                                                    self.src.view_span(text)
                                                );
                                            }
                                            Part::Expression(expr) => {
                                                println!(
                                                    "{base_indent}      expr: {:?}",
                                                    self.src.view_span(expr)
                                                );
                                            }
                                        }
                                    }
                                }

                                AttributeValue::Expression(expr) => {
                                    print!(
                                        "{base_indent}   {}: {}\n",
                                        self.src.view_span(attrib.name),
                                        self.src.view_span(expr)
                                    );
                                }
                            }
                        }

                        println!("{base_indent}   ]");
                    }

                    if let Some(frag) = &elem.fragment {
                        self.display_fragment(frag, level + 2, '|');
                    }
                }

                FragmentNode::Text(text) => {
                    println!(
                        "{base_indent}text: {:?}",
                        self.src.view_span(text)
                    );
                }

                FragmentNode::Expression(expr) => {
                    println!(
                        "{base_indent}expr: {:?}",
                        self.src.view_span(expr)
                    );
                }

                // FragmentNode::Control(block) => {
                //     // println!(
                //     //     "{base_indent}{}: {:?}",
                //     //     self.src.view_span(block.control_type),
                //     //     self.src.view_span(block.expression),
                //     // );
                //     // self.display_fragment(&block.fragment, level + 1, '|');
                // }
                FragmentNode::If(if_block) => {
                    println!(
                        "{base_indent}if: {}",
                        self.src.view_span(if_block.condition)
                    );

                    self.display_fragment(&if_block.body, level + 1, '|');

                    for elif in &if_block.elifs {
                        println!(
                            "{base_indent}elif: {}",
                            self.src.view_span(elif.condition)
                        );

                        self.display_fragment(&elif.body, level + 1, '|');
                    }

                    if let Some(else_body) = &if_block.else_body {
                        println!("{base_indent}else");
                        self.display_fragment(&else_body, level + 1, '|');
                    }
                }

                FragmentNode::Each(each_block) => {
                    println!("{base_indent}each:",);
                    println!(
                        "{base_indent}|  expr: {}",
                        self.src.view_span(each_block.expression)
                    );
                    println!(
                        "{base_indent}|  context: {}",
                        self.src.view_span(each_block.context)
                    );

                    self.display_fragment(&each_block.body, level + 2, '|');
                }
            }
        }
    }

    pub fn display(&self) {
        let Some(fragment) = &self.root.fragment else {
            return;
        };

        self.display_fragment(fragment, 0, '|');
    }
}
