//TODO: Construct symbols for anonymous functions (functions as expressions)

use std::{collections::HashMap, ops::Deref, thread::current};

use crate::ir::template::Expression;
use oxc_allocator::{Allocator as OxcAllocator, Vec as OxcVec};
use oxc_ast::ast::{BindingPattern, Statement};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span as OxcSpan};

#[derive(Debug)]
pub struct SemanticAnalyzer<'a> {
    pub expressions: &'a Vec<Expression>,
    pub context_tree: Vec<SymbolTable>,
}

pub enum Body<'a> {
    Statement(&'a Statement<'a>),
    Statements(&'a OxcVec<'a, Statement<'a>>),
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Variable {
        span: OxcSpan,
        // type_annotation: TypeAnnotation
    },
    Function {
        name: String,
        params: Vec<String>,
        context: usize,
        // return_type: TypeAnnotation
    },
    Paramter {
        span: OxcSpan,
        optional: bool,
        // type_annotation: TypeAnnotation
    },
    ParameterRest {
        span: OxcSpan,
        // type_annotation: TypeAnnotation
    },
}

#[derive(Debug)]
pub struct SymbolTable {
    pub symbols: HashMap<String, Symbol>,
    pub context: usize,
    pub parent: usize,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(expressions: &'a Vec<Expression>) -> Self {
        Self {
            expressions,
            context_tree: Vec::new(),
        }
    }

