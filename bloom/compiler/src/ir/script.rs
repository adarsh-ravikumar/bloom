use std::{collections::HashMap, iter::Skip};

use crate::{common::Span, ir::template::Expression};
use oxc_allocator::{Allocator as OxcAllocator, Vec as OxcVec};

#[rustfmt::skip]
use oxc_ast::ast::{
    Argument as OxcArgument,
    ArrayExpressionElement as OxcArrayExpressionElement, 
    ArrowFunctionBody as OxcArrowFunctionBody, 
    AssignmentTarget as OxcAssignmentTarget, 
    AssignmentTargetMaybeDefault as OxcAssignmentTargetMaybeDefault, 
    AssignmentTargetPattern as OxcAssignmentTargetPattern, 
    AssignmentTargetProperty as OxcAssignmentTargetProperty, 
    BindingPattern as OxcBindingPattern, 
    ChainElement as OxcChainElement, 
    Expression as OxcExpression, 
    ForStatementLeft as OxcForStatementLeft, 
    FormalParameters as OxcFormalParameters, 
    FunctionBody as OxcFunctionBody, 
    MemberExpression as OxcMemberExpression, 
    ObjectPropertyKind as OxcObjectPropertyKind, 
    SimpleAssignmentTarget as OxcSimpleAssignmentTarget, 
    Statement as OxcStatement,
    TSType as OxcTSType,
    VariableDeclaration as OxcVariableDeclaration,
    ForStatementInit as OxcForStatementInit, 
    TSLiteral as OxcTSLiteral, 
    TSTupleElement as OxcTSTupleElement,
    TSTypeName as OxcTSTypeName,
    TSSignature as OxcTSSignature,
    PropertyKey as OxcPropertyKey
};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span as OxcSpan};

#[derive(Debug)]
pub struct SemanticAnalyzer<'a> {
    source: &'a str,

    pub context_tree: Vec<SymbolTable>,
    pub uses: Vec<SymbolUse>,
}

#[derive(Debug, Clone)]
pub enum TypeAnnotation {
    Number(OxcSpan), // 64-bit IEEE-754
    String(OxcSpan), // UTF-8
    Boolean(OxcSpan),

    Array((OxcSpan, Box<TypeAnnotation>)),
    Tuple((OxcSpan, Vec<TypeAnnotation>)),
    NamedTuple((OxcSpan, Vec<(String, TypeAnnotation)>)),

    // this can occur only inside the NamedTuple array, but is a part of
    // TypeAnnotation to preserve the simplicity
    NamedTupleMember((OxcSpan, String, Box<TypeAnnotation>)),

    Union((OxcSpan, Vec<TypeAnnotation>)),
    Literal((OxcSpan, Literal)),

    TypeReference((OxcSpan, String)),

    Disallowed((OxcSpan, &'static str)),
}

#[derive(Debug, Clone)]
pub enum NamedType {
    Alias {
        type_annotation: TypeAnnotation,
        generics: Option<OxcSpan>,
    },

    Interface {
        fields: Vec<(Option<String>, TypeAnnotation)>,
        generics: Option<OxcSpan>,
    },

    Enum(Vec<(String, Option<EnumValue>)>),
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),    // 64-bit IEEE-754
    String(String), // UTF-8
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub enum EnumValue {
    Number(f64),    // 64-bit IEEE-754
    String(String), // UTF-8
    Invalid,
}

#[derive(Debug, Clone)]
pub enum Symbol {
    Variable {
        span: OxcSpan,
        type_annotation: Option<TypeAnnotation>,
    },

    Function {
        name: String,
        context: usize,
        body: Option<OxcSpan>,
        return_type: Option<TypeAnnotation>,
        // bloom does NOT allow for generics.
        // store here if any for diagnostics
        generics: Option<OxcSpan>,
    },

    Parameter {
        span: OxcSpan,
        optional: bool,
        order: usize,
        type_annotation: Option<TypeAnnotation>,
    },

    ParameterRest {
        span: OxcSpan,
    },

    Type(NamedType),
}

#[derive(Debug, Clone)]
pub struct SymbolUse {
    pub span: OxcSpan,
    pub context: usize,
}

#[derive(Debug)]
pub struct SymbolTable {
    pub symbols: HashMap<String, Symbol>,
    pub context: usize,
    pub parent: usize,
}

impl<'a, 'b> SemanticAnalyzer<'b>
where
    'a: 'b,
{
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            context_tree: Vec::new(),
            uses: Vec::new(),
        }
    }

    fn parse(
        source: &'a str,
        oxc_allocator: &'a OxcAllocator,
    ) -> OxcVec<'b, OxcStatement<'b>> {
        let parser = Parser::new(oxc_allocator, source, SourceType::ts());

        let result = parser.parse();

        if !result.diagnostics.is_empty() {
            // TODO: Render diagnostics
            println!("{:?}", result.diagnostics);
        }

        result.program.body
    }

    fn create_context(&mut self, parent: usize) -> usize {
        let new_context = self.context_tree.len();

        self.context_tree.push(SymbolTable {
            symbols: HashMap::new(),
            context: new_context,
            parent,
        });

        new_context
    }

    fn insert_in_context(
        &mut self,
        context: usize,
        name: String,
        symbol: Symbol,
    ) {
        let Some(context) = self.context_tree.get_mut(context) else {
            unreachable!(
                "invalid context {context}. maximum allowed context is  {}",
                self.context_tree.len() - 1
            );
        };

        context.symbols.insert(name, symbol);
    }

