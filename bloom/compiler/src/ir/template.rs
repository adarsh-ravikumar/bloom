use std::mem::take;

use crate::{
    common::{Source, SpannedString},
    module::{Expression, ModuleRegistry},
    parser::{
        Attribute, AttributeValue, Control, EachControl, Fragment,
        FragmentNode, IfControl, Part, Parts, Widget,
    },
};

#[derive(Debug, Clone)]
pub struct TemplateIr<'a> {
    pub ir_nodes: Vec<IrNode>,
    pub expressions: Vec<Expression>,

    ast: &'a Fragment,
    next_node_id: usize,

    registry: &'a ModuleRegistry,
}

type NodeId = usize;
type ExpressionId = usize;

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
    AttributeBindState { from: String, to: String, id: NodeId},
    AttributeBindEvent { from: String, to: String, id: NodeId},
    AttributeTrue,
    EndAttributeList,

    StartIfBlock { test: Option<ExpressionId>, id: NodeId},
    EndIfBlock { id: NodeId, parent: NodeId },

    StartCaseBlock { test: Option<ExpressionId>, id: NodeId},
    EndCaseBlock { id: NodeId, parent: NodeId },

    StartEachBlock { iterable: Option<ExpressionId>, id: NodeId},
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

impl<'a> TemplateIr<'a> {
    pub fn new(ast: &'a Fragment, registry: &'a ModuleRegistry) -> Self {
        Self {
            next_node_id: 1, // 0 represents global scope
            ir_nodes: Vec::new(),
            expressions: Vec::new(),
            ast,
            registry,
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

    fn visit_expression(
        &mut self,
        expr: Expression,
        parent: NodeId,
    ) -> ExpressionId {
        let expr_id = self.expressions.len();
        let id = self.get_node_id();

        self.emit_node(IrNode::Expression {
            expr_id,
            id,
            parent,
        });

        self.expressions.push(expr);

        id
    }

    fn visit_parts(&mut self, parts: &mut Parts, parent: NodeId) {
        let id = self.get_node_id();

        self.emit_node(IrNode::StartParts { id });

        for part in parts {
            match part {
                Part::Text(string) => {
                    self.visit_text(std::mem::take(string), id)
                }
                Part::Expression(expr) => {
                    self.visit_expression(std::mem::take(expr), id);
                }
            }
        }

        self.emit_node(IrNode::EndParts { id, parent });
    }

    fn visit_widget(&mut self, widget: &mut Widget, parent: NodeId) {
        let id = self.get_node_id();

        self.emit_node(IrNode::StartWidget {
            name: take(&mut widget.name.string),
            id,
            is_platform: widget.is_platform,
        });

        self.emit_node(IrNode::StartAttributeList);

        for attrib in &widget.attributes {
            // match attrib {
            //     Attribute::Regular { name, value } => {
            //         self.emit_node(IrNode::AttributeName {
            //             name: attrib
            //             span: attrib.name,
            //         });
            //     }
            // }

            // match &attrib.value {
            //     AttributeValue::Expression(span) => {
            //         //TODO: handle visit expression parent here
            //         self.visit_expression(*span, 0);
            //     }
            //
            //     AttributeValue::Parts(parts) => {
            //         // the parts generated within an attribute will have a value of 0
            //         // the consumer of the IR is expected to handle the attribute's value
            //         // as a special case, in terms of association
            //         self.visit_parts(parts, 0)
            //     }
            //
            //     AttributeValue::True => {
            //         self.emit_node(IrNode::AttributeTrue);
            //     }
            //
            //     AttributeValue::BindingAttribute {
            //         bind_from,
            //         bind_to,
            //         value,
            //     } => {
            //         todo!()
            //     }
            // }
        }

        self.emit_node(IrNode::EndAttributeList);

        if let Some(frag) = &mut widget.fragment {
            self.visit_fragment(frag, id);
        }

        self.emit_node(IrNode::EndWidget { id, parent });
    }

    fn visit_control(&mut self, control: &Control, parent: NodeId) {
        // match control {
        //     Control::If(if_ctrl) => self.visit_if(if_ctrl, parent),
        //     Control::Each(each_ctrl) => self.visit_each(each_ctrl, parent),
        // }
    }

    // fn visit_if(&mut self, if_ctrl: &IfControl, parent: NodeId) {
    //     let if_id = self.get_node_id();
    //
    //     //TODO: handle visit expression parent here
    //     let test = Some(self.visit_expression(if_ctrl.condition, 0));
    //
    //     self.emit_node(IrNode::StartIfBlock { test, id: if_id });
    //
    //     self.visit_fragment(&if_ctrl.body, if_id);
    //
    //     for case in &if_ctrl.elifs {
    //         //TODO: handle visit expression parent here
    //         let test = Some(self.visit_expression(if_ctrl.condition, 0));
    //
    //         let case_id = self.get_node_id();
    //
    //         self.emit_node(IrNode::StartCaseBlock { test, id: case_id });
    //
    //         self.visit_fragment(&case.body, case_id);
    //
    //         self.emit_node(IrNode::EndCaseBlock {
    //             id: case_id,
    //             parent: if_id,
    //         });
    //     }
    //
    //     if let Some(else_ctrl) = &if_ctrl.else_body {
    //         let else_id = self.get_node_id();
    //
    //         self.emit_node(IrNode::StartCaseBlock {
    //             test: None,
    //             id: else_id,
    //         });
    //
    //         self.visit_fragment(else_ctrl, else_id);
    //
    //         self.emit_node(IrNode::EndCaseBlock {
    //             id: else_id,
    //             parent: if_id,
    //         });
    //     }
    //
    //     self.emit_node(IrNode::EndIfBlock { id: if_id, parent });
    // }
    //
    // fn visit_each(&mut self, each_ctrl: &EachControl, parent: NodeId) {
    //     let each_id = self.get_node_id();
    //
    //     //TODO: handle visit expression parent here
    //     let iterable = Some(self.visit_expression(each_ctrl.expression, 0));
    //
    //     let context = self.src.view_span(each_ctrl.context).to_string();
    //
    //     self.emit_node(IrNode::StartEachBlock {
    //         iterable,
    //         id: each_id,
    //     });
    //
    //     self.context_vars.push(context);
    //
    //     self.visit_fragment(&each_ctrl.body, each_id);
    //
    //     self.context_vars.pop();
    //
    //     self.emit_node(IrNode::EndEachBlock {
    //         id: each_id,
    //         parent,
    //     });
    // }

    fn visit_fragment(&mut self, frag: &mut Fragment, parent: NodeId) {
        for node in &mut frag.nodes {
            match node {
                FragmentNode::Text(span) => {
                    self.visit_text(std::mem::take(span), parent)
                }
                FragmentNode::Expression(expr) => {
                    self.visit_expression(std::mem::take(expr), parent);
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
