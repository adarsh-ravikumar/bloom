use crate::{
    module::{ExpressionId, ModuleRegistry},
    parser::{
        Attribute, AttributeValue, Control, EachControl, Fragment,
        FragmentNode, IfControl, Part, Parts, Widget,
    },
};

#[derive(Debug, Clone)]
pub struct TemplateIrGen<'a> {
    pub ir_nodes: Vec<IrNode>,

    ast: &'a Fragment,
    next_node_id: usize,
}

type NodeId = usize;

#[rustfmt::skip]
#[derive(Debug, Clone)]
pub enum IrNode {
    StartParts { id: NodeId },
    Text { value: String, id: NodeId, parent: NodeId },
    Expression { id: NodeId, expr_id: ExpressionId, parent: NodeId },
    EndParts { id: NodeId, parent: NodeId },

    StartWidget { name: String, id: NodeId, is_platform: bool },
    EndWidget { id: NodeId, parent: NodeId },

    StartAttributeList,
    AttributeName { name: String, id: NodeId },
    AttributeBindEvent { event: String, has_value: bool, id: NodeId },
    AttributeBindState { from: String, to: String, id: NodeId },
    AttributeTrue,
    EndAttributeList,

    StartIfBlock { test: ExpressionId, id: NodeId},
    EndIfBlock { id: NodeId, parent: NodeId },

    StartCaseBlock { test: Option<ExpressionId>, id: NodeId},
    EndCaseBlock { id: NodeId, parent: NodeId },

    StartEachBlock { iterable: ExpressionId, id: NodeId},
    EndEachBlock { id: NodeId, parent: NodeId },
}

fn sanitize_text(text: String) -> String {
    let mut san = Vec::new();

    for char in &text.into_bytes() {
        if *char == b'\'' {
            san.push(b'\\');
            san.push(b'\'');
        } else if *char == b'"' {
            san.push(b'\\');
            san.push(b'"');
        } else if *char == b'\'' {
            san.push(b'\\');
            san.push(b'`');
        } else {
            san.push(*char);
        }
    }

    String::from_utf8(san).unwrap()
}

impl<'a> TemplateIrGen<'a> {
    pub fn new(ast: &'a Fragment) -> Self {
        Self {
            next_node_id: 1, // 0 represents global scope
            ir_nodes: Vec::new(),
            ast,
        }
    }

    fn get_node_id(&mut self) -> NodeId {
        let id = self.next_node_id;
        self.next_node_id += 1;
        id
    }

    fn emit_node(&mut self, node: IrNode) {
        self.ir_nodes.push(node)
    }

    fn visit_text(&mut self, value: String, parent: NodeId) {
        let id = self.get_node_id();

        self.emit_node(IrNode::Text {
            id,
            value: sanitize_text(value),
            parent,
        });
    }

    fn visit_expression(&mut self, expr_id: ExpressionId, parent: NodeId) {
        let id = self.get_node_id();

        self.emit_node(IrNode::Expression {
            expr_id,
            id,
            parent,
        });
    }

    fn visit_parts(&mut self, parts: &Parts, parent: NodeId) {
        let id = self.get_node_id();

        self.emit_node(IrNode::StartParts { id });

        for part in parts {
            match part {
                Part::Text(string) => self.visit_text(string.clone(), id),
                Part::Expression(expr_id) => {
                    self.visit_expression(*expr_id, id);
                }
            }
        }

        self.emit_node(IrNode::EndParts { id, parent });
    }

    fn visit_attribute_value(
        &mut self,
        value: &AttributeValue,
        parent: NodeId,
    ) {
        match value {
            AttributeValue::True => self.ir_nodes.push(IrNode::AttributeTrue),
            AttributeValue::Parts(parts) => self.visit_parts(parts, parent),

            AttributeValue::Expression(expr) => {
                self.visit_expression(*expr, parent);
            }
        }
    }

