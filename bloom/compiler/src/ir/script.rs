use std::{collections::HashMap, os::unix::ffi::OsStrExt};

use crate::ir::template::Expression;
use oxc_allocator::{Allocator as OxcAllocator, Vec as OxcVec};
use oxc_ast::ast::{
    Argument, ArrayExpression, ArrayExpressionElement, ArrowFunctionBody,
    ArrowFunctionExpression, AssignmentTarget, AssignmentTargetMaybeDefault,
    AssignmentTargetProperty, BindingPattern as OxcBindingPattern,
    ChainElement, Expression as OxcExpression, ExpressionStatement,
    ForStatementInit, Function, ObjectPropertyKind, ParenthesizedExpression,
    Program, PropertyKey, Statement as OxcStatement, UpdateExpression,
};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span as OxcSpan};

#[derive(Debug)]
pub struct SemanticAnalyzer<'a> {
    pub expressions: &'a Vec<Expression>,
    pub context_tree: Vec<SymbolTable>,
    pub next_context: usize,
    pub uses: Vec<OxcSpan>,
}

pub enum Body<'a> {
    Statement(&'a OxcStatement<'a>),
    Statements(&'a OxcVec<'a, OxcStatement<'a>>),
    Expression(&'a OxcExpression<'a>),
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
            uses: Vec::new(),
            next_context: 0,
        }
    }

    fn parse<'b>(
        source: &'b str,
        oxc_allocator: &'b OxcAllocator,
    ) -> OxcVec<'b, OxcStatement<'b>> {
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

        print!("{new_context} | ");

        new_context
    }

    fn enter_context(&mut self) -> usize {
        let context = self.next_context;

        self.next_context += 1;

        print!("{context} | ");

        context
    }

    fn collect_bindings(&mut self, pattern: &OxcBindingPattern) -> Vec<String> {
        match pattern {
            OxcBindingPattern::BindingIdentifier(id) => {
                vec![id.name.to_string()]
            }

            OxcBindingPattern::ObjectPattern(pattern) => {
                let mut bindings = Vec::new();

                for property in &pattern.properties {
                    bindings.extend(self.collect_bindings(&property.value));
                }

                if let Some(rest) = &pattern.rest {
                    bindings.extend(self.collect_bindings(&rest.argument));
                }

                bindings
            }

            OxcBindingPattern::ArrayPattern(pattern) => {
                let mut bindings = Vec::new();

                for element in &pattern.elements {
                    if let Some(element) = element {
                        bindings.extend(self.collect_bindings(&element));
                    }
                }

                bindings
            }

            OxcBindingPattern::AssignmentPattern(pattern) => {
                self.collect_bindings(&pattern.left)
            }
        }
    }

    fn visit_statement<'b>(
        &mut self,
        parent_context: usize,
        current_context: usize,
        stmt: &OxcStatement<'b>,
    ) {
        match stmt {
            OxcStatement::ExpressionStatement(expr) => {
                if matches!(
                    expr.expression,
                    OxcExpression::ArrowFunctionExpression(_)
                        | OxcExpression::FunctionExpression(_)
                ) {
                    self.build_context(
                        current_context,
                        Body::Expression(&expr.expression),
                    );
                }
            }

            OxcStatement::VariableDeclaration(decl) => {
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

                    if let Some(init) = &decl.init
                        && matches!(
                            init,
                            OxcExpression::ArrowFunctionExpression(_)
                                | OxcExpression::FunctionExpression(_)
                        )
                    {
                        self.build_context(
                            current_context,
                            Body::Expression(init),
                        );
                    }
                }
            }

            // functions -> add params to new context
            OxcStatement::FunctionDeclaration(func) => {
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
            OxcStatement::ForStatement(for_stmt) => {
                let new_context = self.new_context(current_context);

                self.build_context(
                    new_context,
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
                                .get_mut(new_context)
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

            OxcStatement::ForInStatement(for_in_stmt) => {
                let new_context = self.new_context(current_context);

                self.build_context(
                    new_context,
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
                                .get_mut(new_context)
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

            OxcStatement::ForOfStatement(for_of_stmt) => {
                let new_context = self.new_context(current_context);

                self.build_context(
                    new_context,
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
                                .get_mut(new_context)
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
            OxcStatement::IfStatement(stmt) => {
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

            OxcStatement::WhileStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statement(&stmt.body),
                );
            }

            OxcStatement::DoWhileStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statement(&stmt.body),
                );
            }

            OxcStatement::WithStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statement(&stmt.body),
                );
            }

            OxcStatement::LabeledStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statement(&stmt.body),
                );
            }

            OxcStatement::TryStatement(stmt) => {
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

            OxcStatement::SwitchStatement(stmt) => {
                let switch_context = self.new_context(current_context);

                for case in &stmt.cases {
                    for stmt in &case.consequent {
                        self.visit_statement(
                            switch_context,
                            switch_context,
                            stmt,
                        );
                    }
                }
            }

            OxcStatement::BlockStatement(stmt) => {
                self.build_context(
                    current_context,
                    Body::Statements(&stmt.body),
                );
            }
            _ => return,
        }
    }

    fn visit_arrow_statement<'b>(
        &mut self,
        parent_context: usize,
        current_context: usize,
        func: &ArrowFunctionExpression<'b>,
    ) {
        let new_context = self.new_context(current_context);

        match &func.body {
            ArrowFunctionBody::FunctionBody(body) => {
                self.build_context(
                    new_context,
                    Body::Statements(&body.statements),
                );
            }

            _ => (),
        };

        let mut params = Vec::new();

        for param in &func.params.items {
            // let from = self.context_tree.get(fn_context).unwrap();

            let bindings = self.collect_bindings(&param.pattern);

            for binding in bindings {
                params.push(binding.clone());
                self.context_tree
                    .get_mut(new_context)
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
                    .get_mut(new_context)
                    .unwrap()
                    .symbols
                    .insert(binding, Symbol::ParameterRest { span: rest.span });
            }
        }
    }

    fn visit_func_statement<'b>(
        &mut self,
        parent_context: usize,
        current_context: usize,
        func: &Function<'b>,
    ) {
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
                    .insert(binding, Symbol::ParameterRest { span: rest.span });
            }
        }

        if let Some(name) = func.name() {
            self.context_tree
                .get_mut(fn_context)
                .unwrap()
                .symbols
                .insert(
                    name.to_string(),
                    Symbol::Function {
                        name: name.to_string(),
                        params,
                        context: fn_context,
                    },
                );
        }
    }

    fn build_context<'b>(
        &mut self,
        current_context: usize,
        body: Body<'b>,
    ) -> usize {
        match body {
            Body::Statement(stmt) => {
                let enum_entry = format!("{:?}", stmt);
                let idx = enum_entry.find("(");

                if let Some(idx) = idx {
                    println!(
                        "{} | current: {current_context}",
                        &enum_entry[..idx]
                    )
                } else {
                    println!("{enum_entry} | current: {current_context}",)
                }

                self.visit_statement(current_context, current_context, stmt);
                current_context
            }

            Body::Statements(stmts) => {
                let new_context = self.new_context(current_context);

                for stmt in stmts {
                    let enum_entry = format!("{:?}", stmt);
                    let idx = enum_entry.find("(");

                    if let Some(idx) = idx {
                        println!(
                            "{} | parent: {current_context} | current: {new_context}",
                            &enum_entry[..idx]
                        )
                    } else {
                        println!(
                            "{enum_entry} | parent: {current_context} | current: {new_context}",
                        )
                    }

                    self.visit_statement(current_context, new_context, &stmt)
                }

                new_context
            }

            Body::Expression(expr) => match expr {
                OxcExpression::ArrowFunctionExpression(arrow_func) => {
                    println!(
                        "ArrowFunctionExpression | parent: {current_context} | current: {current_context}",
                    );
                    self.visit_arrow_statement(
                        current_context,
                        current_context,
                        &arrow_func,
                    );

                    current_context
                }

                OxcExpression::FunctionExpression(func_expr) => {
                    println!(
                        "FunctionExpression | parent: {current_context} | current: {current_context}",
                    );

                    self.visit_func_statement(
                        current_context,
                        current_context,
                        &func_expr,
                    );

                    current_context
                }

                _ => {
                    unreachable!()
                }
            },
        }
    }

    pub fn resolve_symbol(
        &self,
        present_context: usize,
        symbol_name: &str,
    ) -> Option<Symbol> {
        if present_context >= self.context_tree.len() {
            panic!(
                "invalid present context {}. there are {} contexts",
                present_context,
                self.context_tree.len()
            );
        }

        let ctxt = self.context_tree.get(present_context).unwrap();

        let symbol = ctxt.symbols.get(symbol_name);

        if symbol.is_some() {
            if present_context == 0 {
                // we return Some(Symbol) only IF it was found at the global context
                Some(symbol.unwrap().clone())
            } else {
                None
            }
        } else {
            if ctxt.parent == present_context {
                // global context
                None
            } else {
                self.resolve_symbol(ctxt.parent, symbol_name)
            }
        }
    }

    pub fn analyze_statement<'b>(
        &mut self,
        current_context: usize,
        stmt: &OxcStatement<'b>,
    ) {
        // any statement that has the potential to open a scope, will track it
        // we are guaranteed to visit the expressions in the same order we discovered them

        match stmt {
            OxcStatement::VariableDeclaration(decl) => {
                for decl in &decl.declarations {
                    if let Some(init) = &decl.init {
                        self.analyze_expression(current_context, &init);
                    }
                }
            }

            OxcStatement::FunctionDeclaration(func) => {
                for param in &func.params.items {
                    if let Some(init) = &param.initializer {
                        self.analyze_expression(current_context, init);
                    }
                }

                if let Some(body) = &func.body {
                    let fn_context = self.enter_context();
                    self.analyze_use(
                        fn_context,
                        Body::Statements(&body.statements),
                    );
                }
            }

            OxcStatement::ForStatement(for_stmt) => {
                let for_context = self.enter_context();
                if let Some(init) = &for_stmt.init {
                    match init {
                        ForStatementInit::VariableDeclaration(decl) => {
                            for decl in &decl.declarations {
                                if let Some(init) = &decl.init {
                                    self.analyze_expression(
                                        current_context,
                                        &init,
                                    );
                                }
                            }
                        }
                        _ => {
                            self.analyze_expression(
                                current_context,
                                &init.as_expression().unwrap(),
                            );
                        }
                    }
                }

                self.analyze_use(for_context, Body::Statement(&for_stmt.body));
            }

            OxcStatement::ForInStatement(for_in_stmt) => {
                let for_context = self.enter_context();

                self.analyze_expression(current_context, &for_in_stmt.right);

                self.analyze_use(
                    for_context,
                    Body::Statement(&for_in_stmt.body),
                );
            }

            OxcStatement::ForOfStatement(for_of_stmt) => {
                let for_context = self.enter_context();

                self.analyze_expression(current_context, &for_of_stmt.right);

                self.analyze_use(
                    for_context,
                    Body::Statement(&for_of_stmt.body),
                );
            }

            OxcStatement::IfStatement(stmt) => {
                self.analyze_expression(current_context, &stmt.test);

                self.analyze_use(
                    current_context,
                    Body::Statement(&stmt.consequent),
                );

                if let Some(alternate) = &stmt.alternate {
                    self.analyze_use(
                        current_context,
                        Body::Statement(&alternate),
                    );
                }
            }

            OxcStatement::WhileStatement(stmt) => {
                self.analyze_expression(current_context, &stmt.test);
                self.analyze_use(current_context, Body::Statement(&stmt.body));
            }

            OxcStatement::DoWhileStatement(stmt) => {
                self.analyze_expression(current_context, &stmt.test);

                self.analyze_use(current_context, Body::Statement(&stmt.body));
            }

            OxcStatement::WithStatement(stmt) => {
                self.analyze_expression(current_context, &stmt.object);
                self.analyze_use(current_context, Body::Statement(&stmt.body));
            }

            OxcStatement::LabeledStatement(stmt) => {
                self.analyze_use(current_context, Body::Statement(&stmt.body));
            }

            OxcStatement::TryStatement(stmt) => {
                self.analyze_use(
                    current_context,
                    Body::Statements(&stmt.block.body),
                );

                // catch
                if let Some(catch_block) = &stmt.handler {
                    self.analyze_use(
                        current_context,
                        Body::Statements(&catch_block.body.body),
                    );
                }

                // finally
                if let Some(finally_block) = &stmt.finalizer {
                    self.analyze_use(
                        current_context,
                        Body::Statements(&finally_block.body),
                    );
                }
            }

            OxcStatement::SwitchStatement(stmt) => {
                let switch_context = self.enter_context();

                self.analyze_expression(current_context, &stmt.discriminant);

                for case in &stmt.cases {
                    if let Some(test) = &case.test {
                        self.analyze_expression(current_context, test);
                    }

                    self.analyze_use(
                        switch_context,
                        Body::Statements(&case.consequent),
                    );
                }
            }

            OxcStatement::BlockStatement(stmt) => {
                let block_context = self.enter_context();
                self.analyze_use(block_context, Body::Statements(&stmt.body));
            }

            OxcStatement::ExpressionStatement(expr) => {
                self.analyze_expression(current_context, &expr.expression);
            }

            OxcStatement::ReturnStatement(stmt) => {
                if let Some(argument) = &stmt.argument {
                    self.analyze_expression(current_context, argument);
                }
            }

            OxcStatement::ThrowStatement(stmt) => {
                self.analyze_expression(current_context, &stmt.argument);
            }
            _ => return,
        }
    }

    pub fn analyze_assignment_target<'b>(
        &mut self,
        current_context: usize,
        target: &AssignmentTarget<'b>,
    ) {
        match target {
            AssignmentTarget::AssignmentTargetIdentifier(ident) => {
                let name = ident.name.to_string();

                if let Some(symbol) =
                    self.resolve_symbol(current_context, &name)
                {
                    self.uses.push(ident.span)
                }
            }

            AssignmentTarget::TSAsExpression(as_expr) => {
                self.analyze_expression(current_context, &as_expr.expression);
            }

            AssignmentTarget::TSSatisfiesExpression(satisfies) => {
                self.analyze_expression(current_context, &satisfies.expression);
            }

            AssignmentTarget::TSNonNullExpression(non_null) => {
                self.analyze_expression(current_context, &non_null.expression);
            }

            AssignmentTarget::TSTypeAssertion(assert) => {
                self.analyze_expression(current_context, &assert.expression);
            }

            AssignmentTarget::ArrayAssignmentTarget(target) => {
                for elem in &target.elements {
                    if let Some(elem) = elem {
                        match elem {
                            AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(default) => {
                                self.analyze_assignment_target(current_context, &default.binding);
                                self.analyze_expression(current_context, &default.init);
                            }
                            _ => {
                                if let Some(elem) = elem.as_assignment_target() {

                                self.analyze_assignment_target(current_context, elem);
                                }
                            }
                        }
                    }
                }
            }

            AssignmentTarget::ObjectAssignmentTarget(target) => {
                for prop in &target.properties {
                    match prop {
                        AssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident)=> {
                            if let Some(init) = &ident.init {
                                self.analyze_expression(current_context, &init);
                            }
                        }

                        AssignmentTargetProperty::AssignmentTargetPropertyProperty(prop) => {
                            match &prop.binding {
                                 AssignmentTargetMaybeDefault::AssignmentTargetWithDefault(def) => {
                                    self.analyze_expression(current_context, &def.init);
                                 }

                                 _ => {
                                     self.analyze_assignment_target(current_context, prop.binding.to_assignment_target());
                                 }
                            }
                        }
                    }
                }
            }

            _ => {
                self.analyze_expression(
                    current_context,
                    &target.to_member_expression().as_expression(),
                );
            }
        }
    }

    pub fn analyze_expression<'b>(
        &mut self,
        current_context: usize,
        expr: &OxcExpression<'b>,
    ) {
        match expr {
            OxcExpression::TemplateLiteral(literal) => {
                for expression in &literal.expressions {
                    self.analyze_expression(current_context, expression);
                }
            }

            OxcExpression::Identifier(ident) => {
                let name = ident.name.to_string();

                if let Some(symbol) =
                    self.resolve_symbol(current_context, &name)
                {
                    self.uses.push(ident.span)
                }
            }

            OxcExpression::ArrayExpression(arr) => {
                for elem in &arr.elements {
                    match elem {
                        ArrayExpressionElement::SpreadElement(elem) => {
                            self.analyze_expression(
                                current_context,
                                &elem.argument,
                            );
                        }

                        ArrayExpressionElement::Elision(elision) => {
                            // this node only stores a span. nothing to visit.
                            continue;
                        }

                        _ => {
                            self.analyze_expression(
                                current_context,
                                &elem.to_expression(),
                            );
                        }
                    }
                }
            }

            OxcExpression::ArrowFunctionExpression(arrow_func) => {
                for param in &arrow_func.params.items {
                    if let Some(init) = &param.initializer {
                        self.analyze_expression(current_context, init);
                    }
                }

                let arrow_context = self.enter_context();
                match &arrow_func.body {
                    ArrowFunctionBody::FunctionBody(body) => self.analyze_use(
                        arrow_context,
                        Body::Statements(&body.statements),
                    ),

                    _ => {
                        self.analyze_expression(
                            arrow_context,
                            &arrow_func.body.to_expression(),
                        );
                    }
                }
            }

            OxcExpression::AssignmentExpression(assignment) => {
                self.analyze_assignment_target(
                    current_context,
                    &assignment.left,
                );

                self.analyze_expression(current_context, &assignment.right);
            }

            OxcExpression::BinaryExpression(expr) => {
                self.analyze_expression(current_context, &expr.left);
                self.analyze_expression(current_context, &expr.right);
            }

            OxcExpression::CallExpression(call) => {
                self.analyze_expression(current_context, &call.callee);
                for arg in &call.arguments {
                    match arg {
                        Argument::SpreadElement(elem) => {
                            self.analyze_expression(
                                current_context,
                                &elem.argument,
                            );
                        }

                        _ => {
                            self.analyze_expression(
                                current_context,
                                arg.to_expression(),
                            );
                        }
                    }
                }
            }

            OxcExpression::ChainExpression(chain) => match &chain.expression {
                ChainElement::CallExpression(call) => {
                    self.analyze_expression(current_context, &call.callee);
                    for arg in &call.arguments {
                        match arg {
                            Argument::SpreadElement(elem) => {
                                self.analyze_expression(
                                    current_context,
                                    &elem.argument,
                                );
                            }

                            _ => {
                                self.analyze_expression(
                                    current_context,
                                    arg.to_expression(),
                                );
                            }
                        }
                    }
                }

                ChainElement::TSNonNullExpression(expr) => {
                    self.analyze_expression(current_context, &expr.expression);
                }

                _ => {
                    self.analyze_expression(
                        current_context,
                        chain.expression.to_member_expression().as_expression(),
                    );
                }
            },

            OxcExpression::ConditionalExpression(conditional) => {
                self.analyze_expression(current_context, &conditional.test);
                self.analyze_expression(
                    current_context,
                    &conditional.consequent,
                );
                self.analyze_expression(
                    current_context,
                    &conditional.alternate,
                );
            }

            OxcExpression::FunctionExpression(func) => {
                for param in &func.params.items {
                    if let Some(init) = &param.initializer {
                        self.analyze_expression(current_context, init);
                    }
                }

                if let Some(body) = &func.body {
                    let fn_context = self.enter_context();

                    self.analyze_use(
                        fn_context,
                        Body::Statements(&body.statements),
                    );
                }
            }

            OxcExpression::LogicalExpression(logical) => {
                self.analyze_expression(current_context, &logical.left);
                self.analyze_expression(current_context, &logical.right);
            }

            OxcExpression::NewExpression(new) => {
                self.analyze_expression(current_context, &new.callee);

                for arg in &new.arguments {
                    match arg {
                        Argument::SpreadElement(elem) => {
                            self.analyze_expression(
                                current_context,
                                &elem.argument,
                            );
                        }

                        _ => {
                            self.analyze_expression(
                                current_context,
                                arg.to_expression(),
                            );
                        }
                    }
                }
            }

            OxcExpression::ObjectExpression(obj) => {
                for prop in &obj.properties {
                    match prop {
                        ObjectPropertyKind::ObjectProperty(obj_prop) => {
                            match &obj_prop.key {
                                PropertyKey::StaticIdentifier(_) => (),
                                PropertyKey::PrivateIdentifier(_) => (),
                                _ => self.analyze_expression(
                                    current_context,
                                    obj_prop.key.to_expression(),
                                ),
                            }

                            self.analyze_expression(
                                current_context,
                                &obj_prop.value,
                            );
                        }

                        ObjectPropertyKind::SpreadProperty(spread) => {
                            self.analyze_expression(
                                current_context,
                                &spread.argument,
                            );
                        }
                    }
                }
            }

            OxcExpression::ParenthesizedExpression(paren) => {
                self.analyze_expression(current_context, &paren.expression);
            }

            OxcExpression::SequenceExpression(seq) => {
                for expr in &seq.expressions {
                    self.analyze_expression(current_context, expr);
                }
            }

            OxcExpression::TaggedTemplateExpression(template) => {
                self.analyze_expression(current_context, &template.tag);

                for expr in &template.quasi.expressions {
                    self.analyze_expression(current_context, expr);
                }
            }

            OxcExpression::UnaryExpression(unary) => {
                self.analyze_expression(current_context, &unary.argument);
            }

            OxcExpression::UpdateExpression(update) => {
                self.analyze_assignment_target(
                    current_context,
                    update.argument.as_assignment_target(),
                );
            }

            OxcExpression::YieldExpression(yield_expr) => {
                if let Some(expr) = &yield_expr.argument {
                    self.analyze_expression(current_context, expr)
                }
            }

            OxcExpression::TSAsExpression(as_expr) => {
                self.analyze_expression(current_context, &as_expr.expression);
            }

            OxcExpression::TSSatisfiesExpression(satisfies) => {
                self.analyze_expression(current_context, &satisfies.expression);
            }

            OxcExpression::TSNonNullExpression(non_null) => {
                self.analyze_expression(current_context, &non_null.expression);
            }

            OxcExpression::TSTypeAssertion(assert) => {
                self.analyze_expression(current_context, &assert.expression);
            }

            OxcExpression::TSInstantiationExpression(expr) => {
                self.analyze_expression(current_context, &expr.expression);
            }

            OxcExpression::ComputedMemberExpression(computed_member) => {
                self.analyze_expression(
                    current_context,
                    &computed_member.object,
                );

                self.analyze_expression(
                    current_context,
                    &computed_member.expression,
                );
            }

            OxcExpression::StaticMemberExpression(static_member) => {
                self.analyze_expression(current_context, &static_member.object);
            }

            _ => {
                return;
            }
        }
    }

    fn analyze_use<'b>(&mut self, current_context: usize, body: Body<'b>) {
        /*
         * We are interested in resolving the use of global variables and global functions only,
         * as we need to perform transformations on them during codegen.
         *
         * The gaol is to visit nodes of interest, being:
         * - Identifier
         * - Member Expression
         * - VariableDeclarator (init)
         * - AssignmentExpression (lhs=read, rhs=write)
         * - UpdateExpression (read-write)
         * - CallExpression (callee, arguments)
         * - Expression containers (recursively visit children)
         * - Control-flow expressions (test, init, update, discriminant etc.)
         * - ReturnStatement (argument)
         */

        match body {
            Body::Statement(stmt) => {
                let enum_entry = format!("{:?}", stmt);
                let idx = enum_entry.find("(");

                if let Some(idx) = idx {
                    println!("{} | ctxt: {current_context}", &enum_entry[..idx])
                } else {
                    println!("{enum_entry} | ctxt: {current_context}")
                }

                self.analyze_statement(current_context, stmt);
            }

            Body::Statements(stmts) => {
                for stmt in stmts {
                    let enum_entry = format!("{:?}", stmt);
                    let idx = enum_entry.find("(");

                    if let Some(idx) = idx {
                        println!(
                            "{} | ctxt: {current_context}",
                            &enum_entry[..idx]
                        )
                    } else {
                        println!("{enum_entry} | ctxt: {current_context}")
                    }

                    self.analyze_statement(current_context, stmt);
                }
            }

            Body::Expression(expr) => {
                let enum_entry = format!("{:?}", expr);
                let idx = enum_entry.find("(");

                if let Some(idx) = idx {
                    println!("{} | ctxt: {current_context}", &enum_entry[..idx])
                } else {
                    println!("{enum_entry} | ctxt: {current_context}")
                }

                self.analyze_expression(current_context, expr);
            }
        }
    }

    pub fn analyze_program(
        &mut self,
        oxc_allocator: &OxcAllocator,
        source: &str,
    ) {
        let body = Self::parse(source, oxc_allocator);

        println!("Build Context");
        self.build_context(0, Body::Statements(&body));
        println!("\n\nAnalyze Use");
        let global = self.enter_context();
        self.analyze_use(global, Body::Statements(&body));
        println!();
    }
}
