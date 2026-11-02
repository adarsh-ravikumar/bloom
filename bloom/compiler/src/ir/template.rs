use std::collections::HashMap;

use crate::{
    analysis::PlatformNamespace,
    module::ExpressionId,
    parser::{
        Attribute, AttributeValue, Control, EachControl, Fragment,
        FragmentNode, IfControl, Part, Parts, Widget,
    },
    utils,
};

#[derive(Debug, Clone)]
pub struct TemplateIrGen<'a> {
    pub ir_nodes: Vec<IrNode>,
    // exposed_event_map: &'a HashMap<String, Option<String>>,
    ast: &'a Fragment,
    next_node_id: usize,
}

type NodeId = usize;

#[rustfmt::skip]
#[derive(Debug, Clone)]
pub enum IrNode {
    StartParts { id: NodeId },
    Text { value: String },
    Expression { expr_id: ExpressionId },
    EndParts { id: NodeId, parent: NodeId },

    StartWidget { name: String, id: NodeId, is_platform: bool },
    EndWidget { id: NodeId, parent: NodeId },

    StartAttributeList,
    AttributeName { name: String, id: NodeId },
    AttributeBindEvent { event: String, id: NodeId },
    AttributeExposeEvent { source_event: String,  id: NodeId, widget_id: ExpressionId},
    AttributeBindState { from: String, to: String, id: NodeId, has_value: bool},
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

    // NOTE: we santize `$` to ensure `${` in text doesn't open
    // an expression in a template literal
    for char in &text.into_bytes() {
        if *char == b'\\' {
            san.extend_from_slice(b"\\\\");
        } else if *char == b'$' {
            san.extend_from_slice(b"\\$");
        } else if *char == b'"' {
            san.extend_from_slice(b"\\\"");
        } else if *char == b'`' {
            san.extend_from_slice(b"\\`");
        } else if *char == b'\'' {
            san.extend_from_slice(b"\\'");
        } else if *char == b'\n' {
            san.extend_from_slice(b"\\n");
        } else if *char == b'\t' {
            san.extend_from_slice(b"\\t");
        } else if *char == b'\r' {
            san.extend_from_slice(b"\\r");
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

    fn visit_text(&mut self, value: String) {
        self.emit_node(IrNode::Text {
            value: sanitize_text(value),
        });
    }

    fn visit_expression(&mut self, expr_id: ExpressionId) {
        self.emit_node(IrNode::Expression { expr_id });
    }

    fn visit_parts(&mut self, parts: &Parts, parent: NodeId) {
        let id = self.get_node_id();

        self.emit_node(IrNode::StartParts { id });

        for part in parts {
            match part {
                Part::Text(string) => self.visit_text(string.clone()),
                Part::Expression(expr_id) => {
                    self.visit_expression(*expr_id);
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
                self.visit_expression(*expr);
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

        /* NOTE: Attribute analysis guarantees that if a widget has an id, then it is the
         * first attribute, its value is an expression, and that expression is a lone identifier.
         * If no `id` attribute exists, the widget cannot have exposed events. So widget_id will never
         * be consumed. In that case `0` is a harmless default rather than an actual expression ID
         */
        let widget_id = if let Some(id_attrib_candidate) =
            widget.attributes.first()
            && let Attribute::Regular(attrib) = id_attrib_candidate
            && attrib.name.string == "id"
            && let AttributeValue::Expression(expr) = &attrib.value
        {
            *expr
        } else {
            0
        };

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
                        if attrib.value.is_some() {
                            self.ir_nodes.push(IrNode::AttributeBindEvent {
                                event: attrib.bind_to.string.clone(),
                                id,
                            })
                        } else {
                            let source_event = utils::normalize_case(
                                &attrib.bind_to.string,
                                utils::Case::SnakeCase,
                            );

                            self.ir_nodes.push(IrNode::AttributeExposeEvent {
                                source_event,
                                widget_id,
                                id,
                            })
                        }
                    } else {
                        self.ir_nodes.push(IrNode::AttributeBindState {
                            from: attrib.bind_from.string.clone(),
                            to: attrib.bind_to.string.clone(),
                            has_value: attrib.value.is_some(),
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
                FragmentNode::Text(value) => self.visit_text(value.clone()),
                FragmentNode::Expression(expr) => {
                    self.visit_expression(*expr);
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
