use crate::span::Span;

#[derive(Debug, Clone)]
pub struct Root {
    pub fragment: Option<Fragment>,
}

// Fragment
#[derive(Debug, Clone)]
pub struct Fragment {
    pub nodes: Vec<FragmentNode>,
}

#[derive(Debug, Clone)]
pub enum FragmentNode {
    Element(Element),
    Text(Span),
    Expression(Span),
    ControlBlock(ControlBlock),
}

#[derive(Debug, Clone)]
pub struct Element {
    pub name: Span,
    pub attributes: Vec<Attribute>,
    pub fragment: Option<Fragment>,
}

#[derive(Debug, Clone)]
pub struct ControlBlock {
    pub control_type: Span,
    pub expression: Span,
    pub fragment: Fragment,
}

// Attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Span,
    pub value: AttributeValue,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    Parts(Fragment),
    True,
}
