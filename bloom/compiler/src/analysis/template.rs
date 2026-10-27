use std::collections::HashMap;

use crate::{
    common::Span,
    module::{Expression, ModuleMemberType, ModuleMetadata, ModuleRegistry},
    parser::{
        Attribute, AttributeValue, BindingAttribute, Fragment, FragmentNode,
        RegularAttribute,
    },
};

pub struct AttributeAnalyzer<'a> {
    registry: &'a ModuleRegistry,
    pub interface_delta: HashMap<String, ModuleMetadata>,
    id_set: HashMap<String, Span>,
}

pub struct ModuleContext<'a> {
    pub name: &'a str,
    pub metadata: &'a ModuleMetadata,
    pub expressions: &'a [Expression],
}

impl<'a> AttributeAnalyzer<'a> {
    pub fn new(registry: &'a ModuleRegistry) -> Self {
        // i don't really think this needs to be a struct that
        // holds state? let it be for now, but i'll probably refactor
        Self {
            registry,
            interface_delta: HashMap::new(),
            id_set: HashMap::new(),
        }
    }

    fn member_exists<'b>(
        &'b self,
        module: &'b ModuleContext,
        id: &str,
    ) -> Option<&'b ModuleMemberType> {
        if let Some(member) = module.metadata.get(id) {
            Some(member)
        } else if let Some(member) =
            self.interface_delta.get(module.name).unwrap().get(id)
        {
            Some(member)
        } else {
            None
        }
    }

    fn resolve_regular_attribute(
        &mut self,
        widget_module: &Option<&ModuleMetadata>,
        attrib: &RegularAttribute,
    ) {
        // if we are dealing with an id attribute, we needn't access the module from the registry.
        if &attrib.name.string == "id" {
            println!("encountered ID");
            return;
        }

        // TODO: Handle platform widgets
        if widget_module.is_none() {
            return;
        }

        let module = widget_module.as_ref().unwrap();

        let member = module.get(&attrib.name.string);

        if member.is_none() {
            // TODO: diagnostics
            println!("Unknown attribute `{}` on widget", attrib.name.string,);

            return;
        }

        let member = member.unwrap();

        match member {
            ModuleMemberType::PrivateState(_) => {
                // TODO: diagnostics
                println!("State is private")
            }

            ModuleMemberType::PrivateVariable(_) => {
                // TODO: diagnostics
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
        id: &Option<String>,
        current_module: &ModuleContext,
        widget_module: &Option<&ModuleMetadata>,
        attrib: &BindingAttribute,
    ) {
        // TODO: Handle platform widgets
        if widget_module.is_none() {
            return;
        }

        let widget_module = widget_module.as_ref().unwrap();

        // event
        if attrib.bind_from.string == "on" {
            let event = widget_module.get(&attrib.bind_to.string);

            if event.is_none() {
                // TODO: diagnostics
                println!("No event `{}` on widget", attrib.bind_to.string);
                return;
            }

            // new events are introduced only on exposition or renaming.
            match &attrib.value {
                Some(value) => {
                    // check if value is a lone identifier
                    if let AttributeValue::Expression(e) = value {
                        let expr = &current_module.expressions[*e];
                        if expr.is_lone_identifier {
                            // check if this member exists in the current module
                            let event_name = expr.source.string.trim();

                            if current_module.metadata.get(event_name).is_some()
                            {
                                // remap case. nothing to do.
                                println!(
                                    "remapping event `{}` to `{}`",
                                    attrib.bind_to.string, event_name
                                );
                            } else {
                                // rename case
                                println!(
                                    "renaming event `{}` to `{}`. this introduces `{}` into current scope",
                                    attrib.bind_to.string,
                                    event_name,
                                    event_name
                                );

                                if let Some(member) = self
                                    .member_exists(current_module, event_name)
                                {
                                    println!(
                                        "Cannot rename event to `{event_name}`. Member already exists `{member:?}`",
                                    );
                                    return;
                                }

                                self.interface_delta
                                    .get_mut(current_module.name)
                                    .unwrap()
                                    .insert(
                                        event_name.to_string(),
                                        ModuleMemberType::RenamedEvent {
                                            from: attrib.bind_to.string.clone(),
                                            to: event_name.to_string(),
                                        },
                                    );
                            }
                        }
                    }
                }

                None => {
                    if id.is_none() {
                        println!("ID is required to expose an event");
                    }

                    let id = id.as_ref().unwrap();

                    println!(
                        "exposing event `{}`. this introduces `{id}_{}` into the current scope",
                        attrib.bind_to.string, attrib.bind_to.string
                    );

                    let event_name = format!("{id}_{}", attrib.bind_to.string);

                    if let Some(member) =
                        self.member_exists(current_module, &event_name)
                    {
                        println!(
                            "Cannot introduce new event `{event_name}`. Member already exists `{member:?}`",
                        );
                        return;
                    }

                    self.interface_delta
                        .get_mut(current_module.name)
                        .unwrap()
                        .insert(
                            event_name.clone(),
                            ModuleMemberType::ExposedEvent {
                                prefix: id.to_string(),
                                event: event_name,
                            },
                        );
                }
            }

            return;
        }

        // state
        if let Some(member) =
            self.member_exists(current_module, &attrib.bind_to.string)
        {
            println!(
                "Member `{}` exists. Attempt to bind.",
                attrib.bind_to.string
            );

            // NOTE: since bindings are implicit in nature, it would be nice to have a warning
            // that tells you that a binding of this name already exists, and you have bound
            // something to it already
            match member {
                // can bind to public and private state. nothing to do.
                ModuleMemberType::PublicState(_)
                | ModuleMemberType::PrivateState(_) => (),

                // NOTE: this implementation doesn't check if the particular binding was already promoted,
                // but it is an optimization, not a correctness concern
                ModuleMemberType::PrivateVariable(v) => {
                    // promote to private state
                    let mut v = v.clone();
                    v.is_state = true;

                    self.interface_delta
                        .get_mut(current_module.name)
                        .unwrap()
                        .insert(
                            attrib.bind_to.string.clone(),
                            ModuleMemberType::PrivateState(v),
                        );
                }

                ModuleMemberType::PublicVariable(v) => {
                    // promote to public state
                    let mut v = v.clone();
                    v.is_state = true;

                    self.interface_delta
                        .get_mut(current_module.name)
                        .unwrap()
                        .insert(
                            attrib.bind_to.string.clone(),
                            ModuleMemberType::PublicState(v),
                        );
                }

                _ => {
                    // TODO: diagnostics
                    println!("Cannot bind state to this member `{member:?}`");
                }
            }

            return;
        }

        self.interface_delta
            .get_mut(current_module.name)
            .unwrap()
            .insert(
                attrib.bind_to.string.clone(),
                ModuleMemberType::StateProxy {
                    from: attrib.bind_from.string.clone(),
                    to: attrib.bind_to.string.clone(),
                },
            );

        println!("new state introduced: {}", attrib.bind_to.string);
    }

    fn collect_id(
        &mut self,
        current_module: &ModuleContext,
        attributes: &Vec<Attribute>,
    ) -> Option<String> {
        for (i, attrib) in attributes.iter().enumerate() {
            if let Attribute::Regular(attrib) = &attrib
                && attrib.name.string == "id"
            {
                if let AttributeValue::Expression(expr) = &attrib.value {
                    let expr = &current_module.expressions[*expr];

                    if !expr.is_lone_identifier {
                        // TODO: diagnostics
                        println!("ID must only be an identifier");
                    } else if i != 0 {
                        // TODO: diagnostics
                        println!("ID must be the first attribute");
                    } else {
                        let id_str = expr.source.string.trim().to_string();

                        println!("id: {id_str}");
                        if let Some(site) = self.id_set.get(&id_str) {
                            println!(
                                // TODO: diagnostics
                                "`{id_str}` already used at {site:?}. ID must be unique"
                            );
                        }

                        self.id_set.insert(id_str.clone(), expr.source.span);
                        return Some(id_str);
                    }
                } else {
                    // TODO: diagnostics
                    println!("ID must be an expression");
                }
            }
        }

        None
    }

    fn visit_fragment(
        &mut self,
        current_module: &ModuleContext,
        fragment: &Fragment,
    ) {
        for node in &fragment.nodes {
            if let FragmentNode::Widget(w) = node {
                let widget_module_metadata = if w.is_platform {
                    // TODO: handle platform widgets
                    None
                } else {
                    self.registry
                        .get_module(&w.name.string)
                        .map(|w| &w.metadata)
                };

                // first, we must find the id.
                let id = self.collect_id(current_module, &w.attributes);

                for attrib in &w.attributes {
                    match attrib {
                        Attribute::Regular(attrib) => self
                            .resolve_regular_attribute(
                                &widget_module_metadata,
                                attrib,
                            ),

                        Attribute::Binding(attrib) => self
                            .resolve_binding_attribute(
                                &id,
                                current_module,
                                &widget_module_metadata,
                                attrib,
                            ),
                    }
                }

                if let Some(fragment) = &w.fragment {
                    self.visit_fragment(current_module, fragment);
                }
            }
        }
    }

    pub fn visit_module(
        &mut self,
        root: &Option<Fragment>,
        module_context: &ModuleContext,
    ) {
        if root.is_none() {
            return;
        }

        if let Some(root) = &root {
            println!("\nAnalyzing Module");
            self.id_set.clear();

            self.interface_delta
                .insert(module_context.name.to_string(), HashMap::new());

            // resolve attributes
            self.visit_fragment(module_context, root);
        }
    }
}

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
