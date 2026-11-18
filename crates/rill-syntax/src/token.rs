use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Let,
    Int,
    Bool,

    // Values
    Identifier,
    Integer,
    Boolean,

    // Operators
    Equal,

    // Separators
    Newline,

    // Special
    Eof,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
