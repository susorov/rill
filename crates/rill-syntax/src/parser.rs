use crate::ast::{BinaryOperator, Expression, ExpressionKind, Program, Statement};
use crate::lexer::tokenize;
use crate::span::Span;
use crate::token::{Delimiter, Keyword, Literal, Operator, Token, TokenKind};

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
        match self.current().kind {
            TokenKind::Keyword(Keyword::Print) => self.parse_print_statement(),
            _ => self.parse_let_statement(),
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

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        let left = self.parse_primary()?;

        let operator = match self.current().kind {
            TokenKind::Operator(Operator::Equal) => BinaryOperator::Equal,
            TokenKind::Operator(Operator::NotEqual) => BinaryOperator::NotEqual,
            TokenKind::Operator(Operator::GreaterThan) => BinaryOperator::GreaterThan,
            TokenKind::Operator(Operator::GreaterThanOrEqual) => BinaryOperator::GreaterThanOrEqual,
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
            TokenKind::Literal(Literal::Integer) => {
                let value = self.source[token.span.start..token.span.end]
                    .parse::<i64>()
                    .map_err(|_| ParseError {
                        message: "integer literal is out of range for i64".into(),
                        span: token.span.clone(),
                    })?;

                ExpressionKind::Integer(value)
            }

            TokenKind::Literal(Literal::Float) => {
                let value = self.source[token.span.start..token.span.end]
                    .parse::<f64>()
                    .map_err(|_| ParseError {
                        message: "float literal is out of range for f64".into(),
                        span: token.span.clone(),
                    })?;

                if !value.is_finite() {
                    return Err(ParseError {
                        message: "float literal is out of range for f64".into(),
                        span: token.span.clone(),
                    });
                }

                ExpressionKind::Float(value)
            }

            TokenKind::Literal(Literal::Boolean) => {
                let value = self.source[token.span.start..token.span.end]
                    .parse::<bool>()
                    .map_err(|_| ParseError {
                        message: "bool literal should be true or false".into(),
                        span: token.span.clone(),
                    })?;

                ExpressionKind::Boolean(value)
            }

            TokenKind::Literal(Literal::String) => {
                let value = self.source[token.span.start + 1..token.span.end - 1].to_owned();

                ExpressionKind::String(value)
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
    fn parses_greater_than_with_operand_spans() {
        let source = "let result = 10>-2";
        let program = parse(source).unwrap();
        let Statement::Let { value, .. } = &program.statements[0] else {
            panic!("expected declaration");
        };
        let ExpressionKind::Binary {
            left,
            operator,
            right,
        } = &value.kind
        else {
            panic!("expected comparison");
        };
        assert_eq!(*operator, BinaryOperator::GreaterThan);
        assert_eq!(&source[left.span.start..left.span.end], "10");
        assert_eq!(&source[right.span.start..right.span.end], "-2");
        assert_eq!(&source[value.span.start..value.span.end], "10>-2");
        for source in [
            "print(> 1)",
            "print(1 > = 2)",
            "print(1 > 2 >= 3)",
            "print(1 >)",
            "let b = 1 >",
            "print(1 > 2 > 3)",
            "print(1 > 2 == true)",
            "print(1 == 2 > 3)",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
    }

    #[test]
    fn parses_greater_than_or_equal_with_operand_spans() {
        let source = "let result = 10>=-2";
        let program = parse(source).unwrap();
        let Statement::Let { value, .. } = &program.statements[0] else {
            panic!("expected declaration");
        };
        let ExpressionKind::Binary {
            left,
            operator,
            right,
        } = &value.kind
        else {
            panic!("expected comparison");
        };
        assert_eq!(*operator, BinaryOperator::GreaterThanOrEqual);
        assert_eq!(&source[left.span.start..left.span.end], "10");
        assert_eq!(&source[right.span.start..right.span.end], "-2");
        assert_eq!(&source[value.span.start..value.span.end], "10>=-2");
        for source in [
            "print(>= 1)",
            "print(1 >=)",
            "let b = 1 >=",
            "print(1 >= 2 >= 3)",
            "print(1 >= 2 == true)",
            "print(1 == 2 >= 3)",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
    }

    #[test]
    fn parses_less_than_or_equal_with_operand_spans() {
        let source = "let result = 10<=-2";
        let program = parse(source).unwrap();
        let Statement::Let { value, .. } = &program.statements[0] else {
            panic!("expected declaration");
        };
        let ExpressionKind::Binary {
            left,
            operator,
            right,
        } = &value.kind
        else {
            panic!("expected comparison");
        };
        assert_eq!(*operator, BinaryOperator::LessThanOrEqual);
        assert_eq!(&source[left.span.start..left.span.end], "10");
        assert_eq!(&source[right.span.start..right.span.end], "-2");
        assert_eq!(&source[value.span.start..value.span.end], "10<=-2");
        for source in [
            "print(<= 1)",
            "print(1 <=)",
            "let b = 1 <=",
            "print(1 <= 2 <= 3)",
            "print(1 <= 2 >= 3)",
            "print(1 >= 2 <= 3)",
            "print(1 <= 2 == true)",
            "print(1 == 2 <= 3)",
        ] {
            assert!(parse(source).is_err(), "{source}");
        }
    }

    #[test]
    fn parses_float_declarations_with_source_spans_and_signs() {
        for (literal, expected) in [
            ("0.0", 0.0_f64),
            ("-0.0", -0.0),
            ("1.25", 1.25),
            ("-42.25", -42.25),
            ("42.0", 42.0),
            ("0.1", 0.1),
        ] {
            let source = format!("let value = {literal}");
            let program = parse(&source).unwrap();
            let Statement::Let { name, value, span } = &program.statements[0] else {
                panic!("expected let statement");
            };
            let ExpressionKind::Float(actual) = &value.kind else {
                panic!("expected float expression");
            };

            assert_eq!(name, "value");
            // Bit comparison also distinguishes negative zero from positive zero.
            assert_eq!(actual.to_bits(), expected.to_bits());
            assert_eq!(value.span, Span::new(12, source.len()));
            assert_eq!(*span, Span::new(0, source.len()));
        }
    }

    #[test]
    fn accepts_finite_float_limits_and_subnormal_values() {
        for expected in [f64::MAX, -f64::MAX, f64::MIN_POSITIVE, f64::from_bits(1)] {
            // Fixed-point notation stays within the supported decimal syntax.
            let source = format!("let value = {expected:.324}");
            let program = parse(&source).unwrap();
            let Statement::Let { value, .. } = &program.statements[0] else {
                panic!("expected let statement");
            };
            let ExpressionKind::Float(actual) = &value.kind else {
                panic!("expected float expression");
            };

            assert_eq!(actual.to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn rejects_float_overflow_with_literal_span() {
        for sign in ["", "-"] {
            let source = format!("let value = {sign}{}.0", "9".repeat(309));

            assert_eq!(
                parse(&source).unwrap_err(),
                ParseError {
                    message: "float literal is out of range for f64".to_owned(),
                    span: Span::new(12, source.len()),
                }
            );
        }
    }

    #[test]
    fn parses_string_declarations_with_source_spans() {
        for text in ["", "hello world", "// not a comment", "true (42)"] {
            let prefix = "let text = ";
            let source = format!("{prefix}\"{text}\"");

            assert_eq!(
                parse(&source).unwrap(),
                Program {
                    statements: vec![Statement::Let {
                        name: "text".to_owned(),
                        value: Expression {
                            kind: ExpressionKind::String(text.to_owned()),
                            span: Span::new(prefix.len(), source.len()),
                        },
                        span: Span::new(0, source.len()),
                    }],
                }
            );
        }
    }

    #[test]
    fn preserves_unterminated_string_errors() {
        for suffix in ["", "\n", "\r\n", "\r"] {
            let source = format!("let text = \"hello{suffix}");

            assert_eq!(
                parse(&source).unwrap_err(),
                ParseError {
                    message: "unterminated string".to_owned(),
                    span: Span::new(11, source.len() - suffix.len()),
                }
            );
        }
    }

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
    fn rejects_reserved_words_as_variable_names() {
        for keyword in ["let", "true", "false", "int", "bool", "float"] {
            let source = format!("let {keyword} = 42");

            assert_eq!(
                parse(&source).unwrap_err(),
                ParseError {
                    message: "expected an identifier".to_owned(),
                    span: Span::new(4, 4 + keyword.len()),
                }
            );
        }
    }

    #[test]
    fn rejects_type_keywords_as_expressions() {
        for keyword in ["int", "bool", "float"] {
            let source = format!("let value = {keyword}");

            assert_eq!(
                parse(&source).unwrap_err(),
                ParseError {
                    message: "expected an expression".to_owned(),
                    span: Span::new(12, 12 + keyword.len()),
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
                let Statement::Let { value, .. } = statement else {
                    panic!("expected let statement");
                };

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

        let Statement::Let { value, .. } = &program.statements[0] else {
            panic!("expected let statement");
        };

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
        let error = parse("let x = @").unwrap_err();

        assert_eq!(error.message, "unexpected character '@'");
        assert_eq!(error.span, Span::new(8, 9));
    }
}

#[test]
fn parses_print_statement() {
    for (argument, kind) in [
        ("count", ExpressionKind::Identifier("count".to_owned())),
        ("42", ExpressionKind::Integer(42)),
        ("1.25", ExpressionKind::Float(1.25)),
        ("-0.5", ExpressionKind::Float(-0.5)),
        ("42.0", ExpressionKind::Float(42.0)),
        ("true", ExpressionKind::Boolean(true)),
        ("false", ExpressionKind::Boolean(false)),
        (
            "\"hello world\"",
            ExpressionKind::String("hello world".to_owned()),
        ),
        ("\"\"", ExpressionKind::String(String::new())),
    ] {
        let source = format!("print({argument})");

        assert_eq!(
            parse(&source).unwrap().statements,
            vec![Statement::Print {
                value: Some(Expression {
                    kind,
                    span: Span::new(6, 6 + argument.len()),
                }),
                span: Span::new(0, source.len()),
            }]
        );
    }
}

#[test]
fn parses_empty_print_statement() {
    let program = parse("print()").unwrap();

    assert_eq!(
        program.statements,
        vec![Statement::Print {
            value: None,
            span: Span::new(0, 7),
        }]
    );
}

#[test]
fn rejects_invalid_print_statements() {
    for (source, message, start, end) in [
        ("print(count", "expected ')'", 11, 11),
        ("print count", "expected '('", 6, 11),
        ("print(", "expected an expression", 6, 6),
        ("print(let)", "expected an expression", 6, 9),
        ("print(\n)", "expected an expression", 6, 7),
        ("print(42 43)", "expected ')'", 9, 11),
        ("print(\"hello\"", "expected ')'", 13, 13),
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
fn parses_equality_and_reports_operand_spans() {
    for source in ["let b = 10 == 10", "print (10==10)"] {
        let program = parse(source).unwrap();
        let value = match &program.statements[0] {
            Statement::Let { value, .. } => value,
            Statement::Print {
                value: Some(value), ..
            } => value,
            _ => panic!("expected expression"),
        };
        let ExpressionKind::Binary {
            left,
            operator,
            right,
        } = &value.kind
        else {
            panic!("expected binary expression");
        };
        assert_eq!(*operator, BinaryOperator::Equal);
        assert_eq!(left.kind, ExpressionKind::Integer(10));
        assert_eq!(right.kind, ExpressionKind::Integer(10));
        assert_eq!(&source[left.span.start..left.span.end], "10");
        assert_eq!(&source[right.span.start..right.span.end], "10");
        assert_eq!(value.span, Span::new(left.span.start, right.span.end));
    }
}

#[test]
fn rejects_missing_operands_and_chained_equality() {
    for source in [
        "let b = == 10",
        "let b = 10 ==",
        "print(10 ==)",
        "let b = 1 == 1 == true",
    ] {
        assert!(parse(source).is_err(), "{source}");
    }
}

#[test]
fn parses_not_equal_and_rejects_incomplete_comparisons() {
    for source in ["let b = 10 != 20", "print(10!=20)"] {
        let program = parse(source).unwrap();
        let value = match &program.statements[0] {
            Statement::Let { value, .. } => value,
            Statement::Print {
                value: Some(value), ..
            } => value,
            _ => panic!("expected expression"),
        };
        assert!(matches!(
            value.kind,
            ExpressionKind::Binary {
                operator: BinaryOperator::NotEqual,
                ..
            }
        ));
    }
    for source in [
        "print(10 !=)",
        "let b = != 10",
        "print(1 != 2 == true)",
        "print(1 != 2 != 3)",
    ] {
        assert!(parse(source).is_err(), "{source}");
    }
}
