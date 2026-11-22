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
    IntegerLiteral,
    FloatLiteral,
    BooleanLiteral,
    StringLiteral,

    Operator(Operator),

    // Separators
    Newline,
    LeftParenthesis,
    RightParenthesis,

    // Special
    Eof,
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
