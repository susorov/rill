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
    Keyword(Keyword),
    Identifier,
    Literal(LiteralKind),
    Operator(Operator),
    Delimiter(Delimiter),
    /// A line ending: LF, CRLF, or CR. CRLF spans two bytes.
    Newline,
    /// End of input, with an empty span at `source.len()`.
    Eof,
}

/// Keywords supported by the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Let,
    Int,
    Bool,
    Float,
    Print,
}

/// Literal types. The parser reads the value from the source span
/// and checks numeric ranges.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    Integer,
    Float,
    Boolean,
    String,
}

/// Operators supported by the language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Assign,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
}

/// Parentheses used as delimiters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    LeftParenthesis,
    RightParenthesis,
}

/// A token's type and location in the original source.
///
/// Tokens do not own source text. Use their spans with the same source
/// passed to the lexer.
#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    /// Half-open byte range `[start, end)` in the original UTF-8 source.
    ///
    /// Lexer-produced spans always point to valid character boundaries,
    /// so `&source[span.start..span.end]` returns the complete token text.
    /// String spans include quotes; negative numbers include the minus sign.
    /// EOF has an empty span at the end of the source.
    pub span: Span,
}