    fn expression(
        &mut self,
        context: usize,
        expr: &OxcExpression<'a>,
        uses: &mut Vec<SymbolUse>,
    ) {
        match expr {
            // these need not be visited
            OxcExpression::BooleanLiteral(_) => return,
            OxcExpression::NullLiteral(_) => return,
            OxcExpression::NumericLiteral(_) => return,
            OxcExpression::BigIntLiteral(_) => return,
            OxcExpression::RegExpLiteral(_) => return,
            OxcExpression::StringLiteral(_) => return,
            OxcExpression::V8IntrinsicExpression(_) => return,
            OxcExpression::Super(_) => return,
            OxcExpression::ClassExpression(_) => return,
            OxcExpression::ImportExpression(_) => return,
            OxcExpression::ThisExpression(_) => return,
            OxcExpression::ImportMeta(_) => return,
            OxcExpression::NewTarget(_) => return,

            // jsx expressions are guaranteed to NOT occur
            OxcExpression::JSXElement(_) => unreachable!(),
            OxcExpression::JSXFragment(_) => unreachable!(),

            OxcExpression::TemplateLiteral(expr) => {
                /*
                 * struct oxc_ast::ast::TemplateLiteral<'a>
                 * - expressions: oxc_allocator::Vec<'a, oxc_ast::ast::Expression<'a>>
                 *
                 * quasis is ignored as it only contains static template text.
                 */

                for expr in &expr.expressions {
                    self.expression(context, expr, uses);
                }
            }

            OxcExpression::Identifier(expr) => {
                /*
                 * struct oxc_ast::ast::IdentifierReference<'a>
                 * - span: oxc_span::Span
                 */

                uses.push(SymbolUse {
                    span: expr.span,
                    context,
                });
            }

            OxcExpression::ArrayExpression(expr) => {
                /*
                 * struct oxc_ast::ast::ArrayExpression<'a>
                 * - elements: oxc_allocator::Vec<'a, oxc_ast::ast::ArrayExpressionElement<'a>>)
                 */

                for element in &expr.elements {
                    /*
                     * enum oxc_ast::ast::ArrayExpressionElement<'a>
                     * : SpreadElement(oxc_ast::ast::SpreadElement<'a>),
                     * : Elision(oxc_ast::ast::Elision),
                     * : oxc_ast::ast::Expression<'a> variants
                     */

                    match element {
                        /*
                         * struct oxc_ast::ast::Elision
                         * : this represents the lack of an element, and hence can be skipped
                         */
                        OxcArrayExpressionElement::Elision(_) => continue,

                        /*
                         * struct oxc_ast::ast::SpreadElement<'a>
                         * - argument: Expression<'a>
                         */
                        OxcArrayExpressionElement::SpreadElement(spread) => {
                            self.expression(context, &spread.argument, uses);
                        }

                        _ => {
                            if let Some(expr) = element.as_expression() {
                                self.expression(context, expr, uses);
                            } else {
                                unreachable!(
                                    "unhandled oxc_ast::ast::ArrayExpressionElement | {expr:?}"
                                );
                            }
                        }
                    }
                }
            }

            OxcExpression::ArrowFunctionExpression(expr) => {
                /*
                 * struct oxc_ast::ast::ArrowFunctionExpression<'a>
                 * - params: Box<'a, oxc_ast::ast::FormalParameters<'a>>,
                 * - body: oxc_ast::ast::ArrowFunctionBody<'a>,
                 */

                // creates a new semantic context.
                let new_context = self.create_context(context);

                self.formal_parameters(new_context, &expr.params, uses);

                /*
                 * enum oxc_ast::ast::ArrowFunctionBody
                 * : FunctionBody(Box<'a, oxc_ast::ast::FunctionBody<'a>>)
                 * : oxc_ast::ast::Expression<'a> variants
                 */
                match &expr.body {
                    OxcArrowFunctionBody::FunctionBody(body) => {
                        self.function_body(new_context, body, uses);
                    }

                    _ => {
                        if let Some(expr) = &expr.body.as_expression() {
                            self.expression(new_context, expr, uses);
                        } else {
                            unreachable!(
                                "unhandled oxc_ast::ast::ArrowFunctionBody | {expr:?}"
                            );
                        }
                    }
                }
            }

            OxcExpression::AssignmentExpression(expr) => {
                /*
                 * struct oxc_ast::ast::AssignmentExpression<'a>
                 * - left: oxc_ast::ast::AssignmentTarget<'a>
                 * - right: oxc_ast::ast::Expression<'a>
                 */

                self.assignment_target(context, &expr.left, uses);
                self.expression(context, &expr.right, uses);
            }

            OxcExpression::AwaitExpression(expr) => {
                /*
                 * struct oxc_ast::ast::AwaitExpression<'a>
                 * - argument: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &expr.argument, uses);
            }

            OxcExpression::BinaryExpression(expr) => {
                /*
                 * struct oxc_ast::ast::BinaryExpression<'a>
                 * - left: oxc_ast::ast::Expression<'a>
                 * - right: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &expr.left, uses);
                self.expression(context, &expr.right, uses);
            }

            OxcExpression::CallExpression(expr) => {
                self.callee_arguments(
                    context,
                    &expr.callee,
                    &expr.arguments,
                    uses,
                );
            }

            OxcExpression::ChainExpression(expr) => {
                /*
                 * struct oxc_ast::ast::ChainExpression<'a>
                 * - expression: oxc_ast::ast::ChainElement<'a>
                 */

                match &expr.expression {
                    /*
                     * enum oxc_ast::ast::ChainElement<'a>
                     * : CallExpression(Box<'a, CallExpression<'a>>)
                     * : TSNonNullExpression(Box<'a, TSNonNullExpression<'a>>)
                     * : oxc_ast::ast::MemberExpression<'a> variants
                     */
                    OxcChainElement::CallExpression(expr) => {
                        self.callee_arguments(
                            context,
                            &expr.callee,
                            &expr.arguments,
                            uses,
                        );
                    }

                    OxcChainElement::TSNonNullExpression(expr) => {
                        /*
                         * struct oxc_ast::ast::TSNonNullExpression<'a>
                         * - expression: oxc_ast::ast::Expression<'a>
                         */

                        self.expression(context, &expr.expression, uses);
                    }

                    _ => {
                        if let Some(expr) =
                            expr.expression.as_member_expression()
                        {
                            self.member(context, expr, uses);
                        } else {
                            unreachable!(
                                "unhandled oxc_ast::ast::Element | {expr:?}"
                            );
                        }
                    }
                }
            }

            OxcExpression::ConditionalExpression(expr) => {
                /*
                 * struct oxc_ast::ast::ConditionalExpression<'a>
                 * - test: oxc_ast::ast::Expression<'a>
                 * - consequent: oxc_ast::ast::Expression<'a>
                 * - alternate: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &expr.test, uses);
                self.expression(context, &expr.consequent, uses);
                self.expression(context, &expr.alternate, uses);
            }

            OxcExpression::FunctionExpression(expr) => {
                /*
                 * struct oxc_ast::ast::FunctionExpression<'a>
                 * - params: Box<'a, oxc_ast::ast::FormalParameters<'a>>,
                 * - body: oxc_ast::ast::FunctionBody<'a>,
                 * - id: Option<oxc_ast::ast::BindingIdentifier<'a>>
                 */

                // creates a new semantic context.
                let new_context = self.create_context(context);

                self.formal_parameters(new_context, &expr.params, uses);

                if let Some(body) = &expr.body {
                    let body = self.function_body(new_context, body, uses);

                    // if the body itself doesn't exist, then it doesn't make a
                    // difference if the function's name is tracked
                    if let Some(id) = &expr.id {
                        self.insert_in_context(
                            new_context,
                            id.name.to_string(),
                            Symbol::Function {
                                name: id.name.to_string(),
                                context: new_context,
                                body: Some(body),
                                // we needn't care about the type annotation and
                                // generics for a function expression.
                                return_type: None,
                                generics: None,
                            },
                        );
                    }
                }
            }

            OxcExpression::LogicalExpression(expr) => {
                /*
                 * struct oxc_ast::ast::LogicalExpression<'a>
                 * - left: oxc_ast::ast::Expression<'a>
                 * - right: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &expr.left, uses);
                self.expression(context, &expr.right, uses);
            }

            OxcExpression::NewExpression(expr) => {
                /*
                 * struct oxc_ast::ast::NewExpression<'a>
                 * - callee: oxc_ast::ast::Expression<'a>
                 * - arguments: oxc_allocator::Vec<'a, oxc_ast::ast::<'a>>
                 */

                self.callee_arguments(
                    context,
                    &expr.callee,
                    &expr.arguments,
                    uses,
                );
            }

            OxcExpression::ObjectExpression(expr) => {
                /*
                 * struct oxc_ast::ast::ObjectExpression<'a>
                 * - properties: oxc_allocator::Vec<'a, oxc_ast::ast::ObjectPropertyKind<'a>>
                 */

                for prop in &expr.properties {
                    /*
                     * enum oxc_ast::ast::ObjectPropertyKind<'a>
                     * : ObjectProperty(Box<'a, ObjectProperty<'a>>)
                     * : SpreadProperty(Box<'a, SpreadElement<'a>>)
                     */

                    match prop {
                        /*
                         * struct oxc_ast::ast::ObjectProperty<'a>
                         * - value: oxc_ast::ast::Expression<'a>
                         */
                        OxcObjectPropertyKind::ObjectProperty(obj) => {
                            self.expression(context, &obj.value, uses);
                        }

                        /*
                         * struct oxc_ast::ast::SpreadElement<'a>
                         * - argument: oxc_ast::ast::Expression<'a>
                         */
                        OxcObjectPropertyKind::SpreadProperty(spread) => {
                            self.expression(context, &spread.argument, uses);
                        }
                    }
                }
            }

            OxcExpression::ParenthesizedExpression(expr) => {
                /*
                 * struct oxc_ast::ast::ParenthesizedExpression<'a>
                 * - expression: oxc_ast::ast::Expression<'a>
                 */
                self.expression(context, &expr.expression, uses);
            }

            OxcExpression::SequenceExpression(expr) => {
                /*
                 * struct oxc_ast::ast::SequenceExpression<'a>
                 * - expressions: oxc_allocator::Vec<oxc_ast::ast::Expression<'a>>
                 */

                for expr in &expr.expressions {
                    self.expression(context, expr, uses);
                }
            }

            OxcExpression::TaggedTemplateExpression(expr) => {
                /*
                 * struct oxc_ast::ast::TaggedTemplateLiteral<'a>
                 * - tag: oxc_ast::ast::Expression<'a>
                 * - quasi: oxc_ast::ast::TemplateLiteral<'a>
                 */

                self.expression(context, &expr.tag, uses);

                /*
                 * struct oxc_ast::ast::TemplateLiteral<'a>
                 * - expressions: oxc_allocator::Vec<'a, oxc_ast::ast::Expression<'a>>
                 *
                 * quasis is ignored as it only contains static template text.
                 */

                for expr in &expr.quasi.expressions {
                    self.expression(context, expr, uses);
                }
            }

            OxcExpression::UnaryExpression(expr) => {
                /*
                 * struct oxc_ast::ast::UnaryExpression<'a>
                 * - argument: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &expr.argument, uses);
            }

            OxcExpression::UpdateExpression(expr) => {
                /*
                 * struct oxc_ast::ast::UpdateExpression<'a>
                 * - argument: oxc_ast::ast::SimpleAssignmentTarget<'a>
                 */

                self.simple_assignment_target(context, &expr.argument, uses);
            }

            OxcExpression::YieldExpression(expr) => {
                /*
                 * struct oxc_ast::ast::YieldExpression<'a>
                 * - argument: Option<oxc_ast::ast::Expression<'a>>
                 */

                if let Some(expr) = &expr.argument {
                    self.expression(context, expr, uses);
                }
            }

            OxcExpression::PrivateInExpression(expr) => {
                /*
                 * struct oxc_ast::ast::PrivateInExpression<'a>
                 * - right: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &expr.right, uses);
            }

            OxcExpression::TSAsExpression(expr) => {
                /*
                 * struct oxc_ast::ast::TSAsExpression<'a>
                 * - expression: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &expr.expression, uses);
            }

            OxcExpression::TSSatisfiesExpression(expr) => {
                /*
                 * struct oxc_ast::ast::TSSatisfiesExpression<'a>
                 * - expression: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &expr.expression, uses);
            }

            OxcExpression::TSTypeAssertion(expr) => {
                /*
                 * struct oxc_ast::ast::TSTypeAssertion<'a>
                 * - expression: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &expr.expression, uses);
            }

            OxcExpression::TSNonNullExpression(expr) => {
                /*
                 * struct oxc_ast::ast::TSNonNullExpression<'a>
                 * - expression: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &expr.expression, uses);
            }

            OxcExpression::TSInstantiationExpression(expr) => {
                /*
                 * struct oxc_ast::ast::TSInstantiationExpression<'a>
                 * - expression: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &expr.expression, uses);
            }

            _ => {
                if let Some(expr) = expr.as_member_expression() {
                    self.member(context, expr, uses);
                } else {
                    unreachable!("unhandled oxc_ast::ast::Element | {expr:?}");
                }
            }
        }
    }

    fn assignment_target(
        &mut self,
        context: usize,
        target: &OxcAssignmentTarget<'a>,
        uses: &mut Vec<SymbolUse>,
    ) {
        /*
         * enum oxc_ast::ast::AssignmentTarget<'a>
         * : oxc_ast::ast::SimpleAssignmentTarget<'a> variants
         * : oxc_ast::ast::AssignmentTargetPattern<'a> variants
         */

        if let Some(target) = target.as_simple_assignment_target() {
            self.simple_assignment_target(context, target, uses);
        } else if let Some(target) = target.as_assignment_target_pattern() {
            self.assignment_target_pattern(context, &target, uses);
        } else {
            unreachable!(
                "unhandled oxc_ast::ast::AssignmentTarget<'a> | {target:?}"
            );
        }
    }

    fn assignment_target_maybe_default(
        &mut self,
        context: usize,
        target: &OxcAssignmentTargetMaybeDefault<'a>,
        uses: &mut Vec<SymbolUse>,
    ) {
        /*
         * enum oxc_ast::ast::AssignmentTargetMaybeDefault<'a>
         * : AssignmentTargetWithDefault(Box<'a, oxc_ast::ast::AssignmentTargetWithDefault<'a>>)
         * : oxc_ast::ast::AssignmentTarget<'a> variants
         */

        match &target {
            OxcAssignmentTargetMaybeDefault::AssignmentTargetWithDefault(
                target,
            ) => {
                /*
                 * struct oxc_ast::ast::AssignmentTargetWithDefault<'a>
                 * - binding: oxc_ast::ast::AssignmentTarget<'a>
                 * - init: oxc_ast::ast::Expression<'a>
                 */

                self.assignment_target(context, &target.binding, uses);
                self.expression(context, &target.init, uses);
            }

            _ => {
                if let Some(target) = target.as_assignment_target() {
                    self.assignment_target(context, target, uses);
                } else {
                    unreachable!(
                        "unhandled oxc_ast::ast::AssignmentTargetMaybeDefault<'a> | {target:?}"
                    );
                }
            }
        }
    }

    fn assignment_target_pattern(
        &mut self,
        context: usize,
        target: &OxcAssignmentTargetPattern<'a>,
        uses: &mut Vec<SymbolUse>,
    ) {
        match target {
            OxcAssignmentTargetPattern::ArrayAssignmentTarget(target) => {
                /*
                 * struct oxc_ast::ast::ArrayAssignmentTarget<'a>
                 * - elements: oxc_allocator::Vec<'a, Option<AssignmentTargetMaybeDefault<'a>>>
                 * - rest: Option<Box<'a, AssignmentTargetRest<'a>>>
                 */

                for elem in &target.elements {
                    let Some(elem) = elem else {
                        continue;
                    };

                    self.assignment_target_maybe_default(context, elem, uses);
                }
            }

            OxcAssignmentTargetPattern::ObjectAssignmentTarget(target) => {
                /*
                 * struct oxc_ast::ast::ObjectAssignmentTarget<'a>
                 * - properties: oxc_allocator::Vec<'a, oxc_ast::ast::AssignmentTargetProperty<'a>>
                 * - rest: Option<Box<'a, oxc_ast::ast::AssignmentTargetRest<'a>>>,
                 */

                for prop in &target.properties {
                    /*
                     * enum oxc_ast::ast::AssignmentTargetProperty<'a>
                     * : AssignmentTargetPropertyIdentifier(Box<'a, AssignmentTargetPropertyIdentifier<'a>>)
                     * : AssignmentTargetPropertyProperty(Box<'a, AssignmentTargetPropertyProperty<'a>>)
                     */

                    match prop {
                        /*
                         * struct oxc_ast::ast::AssignmentTargetPropertyIdentifier<'a> 
                         * - binding: oxc_ast::ast::IdentifierReference<'a>
                         * - init: Option<Expression<'a>>
                         */

                        OxcAssignmentTargetProperty::AssignmentTargetPropertyIdentifier(ident) => {
                            /*
                            * struct oxc_ast::ast::IdentifierReference<'a>
                            * - span: oxc_span::Span
                            */

                            uses.push(
                                SymbolUse {
                                    span: ident.span,
                                    context
                                }
                            );

                        }

                        OxcAssignmentTargetProperty::AssignmentTargetPropertyProperty(property) => {
                            /*
                             * struct oxc_ast::ast::AssignmentTargetPropertyProperty<'a>
                             * - binding: oxc_ast::ast::AssignmentTargetMaybeDefault<'a>
                             */

                            self.assignment_target_maybe_default(context, &property.binding, uses);
                        }
                    }
                }
            }
        }
    }

    fn simple_assignment_target(
        &mut self,
        context: usize,
        target: &OxcSimpleAssignmentTarget<'a>,
        uses: &mut Vec<SymbolUse>,
    ) {
        match target {
            OxcSimpleAssignmentTarget::AssignmentTargetIdentifier(target) => {
                /*
                 * struct oxc_ast::ast::IdentifierReference<'a>
                 * - span: oxc_span::Span
                 */

                uses.push(SymbolUse {
                    span: target.span,
                    context,
                });
            }

            OxcSimpleAssignmentTarget::TSAsExpression(target) => {
                /*
                 * struct oxc_ast::ast::TSAsExpression<'a>
                 * - expression: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &target.expression, uses);
            }

            OxcSimpleAssignmentTarget::TSSatisfiesExpression(target) => {
                /*
                 * struct oxc_ast::ast::TSSatisfiesExpression<'a>
                 * - expression: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &target.expression, uses);
            }

            OxcSimpleAssignmentTarget::TSNonNullExpression(target) => {
                /*
                 * struct oxc_ast::ast::TSNonNullExpression<'a>
                 * - expression: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &target.expression, uses);
            }

            OxcSimpleAssignmentTarget::TSTypeAssertion(target) => {
                /*
                 * struct oxc_ast::ast::TSTypeAssertion<'a>
                 * - expression: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &target.expression, uses);
            }

            _ => {
                if let Some(member) = target.as_member_expression() {
                    self.member(context, member, uses);
                } else {
                    unreachable!(
                        "unhandled oxc_ast::ast::SimpleAssignmentTarget<'a> | {target:?}"
                    );
                }
            }
        }
    }

    fn function_body(
        &mut self,
        context: usize,
        body: &OxcFunctionBody<'a>,
        uses: &mut Vec<SymbolUse>,
    ) -> OxcSpan {
        // NOTE: we do NOT create a new context here. instead, we use the context passed in to the
        // parameter. this is done, so creation of the context and scanning the function body can
        // remain uncoupled.

        /*
         * struct oxc_ast::ast::FunctionBody<'a>
         * - span: oxc_ast::ast::Span
         * - statements: oxc_allocator::Vec<'a, oxc_ast::ast::Statement<'a>>
         */

        for stmt in &body.statements {
            self.statement(context, stmt, uses);
        }

        body.span
    }

    fn formal_parameters(
        &mut self,
        context: usize,
        params: &OxcFormalParameters<'a>,
        uses: &mut Vec<SymbolUse>,
    ) {
        /*
         * struct oxc_ast::ast::FormalParameters<'a>
         * - items: oxc_allocator::Vec<'a, FormalParameter<'a>>,
         * - rest: Option<Box<'a, FormalParameterRest<'a>>>,
         */

        for (order, param) in params.items.iter().enumerate() {
            /*
             * struct oxc_ast::ast::FormalParameter<'a>
             * - pattern: oxc_ast::ast::BindingPattern<'a>
             * - initializer: Option<Box<'a, Expression<'a>>>
             */

            let type_annotation = if let Some(t) = &param.type_annotation {
                Some(self.type_annotation(&t.type_annotation))
            } else {
                None
            };

            self.binding(
                context,
                Symbol::Parameter {
                    span: param.span,
                    optional: param.optional,
                    order,
                    type_annotation,
                },
                &param.pattern,
                uses,
            );

            if let Some(init) = &param.initializer {
                self.expression(context, init, uses);
            }
        }

        /*
         * struct oxc_ast::ast::FormalParametersRest<'a>
         * - rest: BindingRestElement<'a>
         */

        if let Some(rest) = &params.rest {
            /*
             * struct oxc_ast::ast::BindingRestElement<'a>
             * - argument: oxc_ast::ast::BindingPattern<'a>
             */
            self.binding(
                context,
                Symbol::ParameterRest { span: rest.span },
                &rest.rest.argument,
                uses,
            );
        }
    }

    fn callee_arguments(
        &mut self,
        context: usize,
        callee: &OxcExpression<'a>,
        args: &OxcVec<'a, OxcArgument<'a>>,
        uses: &mut Vec<SymbolUse>,
    ) {
        self.expression(context, callee, uses);

        for arg in args {
            /*
             * enum oxc_ast::ast::Argument<'a>
             * : SpreadElement(oxc_ast::ast::SpreadElement)
             * : oxc_ast::ast::Element<'a> variants
             */

            match arg {
                /*
                 * struct oxc_ast::ast::SpreadElement<'a>
                 * - argument: oxc_ast::ast::Expression<'a>
                 */
                OxcArgument::SpreadElement(spread) => {
                    self.expression(context, &spread.argument, uses);
                }

                _ => {
                    if let Some(expr) = arg.as_expression() {
                        self.expression(context, expr, uses);
                    } else {
                        unreachable!(
                            "unhandled oxc_ast::ast::Argument<'a> | {arg:?}"
                        );
                    }
                }
            }
        }
    }

    fn binding(
        &mut self,
        context: usize,
        symbol: Symbol,
        pattern: &OxcBindingPattern<'a>,
        uses: &mut Vec<SymbolUse>,
    ) {
        /*
         * enum oxc_ast::ast::BindingPattern<'a>
         * : BindingIdentifier(Box<'a, BindingIdentifier<'a>>)
         * : ObjectPattern(Box<'a, ObjectPattern<'a>>)
         * : ArrayPattern(Box<'a, ArrayPattern<'a>>)
         * : AssignmentPattern(Box<'a, AssignmentPattern<'a>>)
         */

        match pattern {
            OxcBindingPattern::BindingIdentifier(ident) => {
                /*
                 * struct oxc_ast::ast::BindingIdentifier<'a>
                 * - name: Ident<'a>
                 */
                self.insert_in_context(context, ident.name.to_string(), symbol)
            }

            OxcBindingPattern::ObjectPattern(obj) => {
                /*
                 * struct oxc_ast::ast::ObjectPattern<'a>
                 * - properties: oxc_allocator::Vec<'a, oxc_ast::ast::BindingProperty<'a>>
                 */

                for prop in &obj.properties {
                    /*
                     *struct oxc_ast::ast::BindingProperty<'a>
                     *- value: oxc_ast::ast::BindingPattern<'a>
                     */
                    self.binding(context, symbol.clone(), &prop.value, uses)
                }
            }

            OxcBindingPattern::ArrayPattern(arr) => {
                /*
                 * struct oxc_ast::ast::ArrayPattern<'a>
                 * - elements: oxc_allocator::Vec<'a, Option<BindingPattern<'a>>>
                 * - rest: Option<Box<'a, BindingRestElement<'a>>>
                 */

                for elem in &arr.elements {
                    if let Some(pattern) = elem {
                        self.binding(context, symbol.clone(), pattern, uses);
                    }
                }

                if let Some(rest) = &arr.rest {
                    /*
                     * struct oxc_ast::ast::BindingRestElement<'a>
                     * - argument: oxc_ast::ast::BindingPattern<'a>
                     */
                    self.binding(context, symbol, &rest.argument, uses);
                }
            }

            OxcBindingPattern::AssignmentPattern(assign) => {
                /*
                 * struct oxc_ast::ast::AssignmentPattern<'a>
                 * - left: oxc_ast::ast::BindingPattern<'a>
                 * - right: oxc_ast::ast::Expression<'a>
                 */
                self.binding(context, symbol, &assign.left, uses);
                self.expression(context, &assign.right, uses);
            }
        }
    }

    fn member(
        &mut self,
        context: usize,
        expr: &OxcMemberExpression<'a>,
        uses: &mut Vec<SymbolUse>,
    ) {
        /*
         * enum oxc_ast::ast::MemberExpression<'a>
         * : ComputedMemberExpression(Box<'a, ComputedMemberExpression<'a>>)
         * : StaticMemberExpression(Box<'a, StaticMemberExpression<'a>>)
         * : PrivateFieldExpression(Box<'a, PrivateFieldExpression<'a>>)
         */

        match expr {
            /*
             * struct oxc_ast::ast::ComputedMemberExpression<'a>
             * - object: oxc_ast::ast::Expression<'a>
             * - expression: oxc_ast::ast::Expression<'a>
             */
            OxcMemberExpression::ComputedMemberExpression(expr) => {
                self.expression(context, &expr.object, uses);
                self.expression(context, &expr.expression, uses);
            }

            /*
             * struct oxc_ast::ast::StaticMemberExpression<'a>
             * - object: oxc_ast::ast::Expression<'a>
             */
            OxcMemberExpression::StaticMemberExpression(expr) => {
                self.expression(context, &expr.object, uses);
            }

            /*
             * struct oxc_ast::ast::PrivateFieldExpression<'a>
             * - object: oxc_ast::ast::Expression<'a>
             */
            OxcMemberExpression::PrivateFieldExpression(expr) => {
                self.expression(context, &expr.object, uses);
            }
        }
    }

    fn statement(
        &mut self,
        context: usize,
        stmt: &OxcStatement<'a>,
        uses: &mut Vec<SymbolUse>,
    ) {
        match stmt {
            // these need not be visited
            OxcStatement::BreakStatement(_) => return,
            OxcStatement::ContinueStatement(_) => return,
            OxcStatement::DebuggerStatement(_) => return,
            OxcStatement::EmptyStatement(_) => return,
            OxcStatement::ClassDeclaration(_) => return,

            // statements
            OxcStatement::BlockStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::BlockStatement<'a>
                 * - body: oxc_allocator::Vec<'a, oxc_ast::ast::Statement<'a>>
                 */

                let new_context = self.create_context(context);
                self.statements(new_context, &stmt.body, uses);
            }

            OxcStatement::DoWhileStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::DoWhileStatement<'a>
                 * - body: oxc_ast::ast::Statement<'a>
                 */

                self.expression(context, &stmt.test, uses);
                self.statement(context, &stmt.body, uses);
            }

            OxcStatement::ExpressionStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::ExpressionStatement<'a>
                 * - expression: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &stmt.expression, uses);
            }

            OxcStatement::ForInStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::ForInStatement<'a>
                 * - left: oxc_ast::ast::ForStatementLeft<'a>
                 * - right: oxc_ast::ast::Expression<'a>
                 * - body: oxc_ast::ast::Statement<'a>
                 */

                let new_context = self.create_context(context);

                match &stmt.left {
                    /*
                     * enum oxc_ast::ast::ForStatementLeft<'a>
                     * : oxc_ast::ast::VariableDeclaration(Box<'a, oxc_ast::ast:VariableDeclaration<'a>)
                     * : oxc_ast::ast::AssignmentTarget<'a> variants
                     */
                    OxcForStatementLeft::VariableDeclaration(decl) => {
                        self.var_decl(new_context, decl, uses);
                    }

                    _ => {
                        if let Some(stmt) = stmt.left.as_assignment_target() {
                            self.assignment_target(context, stmt, uses);
                        } else {
                            unreachable!(
                                "unhandled oxc_ast::ast::ForStatementLeft<'a> | {:?}",
                                stmt.left
                            )
                        }
                    }
                }

                self.expression(context, &stmt.right, uses);
                self.statement(new_context, &stmt.body, uses);
            }

            OxcStatement::ForOfStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::ForOfStatement<'a>
                 * - left: oxc_ast::ast::ForStatementLeft<'a>
                 * - right: oxc_ast::ast::Expression<'a>
                 * - body: oxc_ast::ast::Statement<'a>
                 */

                let new_context = self.create_context(context);

                match &stmt.left {
                    /*
                     * enum oxc_ast::ast::ForStatementLeft<'a>
                     * : oxc_ast::ast::VariableDeclaration(Box<'a, oxc_ast::ast:VariableDeclaration<'a>)
                     * : oxc_ast::ast::AssignmentTarget<'a> variants
                     */
                    OxcForStatementLeft::VariableDeclaration(decl) => {
                        self.var_decl(new_context, decl, uses);
                    }

                    _ => {
                        if let Some(stmt) = stmt.left.as_assignment_target() {
                            self.assignment_target(context, stmt, uses);
                        } else {
                            unreachable!(
                                "unhandled oxc_ast::ast::ForStatementLeft<'a> | {:?}",
                                stmt.left
                            )
                        }
                    }
                }

                self.expression(context, &stmt.right, uses);
                self.statement(new_context, &stmt.body, uses);
            }

