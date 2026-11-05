use std::collections::HashMap;

use crate::{
    module::{
        Expression, ModuleMemberType, ModuleMetadata, SymbolUse, SymbolUseType,
    },
    utils,
};

pub struct ExpressionAnalyzer<'a> {
    interface_delta: &'a HashMap<String, ModuleMetadata>,
}

impl<'a> ExpressionAnalyzer<'a> {
    pub fn new(interface_delta: &'a HashMap<String, ModuleMetadata>) -> Self {
        Self { interface_delta }
    }

    pub fn analyze_symbol_uses(
        &self,
        module_name: &str,
        symbol_uses: &mut [SymbolUse],
    ) {
        for symbol_use in symbol_uses {
            // we need to check only the delta interface, as any members that were
            // declared before the semantic pass are already resolved.
            //
            // we skip over local variables and template context variables, as they
            // essentially shallow any resolutions to the global scope
            if !matches!(
                symbol_use.r#type,
                SymbolUseType::Local | SymbolUseType::TemplateContext
            ) && let Some(member) = self
                .interface_delta
                .get(module_name)
                .unwrap()
                .get(&symbol_use.r#use.string)
            {
                match &member {
                    ModuleMemberType::Event(_)
                    | ModuleMemberType::Command(_)
                    | ModuleMemberType::Reactive(_)
                    | ModuleMemberType::Function(_)
                    | ModuleMemberType::RenamedEvent { .. }
                    | ModuleMemberType::ExposedEvent { .. } => {
                        symbol_use.r#type = SymbolUseType::GlobalFunction
                    }

                    ModuleMemberType::PrivateState(_)
                    | ModuleMemberType::PublicState(_) => {
                        symbol_use.r#type = SymbolUseType::GlobalState
                    }

                    ModuleMemberType::StateProxy { from, to } => {
                        println!("binding state proxy {from}, {to}");
                        symbol_use.r#type = SymbolUseType::GlobalState
                    }

                    ModuleMemberType::PrivateVariable(_)
                    | ModuleMemberType::PublicVariable(_) => {
                        symbol_use.r#type = SymbolUseType::GlobalVariable
                    }

                    ModuleMemberType::ModuleReference { .. } => {
                        symbol_use.r#type = SymbolUseType::ModuleReference
                    }

                    _ => {
                        // TODO: diagnostics
                        println!(
                            "Symbol `{}` resolves to {member:?}, which is invalid",
                            symbol_use.r#use.string,
                        );
                    }
                }
            }

            if matches!(
                symbol_use.r#type,
                SymbolUseType::Unknown | SymbolUseType::GlobalUnresolved
            ) {
                // TODO: diagnostics
                println!("Unknown symbol `{}`", symbol_use.r#use.string);
            }
        }
    }

    pub fn analyze(
        &mut self,
        module_name: &str,
        expressions: &mut [Expression],
    ) {
        for expression in expressions {
            self.analyze_symbol_uses(module_name, &mut expression.symbol_uses);
        }
    }
}
