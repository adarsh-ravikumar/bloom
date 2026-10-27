use std::collections::HashMap;

use crate::module::{Expression, ModuleMetadata};

pub struct ExpressionAnalyzer<'a> {
    interface_delta: &'a HashMap<String, ModuleMetadata>,
}

impl<'a> ExpressionAnalyzer<'a> {
    pub fn new(interface_delta: &'a HashMap<String, ModuleMetadata>) -> Self {
        Self { interface_delta }
    }
    pub fn analyze(
        &mut self,
        module_name: &str,
        expressions: &mut [Expression],
    ) {
        for expression in expressions {
            for symbol_use in &mut expression.symbol_uses {
                // we need to check only the delta interface, as any members that were
                // declared before the semantic pass are already resolved.

                let member_exists = self
                    .interface_delta
                    .get(module_name)
                    .unwrap()
                    .contains_key(&symbol_use.r#use.string);

                if member_exists
                    && !(symbol_use.is_template_context_use
                        || symbol_use.is_global_use)
                {
                    symbol_use.is_global_use = true;
                }
            }
        }
    }
}
