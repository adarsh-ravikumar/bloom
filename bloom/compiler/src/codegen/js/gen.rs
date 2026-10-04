use crate::{
    common::Source,
    parser::{Fragment, FragmentNode, Root, Widget},
};

#[derive(Debug, Clone)]
pub enum IrNode {
    CreateWidget(CreateWidget),
    CreateScript(String),
    CreateParts(CreateParts),
    AddChild(AddChild),
}

#[derive(Debug, Clone)]
pub struct CreateWidget {
    pub id: usize,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct CreateParts {
    pub id: usize,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone)]
pub enum Part {
    Text(PartText),
    Expression(PartExpression),
}

#[derive(Debug, Clone)]
pub struct PartText {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct PartExpression {
    pub expression: String,
    pub states: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct AddChild {
    pub parent_id: usize,
    pub child_id: usize,
}

pub struct CodegenJs<'a> {
    pub src: &'a Source,
    pub ast: &'a Root,
    pub ir: Vec<IrNode>,
    pub next_id: usize,
}

impl<'a> CodegenJs<'a> {
    pub fn new(src: &'a Source, ast: &'a Root) -> Self {
        Self {
            src,
            ast,
            ir: Vec::new(),
            next_id: 0,
        }
    }

    pub fn get_id(&mut self) -> usize {
        self.next_id += 1;
        assert!(self.next_id != 0); // id 0 is the root
        self.next_id
    }

    pub fn emit(&mut self, node: IrNode) {
        self.ir.push(node);
    }

    pub fn visit_widget(&mut self, widget: &Widget, parent_id: usize) {
        let id = self.get_id();
        let name = self.src.view_span(widget.name).to_string();

        self.emit(IrNode::CreateWidget(CreateWidget { id, name }));
        self.emit(IrNode::AddChild(AddChild {
            parent_id,
            child_id: id,
        }));

        if let Some(frag) = &widget.fragment {
            self.visit_fragment(frag, id);
        }
    }

    pub fn visit_parts(
        &mut self,
        parts: &mut Vec<&FragmentNode>,
        parent_id: usize,
    ) {
        let mut parts_ir = Vec::new();

        for part in parts.iter() {
            match part {
                FragmentNode::Text(text) => {
                    parts_ir.push(Part::Text(PartText {
                        text: self.src.view_span(text).to_string(),
                    }))
                }

                FragmentNode::Expression(expression) => {
                    parts_ir.push(Part::Expression(PartExpression {
                        expression: self.src.view_span(expression).to_string(),
                        states: vec![],
                    }))
                }

                _ => panic!("unexpected part: {:?}", part),
            }
        }

        let id = self.get_id();

        self.ir.push(IrNode::CreateParts(CreateParts {
            id,
            parts: parts_ir,
        }));

        self.ir.push(IrNode::AddChild(AddChild {
            parent_id,
            child_id: id,
        }));

        parts.clear();
    }

    pub fn visit_fragment(&mut self, fragment: &Fragment, id: usize) {
        let mut parts = Vec::new();

        for node in &fragment.nodes {
            match node {
                FragmentNode::Widget(elem) => {
                    if parts.len() > 0 {
                        self.visit_parts(&mut parts, id);
                    }

                    self.visit_widget(elem, id)
                }

                FragmentNode::Text(_) | FragmentNode::Expression(_) => {
                    parts.push(node);
                }

                _ => continue,
            }
        }

        if parts.len() > 0 {
            self.visit_parts(&mut parts, id);
        }
    }

    pub fn generate_code(&mut self) -> String {
        let mut code = String::new();
        code += "const $w0 = document.querySelector(\"#main\");\n";

        let mut scripts = String::new();

        for action in &self.ir {
            match action {
                IrNode::CreateWidget(widget) => {
                    code += format!(
                        "const $w{} = document.createElement(\"{}\");\n",
                        widget.id, widget.name
                    )
                    .as_str();
                }

                IrNode::AddChild(add) => {
                    code += format!(
                        "$w{}.appendChild($w{});\n",
                        add.parent_id, add.child_id
                    )
                    .as_str();
                }

                IrNode::CreateParts(parts) => {
                    let mut string = String::new();

                    for part in &parts.parts {
                        match part {
                            Part::Text(text) => {
                                string += &text.text;
                            }
                            Part::Expression(expr) => {
                                string += format!("${{ {} }}", expr.expression)
                                    .as_str();
                            }
                        }
                    }

                    code += format!(
                            "const $w{} = document.createElement(\"span\");\n$w{}.innerText = `{string}`\n",
                            parts.id, parts.id
                        )
                        .as_str();
                }

                IrNode::CreateScript(source) => {
                    scripts += source;
                }
            }
        }

        scripts += &code;
        scripts += "mount()\n";

        scripts
    }

    pub fn generate(&mut self) -> String {
        if let Some(frag) = &self.ast.fragment {
            self.visit_fragment(&frag, 0);

            let code = self.generate_code();

            return code;
        }

        return "".to_string();
    }
}
