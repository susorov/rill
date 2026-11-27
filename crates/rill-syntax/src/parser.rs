//! Source parsing and syntax diagnostics.
//!
//! Builds owned syntax trees for declarations, print calls, and conditionals.
//! Name resolution and type checking are left to the compiler.

use crate::ast::{BinaryOperator, Expression, ExpressionKind, Program, Statement};
use crate::lexer::{LexError, tokenize};
use crate::span::Span;
use crate::token::{Delimiter, Keyword, LiteralKind, Operator, Token, TokenKind};

/// A lexical or parsing failure and its source location.
#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    /// Human-readable description of the failure.
    pub message: String,
    /// Offending token or literal's byte range; empty at unexpected EOF.
    /// Lexical errors retain the lexer's span.
    pub span: Span,
}

impl From<LexError> for ParseError {
    fn from(error: LexError) -> Self {
        Self {
            message: error.to_string(),
            span: error.span,
        }
    }
}

/// Parse source text into an owned syntax tree with byte spans.
/// Empty input and blank lines are allowed; the final newline is optional.
///
/// # Errors
///
/// Tokenization runs first and preserves lexical error messages and spans.
/// Parsing then stops at the first syntax or numeric-range error, returning no partial tree.
pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = tokenize(source)?;

    Parser {
        source,
        tokens: &tokens,
        pos: 0,
    }
    .parse_program()
}

