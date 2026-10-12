use crate::common::{Span, SpannedString};

// Fragment
#[derive(Debug)]
pub struct Fragment {
    pub nodes: Vec<FragmentNode>,
}

#[derive(Debug)]
pub enum FragmentNode {
    Widget(Widget),
    Parts(Parts),
    Text(String),
    Expression(SpannedString),
    Control(Control),
}

#[derive(Debug, PartialEq, Eq)]
pub enum FragmentEnd {
    Root,
    Widget,
    ControlEnd,
    ControlCase,
}

// Parts
#[derive(Debug)]
pub enum Part {
    Text(String),
    Expression(SpannedString),
}
pub type Parts = Vec<Part>;

impl Part {
    pub fn as_fragment(self) -> FragmentNode {
        match self {
            Self::Text(v) => FragmentNode::Text(v),
            Self::Expression(v) => FragmentNode::Expression(v),
        }
    }
}

// Widget
#[derive(Debug)]
pub struct Widget {
    pub name: SpannedString,
    pub attributes: Vec<Attribute>,
    pub fragment: Option<Fragment>,
    pub is_platform: bool,
}

// Control
#[derive(Debug)]
pub enum Control {
    If(IfControl),
    Each(EachControl),
}

#[derive(Debug)]
pub struct IfControl {
    pub condition: SpannedString,
    pub body: Fragment,
    pub elifs: Vec<ElifControl>,
    pub else_body: Option<Fragment>,
}

#[derive(Debug)]
pub struct ElifControl {
    pub condition: SpannedString,
    pub body: Fragment,
}

#[derive(Debug)]
pub struct EachControl {
    pub expression: SpannedString,
    pub context: SpannedString,
    pub body: Fragment,
}

// Attribute
#[derive(Debug)]
pub struct Attribute {
    pub name: SpannedString,
    pub value: AttributeValue,
}

#[derive(Debug)]
pub enum AttributeValue {
    Expression(SpannedString),
    Parts(Parts),
    BindingAttribute {
        bind_from: SpannedString,
        bind_to: SpannedString,
        value: Option<Box<AttributeValue>>,
    },
    True,
}
