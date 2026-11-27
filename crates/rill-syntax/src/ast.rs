//! Syntax tree built by the parser and used by the compiler.
//!
//! Nodes own names and string values. Spans refer to the original source.

use crate::span::Span;

/// A program's syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    /// Top-level statements in source order.
    pub statements: Vec<Statement>,
}

/// A variable declaration, print call, or conditional statement.
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// A variable declaration with an initializer.
    Let {
        /// Declared variable name.
        name: String,
        /// Initializer expression.
        value: Expression,
        /// Source range from `let` through the initializer.
        span: Span,
    },
    /// A print call with an optional value.
    Print {
        /// None represents print() without a value.
        value: Option<Expression>,
        /// Source range from `print` through the closing parenthesis.
        span: Span,
    },
    /// A conditional statement with an optional else branch.
    If {
        /// Condition expression; boolean type checking belongs to the compiler.
        condition: Expression,
        /// Statements executed when the condition is true.
        then_branch: Vec<Statement>,
        /// Statements executed when the condition is false; `None` means no `else`.
        else_branch: Option<Vec<Statement>>,
        /// Source range from `if` through the final branch's closing brace.
        span: Span,
    },
}

/// An expression and its source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Expression {
    /// The expression's contents.
    pub kind: ExpressionKind,
    /// Source range covering the whole expression, including any literal quotes or sign.
    pub span: Span,
}

/// A literal, variable reference, or binary expression.
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionKind {
    /// A signed 64-bit integer literal.
    Integer(i64),
    /// A 64-bit floating-point literal; the parser accepts only finite values.
    Float(f64),
    /// A `true` or `false` literal.
    Boolean(bool),
    /// String contents without surrounding quotes.
    String(String),
    /// A variable name resolved by the compiler.
    Identifier(String),
    /// A comparison between two expressions.
    Binary {
        /// Left operand.
        left: Box<Expression>,
        /// Comparison to perform.
        operator: BinaryOperator,
        /// Right operand.
        right: Box<Expression>,
    },
}

/// Comparison operators used in expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    /// Equality (`==`).
    Equal,
    /// Inequality (`!=`).
    NotEqual,
    /// Strict greater-than comparison (`>`).
    GreaterThan,
    /// Greater-than-or-equal comparison (`>=`).
    GreaterThanOrEqual,
    /// Strict less-than comparison (`<`).
    LessThan,
    /// Less-than-or-equal comparison (`<=`).
    LessThanOrEqual,
}
