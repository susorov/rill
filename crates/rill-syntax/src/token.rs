use crate::span::Span;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Keywords
    LetKeyword,
    IntKeyword,
    BoolKeyword,
    FloatKeyword,
    PrintKeyword,

    // Values
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
