use crate::{
    analysis::{AttributeAnalyzer, ExpressionAnalyzer, ModuleContext},
    codegen::GenerateTemplateJs,
    common::Source,
    ir::TemplateIrGen,
    module::{Module, ModuleRegistry, ModuleType},
};

use std::{fmt::Write, fs};

mod analysis;
mod codegen;
mod common;
mod diagnostic;
// mod discover;
mod ir;
mod module;
mod parser;
mod utils;

fn main() {
    let sources = [
        (
            "text",
            ModuleType::Widget,
            Source::from_path("sandbox/widgets/text.bloom").unwrap(),
        ),
        (
            "button",
            ModuleType::Widget,
            Source::from_path("sandbox/widgets/button.bloom").unwrap(),
        ),
        (
            "container",
            ModuleType::Widget,
            Source::from_path("sandbox/widgets/container.bloom").unwrap(),
        ),
        // (
        //     "counter",
        //     ModuleType::Widget,
        //     Source::from_path("sandbox/widgets/counter.bloom").unwrap(),
        // ),
        // (
        //     "view",
        //     ModuleType::Widget,
        //     Source::from_path("sandbox/widgets/view.bloom").unwrap(),
        // ),
        // (
        //     "column",
        //     ModuleType::Widget,
        //     Source::from_path("sandbox/widgets/column.bloom").unwrap(),
        // ),
        // (
        //     "row",
        //     ModuleType::Widget,
        //     Source::from_path("sandbox/widgets/column.bloom").unwrap(),
        // ),
        // (
        //     "slider",
        //     ModuleType::Widget,
        //     Source::from_path("sandbox/widgets/slider.bloom").unwrap(),
        // ),
        // (
        //     "inspector",
        //     ModuleType::Panel,
        //     Source::from_path("sandbox/panels/inspector.bloom").unwrap(),
        // ),
    ];

    let mut registry = ModuleRegistry::new();

    for (name, r#type, source) in &sources {
        println!("\n Frontend analysis on module[{:?}]: {}", r#type, name);
        Module::frontend_analysis(
            &mut registry,
            name.to_string(),
            *r#type,
            source,
        );
    }

    // println!("\nPerforming dependency analysis:");
    let order = registry.analyze_dependencies();

    // println!("{order:?}");

    // println!("\nPerforming template analysis:");

    let (interface_delta, exposed_events_map, slots) = {
        let mut attribute_analyzer = AttributeAnalyzer::new(&registry);

        for module in &order {
            let module = &registry.modules[*module];

            println!("\n Attribute analysis on module: {}", module.name);

            // println!(
            //     "\nExpressions before semantic pass in module {}",
            //     module.name
            // );
            // println!("{:?}", module.expressions);

            let context = ModuleContext {
                name: &module.name,
                metadata: &module.metadata,
                expressions: &module.expressions,
            };

            attribute_analyzer.visit_module(&module.root, &context);
        }

        // println!(
        //     "\n\nInterface Delta: {:?}",
        //     attribute_analyzer.interface_delta
        // );

        (
            attribute_analyzer.interface_delta,
            attribute_analyzer.exposed_event_map,
            attribute_analyzer.slots,
        )
    };

    {
        let mut expression_analyzer = ExpressionAnalyzer::new(&interface_delta);

        for module in &order {
            let module = &mut registry.modules[*module];
            println!("\n Expression analysis on module: {}", module.name);
            expression_analyzer.analyze(&module.name, &mut module.expressions);
        }

        registry.apply_delta(interface_delta);
    };

    // println!("\nPerforming Intermediate Representation Generation");

    let mut compiled_result = String::new();

    for module in &order {
        let module = &mut registry.modules[*module];

        // println!(
        //     "\nExpressions after semantic pass in module {}",
        //     module.name
        // );

        // println!("{:?}", module.expressions);

        // println!("\nGenerating module {}", module.name);

        if let Some(root) = &module.root {
            let mut ir_gen = TemplateIrGen::new(root, &slots);
            ir_gen.generate_ir();

            for node in &ir_gen.ir_nodes {
                println!("{node:?}");
            }

            println!("\nCodegen Result:\n");
            let mut codegen = GenerateTemplateJs::new(
                &ir_gen.ir_nodes,
                &module.name,
                &module.metadata,
                &module.symbol_uses,
                &module.expressions,
                &exposed_events_map,
            );

            let res = codegen.generate();

            println!("{res}");
            write!(&mut compiled_result, "{res}");
        }
    }

    let mut prelude =
        String::from_utf8(fs::read("sandbox/out/prelude.js").unwrap()).unwrap();

    prelude += &compiled_result;

    fs::write("sandbox/out/result.js", prelude);
}
