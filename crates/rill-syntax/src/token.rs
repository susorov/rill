use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    Let,
    Int,
    Bool,
    Print,

    // Values
    Identifier,
    Integer,
    Float,
    Boolean,
    String,

    // Operators
    Equal,

    // Separators
    Newline,
    LeftParenthesis,
    RightParenthesis,

    // Special
    Eof,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
