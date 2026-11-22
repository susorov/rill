use crate::span::Span;
use crate::token::{Delimiter, Keyword, Literal, Operator, Token, TokenKind};

#[derive(Debug, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub span: Span,
}

pub fn tokenize(source: &str) -> Result<Vec<Token>, LexError> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut pos = 0;

    while pos < bytes.len() {
        let start = pos;
        let kind = match bytes[pos] {
            b' ' | b'\t' => {
                pos += 1;
                continue;
            }

            b'\n' => {
                pos += 1;
                TokenKind::Newline
            }

            // Treat CRLF as one newline; also accept a standalone CR.
            b'\r' => {
                pos += 1;

                if pos < bytes.len() && bytes[pos] == b'\n' {
                    pos += 1;
                }

                TokenKind::Newline
            }

            b'(' => {
                pos += 1;
                TokenKind::Delimiter(Delimiter::LeftParenthesis)
            }

            b')' => {
                pos += 1;
                TokenKind::Delimiter(Delimiter::RightParenthesis)
            }

            b'/' if bytes.get(pos + 1) == Some(&b'/') => {
                pos += 2;

                while pos < bytes.len() && bytes[pos] != b'\n' && bytes[pos] != b'\r' {
                    pos += 1;
                }

                continue;
            }

            b'=' if bytes.get(pos + 1) == Some(&b'=') => {
                pos += 2;
                TokenKind::Operator(Operator::Equal)
            }

            b'!' if bytes.get(pos + 1) == Some(&b'=') => {
                pos += 2;
                TokenKind::Operator(Operator::NotEqual)
            }

            b'>' if bytes.get(pos + 1) == Some(&b'=') => {
                pos += 2;
                TokenKind::Operator(Operator::GreaterThanOrEqual)
            }

            b'<' if bytes.get(pos + 1) == Some(&b'=') => {
                pos += 2;
                TokenKind::Operator(Operator::LessThanOrEqual)
            }

            b'=' => {
                pos += 1;
                TokenKind::Operator(Operator::Assign)
            }

            b'"' => {
                pos += 1;

                loop {
                    match bytes.get(pos) {
                        Some(b'"') => {
                            pos += 1;
                            break TokenKind::Literal(Literal::String);
                        }
                        Some(b'\n' | b'\r') | None => {
                            return Err(LexError {
                                message: "unterminated string".to_owned(),
                                span: Span::new(start, pos),
                            });
                        }
                        Some(_) => pos += 1,
                    }
                }
            }

            b'0'..=b'9' | b'-'
                if bytes[pos] != b'-' || bytes.get(pos + 1).is_some_and(u8::is_ascii_digit) =>
            {
                pos += 1;

                while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                    pos += 1;
                }

                if bytes.get(pos) == Some(&b'.') {
                    pos += 1;

                    if !bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                        return Err(LexError {
                            message: "expected a digit after '.'".to_owned(),
                            span: Span::new(start, pos),
                        });
                    }

                    while bytes.get(pos).is_some_and(u8::is_ascii_digit) {
                        pos += 1;
                    }

                    TokenKind::Literal(Literal::Float)
                } else {
                    TokenKind::Literal(Literal::Integer)
                }
            }

            // Match keywords only after scanning the full identifier (e.g. `let_count`).
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => {
                pos += 1;

                while pos < bytes.len()
                    && (bytes[pos].is_ascii_alphanumeric() || bytes[pos] == b'_')
                {
                    pos += 1;
                }

                match &source[start..pos] {
                    "let" => TokenKind::Keyword(Keyword::Let),
                    "int" => TokenKind::Keyword(Keyword::Int),
                    "bool" => TokenKind::Keyword(Keyword::Bool),
                    "float" => TokenKind::Keyword(Keyword::Float),
                    "print" => TokenKind::Keyword(Keyword::Print),
                    "true" | "false" => TokenKind::Literal(Literal::Boolean),
                    _ => TokenKind::Identifier,
                }
            }

            _ => {
                // Cover the entire unexpected character in the diagnostic span.
                let ch = source[pos..].chars().next().unwrap();

                return Err(LexError {
                    message: format!("unexpected character '{ch}'"),
                    span: Span::new(pos, pos + ch.len_utf8()),
                });
            }
        };

        tokens.push(Token {
            kind,
            span: Span::new(start, pos),
        });
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(pos, pos),
    });

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_greater_than_or_equal_and_rejects_bare_greater_than() {
        let tokens = tokenize("1>=2").unwrap();
        assert_eq!(
            tokens[1].kind,
            TokenKind::Operator(Operator::GreaterThanOrEqual)
        );
        assert_eq!(tokens[1].span, Span::new(1, 3));
        for source in [">", "1 > 2", "1 > = 2"] {
            let error = tokenize(source).unwrap_err();
            let start = source.find('>').unwrap();
            assert_eq!(error.message, "unexpected character '>'");
            assert_eq!(error.span, Span::new(start, start + 1));
        }
    }

    #[test]
    fn tokenizes_less_than_or_equal_and_rejects_bare_less_than() {
        let tokens = tokenize("1<=2").unwrap();
        assert_eq!(
            tokens[1].kind,
            TokenKind::Operator(Operator::LessThanOrEqual)
        );
        assert_eq!(tokens[1].span, Span::new(1, 3));
        for source in ["<", "1 < 2", "1 < = 2"] {
            let error = tokenize(source).unwrap_err();
            let start = source.find('<').unwrap();
            assert_eq!(error.message, "unexpected character '<'");
            assert_eq!(error.span, Span::new(start, start + 1));
        }
    }

    #[test]
    fn tokenizes_float_literals_with_spans() {
        for literal in ["0.0", "-0.0", "3.14", "-0.5", "42.0"] {
            for suffix in ["", ")"] {
                let source = format!("{literal}{suffix}");
                let tokens = tokenize(&source).unwrap();

                assert_eq!(tokens[0].kind, TokenKind::Literal(Literal::Float));
                assert_eq!(tokens[0].span, Span::new(0, literal.len()));
                if suffix.is_empty() {
                    assert_eq!(tokens.len(), 2);
                } else {
                    assert_eq!(tokens.len(), 3);
                    assert_eq!(
                        tokens[1].kind,
                        TokenKind::Delimiter(Delimiter::RightParenthesis)
                    );
                    assert_eq!(tokens[1].span, Span::new(literal.len(), source.len()));
                }
                let eof = tokens.last().unwrap();
                assert_eq!(eof.kind, TokenKind::Eof);
                assert_eq!(eof.span, Span::new(source.len(), source.len()));
            }
        }
    }

    #[test]
    fn rejects_decimal_points_without_a_following_digit() {
        for (literal, end) in [("1.", 2), ("-1.", 3), ("1.a", 2), ("1.\n", 2), ("1..2", 2)] {
            let source = format!("let x = {literal}");

            assert_eq!(
                tokenize(&source).unwrap_err(),
                LexError {
                    message: "expected a digit after '.'".to_owned(),
                    span: Span::new(8, 8 + end),
                }
            );
        }
    }

    #[test]
    fn tokenizes_mixed_positive_and_negative_integers_with_spans() {
        let tokens = tokenize("0 -0 7 -7 42 -42").unwrap();

        assert_eq!(
            tokens
                .iter()
                .map(|token| (token.kind, token.span.clone()))
                .collect::<Vec<_>>(),
            vec![
                (TokenKind::Literal(Literal::Integer), Span::new(0, 1)),
                (TokenKind::Literal(Literal::Integer), Span::new(2, 4)),
                (TokenKind::Literal(Literal::Integer), Span::new(5, 6)),
                (TokenKind::Literal(Literal::Integer), Span::new(7, 9)),
                (TokenKind::Literal(Literal::Integer), Span::new(10, 12)),
                (TokenKind::Literal(Literal::Integer), Span::new(13, 16)),
                (TokenKind::Eof, Span::new(16, 16)),
            ]
        );
    }

    #[test]
    fn tokenizes_negative_integer_literals() {
        for literal in ["-42", "-0", "-9223372036854775808"] {
            let source = format!("let x = {literal}");
            let tokens = tokenize(&source).unwrap();

            assert_eq!(tokens.len(), 5);
            assert_eq!(tokens[3].kind, TokenKind::Literal(Literal::Integer));
            assert_eq!(tokens[3].span, Span::new(8, source.len()));
            assert_eq!(tokens[4].kind, TokenKind::Eof);
            assert_eq!(tokens[4].span, Span::new(source.len(), source.len()));
        }
    }

    #[test]
    fn rejects_minus_without_an_immediate_digit() {
        for value in ["-", "-name", "- 42", "-\n42", "--42"] {
            let source = format!("let x = {value}");

            assert_eq!(
                tokenize(&source).unwrap_err(),
                LexError {
                    message: "unexpected character '-'".to_owned(),
                    span: Span::new(8, 9),
                }
            );
        }
    }

    #[test]
    fn tokenizes_strings_as_single_tokens_with_quoted_byte_spans() {
        for text in ["", "hello world", "// not a comment", "true (42)"] {
            let source = format!("\"{text}\"");
            let tokens = tokenize(&source).unwrap();

            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0].kind, TokenKind::Literal(Literal::String));
            assert_eq!(tokens[0].span, Span::new(0, source.len()));
            assert_eq!(tokens[1].kind, TokenKind::Eof);
            assert_eq!(tokens[1].span, Span::new(source.len(), source.len()));
        }
    }

    #[test]
    fn resumes_tokenizing_after_strings() {
        let source = "print(\"hello!\") // comment\nprint()";
        let tokens = tokenize(source).unwrap();

        assert_eq!(
            tokens.iter().map(|token| token.kind).collect::<Vec<_>>(),
            vec![
                TokenKind::Keyword(Keyword::Print),
                TokenKind::Delimiter(Delimiter::LeftParenthesis),
                TokenKind::Literal(Literal::String),
                TokenKind::Delimiter(Delimiter::RightParenthesis),
                TokenKind::Newline,
                TokenKind::Keyword(Keyword::Print),
                TokenKind::Delimiter(Delimiter::LeftParenthesis),
                TokenKind::Delimiter(Delimiter::RightParenthesis),
                TokenKind::Eof,
            ]
        );
        assert_eq!(tokens[2].span, Span::new(6, 14));
        assert_eq!(tokens[3].span, Span::new(14, 15));
        assert_eq!(tokens[4].span, Span::new(26, 27));
    }

    #[test]
    fn rejects_unterminated_strings_at_eof() {
        for source in ["\"", "\"hello"] {
            assert_eq!(
                tokenize(source).unwrap_err(),
                LexError {
                    message: "unterminated string".to_owned(),
                    span: Span::new(0, source.len()),
                }
            );
        }
    }

    #[test]
    fn rejects_raw_newlines_in_strings() {
        for newline in ["\n", "\r\n", "\r"] {
            let source = format!("\"hello{newline}world\"");

            assert_eq!(
                tokenize(&source).unwrap_err(),
                LexError {
                    message: "unterminated string".to_owned(),
                    span: Span::new(0, 6),
                }
            );
        }
    }

    #[test]
    fn preserves_newlines_and_byte_offsets_after_comments() {
        let comment = "// comment";
        for newline in ["\n", "\r\n", "\r"] {
            let source = format!("{comment}{newline}let x = 42");
            let start = comment.len() + newline.len();
            let tokens = tokenize(&source).unwrap();

            assert_eq!(
                tokens.iter().map(|t| t.kind).collect::<Vec<_>>(),
                vec![
                    TokenKind::Newline,
                    TokenKind::Keyword(Keyword::Let),
                    TokenKind::Identifier,
                    TokenKind::Operator(Operator::Assign),
                    TokenKind::Literal(Literal::Integer),
                    TokenKind::Eof,
                ]
            );
            assert_eq!(tokens[0].span, Span::new(comment.len(), start));
            assert_eq!(tokens[1].span, Span::new(start, start + 3));
            assert_eq!(tokens[4].span, Span::new(start + 8, start + 10));
            assert_eq!(tokens[5].span, Span::new(source.len(), source.len()));
        }
    }

    #[test]
    fn rejects_slashes_that_do_not_start_line_comments() {
        for source in ["/", "/ /", "/* comment */", "let x = 42 / 2"] {
            let start = source.find('/').unwrap();
            assert_eq!(
                tokenize(source).unwrap_err(),
                LexError {
                    message: "unexpected character '/'".to_owned(),
                    span: Span::new(start, start + 1),
                }
            );
        }
    }

    #[test]
    fn recognizes_boolean_literals_with_source_spans() {
        let tokens = tokenize("true false").unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Literal(Literal::Boolean));
        assert_eq!(tokens[0].span, Span::new(0, 4));
        assert_eq!(tokens[1].kind, TokenKind::Literal(Literal::Boolean));
        assert_eq!(tokens[1].span, Span::new(5, 10));
        assert_eq!(tokens[2].kind, TokenKind::Eof);
    }

    #[test]
    fn keeps_keyword_like_names_as_identifiers() {
        for source in [
            "true_value",
            "false_count",
            "true1",
            "falsehood",
            "int_value",
            "integer",
            "int2",
            "bool_value",
            "boolean",
            "bool2",
            "float_value",
            "floating",
            "float2",
        ] {
            let tokens = tokenize(source).unwrap();

            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0].kind, TokenKind::Identifier);
            assert_eq!(tokens[0].span, Span::new(0, source.len()));
            assert_eq!(tokens[1].kind, TokenKind::Eof);
        }
    }

    #[test]
    fn recognizes_reserved_type_names() {
        let tokens = tokenize("int bool float").unwrap();

        assert_eq!(tokens.len(), 4);
        assert_eq!(tokens[0].kind, TokenKind::Keyword(Keyword::Int));
        assert_eq!(tokens[0].span, Span::new(0, 3));
        assert_eq!(tokens[1].kind, TokenKind::Keyword(Keyword::Bool));
        assert_eq!(tokens[1].span, Span::new(4, 8));
        assert_eq!(tokens[2].kind, TokenKind::Keyword(Keyword::Float));
        assert_eq!(tokens[2].span, Span::new(9, 14));
        assert_eq!(tokens[3].kind, TokenKind::Eof);
    }

    #[test]
    fn tokenizes_declaration() {
        let tokens = tokenize("let x = 42\n").unwrap();

        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();

        assert_eq!(
            kinds,
            vec![
                TokenKind::Keyword(Keyword::Let),
                TokenKind::Identifier,
                TokenKind::Operator(Operator::Assign),
                TokenKind::Literal(Literal::Integer),
                TokenKind::Newline,
                TokenKind::Eof,
            ]
        );

        assert_eq!(tokens[0].span, Span::new(0, 3));
        assert_eq!(tokens[1].span, Span::new(4, 5));
        assert_eq!(tokens[3].span, Span::new(8, 10));
        assert_eq!(tokens[4].span, Span::new(10, 11));
    }

    #[test]
    fn tokenizes_multiple_lines() {
        let source = "let x = 42\nlet y = 10";
        let tokens = tokenize(source).unwrap();

        assert_eq!(tokens[4].kind, TokenKind::Newline);
        assert_eq!(tokens[5].kind, TokenKind::Keyword(Keyword::Let));
        assert_eq!(tokens.last().unwrap().kind, TokenKind::Eof);
    }

    #[test]
    fn handles_empty_input() {
        let tokens = tokenize("").unwrap();

        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].kind, TokenKind::Eof);
        assert_eq!(tokens[0].span, Span::new(0, 0));
    }

    #[test]
    fn supports_windows_newlines() {
        let tokens = tokenize("let x = 42\r\n").unwrap();

        assert_eq!(tokens[4].kind, TokenKind::Newline);
        assert_eq!(tokens[4].span, Span::new(10, 12));
    }

    #[test]
    fn rejects_semicolons() {
        let error = tokenize("let x = 42;").unwrap_err();

        assert_eq!(error.span, Span::new(10, 11));
    }

    #[test]
    fn skips_trailing_spaces_and_tabs() {
        let tokens = tokenize("let x = 42 \t ").unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[3].kind, TokenKind::Literal(Literal::Integer));
        assert_eq!(tokens[3].span, Span::new(8, 10));
        assert_eq!(tokens[4].kind, TokenKind::Eof);
        assert_eq!(tokens[4].span, Span::new(13, 13));
    }

    #[test]
    fn supports_standalone_carriage_returns() {
        let tokens = tokenize("let x = 42\rlet y = 10\r").unwrap();

        assert_eq!(tokens.len(), 11);
        assert_eq!(tokens[4].kind, TokenKind::Newline);
        assert_eq!(tokens[4].span, Span::new(10, 11));
        assert_eq!(tokens[5].kind, TokenKind::Keyword(Keyword::Let));
        assert_eq!(tokens[5].span, Span::new(11, 14));
        assert_eq!(tokens[9].kind, TokenKind::Newline);
        assert_eq!(tokens[9].span, Span::new(21, 22));
        assert_eq!(tokens[10].kind, TokenKind::Eof);
        assert_eq!(tokens[10].span, Span::new(22, 22));
    }

    #[test]
    fn recognizes_full_identifiers() {
        let tokens = tokenize("let_count = 42").unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Identifier);
    }
}

#[test]
fn tokenizes_print_statement() {
    let tokens = tokenize("print(count)").unwrap();

    let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();

    assert_eq!(
        kinds,
        vec![
            TokenKind::Keyword(Keyword::Print),
            TokenKind::Delimiter(Delimiter::LeftParenthesis),
            TokenKind::Identifier,
            TokenKind::Delimiter(Delimiter::RightParenthesis),
            TokenKind::Eof,
        ]
    );

    assert_eq!(tokens[0].span, Span::new(0, 5));
    assert_eq!(tokens[1].span, Span::new(5, 6));
    assert_eq!(tokens[2].span, Span::new(6, 11));
    assert_eq!(tokens[3].span, Span::new(11, 12));
}

#[test]
fn tokenizes_not_equal_with_span() {
    let tokens = tokenize("1!=2").unwrap();
    assert_eq!(tokens[1].kind, TokenKind::Operator(Operator::NotEqual));
    assert_eq!(tokens[1].span, Span::new(1, 3));
    assert!(tokenize("!").is_err());
}
