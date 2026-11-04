use std::collections::HashMap;

use crate::{
    common::{Span, SpannedString},
    parser::{Alias, Enum, Interface, TypeAnnotation},
};

#[derive(Debug, Clone)]
pub enum SymbolUseType {
    GlobalState,
    GlobalVariable,
    GlobalFunction,
    TemplateContext,
    GlobalUnresolved,
    Local,
    Unknown,
}

#[derive(Debug, Clone)]
pub enum SlotMembers {
    Named(HashMap<String, Span>),
    Unnamed,
}

#[derive(Debug, Clone)]
pub struct SymbolUse {
    pub r#use: SpannedString,
    pub canonical_name: String,
    pub r#type: SymbolUseType,
    pub context: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Expression {
    pub source: SpannedString,
    pub is_lone_identifier: bool,
    pub context_vars: Vec<String>,
    pub symbol_uses: Vec<SymbolUse>,
}

pub type ExpressionId = usize;

impl Expression {
    pub fn new(source: SpannedString) -> Self {
        Self {
            source,
            is_lone_identifier: false,
            context_vars: Vec::new(),
            symbol_uses: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub source_name: String,
    pub canonical_name: String,
    pub span: Span,
    pub type_annotation: Option<TypeAnnotation>,
    pub is_let: bool,
    pub is_state: bool,
    pub is_export: bool,
    pub init: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub source_name: String,
    pub canonical_name: String,
    pub context: usize,
    pub body: SpannedString,
    pub params: String,
    pub return_type: Option<TypeAnnotation>,
    // bloom does NOT allow for generics.
    // store here if any for diagnostics
    pub generics: Option<Span>,
}

#[derive(Debug, Clone)]
pub enum ModuleMemberType {
    // event, command and reactive hold Option<Function> because their entries are
    // first populated by the preprocessor. it is only during script analysis that the actual
    // functions are parsed
    Event(Option<Function>),
    Command(Option<Function>),
    Reactive(Option<Function>),

    RenamedEvent { from: String, to: String },
    StateProxy { from: String, to: String },
    ExposedEvent { prefix: String, event: String },

    Function(Function),

    PrivateState(Variable),
    PublicState(Variable),
    PublicVariable(Variable),
    PrivateVariable(Variable),

    Enum(Enum),
    Alias(Alias),
    Interface(Interface),
}

pub type ModuleMetadata = HashMap<String, ModuleMemberType>;
