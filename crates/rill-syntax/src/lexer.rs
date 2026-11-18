use crate::span::Span;
use crate::token::{Token, TokenKind};

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

            b'/' if bytes.get(pos + 1) == Some(&b'/') => {
                pos += 2;

                while pos < bytes.len() && bytes[pos] != b'\n' && bytes[pos] != b'\r' {
                    pos += 1;
                }

                continue;
            }

            b'=' => {
                pos += 1;
                TokenKind::Equal
            }

            b'0'..=b'9' => {
                pos += 1;

                while pos < bytes.len() && bytes[pos].is_ascii_digit() {
                    pos += 1;
                }

                TokenKind::Integer
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
                    "let" => TokenKind::Let,
                    "int" => TokenKind::Int,
                    "bool" => TokenKind::Bool,
                    "true" | "false" => TokenKind::Boolean,
                    _ => TokenKind::Identifier,
                }
            }

            _ => {
                // Cover the entire UTF-8 character in the diagnostic span.
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
                    TokenKind::Let,
                    TokenKind::Identifier,
                    TokenKind::Equal,
                    TokenKind::Integer,
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
        assert_eq!(tokens[0].kind, TokenKind::Boolean);
        assert_eq!(tokens[0].span, Span::new(0, 4));
        assert_eq!(tokens[1].kind, TokenKind::Boolean);
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
        let tokens = tokenize("int bool").unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Int);
        assert_eq!(tokens[0].span, Span::new(0, 3));
        assert_eq!(tokens[1].kind, TokenKind::Bool);
        assert_eq!(tokens[1].span, Span::new(4, 8));
        assert_eq!(tokens[2].kind, TokenKind::Eof);
    }

    #[test]
    fn tokenizes_declaration() {
        let tokens = tokenize("let x = 42\n").unwrap();

        let kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();

        assert_eq!(
            kinds,
            vec![
                TokenKind::Let,
                TokenKind::Identifier,
                TokenKind::Equal,
                TokenKind::Integer,
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
        assert_eq!(tokens[5].kind, TokenKind::Let);
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
    fn rejects_unicode_with_full_byte_span() {
        for (source, end) in [("let x = é", 10), ("let x = 🦀", 12)] {
            let error = tokenize(source).unwrap_err();

            assert_eq!(error.span, Span::new(8, end));
            assert_eq!(&source[error.span.start..error.span.end], &source[8..]);
        }
    }

    #[test]
    fn skips_trailing_spaces_and_tabs() {
        let tokens = tokenize("let x = 42 \t ").unwrap();

        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[3].kind, TokenKind::Integer);
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
        assert_eq!(tokens[5].kind, TokenKind::Let);
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
