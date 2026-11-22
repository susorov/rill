use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Keyword(Keyword),
    Identifier,
    Literal(Literal),
    Operator(Operator),

    // Separators
    Newline,
    LeftParenthesis,
    RightParenthesis,

    // Special
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Let,
    Int,
    Bool,
    Float,
    Print,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Literal {
    Integer,
    Float,
    Boolean,
    String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Assign,
    Equal,
    NotEqual,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
