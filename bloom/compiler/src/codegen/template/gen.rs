use core::slice;
use std::fmt::{Write, write};

use std::collections::HashMap;

use crate::module::SlotMembers;
use crate::{
    common::{EMPTY_SPAN, Span, SpannedString},
    ir::IrNode,
    module::{
        Expression, ModuleMemberType, ModuleMetadata, SymbolUse, SymbolUseType,
    },
    parser::TagNamespace,
    utils,
};

pub struct GenerateTemplateJs<'a> {
    ir: &'a [IrNode],
    metadata: &'a ModuleMetadata,
    symbol_uses: &'a Vec<SymbolUse>,
    expressions: &'a [Expression],
    module_name: &'a str,
    slot: Option<&'a SlotMembers>,
    buffer: String,
    exposed_event_map: &'a HashMap<String, Option<String>>,
    capture_widget_reference: Option<(String, usize)>,
}

macro_rules! write_buffer {
    ($this:ident, $($arg:tt)*) => {
       write!(&mut $this.buffer, $($arg)*).expect("write to buffer failed")
    };
}

impl<'a> GenerateTemplateJs<'a> {
    pub fn new(
        ir: &'a [IrNode],
        module_name: &'a str,
        metadata: &'a ModuleMetadata,
        symbol_uses: &'a Vec<SymbolUse>,
        expressions: &'a [Expression],
        exposed_event_map: &'a HashMap<String, Option<String>>,
        slot: Option<&'a SlotMembers>,
    ) -> Self {
        Self {
            ir,
            module_name,
            metadata,
            symbol_uses,
            expressions,
            buffer: String::new(),
            exposed_event_map,
            capture_widget_reference: None,
            slot,
        }
    }

    fn rewrite_js(
        &self,
        source: &String,
        sites: &[SymbolUse],
        span_offset: usize,
    ) -> String {
        let mut res = String::new();

        let mut prev_use_end = 0usize;

        for r#use in sites {
            let span_str = &r#use.r#use;
            let span = Span::new(
                span_str.span.start - span_offset,
                span_str.span.end - span_offset,
            );

            // push string before use
            res.push_str(&source[prev_use_end..span.start]);

            if r#use.object_property_shorthand {
                res.push_str(&span_str.string);
                res.push_str(": ");
            }

            // transform the use
            match r#use.r#type {
                SymbolUseType::GlobalState => {
                    res.push_str("this.");

                    let state_name = utils::normalize_case(
                        &span_str.string,
                        utils::Case::CamelCase,
                    );

                    res.push_str(&state_name);

                    res.push_str(".value");
                }

                SymbolUseType::GlobalVariable
                | SymbolUseType::ModuleReference
                | SymbolUseType::GlobalFunction => {
                    res.push_str("this.");

                    // let name = utils::normalize_case(
                    //     &span_str.string,
                    //     utils::Case::CamelCase,
                    // );

                    res.push_str(&span_str.string);
                }

                SymbolUseType::TemplateContext => {
                    let state_name = utils::normalize_case(
                        &span_str.string,
                        utils::Case::CamelCase,
                    );

                    res.push_str(&state_name);

                    res.push_str(".value");
                }

                SymbolUseType::Local | SymbolUseType::Unknown => {
                    res.push_str(&span_str.string);
                }