struct Parser<'a> {
    source: &'a str,
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn parse_program(&mut self) -> Result<Program, ParseError> {
        let statements =
            self.parse_statements(TokenKind::Eof, "expected a newline or end of file")?;
        Ok(Program { statements })
    }

    fn parse_statements(
        &mut self,
        end: TokenKind,
        separator_error: &str,
    ) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();

        loop {
            let next = self.next_non_newline_pos();
            let has_separator = next > self.pos;
            self.pos = next;

            let kind = self.current().kind;
            if kind == end || kind == TokenKind::Eof {
                return Ok(statements);
            }
            if !statements.is_empty() && !has_separator {
                return Err(self.error(separator_error));
            }
            statements.push(self.parse_statement()?);
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current().kind {
            TokenKind::Keyword(Keyword::Let) => self.parse_let_statement(),
            TokenKind::Keyword(Keyword::Print) => self.parse_print_statement(),
            TokenKind::Keyword(Keyword::If) => self.parse_if_statement(),
            TokenKind::Keyword(Keyword::Else) => {
                Err(self.error("unexpected 'else' without a matching 'if'"))
            }
            _ => Err(self.error("expected a statement")),
        }
    }

    fn parse_let_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self
            .expect(TokenKind::Keyword(Keyword::Let), "expected 'let'")?
            .span
            .start;
        let name = self.expect(TokenKind::Identifier, "expected an identifier")?;
        let name = self.source[name.span.start..name.span.end].to_owned();

        self.expect(TokenKind::Operator(Operator::Assign), "expected '='")?;

        let value = self.parse_expression()?;
        let span = Span::new(start, value.span.end);

        Ok(Statement::Let { name, value, span })
    }

    fn parse_print_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self
            .expect(TokenKind::Keyword(Keyword::Print), "expected 'print'")?
            .span
            .start;

        self.expect(
            TokenKind::Delimiter(Delimiter::LeftParenthesis),
            "expected '('",
        )?;

        let value = if self.current().kind == TokenKind::Delimiter(Delimiter::RightParenthesis) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        let end = self
            .expect(
                TokenKind::Delimiter(Delimiter::RightParenthesis),
                "expected ')'",
            )?
            .span
            .end;

        Ok(Statement::Print {
            value,
            span: Span::new(start, end),
        })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self
            .expect(TokenKind::Keyword(Keyword::If), "expected 'if'")?
            .span
            .start;

        let condition = self.parse_expression()?;
        let (then_branch, then_end) = self.parse_block()?;

        let next = self.next_non_newline_pos();
        let (else_branch, end) = if self.tokens[next].kind == TokenKind::Keyword(Keyword::Else) {
            self.pos = next + 1;

            let (statements, end) = self.parse_block()?;
            (Some(statements), end)
        } else {
            (None, then_end)
        };

        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
            span: Span::new(start, end),
        })
    }

    fn parse_block(&mut self) -> Result<(Vec<Statement>, usize), ParseError> {
        self.pos = self.next_non_newline_pos();
        self.expect(TokenKind::Delimiter(Delimiter::LeftBrace), "expected '{'")?;

        let statements = self.parse_statements(
            TokenKind::Delimiter(Delimiter::RightBrace),
            "expected a newline or '}'",
        )?;
        let end = self
            .expect(TokenKind::Delimiter(Delimiter::RightBrace), "expected '}'")?
            .span
            .end;

        Ok((statements, end))
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        let left = self.parse_primary()?;

        let operator = match self.current().kind {
            TokenKind::Operator(Operator::Equal) => BinaryOperator::Equal,
            TokenKind::Operator(Operator::NotEqual) => BinaryOperator::NotEqual,
            TokenKind::Operator(Operator::GreaterThan) => BinaryOperator::GreaterThan,
            TokenKind::Operator(Operator::GreaterThanOrEqual) => BinaryOperator::GreaterThanOrEqual,
            TokenKind::Operator(Operator::LessThan) => BinaryOperator::LessThan,
            TokenKind::Operator(Operator::LessThanOrEqual) => BinaryOperator::LessThanOrEqual,
            _ => return Ok(left),
        };

        self.pos += 1;

        let right = self.parse_primary()?;
        let span = Span::new(left.span.start, right.span.end);

        Ok(Expression {
            kind: ExpressionKind::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            },
            span,
        })
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        let token = self.current();

        let kind = match token.kind {
            TokenKind::Literal(LiteralKind::Integer) => {
                let value = self.source[token.span.start..token.span.end]
                    .parse::<i64>()
                    .map_err(|_| ParseError {
                        message: "integer literal is out of range for i64".into(),
                        span: token.span,
                    })?;

                ExpressionKind::Integer(value)
            }

            TokenKind::Literal(LiteralKind::Float) => {
                let value = self.source[token.span.start..token.span.end]
                    .parse::<f64>()
                    .map_err(|_| ParseError {
                        message: "float literal is out of range for f64".into(),
                        span: token.span,
                    })?;

                if !value.is_finite() {
                    return Err(ParseError {
                        message: "float literal is out of range for f64".into(),
                        span: token.span,
                    });
                }

                ExpressionKind::Float(value)
            }

            TokenKind::Literal(LiteralKind::Boolean) => {
                let value = self.source[token.span.start..token.span.end]
                    .parse::<bool>()
                    .map_err(|_| ParseError {
                        message: "bool literal should be true or false".into(),
                        span: token.span,
                    })?;

                ExpressionKind::Boolean(value)
            }

            TokenKind::Literal(LiteralKind::String) => {
                let value = self.source[token.span.start + 1..token.span.end - 1].to_owned();

                ExpressionKind::String(value)
            }

            TokenKind::Identifier => {
                let name = self.source[token.span.start..token.span.end].to_owned();

                ExpressionKind::Identifier(name)
            }

            _ => return Err(self.error("expected an expression")),
        };

        let span = token.span;
        self.pos += 1;

        Ok(Expression { kind, span })
    }

    fn current(&self) -> &'a Token {
        self.tokens.get(self.pos).expect("lexer always appends EOF")
    }

    fn next_non_newline_pos(&self) -> usize {
        let mut pos = self.pos;
        while self.tokens[pos].kind == TokenKind::Newline {
            pos += 1;
        }
        pos
    }

    fn expect(&mut self, kind: TokenKind, message: &str) -> Result<&'a Token, ParseError> {
        let token = self.current();
        if token.kind != kind {
            return Err(self.error(message));
        }

        self.pos += 1;
        Ok(token)
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_owned(),
            span: self.current().span,
        }
    }
}
