use crate::{diagnostic::DiagnosticRenderer, parser::Parser};

mod common;
mod diagnostic;
mod parser;
mod utils;

fn main() {
    let file = match common::IOFile::from_path("./playground.bloom") {
        Ok(f) => f,
        Err(msg) => return println!("{msg}"),
    };

    let mut renderer = DiagnosticRenderer::new(&file);

    let mut parser = Parser::new(&file);

    let _ = match parser.parse() {
        Ok(_) => parser.display(),
        Err(diag) => {
            let mut diag = vec![diag];
            println!("{}", renderer.render(&mut diag))
        }
    };
}
