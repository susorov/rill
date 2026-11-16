use crate::span::Span;

#[derive(Debug)]
pub enum TokenKind {
    // Keywords
    Let,

    // Values
    Identifier,
    Integer,

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
