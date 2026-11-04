use std::collections::HashMap;

use crate::{
    common::EMPTY_SPAN,
    module::{ExpressionId, SlotMembers},
    parser::{
        Attribute, AttributeValue, Control, EachControl, Fragment,
        FragmentNode, IfControl, Part, Parts, Tag, TagName, TagNamespace,
    },
    utils,
};

#[derive(Debug, Clone)]
pub struct TemplateIrGen<'a> {
    pub ir_nodes: Vec<IrNode>,
    // exposed_event_map: &'a HashMap<String, Option<String>>,
    ast: &'a Fragment,
    next_node_id: usize,
    slots: &'a HashMap<String, SlotMembers>,
}

type NodeId = usize;

#[rustfmt::skip]
#[derive(Debug, Clone)]
pub enum IrNode {
    StartParts { id: NodeId },
    Text { value: String },
    Expression { expr_id: ExpressionId },
    EndParts { id: NodeId, parent: NodeId },

    StartTag { name: TagName, id: NodeId},
    UseSlot { name: TagName, id: NodeId, parent: NodeId },
    EndTag { id: NodeId, parent: NodeId },

    DeclareSlot { name: Option<String>, parent: NodeId },

    StartAttributeList,
    AttributeName { name: String, id: NodeId },
    AttributeBindEvent { event: String, id: NodeId },
    AttributeExposeEvent { source_event: String,  id: NodeId, tag_id: ExpressionId},
    AttributeBindState { from: String, to: String, id: NodeId, has_value: bool},
    AttributeTrue,
    EndAttributeList,

    StartIfBlock { test: ExpressionId, id: NodeId},
    EndIfBlock { id: NodeId, parent: NodeId },

    StartCaseBlock { test: Option<ExpressionId>, id: NodeId, parent_block: NodeId},
    EndCaseBlock,

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
    pub fn new(
        ast: &'a Fragment,
        slots: &'a HashMap<String, SlotMembers>,
    ) -> Self {
        Self {
            next_node_id: 1, // 0 represents global scope
            ir_nodes: Vec::new(),
            slots,
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

    fn visit_tag(&mut self, tag: &Tag, parent: NodeId) {
        let id = self.get_node_id();

        match &tag.name.namespace {
            // handle slot declaration
            Some(TagNamespace::Slot) => {
                if tag.fragment.is_none() {
                    self.emit_node(IrNode::DeclareSlot {
                        name: if tag.name.span() == EMPTY_SPAN {
                            None
                        } else {
                            Some(tag.name.string().clone())
                        },
                        parent,
                    });

                    return;
                }
            }

            _ => {
                self.emit_node(IrNode::StartTag {
                    name: tag.name.clone(),
                    id,
                });

                self.emit_node(IrNode::StartAttributeList);

                /* NOTE: Attribute analysis guarantees that if a tag has an id, then it is the
                 * first attribute, its value is an expression, and that expression is a lone identifier.
                 * If no `id` attribute exists, the tag cannot have exposed events. So tag_id will never
                 * be consumed. In that case `0` is a harmless default rather than an actual expression ID
                 */
                let tag_id = if let Some(id_attrib_candidate) =
                    tag.attributes.first()
                    && let Attribute::Regular(attrib) = id_attrib_candidate
                    && attrib.name.string == "id"
                    && let AttributeValue::Expression(expr) = &attrib.value
                {
                    *expr
                } else {
                    0
                };

                for attrib in &tag.attributes {
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
                                    self.ir_nodes.push(
                                        IrNode::AttributeBindEvent {
                                            event: attrib
                                                .bind_to
                                                .string
                                                .clone(),
                                            id,
                                        },
                                    )
                                } else {
                                    let source_event = utils::normalize_case(
                                        &attrib.bind_to.string,
                                        utils::Case::SnakeCase,
                                    );

                                    self.ir_nodes.push(
                                        IrNode::AttributeExposeEvent {
                                            source_event,
                                            tag_id,
                                            id,
                                        },
                                    )
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
            }
        }

        if let Some(frag) = &tag.fragment {
            match &tag.name.namespace {
                Some(TagNamespace::Platform(_)) => {
                    self.visit_fragment(frag, id);
                    self.emit_node(IrNode::EndTag { id, parent });
                }

                Some(TagNamespace::Slot) => {
                    self.emit_node(IrNode::UseSlot {
                        name: tag.name.clone(),
                        parent,
                        id: id,
                    });

                    self.visit_fragment(frag, id);
                }

                _ => {
                    let slot = self.slots.get(tag.name.string());
                    match slot {
                        Some(SlotMembers::Unnamed) => {
                            let slot_id = self.get_node_id();
                            self.emit_node(IrNode::UseSlot {
                                name: TagName::default(),
                                parent: id,
                                id: slot_id,
                            });
                            self.visit_fragment(frag, slot_id);
                        }

                        _ => {
                            self.visit_fragment(frag, id);
                        }
                    }

                    self.emit_node(IrNode::EndTag { id, parent });
                }
            }
        }
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

        self.emit_node(IrNode::StartCaseBlock {
            test: Some(if_ctrl.condition),
            id: if_id,
            parent_block: if_id,
        });

        self.visit_fragment(&if_ctrl.body, if_id);

        self.emit_node(IrNode::EndCaseBlock);

        for case in &if_ctrl.elifs {
            let case_id = self.get_node_id();

            self.emit_node(IrNode::StartCaseBlock {
                test: Some(case.condition),
                id: case_id,
                parent_block: if_id,
            });

            self.visit_fragment(&case.body, case_id);

            self.emit_node(IrNode::EndCaseBlock);
        }

        if let Some(else_ctrl) = &if_ctrl.else_body {
            let else_id = self.get_node_id();

            self.emit_node(IrNode::StartCaseBlock {
                test: None,
                id: else_id,
                parent_block: if_id,
            });

            self.visit_fragment(else_ctrl, else_id);

            self.emit_node(IrNode::EndCaseBlock);
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
                FragmentNode::Tag(tag) => self.visit_tag(tag, parent),
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
