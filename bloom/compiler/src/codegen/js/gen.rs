use oxc_span::Span as OxcSpan;

use crate::ir::{Expression, IrNode, Symbol, SymbolUse};
use std::{collections::HashMap, fmt::Write};

pub struct GenerateJs<'a> {
    ir: Vec<IrNode>,
    expressions: Vec<Expression>,
    global_symbols: Option<&'a HashMap<String, Symbol>>,
    uses_in_script: Option<Vec<SymbolUse>>,
    script_source: Option<&'a String>,
    build: String,
    states: String,
    pub code: String,
}

impl<'a> GenerateJs<'a> {
    pub fn new(
        ir: Vec<IrNode>,
        expressions: Vec<Expression>,
        global_symbols: Option<&'a HashMap<String, Symbol>>,
        script_source: Option<&'a String>,
        uses_in_script: Option<Vec<SymbolUse>>,
    ) -> Self {
        Self {
            ir,
            expressions,
            code: String::new(),
            states: String::new(),
            build: String::new(),
            uses_in_script,
            global_symbols,
            script_source,
        }
    }

    pub fn generate(&mut self) {
        self.generate_build_fn();
        self.generate_states();

        let _ = write!(
            &mut self.code,
            r#"class Test {{
constructor () {{
{}

this.root = document.createElement("div");
this.build()
}}

build() {{
const $0 = this.root;
{}
}}
}}
"#,
            self.states.trim(),
            self.build.trim()
        );
    }

    fn transform_expression(
        &self,
        source: &str,
        global_uses: &[SymbolUse],
    ) -> String {
        let mut transformed = String::new();

        let mut start_from = 0usize;

        for occurance in global_uses {
            let _ = write!(
                &mut transformed,
                "{}",
                &source[start_from..occurance.span.start as usize]
            );

            let _ = write!(&mut transformed, "this.");

            start_from = occurance.span.end as usize;

            let _ = write!(
                &mut transformed,
                "{}",
                &source[occurance.span.start as usize..start_from]
            );
        }

        let _ = write!(&mut transformed, "{}", &source[start_from..]);
        transformed.trim().to_string()
    }

    fn transform_in_script(&self, span: OxcSpan) -> String {
        if let Some(uses) = &self.uses_in_script
            && let Some(source) = self.script_source
        {
            // we need to see if there's any uses within this span
            let mut found_uses = false;
            let mut uses_start = 0usize;

            for (i, use_) in uses.iter().enumerate() {
                if use_.span.start > span.start {
                    found_uses = true;
                    uses_start = i;
                    break;
                }
            }

            if !found_uses {
                return source[span.start as usize..span.end as usize]
                    .to_string();
            }

            let mut uses_end = uses_start;

            for use_ in &uses[uses_start..] {
                uses_end += 1;

                if use_.span.end > span.end {
                    break;
                }
            }

            // now we have the exact set of uses.
            return self.transform_expression(
                &source[span.start as usize..span.end as usize],
                &uses[uses_start..uses_end],
            );
        }

        unreachable!()
    }

    fn generate_states(&mut self) {
        if let Some(symbols) = &self.global_symbols {
            for (name, symbol) in symbols.iter() {
                match symbol {
                    Symbol::Variable { is_state, init, .. } => {
                        let state_prefix =
                            if *is_state { " = $state(" } else { " = " };
                        let state_suffix = if *is_state { ")" } else { "" };

                        let _ = write!(
                            &mut self.states,
                            "this.{name}{state_prefix}"
                        );

                        if let Some(init) = init {
                            let transformed = self.transform_in_script(*init);
                            let _ = write!(&mut self.states, "{transformed}");
                        }

                        let _ = writeln!(&mut self.states, "{state_suffix};");
                    }
                    _ => continue,
                }
            }
        }
    }

    fn generate_build_fn(&mut self) {
        for node in &self.ir {
            match node {
                IrNode::StartParts { id } => {
                    let _ = writeln!(
                        &mut self.build,
                        "const ${id} = document.createElement('span');"
                    );
                }

                IrNode::Text { value, id, parent } => {
                    let value = value.trim().replace("\n", " ");
                    let _ = writeln!(
                        &mut self.build,
                        r#"const ${id} = document.createElement('span');
${id}.innerText = '{value}';
${parent}.appendChild(${id});"#
                    );
                }

                IrNode::Expression {
                    id,
                    expr_id,
                    parent,
                    ..
                } => {
                    let source = self.expressions[*expr_id].source.trim();
                    let uses = &self.expressions[*expr_id].global_uses;

                    let transformed =
                        self.transform_expression(source, uses.as_slice());
                    let _ = writeln!(
                        &mut self.build,
                        r#"const ${id} = document.createElement('span');
${id}.innerText = $reactive(() => {transformed});
${parent}.appendChild(${id});"#
                    );
                }

                IrNode::EndParts { id, parent } => {
                    let _ = writeln!(
                        &mut self.build,
                        "${parent}.appendChild(${id});"
                    );
                    self.build += format!("").as_str();
                }

                IrNode::StartWidget {
                    name,
                    id,
                    is_platform,
                } => {
                    if *is_platform {
                        // actual name of the element is obtained by removing the leading :
                        let name = &name[1..];
                        let _ = writeln!(
                            &mut self.build,
                            "const ${id} = document.createElement('{name}');"
                        );
                    } else {
                        todo!()
                    }
                }

                IrNode::EndWidget { id, parent } => {
                    let _ = writeln!(
                        &mut self.build,
                        "${parent}.appendChild(${id});"
                    );
                }

                IrNode::StartAttributeList => {}
                IrNode::EndAttributeList => {} //
                // IrNode::AttributeName { name, span } => {}
                // IrNode::AttributeTrue => {}
                // IrNode::StartIfBlock { test, id } => {}
                // IrNode::EndIfBlock { id, parent } => {}
                //
                // IrNode::StartCaseBlock { test, id } => {}
                // IrNode::EndCaseBlock { id, parent } => {}
                //
                // IrNode::StartEachBlock { iterable, id } => {}
                // IrNode::EndEachBlock { id, parent } => {}
                //
                _ => todo!(),
            }
        }
    }
}
