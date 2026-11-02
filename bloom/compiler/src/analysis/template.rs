use std::collections::HashMap;

use crate::{
    analysis::Platform,
    common::Span,
    module::{Expression, ModuleMemberType, ModuleMetadata, ModuleRegistry},
    parser::{
        Attribute, AttributeValue, BindingAttribute, Fragment, FragmentNode,
        RegularAttribute,
    },
    utils,
};

pub struct AttributeAnalyzer<'a> {
    registry: &'a ModuleRegistry,
    pub interface_delta: HashMap<String, ModuleMetadata>,
    pub exposed_event_map: HashMap<String, Option<String>>,
    id_set: HashMap<String, Span>,
    platform_data: Platform,
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
            exposed_event_map: HashMap::new(),
            id_set: HashMap::new(),
            platform_data: Platform::init(),
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
        widget_module: &ModuleMetadata,
        attrib: &RegularAttribute,
    ) {
        // the id attribute is already handled for us.
        if &attrib.name.string == "id" {
            println!("encountered ID");
            return;
        }

        let member = widget_module.get(&attrib.name.string);

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

    fn resolve_event_attribute(
        &mut self,
        id: Option<&str>,
        current_module: &ModuleContext,
        canonical_name: &str,
        attrib: &BindingAttribute,
        exposed_event_suffix: Option<String>,
    ) {
        // new events are introduced only on exposition or renaming.
        match &attrib.value {
            Some(value) => {
                // check if value is a lone identifier
                if let AttributeValue::Expression(e) = value {
                    let expr = &current_module.expressions[*e];
                    if expr.is_lone_identifier {
                        // check if this member exists in the current module
                        let event_name = utils::normalize_case(
                            expr.source.string.trim(),
                            utils::Case::SnakeCase,
                        );

                        if current_module.metadata.get(&event_name).is_some() {
                            // remap case. nothing to do.
                            println!(
                                "remapping event `{}` to `{}`",
                                attrib.bind_to.string, event_name
                            );
                        } else {
                            // rename case
                            println!(
                                "renaming event `{}` to `{}`. this introduces `{}` into current scope",
                                canonical_name, event_name, event_name
                            );

                            if let Some(member) =
                                self.member_exists(current_module, &event_name)
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
                } else {
                    // TODO: diagnostics
                    println!(
                        "Binding value for an event must be an expression"
                    );
                }
            }

            None => {
                if id.is_none() {
                    println!("ID is required to expose an event");
                }

                let id = id.as_ref().unwrap();

                let suffix;

                /* NOTE: the canonical name is derived directly from the source code.
                 * but, the final exposed event name can either arise from <id>_<canonical name>
                 * or <id>_<event>. the latter cannot be infered by the IR pass. hence, we
                 * populate a map from <id>_<canonical name> to the final event name.
                 */

                let canonical_event_name = format!("{id}_{canonical_name}");

                let event_name = if let Some(event) = exposed_event_suffix {
                    suffix = event.to_string();
                    let name = format!("{id}_{event}");

                    self.exposed_event_map
                        .insert(canonical_event_name, Some(name.clone()));

                    name
                } else {
                    suffix = canonical_name.to_string();

                    self.exposed_event_map
                        .insert(canonical_event_name.clone(), None);

                    canonical_event_name
                };

                println!(
                    "exposing event `{canonical_name}`. this introduces `{event_name}` into the current scope",
                );

                if let Some(member) =
                    self.member_exists(current_module, &event_name)
                {
                    // TODO: diagnostics
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
                            event: suffix,
                        },
                    );
            }
        }
    }

    fn resolve_binding_attribute(
        &mut self,
        id: Option<&str>,
        current_module: &ModuleContext,
        canonical_name: &str,
        attrib: &BindingAttribute,
        exposed_event_suffix: Option<String>,
    ) {
        // event
        if attrib.bind_from.string == "on" {
            self.resolve_event_attribute(
                id,
                current_module,
                canonical_name,
                attrib,
                exposed_event_suffix,
            );

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
        expressions: &[Expression],
        attributes: &[Attribute],
    ) -> Option<String> {
        for (i, attrib) in attributes.iter().enumerate() {
            if let Attribute::Regular(attrib) = &attrib
                && attrib.name.string == "id"
            {
                if let AttributeValue::Expression(expr) = &attrib.value {
                    let expr = &expressions[*expr];

                    if !expr.is_lone_identifier {
                        // TODO: diagnostics
                        println!("ID must only be an identifier");
                    } else if i != 0 {
                        // TODO: diagnostics
                        println!("ID must be the first attribute");
                    } else {
                        let id_str = utils::normalize_case(
                            expr.source.string.trim(),
                            utils::Case::SnakeCase,
                        );

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
                // first, we must find the id.
                let id =
                    self.collect_id(&current_module.expressions, &w.attributes);

                if w.is_platform {
                    let namespace = w.platform_namespace.as_ref().unwrap();

                    let element = &w.name.string;
                    if !self.platform_data.is_valid_element(namespace, element)
                    {
                        // TODO: diagnostics
                        println!("Invalid platform element {element}");
                        continue;
                    }

                    for attrib in &w.attributes {
                        match attrib {
                            Attribute::Regular(attrib) => {
                                if attrib.name.string == "id" {
                                    continue;
                                }

                                if !self.platform_data.is_valid_attribute(
                                    namespace,
                                    element,
                                    &attrib.name.string,
                                ) {
                                    // TODO: diagnostics
                                    println!(
                                        "Invalid attribute {} on {element}",
                                        attrib.name.string
                                    );
                                }
                            }

                            Attribute::Binding(attrib) => {
                                let canonical_name = utils::normalize_case(
                                    &attrib.bind_to.string,
                                    utils::Case::SnakeCase,
                                );

                                // NOTE: exposed_event_suffix is None because platform events are
                                // NOT exposed events. They are semantically source/terminal events
                                self.resolve_binding_attribute(
                                    id.as_deref(),
                                    current_module,
                                    &canonical_name,
                                    attrib,
                                    None,
                                );
                            }
                        }
                    }

                    if let Some(fragment) = &w.fragment {
                        self.visit_fragment(current_module, fragment);
                    }

                    continue;
                }

                // module is guaranteed to exist at this point.
                let widget_module_metadata =
                    &self.registry.get_module(&w.name.string).unwrap().metadata;

                for attrib in &w.attributes {
                    match attrib {
                        Attribute::Regular(attrib) => self
                            .resolve_regular_attribute(
                                &widget_module_metadata,
                                attrib,
                            ),

                        Attribute::Binding(attrib) => {
                            let canonical_name = utils::normalize_case(
                                &attrib.bind_to.string,
                                utils::Case::SnakeCase,
                            );

                            let event = if let Some(event) =
                                widget_module_metadata.get(&canonical_name)
                            {
                                Some(event)
                            } else if let Some(event) = self
                                .interface_delta
                                .get(&w.name.string)
                                .unwrap()
                                .get(&canonical_name)
                            {
                                Some(event)
                            } else {
                                None
                            };

                            if event.is_none() {
                                // TODO: diagnostics
                                println!(
                                    "No event `{}` on widget",
                                    attrib.bind_to.string
                                );

                                continue;
                            }

                            let event = event.unwrap();

                            let exposed_event_suffix = match event {
                                ModuleMemberType::ExposedEvent {
                                    event,
                                    ..
                                } => Some(event.clone()),
                                _ => None,
                            };

                            self.resolve_binding_attribute(
                                id.as_deref(),
                                current_module,
                                &canonical_name,
                                attrib,
                                exposed_event_suffix,
                            )
                        }
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
            self.id_set.clear();

            self.interface_delta
                .insert(module_context.name.to_string(), HashMap::new());

            // resolve attributes
            self.visit_fragment(module_context, root);
        }
    }
}
