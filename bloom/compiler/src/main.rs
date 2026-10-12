use crate::{
    common::Source,
    module::{Module, ModuleRegistry, ModuleType},
};

// mod codegen;
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
            "button",
            ModuleType::Widget,
            Source::from_path("sandbox/widgets/button.bloom").unwrap(),
        ),
        (
            "text",
            ModuleType::Widget,
            Source::from_path("sandbox/widgets/text.bloom").unwrap(),
        ),
        (
            "column",
            ModuleType::Widget,
            Source::from_path("sandbox/widgets/column.bloom").unwrap(),
        ),
        (
            "row",
            ModuleType::Widget,
            Source::from_path("sandbox/widgets/column.bloom").unwrap(),
        ),
        (
            "slider",
            ModuleType::Widget,
            Source::from_path("sandbox/widgets/slider.bloom").unwrap(),
        ),
        (
            "inspector",
            ModuleType::Panel,
            Source::from_path("sandbox/panels/inspector.bloom").unwrap(),
        ),
        (
            "counter",
            ModuleType::Widget,
            Source::from_path("sandbox/widgets/counter.bloom").unwrap(),
        ),
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
    registry.analyze_dependencies();

    // let mut renderer = DiagnosticRenderer::new(&source);
    //
    // let mut parser = Parser::new(&source);
    //
    // parser.parse();
    //
    // if !parser.diagnostics.is_empty() {
    //     println!("{}", renderer.render(&mut parser.diagnostics));
    //     println!("\nERRONEOUS AST:");
    //     parser.display();
    //     return;
    // }
    //
    // println!("Abstract syntax tree:");
    // parser.display();
    //
    // let mut template_ir = TemplateIr::new(&parser.root, &source);
    // template_ir.generate_ir();
    //
    // println!("\nIR");
    // for node in &template_ir.ir_nodes {
    //     println!("{:?}", node);
    // }
    //
    // println!("\nExpressions");
    // for expr in &template_ir.expressions {
    //     println!("{:?}", expr);
    // }
    //
    // // script
    //
    // let mut analyzer = SemanticAnalyzer::new();
    //
    // let (global_symbols, script_source, uses_in_script) =
    //     if let Some(script) = &parser.root.script {
    //         println!("\nPREPROCESSING");
    //         println!("Stripped script:");
    //         println!("{}", script.source);
    //         println!("Metadata:");
    //         println!("{:?}", script.metadata);
    //
    //         let oxc_allocator = OxcAllocator::new();
    //         analyzer.analyze(
    //             &oxc_allocator,
    //             &script.source,
    //             &mut template_ir.expressions,
    //         );
    //
    //         let global_symbols = &analyzer.context_tree.get(0).unwrap().symbols;
    //
    //         println!("SCRIPT ANALYSIS");
    //         println!("Global symbols:");
    //         for (k, v) in global_symbols {
    //             println!("{} :  {:?}", k, v);
    //         }
    //
    //         println!("References to global variables:");
    //         for r#use in &analyzer.uses {
    //             println!("{:?}", r#use)
    //         }
    //
    //         (
    //             Some(global_symbols),
    //             Some(&script.source),
    //             Some(analyzer.uses),
    //         )
    //     } else {
    //         (None, None, None)
    //     };
    //
    // println!("\nExpressions after resolution:");
    // for expr in &template_ir.expressions {
    //     println!("{:?}", expr);
    // }
    //
    // let mut codegen = GenerateJs::new(
    //     template_ir.ir_nodes,
    //     template_ir.expressions,
    //     global_symbols,
    //     script_source,
    //     uses_in_script,
    // );
    //
    // codegen.generate();
    //
    // println!("\nCODEGEN\n");
    // println!("{}", codegen.code);
}
