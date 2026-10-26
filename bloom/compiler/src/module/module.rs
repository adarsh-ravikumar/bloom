use std::collections::HashMap;

use crate::{
    common::{ERRONEOUS_SPAN, Source, Span},
    diagnostic::DiagnosticRenderer,
    module::*,
    parser::{Fragment, Parser},
};

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
