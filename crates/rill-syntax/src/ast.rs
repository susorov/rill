//! Syntax tree built by the parser and used by the compiler.
//!
//! Nodes own names and string values. Spans refer to the original source.

use crate::span::Span;

/// Statements in source order.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// A variable declaration or print statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Let {
        name: String,
        value: Expression,
        span: Span,
    },
    Print {
        /// None represents print() without a value.
        value: Option<Expression>,
        span: Span,
    },
}

/// An expression and its source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub span: Span,
}

/// A literal, variable reference, or binary expression.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    /// String contents without surrounding quotes.
    String(String),
    /// A variable name resolved by the compiler.
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
}

/// Types used by the compiler to check expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionType {
    Integer,
    Float,
    Boolean,
    String,
}

/// Comparison operators used in expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}
