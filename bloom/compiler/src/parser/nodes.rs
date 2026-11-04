use core::fmt;
use std::fmt::{Write, write};

use crate::{
    analysis::PlatformNamespace,
    common::{Span, SpannedString},
    module::{Expression, ExpressionId},
};

// Fragment
#[derive(Debug)]
pub struct Fragment {
    pub nodes: Vec<FragmentNode>,
}

#[derive(Debug)]
pub enum FragmentNode {
    Tag(Tag),
    Parts(Parts),
    Text(String),
    Expression(ExpressionId),
    Control(Control),
}

#[derive(Debug, PartialEq, Eq)]
pub enum FragmentEnd {
    Root,
    Tag,
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

// Tag
#[derive(Debug)]
pub struct Tag {
    pub name: TagName,
    pub attributes: Vec<Attribute>,
    pub fragment: Option<Fragment>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum TagNamespace {
    Platform(PlatformNamespace),
    Slot,
    Custom(SpannedString),
}

impl TagNamespace {
    pub fn is_name_match(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Platform(a), Self::Platform(b)) => a == b,
            (Self::Slot, Self::Slot) => true,
            (Self::Custom(a), Self::Custom(b)) => a.string == b.string,
            _ => false,
        }
    }
}

impl fmt::Display for TagNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Platform(a) => a.as_str(),
                Self::Slot => "slot",
                Self::Custom(a) => &a.string,
            }
        )
    }
}

#[derive(Debug, Eq, PartialEq, Default, Clone)]
pub struct TagName {
    pub namespace: Option<TagNamespace>,
    pub name: SpannedString,
}

impl TagName {
    pub fn span(&self) -> Span {
        self.name.span
    }

    pub fn string(&self) -> &String {
        &self.name.string
    }

    pub fn is_name_match(&self, other: &Self) -> bool {
        let namespace_match = match (&self.namespace, &other.namespace) {
            (Some(a), Some(b)) => a.is_name_match(b),
            (None, None) => true,
            _ => false,
        };

        namespace_match && self.name.string == other.name.string
    }
}

impl fmt::Display for TagName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(n) = &self.namespace {
            write!(f, "{n}:{}", self.name)
        } else {
            write!(f, "{}", self.name)
        }
    }
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
    pub key: Option<ExpressionId>,
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