                SymbolUseType::GlobalUnresolved => unreachable!(),
            }

            prev_use_end = span.end;
        }

        // flush what's left
        res.push_str(&source[prev_use_end..]);

        res
    }

    fn rewrite_expression(&self, expr_id: usize) -> String {
        let expr = &self.expressions[expr_id];
        let source = &expr.source.string;

        if expr.symbol_uses.is_empty() {
            source.clone()
        } else {
            self.rewrite_js(source, &expr.symbol_uses, 0)
        }
    }

    fn rewrite_span_in_script(&self, source: &SpannedString) -> String {
        let mut l = 0usize;
        let mut r = self.symbol_uses.len();

        // find first use with start >= source.span.start
        while l < r {
            let mid = l + (r - l) / 2;

            if self.symbol_uses[mid].r#use.span.start < source.span.start {
                l = mid + 1;
            } else {
                r = mid;
            }
        }

        let start = l;

        let mut l = start;
        let mut r = self.symbol_uses.len();

        // find first use with start > source.span.end
        while l < r {
            let mid = l + (r - l) / 2;

            if self.symbol_uses[mid].r#use.span.start <= source.span.end {
                l = mid + 1;
            } else {
                r = mid;
            }
        }

        let end = l;

        let symbol_uses = &self.symbol_uses[start..end];

        if symbol_uses.is_empty() {
            source.string.clone()
        } else {
            self.rewrite_js(&source.string, symbol_uses, source.span.start)
        }
    }

    fn generate_attribute_value<'b>(
        &mut self,
        ir: &mut slice::Iter<'b, IrNode>,
    ) {
        let mut in_parts = false;

        loop {
            let value = ir.next().expect("attribute value missing");

            match value {
                IrNode::StartParts { .. } => {
                    in_parts = true;
                    write_buffer!(self, "new Derived(() => `");
                }

                IrNode::EndParts { .. } => {
                    in_parts = false;
                    write_buffer!(self, "`)");
                }

                IrNode::Text { value } => {
                    write_buffer!(self, "{value}");
                }

                IrNode::Expression { expr_id } => {
                    let expr = self.rewrite_expression(*expr_id);

                    if in_parts {
                        write_buffer!(self, "${{{expr}}}");
                    } else {
                        write_buffer!(self, "new Derived(() => {expr})");
                    }
                }

                _ => {
                    panic!("invalid attribute value");
                }
            }

            if !in_parts {
                break;
            }
        }
    }

    fn generate_build(&mut self) {
        let mut ir = self.ir.iter();

        while let Some(node) = ir.next() {
            match node {
                IrNode::StartParts { id } => {
                    write_buffer!(self, "const w{id} = new Parts([");
                }

                IrNode::EndParts { id, parent } => {
                    if *parent == 0 {
                        write_buffer!(self, "]);\nthis.appendChild(w{id});\n");
                    } else {
                        write_buffer!(
                            self,
                            "]);\nw{parent}.appendChild(w{id});\n"
                        );
                    }
                }

                IrNode::Text { value } => {
                    write_buffer!(self, "`{value}`,");
                }

                IrNode::Expression { expr_id } => {
                    let expr = self.rewrite_expression(*expr_id);

                    write_buffer!(self, "new Derived(() => `${{{expr}}}`),",);
                }

                IrNode::StartTag { name, id } => {
                    write_buffer!(self, "const w{id} = ",);

                    match &name.namespace {
                        Some(TagNamespace::Platform(_)) => {
                            write_buffer!(
                                self,
                                "new Platform(\"{}\");\n",
                                name.string()
                            );
                        }

                        _ => {
                            write_buffer!(
                                self,
                                "new {}();\n",
                                utils::normalize_case(
                                    name.string(),
                                    utils::Case::PascalCase
                                )
                            );
                        }
                    }
                }

                IrNode::UseSlot { name, id, parent } => {
                    write_buffer!(self, "const w{id} = ",);

                    if name.span() == EMPTY_SPAN {
                        write_buffer!(self, "w{parent}.slot;\n",);
                    } else {
                        write_buffer!(
                            self,
                            "w{parent}.slot[\"{}\"];\n",
                            name.string()
                        );
                    }
                }

                IrNode::EndTag { id, parent } => {
                    write_buffer!(self, "w{id}.build();\n");

                    if let Some((source_id, id)) =
                        &self.capture_widget_reference
                    {
                        write_buffer!(
                            self,
                            "this.{source_id} = w{id}.interface;\n"
                        );

                        self.capture_widget_reference = None;
                    }

                    if *parent == 0 {
                        write_buffer!(self, "this.appendChild(w{id});\n");
                    } else {
                        write_buffer!(self, "w{parent}.appendChild(w{id});\n");
                    }
                }

                IrNode::StartAttributeList => {
                    continue;
                }

                IrNode::EndAttributeList => {
                    continue;
                }

                IrNode::AttributeName { name, id } => {
                    let name =
                        utils::normalize_case(name, utils::Case::CamelCase);

                    if name == "id" {
                        let IrNode::Expression { expr_id } = ir.next().unwrap()
                        else {
                            panic!("template id must be an expression")
                        };

                        let source_id =
                            &self.expressions[*expr_id].source.string;

                        let logical_id = utils::normalize_case(
                            &self.expressions[*expr_id].source.string,
                            utils::Case::SnakeCase,
                        );

                        write_buffer!(
                            self,
                            "w{id}.templateId = \"{logical_id}\";\n",
                        );

                        self.capture_widget_reference =
                            Some((source_id.clone(), *id));

                        continue;
                    }

                    write_buffer!(self, "w{id}.setAttribute(\"{name}\", ");

                    self.generate_attribute_value(&mut ir);
                    write_buffer!(self, ");\n");
                }

                IrNode::AttributeBindEvent { event, id } => {
                    let IrNode::Expression { expr_id } =
                        ir.next().expect("Expected value on event binding")
                    else {
                        panic!("Expected expression for event binding")
                    };

                    let expr = self.rewrite_expression(*expr_id);

                    write_buffer!(
                        self,
                        "$._cleanup.push(w{id}.events.on(\"{event}\", (e) => {{(\n{expr}\n)(e);\n}}));\n"
                    );
                }

                IrNode::AttributeExposeEvent {
                    source_event,
                    tag_id,
                    id,
                } => {
                    let logical_id = utils::normalize_case(
                        &self.expressions[*tag_id].source.string,
                        utils::Case::SnakeCase,
                    );

                    let logical_event_name =
                        format!("{logical_id}_{source_event}",);

                    let event_name = if let Some(name) =
                        self.exposed_event_map.get(&logical_event_name)
                    {
                        name.as_deref().unwrap_or(&logical_event_name)
                    } else {
                        &logical_event_name
                    };

                    let event_string = utils::normalize_case(
                        &source_event,
                        utils::Case::SnakeCase,
                    );

                    let fn_name = utils::normalize_case(
                        &event_name,
                        utils::Case::CamelCase,
                    );

                    write_buffer!(
                        self,
                        "$._cleanup.push(w{id}.events.on(\"{event_string}\", (e) => {{this.{fn_name}(e)}}));\n"
                    );
                }

                IrNode::AttributeBindState {
                    from,
                    to,
                    id,
                    has_value,
                } => {
                    let from =
                        utils::normalize_case(from, utils::Case::CamelCase);

                    let to = utils::normalize_case(to, utils::Case::CamelCase);

                    write_buffer!(self, "this.{to} = w{id}.{from};\n");

                    if *has_value {
                        write_buffer!(self, "w{id}.{from}.value = ");
                        self.generate_attribute_value(&mut ir);
                        write_buffer!(self, ";\n");
                    }
                }

                IrNode::DeclareSlot { name, parent } => {
                    let accessor = if let Some(name) = name {
                        format!("this.slot[\"{name}\"]")
                    } else {
                        String::from("this.slot")
                    };

                    if *parent == 0usize {
                        write_buffer!(self, "this.appendChild({accessor});\n")
                    } else {
                        write_buffer!(
                            self,
                            "w{parent}.appendChild({accessor});\n"
                        )
                    }
                }

                IrNode::StartIfBlock { id } => {
                    write_buffer!(
                        self,
                        "const condition{id} = new Conditional();\n"
                    );
                }

                IrNode::EndIfBlock { id, parent } => {
                    if *parent == 0 {
                        write_buffer!(
                            self,
                            "this.appendChild(condition{id});\n"
                        )
                    } else {
                        write_buffer!(
                            self,
                            "w{parent}.appendChild(condition{id});\n"
                        )
                    }

                    write_buffer!(self, "condition{id}.update();\n")
                }

                IrNode::StartCaseBlock {
                    test,
                    id,
                    parent_block,
                } => match test {
                    Some(test) => {
                        let expression = self.rewrite_expression(*test);

                        write_buffer!(
                            self,
                            "condition{parent_block}.case(new Derived(() => {expression}), (w{id}) => {{\n"
                        )
                    }

                    None => {
                        write_buffer!(
                            self,
                            "condition{parent_block}.default((w{id}) => {{\n"
                        )
                    }
                },

                IrNode::EndCaseBlock => {
                    write_buffer!(self, "}});\n")
                }

                IrNode::StartEachBlock {
                    iterable,
                    id,
                    context,
                    key,
                } => {
                    let expression = self.rewrite_expression(*iterable);

                    write_buffer!(self, "const each{id} = new Each(");

                    if let Some(key) = key {
                        let key = self.rewrite_expression(*key);
                        write_buffer!(self, "({context}) => {key}, ")
                    } else {
                        write_buffer!(self, "undefined, ")
                    }

                    write_buffer!(
                        self,
                        "new Derived(() => {expression}), ($, {context}, w{id}) => {{\n"
                    );
                }

                IrNode::EndEachBlock { id, parent } => {
                    if *parent == 0 {
                        write_buffer!(
                            self,
                            "}});\nthis.appendChild(each{id});"
                        );
                    } else {
                        write_buffer!(
                            self,
                            "}});\nw{parent}.appendChild(each{id});\n"
                        );
                    }

                    write_buffer!(self, "each{id}.update();\n");
                }
                _ => continue,
            }
        }
    }

    fn generate_module_functions(&mut self) {
        for (name, member) in self.metadata {
            match member {
                ModuleMemberType::Event(e) => {
                    let e = e.as_ref().unwrap_or_else(|| {
                        panic!("Function {name} cannot be None")
                    });

                    let body = self.rewrite_span_in_script(&e.body);

                    write_buffer!(
                        self,
                        "{}(...params) {{\nconst value = ({} => {}\n)(...params);\n",
                        utils::normalize_case(
                            &e.source_name,
                            utils::Case::CamelCase
                        ),
                        e.params,
                        body
                    );

                    let logical_event_name = utils::normalize_case(
                        &e.source_name,
                        utils::Case::SnakeCase,
                    );

                    write_buffer!(
                        self,
                        "this.events.emit(\"{}\", value);\nreturn value;\n}}\n",
                        logical_event_name
                    );
                }

                ModuleMemberType::Command(f) => {
                    let f = f.as_ref().unwrap();
                    let body = self.rewrite_span_in_script(&f.body);

                    write_buffer!(
                        self,
                        "{}{}{}\n",
                        utils::normalize_case(
                            &f.source_name,
                            utils::Case::CamelCase
                        ),
                        f.params,
                        body
                    );
                }

                ModuleMemberType::Function(f) => {
                    let body = self.rewrite_span_in_script(&f.body);

                    write_buffer!(
                        self,
                        "{}{}{}\n",
                        utils::normalize_case(
                            &f.source_name,
                            utils::Case::CamelCase
                        ),
                        f.params,
                        body
                    );
                }

                ModuleMemberType::RenamedEvent { to, .. } => {
                    let to_fn_name =
                        utils::normalize_case(to, utils::Case::CamelCase);

                    let to_event_name =
                        utils::normalize_case(to, utils::Case::SnakeCase);

                    write_buffer!(
                        self,
                        "{to_fn_name}(e) {{\nthis.events.emit(\"{to_event_name}\", e);\nreturn e;\n}}\n"
                    )
                }

                ModuleMemberType::ExposedEvent { prefix, event } => {
                    let event_name = format!("{prefix}_{event}",);

                    let event_string = utils::normalize_case(
                        &event_name,
                        utils::Case::SnakeCase,
                    );

                    let fn_name = utils::normalize_case(
                        &event_name,
                        utils::Case::CamelCase,
                    );

                    write_buffer!(
                        self,
                        "{fn_name}(e) {{\nthis.events.emit(\"{event_string}\", e);\nreturn e;\n}}\n"
                    );
                }

                _ => continue,
            }
        }
    }

    fn generate_interface_getter(&mut self) {
        write_buffer!(self, "get interface () {{\nreturn {{\n");
        for (name, member) in self.metadata {
            match member {
                ModuleMemberType::ModuleReference { source_id, .. } => {
                    write_buffer!(
                        self,
                        "{}: this.{},\n",
                        source_id.0,
                        source_id.0
                    )
                }

                ModuleMemberType::Event(e) => {
                    let e = e.as_ref().unwrap();
                    write_buffer!(
                        self,
                        "{}: (...args) => this.{}(...args),\n",
                        e.source_name,
                        e.source_name
                    )
                }

                ModuleMemberType::ExposedEvent { prefix, event } => {
                    let logical_name = format!("{prefix}_{event}");

                    let event_name = utils::normalize_case(
                        &logical_name,
                        utils::Case::CamelCase,
                    );

                    write_buffer!(
                        self,
                        "{event_name}: (...args) => this.{event_name}(...args),\n",
                    )
                }

                ModuleMemberType::RenamedEvent { to, .. } => {
                    write_buffer!(self, "{to}: this.{to},\n",)
                }

                ModuleMemberType::Command(e) => {
                    let e = e.as_ref().unwrap();
                    write_buffer!(
                        self,
                        "{}: (...args) => this.{}(...args),\n",
                        e.source_name,
                        e.source_name
                    )
                }

                ModuleMemberType::PublicState(v) => {
                    write_buffer!(
                        self,
                        "{}: this.{},\n",
                        v.source_name,
                        v.source_name
                    )
                }

                ModuleMemberType::PublicVariable(v) => {
                    write_buffer!(
                        self,
                        "{}: this.{},\n",
                        v.source_name,
                        v.source_name
                    )
                }

                _ => continue,
            }
        }

        write_buffer!(self, "}};\n}}\n");
    }

    fn generate_module_variables(&mut self) {
        let mut event_list = Vec::new();

        write_buffer!(
            self,
            "this.setClasses(\"BLOOM-INTRINSIC\", \"b-{}\");\n",
            self.module_name
        );

        for (_, member) in self.metadata {
            match member {
                ModuleMemberType::PrivateVariable(v) => {
                    write_buffer!(
                        self,
                        "this.{} = {};\n",
                        utils::normalize_case(
                            &v.source_name,
                            utils::Case::CamelCase
                        ),
                        v.init.as_deref().unwrap_or("undefined")
                    );
                }

                ModuleMemberType::PublicVariable(v) => {
                    write_buffer!(
                        self,
                        "this.{} = {};\n",
                        utils::normalize_case(
                            &v.source_name,
                            utils::Case::CamelCase
                        ),
                        v.init.as_deref().unwrap_or("undefined")
                    );
                }

                ModuleMemberType::PrivateState(v) => {
                    write_buffer!(
                        self,
                        "this.{} = new State({});\n",
                        utils::normalize_case(
                            &v.source_name,
                            utils::Case::CamelCase
                        ),
                        v.init.as_deref().unwrap_or("undefined")
                    );
                }

                ModuleMemberType::PublicState(v) => {
                    write_buffer!(
                        self,
                        "this.{} = new State({});\n",
                        utils::normalize_case(
                            &v.source_name,
                            utils::Case::CamelCase
                        ),
                        v.init.as_deref().unwrap_or("undefined")
                    );
                }

                ModuleMemberType::StateProxy { to, .. } => {
                    write_buffer!(
                        self,
                        "this.{} = undefined;\n",
                        utils::normalize_case(&to, utils::Case::CamelCase),
                    );
                }

                ModuleMemberType::Event(f) => {
                    let source_name = &f.as_ref().unwrap().source_name;

                    write_buffer!(
                        self,
                        "this.{source_name} = this.{source_name}.bind(this);\n",
                    );

                    let logical_event_name = utils::normalize_case(
                        source_name,
                        utils::Case::SnakeCase,
                    );

                    event_list.push(logical_event_name)
                }

                ModuleMemberType::RenamedEvent { to, .. } => {
                    write_buffer!(self, "this.{to} = this.{to}.bind(this);\n",);

                    event_list.push(utils::normalize_case(
                        to,
                        utils::Case::SnakeCase,
                    ));
                }

                ModuleMemberType::ExposedEvent { prefix, event } => {
                    let fn_name = utils::normalize_case(
                        &format!("{prefix}_{event}"),
                        utils::Case::CamelCase,
                    );

                    write_buffer!(
                        self,
                        "this.{fn_name} = this.{fn_name}.bind(this);\n",
                    );

                    event_list.push(utils::normalize_case(
                        &format!("{prefix}_{event}"),
                        utils::Case::SnakeCase,
                    ));
                }

                ModuleMemberType::Command(f) => {
                    let source_name = &f.as_ref().unwrap().source_name;

                    write_buffer!(
                        self,
                        "this.{source_name} = this.{source_name}.bind(this);\n",
                    );
                }

                ModuleMemberType::Function(f) => {
                    let source_name = &f.source_name;

                    write_buffer!(
                        self,
                        "this.{source_name} = this.{source_name}.bind(this);\n",
                    );
                }

                ModuleMemberType::ModuleReference { source_id, .. } => {
                    write_buffer!(self, "this.{} = undefined;\n", source_id.0)
                }

                _ => continue,
            }
        }

        // write the event list
        write_buffer!(self, "this.events = new EventMap([");
        for e in &event_list {
            write_buffer!(self, "\"{e}\", ");
        }
        write_buffer!(self, "]);\n");

        // slots
        if let Some(slot) = self.slot {
            match slot {
                SlotMembers::Unnamed => {
                    write_buffer!(self, "this.slot = new Slot();\n");
                }

                SlotMembers::Named(slots) => {
                    write_buffer!(self, "this.slot = {{");
                    for (name, _) in slots {
                        write_buffer!(self, "\"{name}\": new Slot(),");
                    }
                    write_buffer!(self, "}};\n");
                }
            }
        }
    }

    pub fn generate(&mut self, promote_root: bool) -> String {
        let mut res = String::new();

        let module_name =
            utils::normalize_case(self.module_name, utils::Case::PascalCase);

        write!(
            &mut res,
            "class {module_name} extends Widget {{\nconstructor() {{\nsuper();\n"
        ).expect("Write to result buffer failed");

        self.generate_module_variables();

        write!(&mut res, "{}\n}}\n", self.buffer)
            .expect("Write to result buffer failed");

        self.buffer.clear();

        self.generate_module_functions();

        write!(&mut res, "{}\n", self.buffer)
            .expect("Write to result buffer failed");

        self.buffer.clear();

        self.generate_interface_getter();

        write!(&mut res, "{}\n", self.buffer)
            .expect("Write to result buffer failed");

        self.buffer.clear();

        self.generate_build();

        write!(&mut res, "build () {{\nconst $ = this;\n{}\n", self.buffer,)
            .expect("Write to result buffer failed");

        if promote_root {
            write!(&mut res, "this.setRoot(w1);\n",)
                .expect("Write to result buffer failed");
        }

        write!(&mut res, "}}\n").expect("Write to result buffer failed");

        write!(&mut res, "}}\n").expect("Write to result buffer failed");

        res
    }
}
