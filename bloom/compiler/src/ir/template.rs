use crate::{
    common::Source,
    parser::{
        Control, EachControl, Fragment, FragmentNode, IfControl, Root, Widget,
    },
};

#[derive(Debug, Clone)]
pub struct TemplateIr<'a> {
    pub nodes: Vec<IrNode>,
    pub expressions: Vec<Expression>,

    ast: &'a Root,
    src: &'a Source,
    next_id: usize,
}

type WidgetId = usize;
#[derive(Debug, Clone)]
pub enum IrNode {
    CreateWidget(CreateWidget),
    CreateParts(CreateParts),
    CreateIfBlock(CreateIfBlock),
    CreateEachBlock(CreateEachBlock),
    EndIfBlock,
    EndIfCase,
    EndEachBlock,
    AddChild(AddChild),
}

#[derive(Debug, Clone)]
pub struct CreateIfBlock {
    pub id: WidgetId,
    pub cases: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct CreateEachBlock {
    pub context: String,
    pub expression: Expression,
    pub id: WidgetId,
}

#[derive(Debug, Clone)]
pub struct CreateWidget {
    pub id: WidgetId,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CreateParts {
    pub id: WidgetId,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone)]
pub enum Part {
    Text(Text),
    Expression(Expression),
}

#[derive(Debug, Clone)]
pub struct Text {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub expression: String,
    pub context: Vec<String>,
    pub symbols: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AddChild {
    pub parent_id: WidgetId,
    pub child_id: WidgetId,
}

impl<'a> TemplateIr<'a> {
    pub fn new(src: &'a Source, ast: &'a Root) -> Self {
        Self {
            src,
            ast,
            expressions: Vec::new(),
            nodes: Vec::new(),
            next_id: 0,
        }
    }

    fn get_id(&mut self) -> WidgetId {
        self.next_id += 1;
        assert!(self.next_id != 0); // id 0 is the root
        self.next_id
    }

    fn emit(&mut self, node: IrNode) {
        self.nodes.push(node);
    }

    fn visit_widget(
        &mut self,
        widget: &Widget,
        context: Vec<String>,
        parent_id: usize,
    ) {
        let id = self.get_id();
        let name = self.src.view_span(widget.name).to_string();

        self.emit(IrNode::CreateWidget(CreateWidget { id, name }));
        self.emit(IrNode::AddChild(AddChild {
            parent_id,
            child_id: id,
        }));

        if let Some(frag) = &widget.fragment {
            self.visit_fragment(frag, context.clone(), id);
        }
    }

    fn visit_parts(
        &mut self,
        parts: &mut Vec<&FragmentNode>,
        context: Vec<String>,
        parent_id: usize,
    ) {
        let mut parts_ir = Vec::new();

        for part in parts.iter() {
            match part {
                FragmentNode::Text(text) => parts_ir.push(Part::Text(Text {
                    text: self.src.view_span(text).to_string(),
                })),

                FragmentNode::Expression(expression) => {
                    let expr = Expression {
                        expression: self.src.view_span(expression).to_string(),
                        symbols: vec![],
                        context: context.clone(), // expensive operation, but affordable
                    };

                    parts_ir.push(Part::Expression(expr.clone()));
                    self.expressions.push(expr.clone());
                }

                _ => panic!("unexpected part: {:?}", part),
            }
        }

        let id = self.get_id();

        self.nodes.push(IrNode::CreateParts(CreateParts {
            id,
            parts: parts_ir,
        }));

        self.nodes.push(IrNode::AddChild(AddChild {
            parent_id,
            child_id: id,
        }));

        parts.clear();
    }

    fn visit_if_ctrl(&mut self, context: Vec<String>, if_ctrl: &IfControl) {
        let mut conditions = Vec::new();

        if if_ctrl.body.nodes.len() > 0 {
            let expr = Expression {
                expression: self.src.view_span(if_ctrl.condition).to_string(),
                symbols: vec![],
                context: context.clone(), // expensive operation, but affordable
            };

            conditions.push(expr.clone());
            self.expressions.push(expr.clone());
        }

        for elif in &if_ctrl.elifs {
            if elif.body.nodes.len() == 0 {
                continue;
            }

            let expr = Expression {
                expression: self.src.view_span(elif.condition).to_string(),
                symbols: vec![],
                context: context.clone(), // expensive operation, but affordable
            };

            conditions.push(expr.clone());
            self.expressions.push(expr.clone());
        }

        let if_id = self.get_id();

        self.emit(IrNode::CreateIfBlock(CreateIfBlock {
            id: if_id,
            cases: conditions,
        }));

        self.visit_fragment(&if_ctrl.body, context.clone(), if_id);

        self.emit(IrNode::EndIfCase);

        for elif in &if_ctrl.elifs {
            if elif.body.nodes.len() == 0 {
                continue;
            }

            self.visit_fragment(&elif.body, context.clone(), if_id);
            self.emit(IrNode::EndIfCase);
        }

        if let Some(body) = &if_ctrl.else_body {
            self.visit_fragment(body, context.clone(), if_id);
            self.emit(IrNode::EndIfCase);
        }

        self.emit(IrNode::EndIfBlock);
    }

    fn visit_each_ctrl(
        &mut self,
        context: Vec<String>,
        each_ctrl: &EachControl,
    ) {
        let each_context = self.src.view_span(&each_ctrl.context).to_string();

        let mut context = context.clone();

        let expression = Expression {
            expression: self.src.view_span(each_ctrl.expression).to_string(),
            symbols: vec![],
            context: context.clone(), // expensive operation, but affordable
        };

        self.expressions.push(expression.clone());

        context.push(each_context.clone());

        let id = self.get_id();
        self.emit(IrNode::CreateEachBlock(CreateEachBlock {
            context: each_context,
            expression,
            id,
        }));

        self.visit_fragment(&each_ctrl.body, context.clone(), id);

        self.emit(IrNode::EndEachBlock);
    }

    fn visit_fragment(
        &mut self,
        fragment: &Fragment,
        context: Vec<String>,
        id: usize,
    ) {
        let mut parts = Vec::new();

        for node in &fragment.nodes {
            match node {
                FragmentNode::Widget(elem) => {
                    if parts.len() > 0 {
                        self.visit_parts(&mut parts, context.clone(), id);
                    }

                    self.visit_widget(elem, context.clone(), id)
                }

                FragmentNode::Text(_) | FragmentNode::Expression(_) => {
                    parts.push(node);
                }

                FragmentNode::Control(control) => {
                    if parts.len() > 0 {
                        self.visit_parts(&mut parts, context.clone(), id);
                    }

                    match &control {
                        Control::If(if_ctrl) => {
                            self.visit_if_ctrl(context.clone(), if_ctrl);
                        }

                        Control::Each(each_ctrl) => {
                            self.visit_each_ctrl(context.clone(), each_ctrl);
                        }
                    }
                }
            }
        }

        if parts.len() > 0 {
            self.visit_parts(&mut parts, context.clone(), id);
        }
    }

    pub fn generate_ir(&mut self) {
        if let Some(frag) = &self.ast.fragment {
            self.visit_fragment(frag, vec![], 0);
        }
    }
}
