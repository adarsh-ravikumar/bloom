use crate::{
    analysis::{AttributeAnalyzer, ExpressionAnalyzer, ModuleContext},
    common::Source,
    module::{Expression, Module, ModuleRegistry, ModuleType},
};

// mod codegen;
mod analysis;
mod common;
mod diagnostic;
// mod discover;
// mod ir;
mod module;
mod parser;
mod utils;

fn main() {
    let sources = [
        (
            "button",
            ModuleType::Widget,
            Source::from_path("sandbox/widgets/button.bloom").unwrap(),
        ),
        (
            "text",
            ModuleType::Widget,
            Source::from_path("sandbox/widgets/text.bloom").unwrap(),
        ),
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
        // (
        //     "counter",
        //     ModuleType::Widget,
        //     Source::from_path("sandbox/widgets/counter.bloom").unwrap(),
        // ),
    ];

    let mut registry = ModuleRegistry::new();

    for (name, r#type, source) in &sources {
        println!("\nAnalyzing module[{:?}]: {}", r#type, name);
        Module::frontend_analysis(
            &mut registry,
            name.to_string(),
            *r#type,
            source,
        );
    }

    println!("\nPerforming dependency analysis:");
    let order = registry.analyze_dependencies();

    println!("{order:?}");

    println!("\nPerforming template analysis:");

    let interface_delta = {
        let mut attribute_analyzer = AttributeAnalyzer::new(&registry);

        for module in &order {
            let module = &registry.modules[*module];

            println!(
                "\nExpressions before semantic pass in module {}",
                module.name
            );
            println!("{:?}", module.expressions);

            let context = ModuleContext {
                name: &module.name,
                metadata: &module.metadata,
                expressions: &module.expressions,
            };

            attribute_analyzer.visit_module(&module.root, &context);
        }

        attribute_analyzer.interface_delta
    };

    {
        let mut expression_analyzer = ExpressionAnalyzer::new(&interface_delta);

        for module in &order {
            let module = &mut registry.modules[*module];
            expression_analyzer.analyze(&module.name, &mut module.expressions);
        }
    };

    for module in &order {
        let module = &registry.modules[*module];
        println!(
            "\nExpressions after semantic pass in module {}",
            module.name
        );
        println!("{:?}", module.expressions);
    }
}
