use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::{
    analysis::{Platform, PlatformNamespace},
    common::{EMPTY_SPAN, Span},
    module::{
        Expression, LogicalEventName, LogicalId, ModuleMemberType,
        ModuleMetadata, ModuleRegistry, SlotMembers, SourceEventName, SourceId,
    },
    parser::{
        Attribute, AttributeValue, BindingAttribute, Fragment, FragmentNode,
        RegularAttribute, Tag, TagNamespace,
    },
    utils::{self, normalize_case},
};

pub struct AttributeAnalyzer<'a> {
    registry: &'a ModuleRegistry,
    pub interface_delta: HashMap<String, ModuleMetadata>,
    pub logical_to_source_delta:
        HashMap<String, HashMap<LogicalEventName, SourceEventName>>,
    pub exposed_event_map: HashMap<String, Option<String>>,
    pub slots: HashMap<String, SlotMembers>,
    pub id_table: HashMap<String, HashMap<LogicalId, SourceId>>,
    platform_data: Platform,
}

pub struct ModuleContext<'a> {
    pub name: &'a str,
    pub metadata: &'a ModuleMetadata,
    pub expressions: &'a [Expression],
    pub logical_to_source_map: &'a HashMap<LogicalEventName, SourceEventName>,
}

impl<'a> AttributeAnalyzer<'a> {
    pub fn new(registry: &'a ModuleRegistry) -> Self {
        Self {
            registry,
            interface_delta: HashMap::new(),
            exposed_event_map: HashMap::new(),
            id_table: HashMap::new(),
            slots: HashMap::new(),
            platform_data: Platform::init(),
            logical_to_source_delta: HashMap::new(),
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
        tag: &Tag,
        current_module: &ModuleContext,
        attrib: &BindingAttribute,
        exposed_event_suffix: Option<LogicalEventName>,
    ) {
        let logical_event_name =
            &LogicalEventName(attrib.bind_to.string.clone());

        let bind_target = if let Some(target) = self
            .registry
            .get_module(tag.name.string())
            .unwrap()
            .logical_to_source_event
            .get(&logical_event_name)
        {
            target
        } else if let Some(target) = self
            .logical_to_source_delta
            .get(tag.name.string())
            .unwrap()
            .get(&logical_event_name)
        {
            target
        } else {
            // TODO: diagnostics
            println!(
                "No event `{}` on tag`{}`",
                attrib.bind_to.string,
                tag.name.string()
            );
            return;
        };

        // new events are introduced only on exposition or renaming.
        match &attrib.value {
            Some(value) => {
                // check if value is a lone identifier
                if let AttributeValue::Expression(e) = value {
                    let expr = &current_module.expressions[*e];
                    if expr.is_lone_identifier {
                        // check if this member exists in the current module
                        let event_name = expr.source.string.trim();

                        if current_module.metadata.get(event_name).is_some() {
                            // remap case. nothing to do.
                            println!(
                                "remapping event `{}` to `{}`",
                                attrib.bind_to.string, event_name
                            );
                        } else {
                            // rename case
                            println!(
                                "renaming event `{}` to `{}`. this introduces `{}` into current scope",
                                attrib.bind_to.string, event_name, event_name
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

                            let logical_name =
                                LogicalEventName(utils::normalize_case(
                                    event_name,
                                    utils::Case::SnakeCase,
                                ));

                            self.logical_to_source_delta
                                .get_mut(current_module.name)
                                .unwrap()
                                .insert(
                                    logical_name,
                                    SourceEventName(event_name.to_string()),
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

                let canonical_event_name =
                    format!("{id}_{}", attrib.bind_to.string);

                let event_name = if let Some(event) = exposed_event_suffix {
                    suffix = event.0.clone();

                    let name = format!("{id}_{}", event.0);

                    self.exposed_event_map
                        .insert(canonical_event_name, Some(name.clone()));

                    name
                } else {
                    suffix = attrib.bind_to.string.clone();

                    self.exposed_event_map
                        .insert(canonical_event_name.clone(), None);

                    canonical_event_name
                };

                println!(
                    "exposing event `{}`. this introduces `{event_name}` into the current scope",
                    attrib.bind_to.string.clone()
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

                self.logical_to_source_delta
                    .get_mut(current_module.name)
                    .unwrap()
                    .insert(
                        LogicalEventName(event_name.clone()),
                        SourceEventName(event_name.clone()),
                    );
            }
        }
    }

    fn resolve_binding_attribute(
        &mut self,
        id: Option<&str>,
        tag: &Tag,
        current_module: &ModuleContext,
        attrib: &BindingAttribute,
        exposed_event_suffix: Option<LogicalEventName>,
    ) {
        // event
        if attrib.bind_from.string == "on" {
            self.resolve_event_attribute(
                id,
                tag,
                current_module,
                attrib,
                exposed_event_suffix,
            );

            return;
        }

        if tag.name.namespace.is_none() {
            let bind_target = if let Some(target) = self
                .registry
                .get_module(tag.name.string())
                .unwrap()
                .metadata
                .get(&attrib.bind_from.string)
            {
                target
            } else if let Some(target) = self
                .interface_delta
                .get(tag.name.string())
                .unwrap()
                .get(&attrib.bind_from.string)
            {
                target
            } else {
                // TODO: diagnostics
                println!(
                    "No attribute `{}` on tag`{}`",
                    attrib.bind_from.string,
                    tag.name.string()
                );
                return;
            };

            match bind_target {
                ModuleMemberType::PrivateState(_)
                | ModuleMemberType::PrivateVariable(_) => {
                    // TODO: diagnostics
                    println!(
                        "Member `{}` on tag `{}` is private",
                        attrib.bind_from.string,
                        tag.name.string()
                    );

                    return;
                }

                _ => (),
            }
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
        module_name: &str,
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
                        let source_id = expr.source.string.trim();

                        let id_str = LogicalId(utils::normalize_case(
                            source_id,
                            utils::Case::SnakeCase,
                        ));

                        let id_table =
                            self.id_table.get_mut(module_name).unwrap();

                        if let Some(site) = id_table.get(&id_str) {
                            // TODO: diagnostics
                            println!(
                                "`{id_str:?}` already used at {site:?}. ID must be unique"
                            );
                        } else if let Some(existing) = self
                            .registry
                            .get_module(&module_name.to_string())
                            .unwrap()
                            .metadata
                            .get(&id_str.0)
                        {
                            // TODO: diagnostics
                            println!(
                                "There already exists a module member `{existing:?}` with the same symbol as this id. ID's introduce a refernce of the same name into the module's scope."
                            );
                        }

                        id_table.insert(
                            id_str.clone(),
                            SourceId(source_id.to_string()),
                        );

                        self.interface_delta
                            .get_mut(module_name)
                            .unwrap()
                            .insert(
                                source_id.to_string(),
                                ModuleMemberType::ModuleReference {
                                    source_id: SourceId(source_id.to_string()),
                                    logical_id: id_str.clone(),
                                },
                            );

                        return Some(id_str.0);
                    }
                } else {
                    // TODO: diagnostics
                    println!("ID must be an expression");
                }
            }
        }

        None
    }

    fn visit_platform_tag(
        &mut self,
        current_module: &ModuleContext,
        id: Option<&str>,
        namespace: &PlatformNamespace,
        tag: &Tag,
    ) {
        let element = tag.name.string();

        if !self.platform_data.is_valid_element(namespace, element) {
            // TODO: diagnostics
            println!("Invalid platform element {element}");
            return;
        }

        let mut visited = HashSet::new();

        for attrib in &tag.attributes {
            match attrib {
                Attribute::Regular(attrib) => {
                    if attrib.name.string == "id" {
                        continue;
                    }

                    if visited.contains(&attrib.name.string) {
                        // TODO: diagnostics
                        println!(
                            "Multiple assignments to attribute {}",
                            attrib.name.string
                        );
                        continue;
                    }

                    visited.insert(attrib.name.string.clone());

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
                    /* NOTE: exposed_event_suffix is None because platform events can
                     * NOT exposed events. They are semantically source/terminal events
                     */

                    if attrib.bind_from.string == "on" {
                        if visited.contains(&attrib.bind_to.string) {
                            // TODO: diagnostics
                            println!(
                                "Multiple assignments to attribute {}",
                                attrib.bind_to.string
                            );
                            continue;
                        }

                        visited.insert(attrib.bind_to.string.clone());
                    } else {
                        if visited.contains(&attrib.bind_from.string) {
                            // TODO: diagnostics
                            println!(
                                "Multiple assignments to attribute {}",
                                attrib.bind_from.string
                            );
                            continue;
                        }

                        visited.insert(attrib.bind_from.string.clone());
                    }

                    self.resolve_binding_attribute(
                        id.as_deref(),
                        tag,
                        current_module,
                        attrib,
                        None,
                    );
                }
            }
        }
    }

    fn visit_regular_tag(
        &mut self,
        current_module: &ModuleContext,
        id: Option<&str>,
        tag: &Tag,
    ) {
        // module is guaranteed to exist at this point.
        let widget_module =
            &self.registry.get_module(tag.name.string()).unwrap();

        let mut visited = HashSet::new();

        for attrib in &tag.attributes {
            match attrib {
                Attribute::Regular(attrib) => {
                    if visited.contains(&attrib.name.string) {
                        // TODO: diagnostics
                        println!(
                            "Multiple assignments to attribute {}",
                            attrib.name.string
                        );
                        continue;
                    }

                    visited.insert(attrib.name.string.clone());

                    self.resolve_regular_attribute(
                        &widget_module.metadata,
                        attrib,
                    );
                }

                Attribute::Binding(attrib) => {
                    let exposed_event_suffix = if attrib.bind_from.string
                        == "on"
                    {
                        if visited.contains(&attrib.bind_to.string) {
                            // TODO: diagnostics
                            println!(
                                "Multiple assignments to attribute {}",
                                attrib.bind_to.string
                            );
                            continue;
                        }

                        visited.insert(attrib.bind_to.string.clone());

                        let logical_event_name =
                            &LogicalEventName(attrib.bind_to.string.clone());
                        // obtain the exposed event suffix, if any
                        let event = if let Some(event) = widget_module
                            .logical_to_source_event
                            .get(&logical_event_name)
                        {
                            widget_module.metadata.get(&event.0).unwrap()
                        } else if let Some(event) = self
                            .logical_to_source_delta
                            .get(tag.name.string())
                            .unwrap()
                            .get(&logical_event_name)
                        {
                            self.interface_delta
                                .get(tag.name.string())
                                .unwrap()
                                .get(&event.0)
                                .unwrap()
                        } else {
                            // TODO: diagnostics
                            println!(
                                "No event `{}` on widget",
                                attrib.bind_to.string
                            );

                            continue;
                        };

                        match event {
                            ModuleMemberType::ExposedEvent {
                                event, ..
                            } => Some(LogicalEventName(event.clone())),
                            _ => None,
                        }
                    } else {
                        None
                    };

                    if visited.contains(&attrib.bind_from.string) {
                        // TODO: diagnostics
                        println!(
                            "Multiple assignments to attribute {}",
                            attrib.bind_from.string
                        );
                        continue;
                    }

                    visited.insert(attrib.bind_from.string.clone());

                    self.resolve_binding_attribute(
                        id.as_deref(),
                        tag,
                        current_module,
                        attrib,
                        exposed_event_suffix,
                    )
                }
            }
        }
    }

    fn visit_slot_tag(
        &mut self,
        current_module: &ModuleContext,
        parent_tag: Option<&Tag>,
        tag: &Tag,
    ) {
        let is_declaration = tag.fragment.is_none();

        if parent_tag.is_none() && !is_declaration {
            // TODO: diagnostics
            println!("Slots can only be used inside tags");
            return;
        }

        // if it is a declaration, then we record it.
        if is_declaration {
            if tag.name.span() == EMPTY_SPAN {
                // unnamed declaration.
                if let Some(existing) = self.slots.get_mut(current_module.name)
                {
                    match existing {
                        SlotMembers::Unnamed => {
                            // TODO: diagnostics
                            println!("Multiple unnamed slots");
                        }

                        SlotMembers::Named(_) => {
                            // TODO: diagnostics
                            println!("Cannot use both named and unnamed slots");
                        }
                    }
                } else {
                    self.slots.insert(
                        current_module.name.to_string(),
                        SlotMembers::Unnamed,
                    );
                };
            } else {
                // named declaration
                let entry = if let Some(existing) =
                    self.slots.get_mut(current_module.name)
                {
                    existing
                } else {
                    self.slots.insert(
                        current_module.name.to_string(),
                        SlotMembers::Named(HashMap::new()),
                    );

                    // get mutable reference.
                    self.slots.get_mut(current_module.name).unwrap()
                };

                match entry {
                    SlotMembers::Unnamed => {
                        // TODO: diagnostics
                        println!("Cannot use both named and unnamed slots");
                    }

                    SlotMembers::Named(slots) => {
                        if slots.contains_key(tag.name.string()) {
                            // TODO: diagnostics
                            println!(
                                "Named slot {} already exists",
                                tag.name.string()
                            );
                        } else {
                            slots.insert(
                                tag.name.string().clone(),
                                tag.name.span(),
                            );
                        }
                    }
                }
            }
        } else {
            let Some(parent_tag) = parent_tag else {
                // TODO: diagnostics
                println!("Slot uses may only occur inside some parent tag");
                return;
            };

            if matches!(parent_tag.name.namespace, Some(TagNamespace::Slot)) {
                // TODO: diagnostics
                println!("Cannot nest slots");
                return;
            }

            let slots = self.slots.get(parent_tag.name.string());

            if slots.is_none() {
                // TODO: diagnostics
                println!("Tag {} does not allow slots", parent_tag.name.name);
                return;
            }

            let slots = slots.unwrap();
            match slots {
                SlotMembers::Unnamed => {
                    if tag.name.span() != EMPTY_SPAN {
                        // TODO: diagnostics
                        println!(
                            "Tag {} has an unnamed slot. Cannot use named slot {} to populate it.",
                            parent_tag.name.name, tag.name.name
                        )
                    } else {
                        // TODO: diagnostics
                        println!(
                            "Tag {} has an unnamed slot. Cannot explicitly use <slot>...</slot>",
                            parent_tag.name.name
                        )
                    }
                }

                SlotMembers::Named(slots) => {
                    if !slots.contains_key(tag.name.string()) {
                        // TODO: diagnostics
                        println!(
                            "Tag {} does not contain a slot with name {}",
                            parent_tag.name.name, tag.name.name
                        )
                    }
                }
            }
        }

        if !tag.attributes.is_empty() {
            // TODO: diagnostics
            println!("Slot does not take attributes");
        }
    }

    fn visit_fragment(
        &mut self,
        current_module: &ModuleContext,
        parent_tag: Option<&Tag>,
        fragment: &Fragment,
    ) {
        // we must check if the parent tag allows children
        let allows_named_slots_only = if let Some(parent_tag) = parent_tag {
            match &parent_tag.name.namespace {
                Some(TagNamespace::Platform(namespace)) => {
                    if !self
                        .platform_data
                        .allows_children(&namespace, parent_tag.name.string())
                    {
                        // TODO: diagnostics
                        println!(
                            "Tag {} does not allow children",
                            parent_tag.name.name
                        );
                        return;
                    }

                    false
                }

                _ => {
                    if matches!(
                        parent_tag.name.namespace,
                        Some(TagNamespace::Slot)
                    ) {
                        false
                    } else if let Some(parent_slot) =
                        self.slots.get(parent_tag.name.string())
                    {
                        match parent_slot {
                            SlotMembers::Named(_) => true,
                            _ => false,
                        }
                    } else {
                        // TODO: diagnostics
                        println!(
                            "Tag {} does not allow children",
                            parent_tag.name.name
                        );
                        return;
                    }
                }
            }
        } else {
            false
        };

        for node in &fragment.nodes {
            if let FragmentNode::Tag(tag) = node {
                // first, we must find the id.
                let id = self.collect_id(
                    current_module.name,
                    &current_module.expressions,
                    &tag.attributes,
                );

                match &tag.name.namespace {
                    Some(TagNamespace::Platform(namespace))
                        if !allows_named_slots_only =>
                    {
                        self.visit_platform_tag(
                            current_module,
                            id.as_deref(),
                            namespace,
                            tag,
                        );

                        if let Some(fragment) = &tag.fragment {
                            self.visit_fragment(
                                current_module,
                                Some(tag),
                                fragment,
                            );
                        }
                        continue;
                    }

                    Some(TagNamespace::Slot) => {
                        self.visit_slot_tag(current_module, parent_tag, tag);

                        if id.is_some() {
                            // TODO: diagnostics
                            println!("Slots do not take attributes");
                        }

                        if let Some(fragment) = &tag.fragment {
                            self.visit_fragment(
                                current_module,
                                Some(tag),
                                fragment,
                            );
                        }
                    }

                    Some(TagNamespace::Custom(_))
                        if !allows_named_slots_only =>
                    {
                        // TODO: handle custom namespaces
                        todo!("Custom namespaces are not supported yet");
                    }

                    None if !allows_named_slots_only => {
                        self.visit_regular_tag(
                            current_module,
                            id.as_deref(),
                            tag,
                        );

                        if let Some(fragment) = &tag.fragment {
                            self.visit_fragment(
                                current_module,
                                Some(&tag),
                                fragment,
                            );
                        }
                    }

                    _ => {
                        // TODO: diagnostics
                        println!(
                            "Tag {} allows named slots only",
                            // unwrap parent_tag, because if we hit this branch,
                            // there's no way we are in the global scope, and hence
                            // parent must exist
                            parent_tag.unwrap().name.string()
                        )
                    }
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
            self.id_table
                .insert(module_context.name.to_string(), HashMap::new());

            self.interface_delta
                .insert(module_context.name.to_string(), HashMap::new());

            self.logical_to_source_delta
                .insert(module_context.name.to_string(), HashMap::new());

            // resolve attributes
            self.visit_fragment(module_context, None, root);
        }
    }
}
