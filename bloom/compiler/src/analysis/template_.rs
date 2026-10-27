use std::collections::HashSet;

use crate::{
    common::SpannedString,
    module::{Module, ModuleMemberType, ModuleRegistry},
    parser::{Attribute, AttributeValue, Fragment, FragmentNode, Part, Widget},
};

pub struct AttributeAnalyzer<'a> {
    registry: &'a ModuleRegistry,
    current_module: Option<&'a Module>,
    ids: HashSet<&'a str>,
}

impl<'a> AttributeAnalyzer<'a> {
    fn visit_fragment(&self, frag: &Fragment) {
        for node in &frag.nodes {
            if let FragmentNode::Widget(w) = node {
                self.visit_widget(w);
            }
        }
    }

    // TODO: find the target module, and attempt matching attributes on that
    // binding targets are to be checked in the current module

    fn validate_regular_attribute(
        &self,
        attrib: Option<&ModuleMemberType>,
        name: &SpannedString,
    ) {
        if attrib.is_none() {
            // TODO: diagnostics
            println!("Attribute {} does not exist", name.string);
            return;
        }

        match attrib.unwrap() {
            ModuleMemberType::PublicState(_) => {
                // allowed. nothing to do.
                return;
            }

            // dissallowed
            _ => {
                // TODO: diagnostics
                println!("Attempt to assign to invalid attribute");
                return;
            } // ModuleMemberType::Event(_) => {}
              // ModuleMemberType::Command(_) => {}
              // ModuleMemberType::Reactive(_) => {}
              // ModuleMemberType::Function(_) => {}
              //
              // ModuleMemberType::PrivateState(_) => {}
              // ModuleMemberType::PublicVariable(_) => {}
              // ModuleMemberType::PrivateVariable(_) => {}
              //
              // ModuleMemberType::Enum(_) => {}
              // ModuleMemberType::Alias(_) => {}
              // ModuleMemberType::Interface(_) => {}
        }
    }

    fn visit_widget(&self, widget: &Widget) {
        // for the first version, we will only check the existence of the attributes
        // we are NOT going to perform type analysis on the attribute value

        let target_module = self.get_module(&widget.name.string);

        for attr in &widget.attributes {
            match attr {
                Attribute::Regular { name, value } => {
                    if name.string == "id" {
                        if let AttributeValue::Parts(id) = &value {
                            if id.len() == 0 {
                                // TODO: diagnostics
                                println!("ID cannot be empty")
                            }

                            if id.len() > 1 {
                                // TODO: diagnostics
                                println!("Too many parts.")
                            }

                            let first_part = id.first().unwrap();

                            if id.len() == 1
                                && let Part::Text(id) = first_part
                            {
                                if self.ids.contains(id.as_str()) {
                                    // TODO: diagnostics
                                    println!("Duplicate id");
                                }
                            } else {
                                println!("ID must be a string")
                            }
                        }
                    }

                    let attrib = target_module.metadata.get(&name.string);
                    self.validate_regular_attribute(attrib, name);
                }

                Attribute::Binding {
                    bind_from,
                    bind_to,
                    value,
                } => {
                    // event binding
                    if bind_from.string == "on" {
                        let attrib =
                            target_module.metadata.get(&bind_to.string);

                        if attrib.is_none() {
                            // TODO: diagnostics
                            println!("Event {} does not exist", bind_to.string);
                            continue;
                        }

                        match attrib.unwrap() {
                            ModuleMemberType::Event(_) => {
                                // look at the value.
                                // if the value is none, then we have an exposition. id required.
                                // else, if the value is an identifier, it could either be a remap
                                // or a rename.
                                // if the value is an expression, then that gets used as is.
                                //
                                // for the first pass, we'll only track None and Some

                                if value.is_none() {}
                            }

                            // dissallowed
                            _ => {
                                // TODO: diagnostics
                                println!("{} is not an event", bind_to.string);
                                continue;
                            }
                        }
                    }

                    let attrib = target_module.metadata.get(&bind_from.string);

                    self.validate_regular_attribute(attrib, bind_from);

                    // let target = module.metadata;
                }
            }
        }
    }

    pub fn new(registry: &'a ModuleRegistry) -> Self {
        Self {
            registry,
            current_module: None,
            ids: HashSet::new(),
        }
    }

    fn get_module(&self, module: &String) -> &'a Module {
        self.registry.get_module(module).unwrap() // guaranteed to exist
    }

    pub fn analyze(&mut self) {
        // we will analyze all the modules
        for module in self.registry.modules() {
            self.current_module = Some(self.get_module(module));
            self.ids.clear();

            if let Some(root) = &self.current_module.unwrap().root {
                self.visit_fragment(root);
            }
        }
    }
}
