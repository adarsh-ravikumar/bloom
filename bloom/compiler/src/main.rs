use crate::{
    io::IOFile,
    node::{AttributeValue, FragmentNode},
    parser::Parser,
};

mod err;
mod io;
mod node;
mod parser;
mod span;

fn main() {
    let file = match IOFile::from_path("./html_test.bloom") {
        Ok(f) => f,
        Err(msg) => return println!("{msg}"),
    };

    let mut parser = Parser::new(file);
    let res = match parser.parse() {
        Ok(_) => parser.display(),
        Err(err) => println!("{:?}", err),
    };
}
