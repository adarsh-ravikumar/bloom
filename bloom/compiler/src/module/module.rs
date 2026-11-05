use std::collections::{HashMap, HashSet};

use crate::{
    common::{EMPTY_SPAN, Source, Span},
    diagnostic::DiagnosticRenderer,
    module::{
        Expression, ExpressionId, ModuleMetadata, ModuleRegistry, SymbolUse,
    },
    parser::{Fragment, Parser},
};

#[derive(Debug, Clone, Copy)]
pub enum ModuleType {
    Widget,
    Panel,
}

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct LogicalEventName(pub String);

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct SourceEventName(pub String);

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct LogicalId(pub String);

#[derive(Debug, Eq, Hash, PartialEq, Clone)]
pub struct SourceId(pub String);

#[derive(Debug)]
pub struct Module {
    pub root: Option<Fragment>,
    pub r#type: ModuleType,
    pub script: (Span, String),
    pub name: String,
    pub metadata: ModuleMetadata,
    pub dependencies: HashMap<String, Span>, // store the span of the first occurance
    pub symbol_uses: Vec<SymbolUse>,
    pub expressions: Vec<Expression>,
    pub has_slots: bool,
    pub named_slots: HashMap<String, Span>,
    pub logical_to_source_event: HashMap<LogicalEventName, SourceEventName>,
    pub logical_to_source_id: HashMap<LogicalId, SourceId>,
}

impl Default for Module {
    fn default() -> Self {
        Self {
            root: None,
            r#type: ModuleType::Widget,
            script: (EMPTY_SPAN, String::new()),
            metadata: ModuleMetadata::new(),
            name: String::new(),
            dependencies: HashMap::new(),
            symbol_uses: Vec::new(),
            expressions: Vec::new(),
            has_slots: false,
            named_slots: HashMap::new(),
            logical_to_source_event: HashMap::new(),
            logical_to_source_id: HashMap::new(),
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
        module.name = name.clone();

        let mut renderer = DiagnosticRenderer::new(&source);

        let mut parser = Parser::new(&source, &mut module);

        parser.parse();

        if !parser.diagnostics.is_empty() {
            println!("{}", renderer.render(&mut parser.diagnostics));
            return;
        }

        module.display();

        // we move the metadata to the registry, as we can retrieve it from there when needed.
        registry.register(name, module);
    }

    pub fn insert_expression(&mut self, expr: Expression) -> ExpressionId {
        let id = self.expressions.len();
        self.expressions.push(expr);

        id
    }

    pub fn get_expression(&self, id: ExpressionId) -> &Expression {
        self.expressions.get(id).unwrap()
    }
}
