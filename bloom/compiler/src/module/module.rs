use std::collections::{HashMap, HashSet};

use crate::{
    common::{ERRONEOUS_SPAN, Source, Span, SpannedString},
    diagnostic::DiagnosticRenderer,
    ir::{Alias, Enum, Interface, NamedType, SemanticAnalyzer, TypeAnnotation},
    module::ModuleRegistry,
    parser::{Fragment, Parser},
};

#[derive(Debug, Clone)]
pub struct State {
    span: Span,
    r#type: TypeAnnotation,
    is_export: bool,
}

#[derive(Debug, Clone)]
pub struct SymbolUse {
    pub r#use: SpannedString,
    pub context: usize,
}

#[derive(Debug)]
pub struct Expression {
    pub source: SpannedString,
    pub context_vars: Vec<String>,
    pub global_uses: Vec<SymbolUse>,
    pub context_uses: Vec<SymbolUse>,
}

impl Expression {
    pub fn new(source: SpannedString) -> Self {
        Self {
            source,
            context_vars: Vec::new(),
            global_uses: Vec::new(),
            context_uses: Vec::new(),
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
    Event(Option<Function>),
    Command(Option<Function>),
    Reactive(Option<Function>),
    Function(Option<Function>),

    PrivateState(Variable),
    PublicState(Variable),
    PublicVariable(Variable),
    PrivateVariable(Variable),

    Enum(Enum),
    Alias(Alias),
    Interface(Interface),
}

pub type ModuleMetadata = HashMap<String, ModuleMemberType>;

#[derive(Debug, Clone, Copy)]
pub enum ModuleType {
    Widget,
    Panel,
}

#[derive(Debug)]
pub struct Module {
    pub root: Option<Fragment>,
    pub r#type: ModuleType,
    pub script: (Span, String),
    pub metadata: ModuleMetadata,
    pub dependencies: HashMap<String, Span>, // store the span of the first occurance
    pub symbol_uses: Vec<SymbolUse>,
    pub expressions: Vec<Expression>,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            root: None,
            r#type: ModuleType::Widget,
            script: (ERRONEOUS_SPAN, String::new()),
            metadata: ModuleMetadata::new(),
            dependencies: HashMap::new(),
            symbol_uses: Vec::new(),
            expressions: Vec::new(),
        }
    }
}

impl Module {
    pub fn frontend_analysis(
        registry: &mut ModuleRegistry,
        name: String,
        r#type: ModuleType,
        source: &Source,
    ) {
        let mut module = Self::default();
        module.r#type = r#type;

        let mut renderer = DiagnosticRenderer::new(&source);

        let mut parser = Parser::new(&source, &mut module);

        parser.parse();

        if !parser.diagnostics.is_empty() {
            println!("{}", renderer.render(&mut parser.diagnostics));
            return;
        }

        // script
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&mut module);

        module.display();

        // we move the metadata to the registry, as we can retrieve it from there
        // if and when needed.
        registry.register(name, module);
    }
}
