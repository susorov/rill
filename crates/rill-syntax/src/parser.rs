use crate::ast::{Expression, ExpressionKind, Program, Statement};
use crate::lexer::tokenize;
use crate::span::Span;
use crate::token::{Token, TokenKind};

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub span: Span,
}

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let tokens = tokenize(source).map_err(|error| ParseError {
        message: error.message,
        span: error.span,
    })?;

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
        let mut statements = Vec::new();

        loop {
            while self.current().kind == TokenKind::Newline {
                self.pos += 1;
            }

            if self.current().kind == TokenKind::Eof {
                break;
            }

            statements.push(self.parse_statement()?);

            if !matches!(self.current().kind, TokenKind::Newline | TokenKind::Eof) {
                return Err(self.error("expected a newline or end of file"));
            }
        }

        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        let start = self.expect(TokenKind::Let, "expected 'let'")?.span.start;
        let name = self.expect(TokenKind::Identifier, "expected an identifier")?;
        let name = self.source[name.span.start..name.span.end].to_owned();
        self.expect(TokenKind::Equal, "expected '='")?;
        let value = self.parse_expression()?;
        let span = Span::new(start, value.span.end);

        Ok(Statement::Let { name, value, span })
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        let token = self.current();

        let kind = match token.kind {
            TokenKind::Integer => {
                let value = self.source[token.span.start..token.span.end]
                    .parse::<i64>()
                    .map_err(|_| ParseError {
                        message: "integer literal is out of range for i64".into(),
                        span: token.span.clone(),
                    })?;

                ExpressionKind::Integer(value)
            }

            TokenKind::Boolean => {
                let value = self.source[token.span.start..token.span.end]
                    .parse::<bool>()
                    .map_err(|_| ParseError {
                        message: "bool literal should be true or false".into(),
                        span: token.span.clone(),
                    })?;

                ExpressionKind::Boolean(value)
            }

            TokenKind::Identifier => {
                let name = self.source[token.span.start..token.span.end].to_owned();

                ExpressionKind::Identifier(name)
            }

            _ => return Err(self.error("expected an expression")),
        };

        let span = token.span.clone();
        self.pos += 1;

        Ok(Expression { kind, span })
    }

    fn current(&self) -> &'a Token {
        self.tokens.get(self.pos).expect("lexer always appends EOF")
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
            span: self.current().span.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_boolean_declarations_with_source_spans() {
        for (literal, value) in [("true", true), ("false", false)] {
            let source = format!("let enabled = {literal}");
            let end = 14 + literal.len();

            assert_eq!(
                parse(&source).unwrap(),
                Program {
                    statements: vec![Statement::Let {
                        name: "enabled".to_owned(),
                        value: Expression {
                            kind: ExpressionKind::Boolean(value),
                            span: Span::new(14, end),
                        },
                        span: Span::new(0, end),
                    }],
                }
            );
        }
    }

    #[test]
    fn rejects_boolean_literals_as_variable_names() {
        for literal in ["true", "false"] {
            let source = format!("let {literal} = 42");

            assert_eq!(
                parse(&source).unwrap_err(),
                ParseError {
                    message: "expected an identifier".to_owned(),
                    span: Span::new(4, 4 + literal.len()),
                }
            );
        }
    }

    #[test]
    fn parses_declaration_with_source_spans() {
        let program = parse("  let count = 42 \t").unwrap();

        assert_eq!(
            program,
            Program {
                statements: vec![Statement::Let {
                    name: "count".to_owned(),
                    value: Expression {
                        kind: ExpressionKind::Integer(42),
                        span: Span::new(14, 16),
                    },
                    span: Span::new(2, 16),
                }],
            }
        );
    }

    #[test]
    fn parses_identifier_expressions_with_source_spans() {
        for name in ["count", "let_count", "_count2"] {
            let source = format!("let limit = {name}");
            let end = 12 + name.len();

            assert_eq!(
                parse(&source).unwrap(),
                Program {
                    statements: vec![Statement::Let {
                        name: "limit".to_owned(),
                        value: Expression {
                            kind: ExpressionKind::Identifier(name.to_owned()),
                            span: Span::new(12, end),
                        },
                        span: Span::new(0, end),
                    }],
                }
            );
        }
    }

    #[test]
    fn parses_chained_variable_references() {
        for newline in ["\n", "\r\n", "\r"] {
            let source = format!(
                "let count = 42{newline}let limit = count{newline}let maximum = limit{newline}"
            );
            let program = parse(&source).unwrap();

            assert_eq!(program.statements.len(), 3);
            for (statement, expected) in program.statements.iter().zip([
                ExpressionKind::Integer(42),
                ExpressionKind::Identifier("count".to_owned()),
                ExpressionKind::Identifier("limit".to_owned()),
            ]) {
                let Statement::Let { value, .. } = statement;
                assert_eq!(value.kind, expected);
            }
        }
    }

    #[test]
    fn parses_declarations_separated_by_newlines_and_blank_lines() {
        for newline in ["\n", "\r\n", "\r"] {
            let source = format!("{newline}let x = 0{newline}{newline}let y = 10{newline}");
            let program = parse(&source).unwrap();
            let first_start = newline.len();
            let second_start = first_start + 9 + 2 * newline.len();

            assert_eq!(
                program.statements,
                vec![
                    Statement::Let {
                        name: "x".to_owned(),
                        value: Expression {
                            kind: ExpressionKind::Integer(0),
                            span: Span::new(first_start + 8, first_start + 9),
                        },
                        span: Span::new(first_start, first_start + 9),
                    },
                    Statement::Let {
                        name: "y".to_owned(),
                        value: Expression {
                            kind: ExpressionKind::Integer(10),
                            span: Span::new(second_start + 8, second_start + 10),
                        },
                        span: Span::new(second_start, second_start + 10),
                    },
                ]
            );
        }
    }

    #[test]
    fn accepts_empty_programs() {
        for source in ["", " \t ", "\n\r\n\r \t"] {
            assert!(parse(source).unwrap().statements.is_empty());
        }
    }

    #[test]
    fn reports_missing_or_unexpected_declaration_parts() {
        for (source, message, start, end) in [
            ("x = 1", "expected 'let'", 0, 1),
            ("let", "expected an identifier", 3, 3),
            ("let = 1", "expected an identifier", 4, 5),
            ("let let = 1", "expected an identifier", 4, 7),
            ("let x", "expected '='", 5, 5),
            ("let x 1", "expected '='", 6, 7),
            ("let x =", "expected an expression", 7, 7),
            ("let x = let", "expected an expression", 8, 11),
            ("let x = =", "expected an expression", 8, 9),
            ("let x =\n1", "expected an expression", 7, 8),
        ] {
            assert_eq!(
                parse(source).unwrap_err(),
                ParseError {
                    message: message.to_owned(),
                    span: Span::new(start, end),
                },
                "source: {source:?}"
            );
        }
    }

    #[test]
    fn rejects_trailing_tokens_on_the_same_line() {
        for (source, end) in [
            ("let x = 1 let y = 2", 13),
            ("let x = 1 y", 11),
            ("let x = y let z = 2", 13),
            ("let x = y z", 11),
            ("let x = y 1", 11),
        ] {
            assert_eq!(
                parse(source).unwrap_err(),
                ParseError {
                    message: "expected a newline or end of file".to_owned(),
                    span: Span::new(10, end),
                }
            );
        }
    }

    #[test]
    fn accepts_largest_i64_literal() {
        let program = parse("let x = 9223372036854775807").unwrap();
        let Statement::Let { value, .. } = &program.statements[0];

        assert_eq!(value.kind, ExpressionKind::Integer(i64::MAX));
    }

    #[test]
    fn rejects_integer_overflow() {
        let error = parse("let x = 9223372036854775808").unwrap_err();

        assert_eq!(error.message, "integer literal is out of range for i64");
        assert_eq!(error.span, Span::new(8, 27));
    }

    #[test]
    fn preserves_lexical_errors() {
        let error = parse("let x = é").unwrap_err();

        assert_eq!(error.message, "unexpected character 'é'");
        assert_eq!(error.span, Span::new(8, 10));
    }
}
