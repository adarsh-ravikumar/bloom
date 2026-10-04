use oxc_allocator::Allocator;
use oxc_ast::ast::{
    BindingPattern, Program, Statement, VariableDeclaration,
    VariableDeclarationKind,
};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

struct Visitor<'a> {
    variables: Vec<String>,
    src: String,
    ast: &'a Program<'a>,
}

impl<'a> Visitor<'a> {
    pub fn new(src: String, ast: &'a Program) -> Self {
        Self {
            variables: Vec::new(),
            src,
            ast,
        }
    }

    pub fn display(&self) {
        println!("Variables: ");

        for var in &self.variables {
            println!("{var}");
        }
    }

    fn analze_program(&mut self) {
        for stmt in &self.ast.body {
            match stmt {
                Statement::VariableDeclaration(decl) => {
                    self.analyze_var_decl(decl)
                }
                _ => continue,
            }
        }
    }

    fn analyze_var_decl(&mut self, decl: &VariableDeclaration) {
        if decl.kind != VariableDeclarationKind::Let {
            return;
        }

        for decl in &decl.declarations {
            match &decl.id {
                BindingPattern::BindingIdentifier(ident) => {
                    self.variables.push(ident.name.to_string());
                    // we need either the
                }

                _ => panic!("unknown binding pattern: {:?}", decl.id),
            }
        }
    }

    fn analyze_type_annotation(&mut self) {}
}

fn main() {
    let source = r#"
        interface Person {
            name: string;
            address: string;
            age: number;
        };

        let x: number = 10;
        const y: number = x + 20;

        const bob: Person = {
            name: "bob",
            address: "123, Lincoln street",
            age: 32
        }

        function test () {
            let a = 4;
            let hehe = "213"
        }

        function another_test () {
            let b = 4;
            let hehe = "213"
        }

        console.log(y);
    "#;

    let allocator = Allocator::default();

    let parser = Parser::new(&allocator, source, SourceType::ts());

    let result = parser.parse();

    let mut visitor = Visitor::new(source.to_string(), &result.program);

    visitor.analze_program();
    visitor.display();
}
