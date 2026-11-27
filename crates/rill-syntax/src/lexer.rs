//! Byte-oriented lexer for the language's ASCII syntax.
//!
//! Spans are half-open UTF-8 byte ranges. Identifiers are ASCII-only;
//! strings may contain Unicode, but have no escape processing and cannot
//! contain raw newlines. A minus immediately followed by a digit is part
//! of a numeric literal. A decimal point must be followed by a digit.
//! Spaces, tabs and `//` comments are ignored; LF, CRLF and CR produce
//! newline tokens. Tokenization stops at the first error. On success,
//! exactly one zero-width EOF token is appended.

use crate::span::Span;
use crate::token::{Delimiter, Keyword, LiteralKind, Operator, Token, TokenKind};
use std::fmt;

/// The reason tokenization failed.
#[derive(Debug, PartialEq, Eq)]
pub enum LexErrorKind {
    /// A character that cannot start a token.
    UnexpectedCharacter(char),
    /// A string reached a line ending or EOF before its closing quote.
    UnterminatedString,
    /// A decimal point was not immediately followed by a digit.
    ExpectedDigitAfterDecimalPoint,
}

impl fmt::Display for LexErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter(ch) => write!(f, "unexpected character '{ch}'"),
            Self::UnterminatedString => f.write_str("unterminated string"),
            Self::ExpectedDigitAfterDecimalPoint => f.write_str("expected a digit after '.'"),
        }
    }
}

/// A tokenization failure and its source location.
#[derive(Debug, PartialEq, Eq)]
pub struct LexError {
    /// The reason tokenization stopped.
    pub kind: LexErrorKind,
    /// Half-open byte range in the original source.
    /// Unexpected characters cover one full UTF-8 character. Unterminated strings
    /// span from the opening quote to the line ending or EOF, excluding the newline.
    /// Malformed decimals span the numeric prefix through the decimal point.
    pub span: Span,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.kind, f)
    }
}

impl std::error::Error for LexError {}

/// Tokenize source text, retaining byte spans into the original source.
///
/// # Errors
///
/// Returns the first [`LexError`] without partial tokens. Numeric ranges are
/// checked by the parser.
pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    Lexer::new(source).tokenize()
}

struct Lexer<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    fn tokenize(mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();

        while let Some(byte) = self.peek() {
            // The cursor is at a UTF-8 boundary at each dispatch. Scanning
            // strings and comments may pass through interior UTF-8 bytes.
            let start = self.pos;
            let kind = match byte {
                b' ' | b'\t' => {
                    self.advance();
                    continue;
                }
                b'\n' => {
                    self.advance();
                    TokenKind::Newline
                }
                b'\r' => {
                    self.advance();
                    self.consume_if(b'\n');
                    TokenKind::Newline
                }
                b'(' => {
                    self.advance();
                    TokenKind::Delimiter(Delimiter::LeftParenthesis)
                }
                b')' => {
                    self.advance();
                    TokenKind::Delimiter(Delimiter::RightParenthesis)
                }
                b'/' if self.peek_next() == Some(b'/') => {
                    self.skip_line_comment();
                    continue;
                }
                b'=' => self.scan_operator(Operator::Assign, Operator::Equal),
                b'!' if self.peek_next() == Some(b'=') => {
                    self.advance();
                    self.advance();
                    TokenKind::Operator(Operator::NotEqual)
                }
                b'>' => self.scan_operator(Operator::GreaterThan, Operator::GreaterThanOrEqual),
                b'<' => self.scan_operator(Operator::LessThan, Operator::LessThanOrEqual),
                b'"' => self.scan_string(start)?,
                b'0'..=b'9' => self.scan_number(start)?,
                b'-' if self.peek_next().is_some_and(|b| b.is_ascii_digit()) => {
                    self.scan_number(start)?
                }
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.scan_identifier(start),
                _ => return Err(self.unexpected_character()),
            };

            tokens.push(Token {
                kind,
                span: Span::new(start, self.pos),
            });
        }

        tokens.push(Token {
            kind: TokenKind::Eof,
            span: Span::new(self.pos, self.pos),
        });
        Ok(tokens)
    }

    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos + 1).copied()
    }

    /// Advance one byte; callers must have confirmed a byte is present.
    fn advance(&mut self) {
        debug_assert!(self.peek().is_some());
        self.pos += 1;
    }

    fn consume_if(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn skip_line_comment(&mut self) {
        // The dispatch has already confirmed the two-byte comment prefix.
        self.advance();
        self.advance();
        while let Some(byte) = self.peek() {
            if matches!(byte, b'\n' | b'\r') {
                break;
            }
            self.advance();
        }
    }

    fn scan_operator(&mut self, single: Operator, with_equal: Operator) -> TokenKind {
        self.advance();
        let operator = if self.consume_if(b'=') {
            with_equal
        } else {
            single
        };
        TokenKind::Operator(operator)
    }

    fn scan_string(&mut self, start: usize) -> Result<TokenKind, LexError> {
        self.advance(); // Opening quote.
        loop {
            match self.peek() {
                Some(b'"') => {
                    self.advance();
                    return Ok(TokenKind::Literal(LiteralKind::String));
                }
                Some(b'\n' | b'\r') | None => {
                    return Err(self.error(start, LexErrorKind::UnterminatedString));
                }
                Some(_) => self.advance(),
            }
        }
    }

    fn scan_number(&mut self, start: usize) -> Result<TokenKind, LexError> {
        self.consume_if(b'-');
        self.scan_digits();

        let kind = if self.consume_if(b'.') {
            if !self.peek().is_some_and(|b| b.is_ascii_digit()) {
                return Err(self.error(start, LexErrorKind::ExpectedDigitAfterDecimalPoint));
            }
            self.scan_digits();
            LiteralKind::Float
        } else {
            LiteralKind::Integer
        };
        Ok(TokenKind::Literal(kind))
    }

    fn scan_digits(&mut self) {
        while self.peek().is_some_and(|b| b.is_ascii_digit()) {
            self.advance();
        }
    }

    fn scan_identifier(&mut self, start: usize) -> TokenKind {
        while self
            .peek()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            self.advance();
        }

        // Classify only after consuming the complete identifier.
        match &self.source[start..self.pos] {
            "let" => TokenKind::Keyword(Keyword::Let),
            "int" => TokenKind::Keyword(Keyword::Int),
            "bool" => TokenKind::Keyword(Keyword::Bool),
            "float" => TokenKind::Keyword(Keyword::Float),
            "print" => TokenKind::Keyword(Keyword::Print),
            "true" | "false" => TokenKind::Literal(LiteralKind::Boolean),
            _ => TokenKind::Identifier,
        }
    }

    fn error(&self, start: usize, kind: LexErrorKind) -> LexError {
        LexError {
            kind,
            span: Span::new(start, self.pos),
        }
    }

    fn unexpected_character(&self) -> LexError {
        // Only called at dispatch, with a byte present at a UTF-8 boundary.
        let ch = self.source[self.pos..]
            .chars()
            .next()
            .expect("lexer cursor must point to a character");
        LexError {
            kind: LexErrorKind::UnexpectedCharacter(ch),
            span: Span::new(self.pos, self.pos + ch.len_utf8()),
        }
    }
}