            OxcStatement::ForStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::ForStatement<'a>
                 * - init: Option<oxc_ast::ast::ForStatementInit<'a>>
                 * - test: Option<oxc_ast::ast::Expression<'a>>
                 * - update: Option<oxc_ast::ast::Expression<'a>>
                 * - body: oxc_ast::ast::Statement<'a>
                 */

                let new_context = self.create_context(context);

                if let Some(init) = &stmt.init {
                    match init {
                        /*
                         * enum oxc_ast::ast::ForStatementInit<'a>
                         * : oxc_ast::ast::VariableDeclaration(Box<'a, oxc_ast::ast:VariableDeclaration<'a>)
                         * : oxc_ast::ast::Expression<'a> variants
                         */
                        OxcForStatementInit::VariableDeclaration(decl) => {
                            self.var_decl(new_context, decl, uses);
                        }

                        _ => {
                            if let Some(expr) = init.as_expression() {
                                self.expression(context, expr, uses);
                            } else {
                                unreachable!(
                                    "unhandled oxc_ast::ast::ForStatementInit<'a> | {init:?}"
                                )
                            }
                        }
                    }
                }
            }

            OxcStatement::IfStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::IfStatement<'a>
                 * - test: oxc_ast::ast::Expression<'a>
                 * - consequent: oxc_ast::ast::Statement<'a>
                 * - alternate: Option<oxc_ast::ast::Statement<'a>>
                 */
                self.expression(context, &stmt.test, uses);
                self.statement(context, &stmt.consequent, uses);

                if let Some(alternate) = &stmt.alternate {
                    self.statement(context, alternate, uses);
                }
            }

            OxcStatement::LabeledStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::LabeledStatement<'a>
                 * - body: Option<oxc_ast::ast::Statement<'a>>
                 */
                self.statement(context, &stmt.body, uses);
            }

            OxcStatement::ReturnStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::ReturnStatement<'a>
                 * - argument: Option<oxc_ast::ast::Expression<'a>>
                 */

                if let Some(arg) = &stmt.argument {
                    self.expression(context, arg, uses);
                }
            }

            OxcStatement::SwitchStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::SwitchStatement<'a>
                 * - discriminant: oxc_ast::ast::Expression<'a>
                 * - cases: oxc_allocator::Vec<'a, oxc_ast::ast::SwitchCase<'a>>
                 */

                let new_context = self.create_context(context);

                self.expression(context, &stmt.discriminant, uses);

                for case in &stmt.cases {
                    /*
                     * struct oxc_ast::ast::SwitchCase<'a>
                     * - test: Option<oxc_ast::ast::Expression<'a>> - consequent: oxc_allocator::Vec<'a, oxc_ast::ast::Statement<'a>>
                     */

                    if let Some(test) = &case.test {
                        self.expression(context, test, uses);
                    }

                    self.statements(new_context, &case.consequent, uses);
                }
            }

            OxcStatement::ThrowStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::ThrowStatement<'a>
                 * - argument: oxc_ast::ast::Expression<'a>
                 */

                self.expression(context, &stmt.argument, uses);
            }

            OxcStatement::TryStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::TryStatement<'a>
                 * - block: Box<'a, oxc_ast::ast::BlockStatement<'a>>
                 * - handler: Option<Box<'a, CatchClause<'a>>>
                 * - finalizer: Option<Box<'a, BlockStatement<'a>>>
                 */

                let try_context = self.create_context(context);
                self.statements(try_context, &stmt.block.body, uses);

                if let Some(catch) = &stmt.handler {
                    /*
                     * struct oxc_ast::ast::CatchClause<'a>
                     * - param: Option<oxc_ast::ast::CatchParameter<'a>>
                     * - body: Box<'a, BlockStatement<'a>>
                     */

                    let catch_context = if let Some(param) = &catch.param {
                        // this creates a `context -> [clause context] -> block statement` kind of structure
                        // on the contex tree, but it gets the job done without over-complicating the
                        // signatures or structures
                        let new_context = self.create_context(context);
                        self.binding(
                            new_context,
                            Symbol::Variable {
                                span: param.span,
                                type_annotation: None,
                            },
                            &param.pattern,
                            uses,
                        );

                        new_context
                    } else {
                        context
                    };

                    self.statements(catch_context, &catch.body.body, uses);
                }

                if let Some(fin) = &stmt.finalizer {
                    let fin_context = self.create_context(context);
                    self.statements(fin_context, &fin.body, uses);
                }
            }

            OxcStatement::WhileStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::WhileStatement<'a>
                 * - test: Option<oxc_ast::ast::Expression<'a>>
                 * - body: Option<oxc_ast::ast::Statement<'a>>
                 */
                self.expression(context, &stmt.test, uses);
                self.statement(context, &stmt.body, uses);
            }

            OxcStatement::WithStatement(stmt) => {
                /*
                 * struct oxc_ast::ast::WithStatement<'a>
                 * - object: oxc_ast::ast::Expression<'a>
                 * - body: Option<oxc_ast::ast::Statement<'a>>
                 */

                self.expression(context, &stmt.object, uses);
                self.statement(context, &stmt.body, uses);
            }

            // declarations
            OxcStatement::VariableDeclaration(stmt) => {
                self.var_decl(context, &stmt, uses);
            }

            OxcStatement::FunctionDeclaration(stmt) => {
                /*
                 * - params: Box<'a, oxc_ast::ast::FormalParameters<'a>>,
                 * struct oxc_ast::ast::FunctionDeclaration<'a>
                 * - body: oxc_ast::ast::FunctionBody<'a>,
                 * - id: Option<oxc_ast::ast::BindingIdentifier<'a>> // required
                 * - type_parameters: Option<Box<'a, oxc_ast::ast::TSTypeParameterDeclaration>>
                 * - return_type: Option<Box<'a, oxc_ast::ast::TSTypeAnnotation>>
                 */

                let new_context = self.create_context(context);

                self.formal_parameters(new_context, &stmt.params, uses);

                let body = if let Some(body) = &stmt.body {
                    Some(self.function_body(new_context, body, uses))
                } else {
                    None
                };

                let return_type = if let Some(an) = &stmt.return_type {
                    Some(self.type_annotation(&an.type_annotation))
                } else {
                    None
                };

                let generics = if let Some(generic) = &stmt.type_parameters {
                    Some(generic.span)
                } else {
                    None
                };

                if let Some(id) = &stmt.id {
                    self.insert_in_context(
                        context,
                        id.name.to_string(),
                        Symbol::Function {
                            name: id.name.to_string(),
                            context: context,
                            body,
                            return_type,
                            generics,
                        },
                    );
                }
            }

            OxcStatement::TSTypeAliasDeclaration(stmt) => {
                /*
                 * struct oxc_ast::ast::TSTypeAliasDeclaration
                 * - id: oxc_ast::ast::BindingIdentifier<'a>
                 * - type_parameters: Option<Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>>
                 * - type_annotation: oxc_ast::ast::TSTypeAnnotation<'a>
                 */

                let type_annotation =
                    self.type_annotation(&stmt.type_annotation);

                let generics = if let Some(generic) = &stmt.type_parameters {
                    Some(generic.span)
                } else {
                    None
                };

                self.insert_in_context(
                    context,
                    stmt.id.name.to_string(),
                    Symbol::Type(NamedType::Alias {
                        type_annotation,
                        generics,
                    }),
                );
            }

            OxcStatement::TSInterfaceDeclaration(stmt) => {
                /*
                 * struct oxc_ast::ast::TSInterfaceDeclaration
                 * - id: oxc_ast::ast::BindingIdentifier<'a>
                 * - type_parameters: Option<Box<'a, oxc_ast::ast::TSTypeParameterDeclaration<'a>>>
                 * - body: Box<'a, oxc_ast::ast::TSInterfaceBody<'a>>
                 */

                let generics = if let Some(generic) = &stmt.type_parameters {
                    Some(generic.span)
                } else {
                    None
                };

                let mut fields = Vec::new();

                /*
                 * struct oxc_ast::ast::TSInterfaceBody
                 * - body: Box<'a, oxc_ast::ast::TSSignature<'a>>
                 */
                for field in &stmt.body.body {
                    match field {
                        OxcTSSignature::TSIndexSignature(sig) => fields.push((
                            None,
                            TypeAnnotation::Disallowed((
                                sig.span,
                                "index signature",
                            )),
                        )),

                        OxcTSSignature::TSPropertySignature(sig) => {
                            match &sig.key {
                                OxcPropertyKey::StaticIdentifier(ident) => {
                                    if let Some(t) = &sig.type_annotation {
                                        fields.push((
                                            Some(ident.name.to_string()),
                                            self.type_annotation(
                                                &t.type_annotation,
                                            ),
                                        ))
                                    } else {
                                        fields.push((
                                            None,
                                            TypeAnnotation::Disallowed((
                                                ident.span,
                                                "type annotation required",
                                            )),
                                        ))
                                    }
                                }

                                _ => fields.push((
                                    None,
                                    TypeAnnotation::Disallowed((
                                        sig.span,
                                        "only static identifiers are allowed",
                                    )),
                                )),
                            }
                        }

                        OxcTSSignature::TSCallSignatureDeclaration(sig) => {
                            fields.push((
                                None,
                                TypeAnnotation::Disallowed((
                                    sig.span,
                                    "index signature",
                                )),
                            ))
                        }

                        OxcTSSignature::TSConstructSignatureDeclaration(
                            sig,
                        ) => fields.push((
                            None,
                            TypeAnnotation::Disallowed((
                                sig.span,
                                "index signature",
                            )),
                        )),

                        OxcTSSignature::TSMethodSignature(sig) => {
                            fields.push((
                                None,
                                TypeAnnotation::Disallowed((
                                    sig.span,
                                    "index signature",
                                )),
                            ))
                        }
                    }
                }

                self.insert_in_context(
                    context,
                    stmt.id.name.to_string(),
                    Symbol::Type(NamedType::Interface { fields, generics }),
                );
            }

            OxcStatement::TSEnumDeclaration(stmt) => {
                /*
                 * struct oxc_ast::ast::TSEnumDeclaration
                 * - id: oxc_ast::ast::BindingIdentifier<'a>
                 * - body: Box<'a, TSEnumBody<'a>>
                 */

                let mut variants = Vec::new();

                /*
                 * struct oxc_ast::ast::TSEnumBody
                 * - members: Box<'a, oxc_ast::ast::TSEnumMember<'a>>
                 */
                for mem in &stmt.body.members {
                    let id = mem.id.static_name().to_string();

                    let init = if let Some(init) = &mem.initializer {
                        match init {
                            OxcExpression::NumericLiteral(lit) => {
                                Some(EnumValue::Number(lit.value))
                            }

                            OxcExpression::StringLiteral(lit) => {
                                Some(EnumValue::String(lit.value.to_string()))
                            }

                            _ => Some(EnumValue::Invalid),
                        }
                    } else {
                        None
                    };

                    variants.push((id, init));
                }

                self.insert_in_context(
                    context,
                    stmt.id.name.to_string(),
                    Symbol::Type(NamedType::Enum(variants)),
                );
            }

            OxcStatement::TSExternalModuleDeclaration(_) => return,

            OxcStatement::TSNamespaceDeclaration(_) => return,

            OxcStatement::TSGlobalDeclaration(_) => return,

            OxcStatement::TSImportEqualsDeclaration(_) => return,

            // imports
            OxcStatement::ImportDeclaration(_) => return,

            OxcStatement::ExportAllDeclaration(_) => return,

            OxcStatement::ExportDefaultDeclaration(_) => return,

            OxcStatement::ExportDeclaration(_) => return,

            OxcStatement::ExportNamedDeclaration(_) => return,

            OxcStatement::ExportFromDeclaration(_) => return,

            OxcStatement::TSExportAssignment(_) => return,

            OxcStatement::TSNamespaceExportDeclaration(_) => return,
        }
    }

    fn var_decl(
        &mut self,
        context: usize,
        decl: &OxcVariableDeclaration<'a>,
        uses: &mut Vec<SymbolUse>,
    ) {
        for decl in &decl.declarations {
            // if the current variable is NOT in the global context,
            // it will never cross the semantic boundary, and hence needn't be analyzed
            let r#type: Option<TypeAnnotation> = if context != 0 {
                None
            } else {
                if let Some(annotation) = &decl.type_annotation {
                    Some(self.type_annotation(&annotation.type_annotation))
                } else {
                    None
                }
            };

            self.binding(
                context,
                Symbol::Variable {
                    span: decl.span,
                    type_annotation: r#type,
                },
                &decl.id,
                uses,
            );
        }
    }

    fn type_annotation(
        &mut self,
        annotation: &OxcTSType<'a>,
    ) -> TypeAnnotation {
        match annotation {
            OxcTSType::TSBooleanKeyword(_type) => {
                TypeAnnotation::Boolean(_type.span)
            }

            OxcTSType::TSNumberKeyword(_type) => {
                TypeAnnotation::Number(_type.span)
            }

            OxcTSType::TSStringKeyword(_type) => {
                TypeAnnotation::String(_type.span)
            }

            OxcTSType::TSArrayType(_type) => {
                /*
                 * struct oxc_ast::ast::TSArrayType<'a>
                 * element_type: oxc_ast::ast::TSType<'a>
                 */

                TypeAnnotation::Array((
                    _type.span,
                    Box::new(self.type_annotation(&_type.element_type)),
                ))
            }

            OxcTSType::TSLiteralType(_type) => match &_type.literal {
                OxcTSLiteral::BooleanLiteral(lit) => TypeAnnotation::Literal((
                    _type.span,
                    Literal::Boolean(lit.value),
                )),

                OxcTSLiteral::NumericLiteral(lit) => TypeAnnotation::Literal((
                    _type.span,
                    Literal::Number(lit.value),
                )),

                OxcTSLiteral::StringLiteral(lit) => TypeAnnotation::Literal((
                    _type.span,
                    Literal::String(lit.value.to_string()),
                )),

                OxcTSLiteral::BigIntLiteral(_type) => {
                    TypeAnnotation::Disallowed((_type.span, "bigint literal"))
                }

                OxcTSLiteral::UnaryExpression(_type) => {
                    TypeAnnotation::Disallowed((
                        _type.span,
                        "unary expression literal",
                    ))
                }

                OxcTSLiteral::TemplateLiteral(_type) => {
                    TypeAnnotation::Disallowed((_type.span, "template literal"))
                }
            },

            OxcTSType::TSNamedTupleMember(_type) => {
                /*
                 * struct oxc_ast::ast::TSNamedTupleMember<'a>
                 * - label: oxc_ast::ast::IdentifierName<'a>
                 * - element_type: oxc_ast::ast::TSTupleElement<'a>
                 */

                match &_type.element_type {
                    /*
                     * enum oxc_ast::ast::TSTupleElement<'a>
                     * : TSOptionalType(_) => Disallowed
                     * : TSRestType(_) => Disallowed
                     * : oxc_ast::ast::TSType<'a> variants
                     */
                    OxcTSTupleElement::TSOptionalType(_type) => {
                        TypeAnnotation::Disallowed((_type.span, "optional"))
                    }

                    OxcTSTupleElement::TSRestType(_type) => {
                        TypeAnnotation::Disallowed((_type.span, "rest"))
                    }

                    _ => {
                        if let Some(t) = _type.element_type.as_ts_type() {
                            TypeAnnotation::NamedTupleMember((
                                _type.span,
                                _type.label.to_string(),
                                Box::new(self.type_annotation(t)),
                            ))
                        } else {
                            unreachable!(
                                "unhandled oxc_ast::ast::TSType {:?}",
                                _type.element_type
                            )
                        }
                    }
                }
            }

            OxcTSType::TSTupleType(_type) => {
                /*
                 * struct oxc_ast::ast::TSTupleType<'a>
                 * - element_types: oxc_allocator::Vec<'a, oxc_ast::ast::TSTupleElement<'a>>
                 */

                let mut types = Vec::new();

                for t in &_type.element_types {
                    match &t {
                        /*
                         * enum oxc_ast::ast::TSTupleElement<'a>
                         * : TSOptionalType(_) => Disallowed
                         * : TSRestType(_) => Disallowed
                         * : oxc_ast::ast::TSType<'a> variants
                         */
                        OxcTSTupleElement::TSOptionalType(_type) => {
                            types.push(TypeAnnotation::Disallowed((
                                _type.span, "optional",
                            )))
                        }

                        OxcTSTupleElement::TSRestType(_type) => types.push(
                            TypeAnnotation::Disallowed((_type.span, "rest")),
                        ),

                        _ => {
                            if let Some(t) = t.as_ts_type() {
                                types.push(self.type_annotation(t))
                            } else {
                                unreachable!(
                                    "unhandled oxc_ast::ast::TSType {t:?}",
                                )
                            }
                        }
                    }
                }

                TypeAnnotation::Tuple((_type.span, types))
            }

            OxcTSType::TSTypeReference(_type) => {
                /*
                 * struct oxc_ast::ast::TSTypeReference<'a>
                 *
                 * - type_name: oxc_ast::ast::TSTypeName<'a>
                 */

                match &_type.type_name {
                    OxcTSTypeName::IdentifierReference(ident) => {
                        if _type.type_arguments.is_some() {
                            TypeAnnotation::Disallowed((_type.span, "generics"))
                        } else {
                            TypeAnnotation::TypeReference((
                                _type.span,
                                ident.name.to_string(),
                            ))
                        }
                    }

                    OxcTSTypeName::QualifiedName(_) => {
                        TypeAnnotation::Disallowed((
                            _type.span,
                            "qualified name",
                        ))
                    }

                    OxcTSTypeName::ThisExpression(_) => {
                        TypeAnnotation::Disallowed((
                            _type.span,
                            "this expression",
                        ))
                    }
                }
            }

            OxcTSType::TSUnionType(_type) => {
                /*
                 * struct oxc_ast::ast::TSUnionType<'a>
                 * - types: oxc_allocator::Vec<'a, oxc_ast::ast::TSType<'a>>
                 */

                let mut types = Vec::new();

                for t in &_type.types {
                    types.push(self.type_annotation(t))
                }

                TypeAnnotation::Union((_type.span, types))
            }

            OxcTSType::TSParenthesizedType(_type) => {
                /*
                 * struct oxc_ast::ast::TSParanthesizedType<'a>
                 * - type_annotation: oxc_allocator::Vec<'a, oxc_ast::ast::TSType<'a>>
                 */

                self.type_annotation(&_type.type_annotation)
            }

            OxcTSType::TSTypeLiteral(_type) => {
                TypeAnnotation::Disallowed((_type.span, "type literal"))
            }

            OxcTSType::TSAnyKeyword(_type) => {
                TypeAnnotation::Disallowed((_type.span, "any"))
            }

            OxcTSType::TSBigIntKeyword(_type) => {
                TypeAnnotation::Disallowed((_type.span, "bigint"))
            }

            OxcTSType::TSIntrinsicKeyword(_type) => {
                TypeAnnotation::Disallowed((_type.span, "intrinsic"))
            }

            OxcTSType::TSNeverKeyword(_type) => {
                TypeAnnotation::Disallowed((_type.span, "never"))
            }

            OxcTSType::TSNullKeyword(_type) => {
                TypeAnnotation::Disallowed((_type.span, "null"))
            }

            OxcTSType::TSObjectKeyword(_type) => {
                TypeAnnotation::Disallowed((_type.span, "object"))
            }

            OxcTSType::TSSymbolKeyword(_type) => {
                TypeAnnotation::Disallowed((_type.span, "symbol"))
            }

            OxcTSType::TSUndefinedKeyword(_type) => {
                TypeAnnotation::Disallowed((_type.span, "undefined"))
            }

            OxcTSType::TSUnknownKeyword(_type) => {
                TypeAnnotation::Disallowed((_type.span, "unknown"))
            }

            OxcTSType::TSVoidKeyword(_type) => {
                TypeAnnotation::Disallowed((_type.span, "void"))
            }

            OxcTSType::TSConditionalType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "conditional"))
            }

            OxcTSType::TSConstructorType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "constructor"))
            }

            OxcTSType::TSFunctionType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "function"))
            }

            OxcTSType::TSImportType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "import"))
            }

            OxcTSType::TSIndexedAccessType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "indexed access"))
            }

            OxcTSType::TSInferType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "infer"))
            }

            OxcTSType::TSIntersectionType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "intersection"))
            }

            OxcTSType::TSMappedType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "mapped"))
            }

            OxcTSType::TSTemplateLiteralType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "template literal"))
            }

            OxcTSType::TSThisType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "this"))
            }

            OxcTSType::TSTypeOperatorType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "type operator"))
            }

            OxcTSType::TSTypePredicate(_type) => {
                TypeAnnotation::Disallowed((_type.span, "type predicate"))
            }

            OxcTSType::TSTypeQuery(_type) => {
                TypeAnnotation::Disallowed((_type.span, "type query"))
            }

            OxcTSType::JSDocNullableType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "JS Doc nullable"))
            }

            OxcTSType::JSDocNonNullableType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "JS Doc non-nullable"))
            }

            OxcTSType::JSDocUnknownType(_type) => {
                TypeAnnotation::Disallowed((_type.span, "JS Doc unknown"))
            }
        }
    }

    fn statements(
        &mut self,
        context: usize,
        stmts: &OxcVec<'a, OxcStatement<'a>>,
        uses: &mut Vec<SymbolUse>,
    ) {
        for stmt in stmts {
            self.statement(context, stmt, uses)
        }
    }

    pub fn view_span(&self, source: &'a str, span: OxcSpan) -> &str {
        &source[span.start as usize..span.end as usize]
    }

    pub fn is_global_symbol(&self, symbol: &str, context: usize) -> bool {
        let ctxt = self.context_tree.get(context).unwrap();

        if ctxt.symbols.contains_key(symbol) {
            context == 0
        } else {
            if context == 0 {
                false
            } else {
                self.is_global_symbol(symbol, ctxt.parent)
            }
        }
    }
    pub fn analyze_script(
        &mut self,
        script: &'a str,
        oxc_allocator: &'a OxcAllocator,
    ) {
        let body = Self::parse(script, oxc_allocator);

        let global_context = self.create_context(0);

        let mut uses = Vec::new();

        self.statements(global_context, &body, &mut uses);

        // collect global uses
        let global_uses = uses
            .iter()
            .filter(|u| {
                self.is_global_symbol(
                    self.view_span(&script, u.span),
                    u.context,
                )
            })
            .cloned()
            .collect();

        self.uses = global_uses;
    }

    pub fn analyze(
        &mut self,
        oxc_allocator: &'a OxcAllocator,
        script: &'a str,
        expressions: &'a mut Vec<Expression>,
    ) {
        self.analyze_script(script, oxc_allocator);
        self.analyze_expressions(expressions, oxc_allocator);
    }

    pub fn analyze_expressions(
        &mut self,
        expressions: &'a mut Vec<Expression>,
        oxc_allocator: &'a OxcAllocator,
    ) {
        for expr in expressions {
            let body = Self::parse(&expr.source, oxc_allocator);

            let mut uses = Vec::new();

            self.statements(0, &body, &mut uses);

            let mut context_uses = Vec::new();
            let mut global_uses = Vec::new();

            // collect global and context uses
            for _use in uses {
                let use_name =
                    &self.view_span(&expr.source, _use.span).to_string();

                if expr.context_vars.contains(use_name) {
                    context_uses.push(_use);
                } else if self.is_global_symbol(use_name, _use.context) {
                    global_uses.push(_use);
                }
            }

            expr.global_uses = global_uses;
            expr.context_uses = context_uses;
        }
    }
}
