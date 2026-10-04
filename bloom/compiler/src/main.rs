use std::{fmt::write, fs, path::Path};

use crate::{diagnostic::DiagnosticRenderer, parser::Parser};

mod codegen;
mod common;
mod diagnostic;
mod parser;
mod utils;

fn write_to_test(generated: String) {
    let code = format!(
        "\n \
<!DOCTYPE html>\n \
<html lang=\"en\">\n \
<head>\n \
  <meta charset=\"UTF-8\">\n \
  <meta name=\"viewport\" content=\"width=, initial-scale=1.0\">\n \
  <title>Document</title>\n \
</head>\n \
<body>\n \
  <div id=\"main\"> </div>\n \
\n \
  <script>\n \
    {generated}\n \
  </script>\n \
</body>\n \
</html>\
        "
    );

    fs::write("./index.html", code).unwrap();
}

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

    print!("Abstract syntax tree:");
    parser.display();

    let mut codegen_js = codegen::CodegenJs::new(&source, &parser.root);

    let generated_code = codegen_js.generate();

    write_to_test(generated_code);
}