    fn parse<'b>(
        source: &'b str,
        oxc_allocator: &'b OxcAllocator,
    ) -> OxcVec<'b, Statement<'b>> {
        let parser = Parser::new(oxc_allocator, source, SourceType::ts());

        let result = parser.parse();

        if !result.diagnostics.is_empty() {
            // TODO: Render diagnostics
            println!("{:?}", result.diagnostics);
        }

        result.program.body
    }

    // context tree
    fn new_context(&mut self, parent: usize) -> usize {
        let new_context = self.context_tree.len();
        self.context_tree.push(SymbolTable {
            symbols: HashMap::new(),
            context: new_context,
            parent,
        });

        new_context
    }

    fn collect_bindings(&mut self, pattern: &BindingPattern) -> Vec<String> {
        match pattern {
            BindingPattern::BindingIdentifier(id) => {
                vec![id.name.to_string()]
            }

            BindingPattern::ObjectPattern(pattern) => {
                let mut bindings = Vec::new();

                for property in &pattern.properties {
                    bindings.extend(self.collect_bindings(&property.value));
                }

                if let Some(rest) = &pattern.rest {
                    bindings.extend(self.collect_bindings(&rest.argument));
                }

                bindings
            }

            BindingPattern::ArrayPattern(pattern) => {
                let mut bindings = Vec::new();

                for element in &pattern.elements {
                    if let Some(element) = element {
                        bindings.extend(self.collect_bindings(&element));
                    }
                }

                bindings
            }

            BindingPattern::AssignmentPattern(pattern) => {
                self.collect_bindings(&pattern.left)
            }
        }
    }

    fn visit_statement<'b>(
        &mut self,
        parent_context: usize,
        current_context: usize,
        stmt: &Statement<'b>,
    ) {
        match stmt {
            Statement::VariableDeclaration(decl) => {
                for decl in &decl.declarations {
                    let bindings = self.collect_bindings(&decl.id);
                    // add the bindings to the current scope
                    for binding in bindings {
                        self.context_tree
                            .get_mut(current_context)
                            .unwrap()
                            .symbols
                            .insert(
                                binding,
                                Symbol::Variable { span: decl.span },
                            );
                    }
                }
            }

            // functions -> add params to new context
            Statement::FunctionDeclaration(func) => {
                let fn_context = if let Some(body) = &func.body {
                    self.build_context(
                        current_context,
                        Body::Statements(&body.statements),
                    )
                } else {
                    current_context
                };

                let mut params = Vec::new();

                for param in &func.params.items {
                    // let from = self.context_tree.get(fn_context).unwrap();

                    let bindings = self.collect_bindings(&param.pattern);

                    for binding in bindings {
                        params.push(binding.clone());
                        self.context_tree
                            .get_mut(fn_context)
                            .unwrap()
                            .symbols
                            .insert(
                                binding,
                                Symbol::Paramter {
                                    span: param.span,
                                    optional: param.optional,
                                },
                            );
                    }
                }

                if let Some(rest) = &func.params.rest {
                    let bindings = self.collect_bindings(&rest.rest.argument);

                    for binding in bindings {
                        params.push(binding.clone());
                        self.context_tree
                            .get_mut(fn_context)
                            .unwrap()
                            .symbols
                            .insert(
                                binding,
                                Symbol::ParameterRest { span: rest.span },
                            );
                    }
                }

                // add the function to the current context
                let fn_name = if let Some(name) = func.name() {
                    name.to_string()
                } else {
                    "".to_string()
                };

                self.context_tree
                    .get_mut(parent_context)
                    .unwrap()
                    .symbols
                    .insert(
                        fn_name.clone(),
                        Symbol::Function {
                            name: fn_name,
                            context: fn_context,
                            params,
                        },
                    );
            }

            // arrow-function -> add params to new context

            // for, for-in, for-of       -> add iterator to new context
            Statement::ForStatement(for_stmt) => {
                let for_context = self.build_context(
                    current_context,
                    Body::Statement(&for_stmt.body),
                );

                if let Some(
                    oxc_ast::ast::ForStatementInit::VariableDeclaration(init),
                ) = &for_stmt.init
                {
                    for decl in &init.declarations {
                        let bindings = self.collect_bindings(&decl.id);
                        // add the bindings to the current scope
                        for binding in bindings {
                            self.context_tree
                                .get_mut(for_context)
                                .unwrap()
                                .symbols
                                .insert(
                                    binding,
                                    Symbol::Variable { span: decl.span },
                                );
                        }
                    }
                }
            }

            Statement::ForInStatement(for_in_stmt) => {
                let for_context = self.build_context(
                    current_context,
                    Body::Statement(&for_in_stmt.body),
                );

                if let oxc_ast::ast::ForStatementLeft::VariableDeclaration(
                    init,
                ) = &for_in_stmt.left
                {
                    for decl in &init.declarations {
                        let bindings = self.collect_bindings(&decl.id);
                        // add the bindings to the current scope
                        for binding in bindings {
                            self.context_tree
                                .get_mut(for_context)
                                .unwrap()
                                .symbols
                                .insert(
                                    binding,
                                    Symbol::Variable { span: decl.span },
                                );
                        }
                    }
                }
            }

            Statement::ForOfStatement(for_of_stmt) => {
                let for_context = self.build_context(
                    current_context,
                    Body::Statement(&for_of_stmt.body),
                );

                if let oxc_ast::ast::ForStatementLeft::VariableDeclaration(
                    init,
                ) = &for_of_stmt.left
                {
                    for decl in &init.declarations {
                        let bindings = self.collect_bindings(&decl.id);
                        // add the bindings to the current scope
                        for binding in bindings {
                            self.context_tree
                                .get_mut(for_context)
                                .unwrap()
                                .symbols
                                .insert(
                                    binding,
                                    Symbol::Variable { span: decl.span },
                                );
                        }
                    }
                }
            }

            // if, while, do-while, with, labeled, try, catch, switch -> build_context
            Statement::IfStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statement(&stmt.consequent),
                );

                if let Some(alternate) = &stmt.alternate {
                    self.build_context(
                        current_context,
                        Body::Statement(alternate),
                    );
                }
            }

            Statement::WhileStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statement(&stmt.body),
                );
            }

            Statement::DoWhileStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statement(&stmt.body),
                );
            }

            Statement::WithStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statement(&stmt.body),
                );
            }

            Statement::LabeledStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statement(&stmt.body),
                );
            }

            Statement::TryStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statements(&stmt.block.body),
                );

                // catch
                if let Some(catch_block) = &stmt.handler {
                    let catch_context = self.build_context(
                        current_context,
                        Body::Statements(&catch_block.body.body),
                    );

                    if let Some(param) = &catch_block.param {
                        let bindings = self.collect_bindings(&param.pattern);
                        for binding in bindings {
                            self.context_tree
                                .get_mut(catch_context)
                                .unwrap()
                                .symbols
                                .insert(
                                    binding,
                                    Symbol::Variable { span: param.span },
                                );
                        }
                    }
                }

                // finally
                if let Some(finally_block) = &stmt.finalizer {
                    self.build_context(
                        current_context,
                        Body::Statements(&finally_block.body),
                    );
                }
            }

            Statement::SwitchStatement(stmt) => {
                let switch_context = self.new_context(current_context);

                for case in &stmt.cases {
                    for stmt in &case.consequent {
                        self.visit_statement(
                            current_context,
                            switch_context,
                            stmt,
                        );
                    }
                }
            }

            Statement::BlockStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statements(&stmt.body),
                );
            }
            _ => return,
        }
    }

    fn build_context<'b>(
        &mut self,
        parent_context: usize,
        body: Body<'b>,
    ) -> usize {
        let new_context = self.new_context(parent_context);

        match body {
            Body::Statement(stmt) => {
                self.visit_statement(parent_context, new_context, stmt)
            }

            Body::Statements(stmts) => {
                for stmt in stmts.deref() {
                    self.visit_statement(parent_context, new_context, &stmt)
                }
            }
        }

        new_context
    }

    pub fn collect_symbols(
        &mut self,
        oxc_allocator: &OxcAllocator,
        source: &str,
    ) {
        let body = Self::parse(source, oxc_allocator);

        self.build_context(0, Body::Statements(&body));
    }
}
