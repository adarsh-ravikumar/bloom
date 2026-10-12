use crate::{
    module::Module,
    parser::{
        Widget,
        nodes::{AttributeValue, Control, Fragment, FragmentNode, Part},
    },
};

impl Module {
    fn indent(level: usize, ch: char) -> String {
        format!("{ch}  ").repeat(level)
    }

    fn display_part(&self, part: &Part, indent: &str) {
        match part {
            Part::Text(text) => {
                println!("{indent}text: {:?}", text);
            }
            Part::Expression(expr) => {
                println!("{indent}expr: {:?}", expr.string);
            }
        }
    }

    fn display_parts(&self, parts: &[Part], indent: &str) {
        for part in parts {
            self.display_part(part, indent);
        }
    }

    fn display_attribute_value(&self, value: &AttributeValue, indent: &str) {
        match value {
            AttributeValue::True => {}
            AttributeValue::Expression(expr) => {
                println!("{:?}", expr.string);
            }
            AttributeValue::Parts(parts) => {
                self.display_parts(parts, indent);
            }
            AttributeValue::BindingAttribute { .. } => {}
        }
    }

    fn display_attributes(&self, elem: &Widget, indent: &str) {
        print!("{indent}attribs: [");

        if elem.attributes.is_empty() {
            println!("]");
            return;
        }

        println!();

        for attrib in &elem.attributes {
            let name = &attrib.name.string;

            match &attrib.value {
                AttributeValue::True => {
                    println!("{indent}   {name},");
                }

                AttributeValue::Expression(expr) => {
                    println!("{indent}   {name}: {}", expr.string);
                }

                AttributeValue::Parts(parts) => {
                    println!("{indent}   {name}:");
                    self.display_parts(parts, &format!("{indent}      "));
                }

                AttributeValue::BindingAttribute {
                    value,
                    bind_from,
                    bind_to,
                } => {
                    print!(
                        "{indent}   {} -> {}",
                        bind_from.string, bind_to.string
                    );

                    if let Some(value) = value {
                        match &**value {
                            AttributeValue::Expression(expr) => {
                                println!(" = {}", expr.string);
                            }
                            AttributeValue::Parts(parts) => {
                                println!(" =");
                                self.display_parts(
                                    parts,
                                    &format!("{indent}      "),
                                );
                            }
                            _ => println!(),
                        }
                    } else {
                        println!();
                    }
                }
            }
        }

        println!("{indent}]");
    }

    fn display_fragment(&self, frag: &Fragment, level: usize, ch: char) {
        let base_indent = Self::indent(level, ch);
        let child_indent = format!("{base_indent}{ch}  ");

        for node in &frag.nodes {
            match node {
                FragmentNode::Widget(elem) => {
                    println!("{base_indent}tag");
                    println!("{child_indent}name: {}", elem.name.string);

                    self.display_attributes(elem, &child_indent);

                    if let Some(fragment) = &elem.fragment {
                        println!("{child_indent}fragment:");
                        self.display_fragment(fragment, level + 2, '|');
                    }
                }

                FragmentNode::Text(text) => {
                    println!("{base_indent}text: {:?}", text);
                }

                FragmentNode::Expression(expr) => {
                    println!("{base_indent}expr: {:?}", expr.string);
                }

                FragmentNode::Parts(parts) => {
                    println!("{base_indent}parts:");
                    self.display_parts(parts, &child_indent);
                }

                FragmentNode::Control(control) => match control {
                    Control::If(if_block) => {
                        println!(
                            "{base_indent}if: {:?}",
                            if_block.condition.string
                        );

                        self.display_fragment(&if_block.body, level + 1, '|');

                        for elif in &if_block.elifs {
                            println!(
                                "{base_indent}elif: {:?}",
                                elif.condition.string
                            );

                            self.display_fragment(&elif.body, level + 1, '|');
                        }

                        if let Some(else_body) = &if_block.else_body {
                            println!("{base_indent}else");
                            self.display_fragment(else_body, level + 1, '|');
                        }
                    }

                    Control::Each(each_block) => {
                        println!("{base_indent}each:");
                        println!(
                            "{child_indent}expr: {:?}",
                            each_block.expression.string
                        );
                        println!(
                            "{child_indent}context: {:?}",
                            each_block.context.string
                        );

                        self.display_fragment(&each_block.body, level + 2, '|');
                    }
                },
            }
        }
    }

    pub fn display(&self) {
        println!("AST");
        if let Some(root) = &self.root {
            self.display_fragment(root, 0, '|');
        }

        println!("\nMetadata");
        for (name, mem) in &self.metadata {
            println!("{}: {:?}", name, mem);
        }

        println!("\nDependencies");
        for (name, occ) in &self.dependencies {
            println!("{} [{}]", name, occ);
        }
    }
}
