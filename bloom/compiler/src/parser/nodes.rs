use crate::{
    analysis::PlatformNamespace,
    common::SpannedString,
    module::{Expression, ExpressionId},
};

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
    Expression(ExpressionId),
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
    Expression(ExpressionId),
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
    pub platform_namespace: Option<PlatformNamespace>,
}

// Control
#[derive(Debug)]
pub enum Control {
    If(IfControl),
    Each(EachControl),
}

#[derive(Debug)]
pub struct IfControl {
    pub condition: ExpressionId,
    pub body: Fragment,
    pub elifs: Vec<ElifControl>,
    pub else_body: Option<Fragment>,
}

#[derive(Debug)]
pub struct ElifControl {
    pub condition: ExpressionId,
    pub body: Fragment,
}

#[derive(Debug)]
pub struct EachControl {
    pub expression: ExpressionId,
    pub context: SpannedString,
    pub body: Fragment,
}

// Attribute
#[derive(Debug)]
pub enum Attribute {
    Regular(RegularAttribute),
    Binding(BindingAttribute),
}

#[derive(Debug)]
pub struct BindingAttribute {
    pub bind_from: SpannedString,
    pub bind_to: SpannedString,
    pub value: Option<AttributeValue>,
}

#[derive(Debug)]
pub struct RegularAttribute {
    pub name: SpannedString,
    pub value: AttributeValue,
}

#[derive(Debug)]
pub enum AttributeValue {
    Expression(ExpressionId),
    Parts(Parts),
    True,
}
