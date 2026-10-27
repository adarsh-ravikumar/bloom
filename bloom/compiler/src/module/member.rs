use std::collections::HashMap;

use crate::{
    common::{Span, SpannedString},
    parser::{Alias, Enum, Interface, TypeAnnotation},
};

#[derive(Debug, Clone)]
pub struct SymbolUse {
    pub r#use: SpannedString,
    pub context: usize,
    pub is_global_use: bool,
    pub is_template_context_use: bool,
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
    pub span: Span,
    pub type_annotation: Option<TypeAnnotation>,
    pub is_let: bool,
    pub is_state: bool,
    pub is_export: bool,
    pub init: Option<Span>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub context: usize,
    pub body: Option<String>,
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
