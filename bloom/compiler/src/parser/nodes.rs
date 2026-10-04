use crate::common::Span;

#[derive(Debug, Clone)]
pub struct Root {
    pub fragment: Option<Fragment>,
    pub script: Option<Script>,
}

#[derive(Debug, Clone)]
pub struct Script {
    pub open: Span,
    pub source: Span,
}

// Fragment
#[derive(Debug, Clone)]
pub struct Fragment {
    pub nodes: Vec<FragmentNode>,
}

#[derive(Debug, Clone)]
pub enum FragmentNode {
    Widget(Widget),
    Text(Span),
    Expression(Span),
    Control(Control),
}

#[derive(Debug, Clone)]
pub enum Part {
    Text(Span),
    Expression(Span),
}
pub type Parts = Vec<Part>;

#[derive(Debug, Clone)]
pub struct Widget {
    pub name: Span,
    pub attributes: Vec<Attribute>,
    pub fragment: Option<Fragment>,
}

#[derive(Debug, Clone)]
pub enum Control {
    If(IfControl),
    Each(EachControl),
}

#[derive(Debug, Clone)]
pub struct IfControl {
    pub condition: Span,
    pub body: Fragment,
    pub elifs: Vec<ElifControl>,
    pub else_body: Option<Fragment>,
}

#[derive(Debug, Clone)]
pub struct ElifControl {
    pub condition: Span,
    pub body: Fragment,
}

#[derive(Debug, Clone)]
pub struct EachControl {
    pub expression: Span,
    pub context: Span,
    pub body: Fragment,
}

// Attribute
#[derive(Debug, Clone)]
pub struct Attribute {
    pub name: Span,
    pub value: AttributeValue,
}

#[derive(Debug, Clone)]
pub enum AttributeValue {
    Expression(Span),
    Parts(Parts),
    True,
}
