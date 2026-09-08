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
    RegularElement(RegularElement),
    Text(Text),
}

#[derive(Debug, Clone)]
pub struct Text {
    pub data: Span,
}

#[derive(Debug, Clone)]
pub struct RegularElement {
    pub name: Span,
    pub attributes: Vec<Attribute>,
    pub fragment: Option<Fragment>,
}

// Attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Span,
    pub value: AttributeValue,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    Text(Text),
    True,
}
