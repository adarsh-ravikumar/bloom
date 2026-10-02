use crate::{diagnostic::DiagnosticRenderer, parser::Parser};

mod common;
mod diagnostic;
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
        println!("\nERRONEOUS TREE:");
    }

    parser.display();
}
