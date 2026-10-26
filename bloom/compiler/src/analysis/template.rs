use std::mem;

use crate::{
    common::SpannedString,
    module::{Module, ModuleMemberType, ModuleRegistry},
    parser::{Attribute, AttributeValue, Fragment, FragmentNode},
};

pub struct TemplateAnalyzer<'a> {
    registry: &'a ModuleRegistry,
}

impl<'a> TemplateAnalyzer<'a> {
    pub fn new(registry: &'a ModuleRegistry) -> Self {
        // i don't really think this needs to be a struct that
        // holds state? let it be for now, but i'll probably refactor
        Self { registry }
    }

    fn resolve_regular_attribute(
        &mut self,
        widget: &SpannedString,
        name: &SpannedString,
        value: &AttributeValue,
    ) {
        // if we are dealing with an id attribute, we needn't access the module from the registry.
        if &name.string == "id" {
            println!("encountered ID");
            return;
        }

        // we can unwrap, because missing dependencies are already handled in the prev. stage
        let module = self.registry.get_module(&widget.string).unwrap();

        let member = module.metadata.get(&name.string);

        if member.is_none() {
            // TODO: diagnostics
            println!(
                "Unknown attribute `{}` on widget `{}`",
                name.string, widget.string
            );

            return;
        }

        let member = member.unwrap();

        match member {
            ModuleMemberType::PrivateState(state) => {
                // TODO: diagnostics
                println!("State is private")
            }

            ModuleMemberType::PrivateVariable(variable) => {
                // TODO diagnostics
                println!("Variable is private")
            }

            ModuleMemberType::PublicState(state) => {
                println!("Assigning to state `{:?}`", state.span);
            }

            ModuleMemberType::PublicVariable(variable) => {
                println!("Assigning to variable `{:?}`", variable.span);
            }

            _ => {
                // TODO: unique diagnostics for each case.
                println!("Cannot assign to this member");
            }
        }
    }

    fn resolve_binding_attribute(
        &mut self,
        widget: &SpannedString,
        bind_from: &SpannedString,
        bind_to: &Option<SpannedString>,
        value: &Option<AttributeValue>,
    ) {
    }

    fn visit_fragment(&mut self, fragment: &Fragment) {
        for node in &fragment.nodes {
            if let FragmentNode::Widget(w) = node {
                for attrib in &w.attributes {
                    match attrib {
                        Attribute::Regular { name, value } => {
                            self.resolve_regular_attribute(&w.name, name, value)
                        }

                        Attribute::Binding {
                            bind_from,
                            bind_to,
                            value,
                        } => self.resolve_binding_attribute(
                            &w.name, bind_from, bind_to, value,
                        ),
                    }
                }

                if let Some(fragment) = &w.fragment {
                    self.visit_fragment(fragment);
                }
            }
        }
    }

    pub fn visit_module(&mut self, module: &Module) {
        if module.root.is_none() {
            return;
        }

        if let Some(root) = &module.root {
            self.visit_fragment(root);
        }
    }
}
