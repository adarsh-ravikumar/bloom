use std::{fmt::write, fs, path::Path};

use oxc_allocator::Allocator as OxcAllocator;

use crate::{
    diagnostic::DiagnosticRenderer,
    ir::{SemanticAnalyzer, TemplateIr},
    parser::Parser,
};

mod common;
mod diagnostic;
mod ir;
mod parser;
mod utils;

fn main() {
    let source = match common::Source::from_path("./playground.bloom") {
        Ok(f) => f,
        Err(msg) => return println!("{msg}"),
    };

    let mut renderer = DiagnosticRenderer::new(&source);

    let mut parser = Parser::new(&source);

    parser.parse();

    if !parser.diagnostics.is_empty() {
        println!("{}", renderer.render(&mut parser.diagnostics));
        println!("\nERRONEOUS AST:");
        parser.display();
        return;
    }

    // print!("Abstract syntax tree:");
    // parser.display();

    // println!("\nIR");
    let mut template_ir = TemplateIr::new(&source, &parser.root);
    template_ir.generate_ir();

    // for node in &template_ir.nodes {
    //     println!("{:?}", node);
    // }
    //
    // println!("\nExpressions");
    // for expr in &template_ir.expressions {
    //     println!("{:?}", expr);
    // }

    // script
    if let Some(script) = &parser.root.script {
        let mut analyzer = SemanticAnalyzer::new(&template_ir.expressions);

        let oxc_allocator = OxcAllocator::new();
        analyzer.collect_symbols(&oxc_allocator, &script.source);

        println!("Context Tree");
        for context in analyzer.context_tree {
            println!(
                "\ncontext {} (parent: {})",
                context.context, context.parent
            );

            println!("symbols:");
            for (k, v) in context.symbols {
                println!("{} :  {:?}", k, v);
            }
        }

        // println!("Stripped script");
        // println!("{}", script.source);
        // println!("Metadata");
        // println!("{:?}", script.pre);
    }
}