    fn visit_widget(&mut self, widget: &Widget, parent: NodeId) {
        let id = self.get_node_id();

        self.emit_node(IrNode::StartWidget {
            name: widget.name.string.clone(),
            id,
            is_platform: widget.is_platform,
        });

        self.emit_node(IrNode::StartAttributeList);

        // TODO: attribute list
        for attrib in &widget.attributes {
            match attrib {
                Attribute::Regular(attrib) => {
                    self.ir_nodes.push(IrNode::AttributeName {
                        name: attrib.name.string.clone(),
                        id,
                    });

                    self.visit_attribute_value(&attrib.value, id);
                }

                Attribute::Binding(attrib) => {
                    if attrib.bind_from.string == "on" {
                        self.ir_nodes.push(IrNode::AttributeBindEvent {
                            event: attrib.bind_to.string.clone(),
                            has_value: attrib.value.is_some(),
                            id,
                        })
                    } else {
                        self.ir_nodes.push(IrNode::AttributeBindState {
                            from: attrib.bind_from.string.clone(),
                            to: attrib.bind_to.string.clone(),
                            id,
                        })
                    }

                    if let Some(value) = &attrib.value {
                        self.visit_attribute_value(value, id);
                    }
                }
            }
        }

        self.emit_node(IrNode::EndAttributeList);

        if let Some(frag) = &widget.fragment {
            self.visit_fragment(frag, id);
        }

        self.emit_node(IrNode::EndWidget { id, parent });
    }

    fn visit_control(&mut self, control: &Control, parent: NodeId) {
        match control {
            Control::If(if_ctrl) => self.visit_if(if_ctrl, parent),
            Control::Each(each_ctrl) => self.visit_each(each_ctrl, parent),
        }
    }

    fn visit_if(&mut self, if_ctrl: &IfControl, parent: NodeId) {
        let if_id = self.get_node_id();

        self.emit_node(IrNode::StartIfBlock {
            test: if_ctrl.condition,
            id: if_id,
        });

        self.visit_fragment(&if_ctrl.body, if_id);

        for case in &if_ctrl.elifs {
            let case_id = self.get_node_id();

            self.emit_node(IrNode::StartCaseBlock {
                test: Some(case.condition),
                id: case_id,
            });

            self.visit_fragment(&case.body, case_id);

            self.emit_node(IrNode::EndCaseBlock {
                id: case_id,
                parent: if_id,
            });
        }

        if let Some(else_ctrl) = &if_ctrl.else_body {
            let else_id = self.get_node_id();

            self.emit_node(IrNode::StartCaseBlock {
                test: None,
                id: else_id,
            });

            self.visit_fragment(else_ctrl, else_id);

            self.emit_node(IrNode::EndCaseBlock {
                id: else_id,
                parent: if_id,
            });
        }

        self.emit_node(IrNode::EndIfBlock { id: if_id, parent });
    }

    fn visit_each(&mut self, each_ctrl: &EachControl, parent: NodeId) {
        let each_id = self.get_node_id();

        // TODO: handle visit expression parent here
        self.emit_node(IrNode::StartEachBlock {
            iterable: each_ctrl.expression,
            id: each_id,
        });

        self.visit_fragment(&each_ctrl.body, each_id);

        self.emit_node(IrNode::EndEachBlock {
            id: each_id,
            parent,
        });
    }

    fn visit_fragment(&mut self, frag: &Fragment, parent: NodeId) {
        for node in &frag.nodes {
            match node {
                FragmentNode::Text(value) => {
                    self.visit_text(value.clone(), parent)
                }
                FragmentNode::Expression(expr) => {
                    self.visit_expression(*expr, parent);
                }
                FragmentNode::Widget(widget) => {
                    self.visit_widget(widget, parent)
                }
                FragmentNode::Control(control) => {
                    self.visit_control(control, parent)
                }
                FragmentNode::Parts(parts) => {
                    self.visit_parts(parts, parent);
                }
            }
        }
    }

    pub fn generate_ir(&mut self) {
        self.visit_fragment(self.ast, 0)
    }
}
