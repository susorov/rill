//! Token definitions shared by the lexer and parser.
//!
//! On success, [`crate::lexer::tokenize`] returns tokens in source order,
//! ending with exactly one [`TokenKind::Eof`], even for empty input.
//! Spaces, tabs, and line comments are omitted outside strings.
//! Newlines are preserved.

use crate::span::Span;

/// A token's classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// A keyword or reserved type name.
    Keyword(Keyword),
    /// An ASCII variable name.
    Identifier,
    /// A literal whose value is read by the parser.
    Literal(LiteralKind),
    /// An assignment or comparison operator.
    Operator(Operator),
    /// An opening or closing parenthesis or brace.
    Delimiter(Delimiter),
    /// A line ending: LF, CRLF, or CR. CRLF spans two bytes.
    Newline,
    /// End of input, with an empty span at `source.len()`.
    Eof,
}

/// Keywords and reserved type names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    /// Variable declaration keyword `let`.
    Let,
    /// Conditional keyword `if`.
    If,
    /// Alternative branch keyword `else`.
    Else,
    /// Reserved word `int`.
    Int,
    /// Reserved word `bool`.
    Bool,
    /// Reserved word `float`.
    Float,
    /// Output keyword `print`.
    Print,
}

/// Literal types. The parser reads the value from the source span
/// and checks numeric ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    /// Decimal digits with an optional leading minus sign.
    Integer,
    /// A decimal with digits on both sides of the point and an optional minus sign.
    Float,
    /// `true` or `false`.
    Boolean,
    /// Double-quoted text without escape processing.
    String,
}

/// Operators supported by the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// Initializer assignment (`=`).
    Assign,
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

/// Parentheses and braces used as delimiters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    /// Opening parenthesis (`(`).
    LeftParenthesis,
    /// Closing parenthesis (`)`).
    RightParenthesis,
    /// Opening brace (`{`).
    LeftBrace,
    /// Closing brace (`}`).
    RightBrace,
}

/// A token's type and location in the original source.
///
/// Tokens do not own source text. Use their spans with the same source
/// passed to the lexer.
#[derive(Debug)]
pub struct Token {
    /// The token's classification.
    pub kind: TokenKind,
    /// Half-open byte range `[start, end)` in the original UTF-8 source.
    ///
    /// Lexer-produced spans always point to valid character boundaries,
    /// so `&source[span.start..span.end]` returns the complete token text.
    /// String spans include quotes; negative numbers include the minus sign.
    /// EOF has an empty span at the end of the source.
    pub span: Span,
}
