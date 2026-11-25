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

#[derive(Debug, PartialEq, Eq)]
pub enum LexErrorKind {
    UnexpectedCharacter(char),
    UnterminatedString,
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

#[derive(Debug, PartialEq, Eq)]
pub struct LexError {
    pub kind: LexErrorKind,
    pub span: Span,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.kind, f)
    }
}

impl std::error::Error for LexError {}

/// Tokenize source text, retaining byte spans into the original source.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenizes_greater_than_and_greater_than_or_equal() {
        for (source, operator, end) in [
            ("1>2", Operator::GreaterThan, 2),
            ("1>=2", Operator::GreaterThanOrEqual, 3),
            ("1> =2", Operator::GreaterThan, 2),
        ] {
            let tokens = tokenize(source).unwrap();
            assert_eq!(tokens[1].kind, TokenKind::Operator(operator));
            assert_eq!(tokens[1].span, Span::new(1, end));
        }
    }

    #[test]
    fn tokenizes_less_than_and_less_than_or_equal() {
        for (source, operator, end) in [
            ("1<2", Operator::LessThan, 2),
            ("1<=2", Operator::LessThanOrEqual, 3),
            ("1< =2", Operator::LessThan, 2),
        ] {
            let tokens = tokenize(source).unwrap();
            assert_eq!(tokens[1].kind, TokenKind::Operator(operator));
            assert_eq!(tokens[1].span, Span::new(1, end));
        }
    }

    #[test]
    fn tokenizes_float_literals_with_spans() {
        for literal in ["0.0", "-0.0", "3.14", "-0.5", "42.0"] {
            for suffix in ["", ")"] {
                let source = format!("{literal}{suffix}");
                let tokens = tokenize(&source).unwrap();

                assert_eq!(tokens[0].kind, TokenKind::Literal(LiteralKind::Float));
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
                    kind: LexErrorKind::ExpectedDigitAfterDecimalPoint,
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
                .map(|token| (token.kind, token.span))
                .collect::<Vec<_>>(),
            vec![
                (TokenKind::Literal(LiteralKind::Integer), Span::new(0, 1)),
                (TokenKind::Literal(LiteralKind::Integer), Span::new(2, 4)),
                (TokenKind::Literal(LiteralKind::Integer), Span::new(5, 6)),
                (TokenKind::Literal(LiteralKind::Integer), Span::new(7, 9)),
                (TokenKind::Literal(LiteralKind::Integer), Span::new(10, 12)),
                (TokenKind::Literal(LiteralKind::Integer), Span::new(13, 16)),
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
            assert_eq!(tokens[3].kind, TokenKind::Literal(LiteralKind::Integer));
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
                    kind: LexErrorKind::UnexpectedCharacter('-'),
                    span: Span::new(8, 9),
                }
            );
        }
    }

    #[test]
    fn tokenizes_strings_as_single_tokens_with_quoted_byte_spans() {
        for text in [
            "",
            "hello world",
            "// not a comment",
            "true (42)",
            "héЩ",
            "\\n",
            "\\",
        ] {
            let source = format!("\"{text}\"");
            let tokens = tokenize(&source).unwrap();

            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0].kind, TokenKind::Literal(LiteralKind::String));
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
                TokenKind::Literal(LiteralKind::String),
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
        for source in ["\"", "\"hello", "\"héᕮ"] {
            assert_eq!(
                tokenize(source).unwrap_err(),
                LexError {
                    kind: LexErrorKind::UnterminatedString,
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
                    kind: LexErrorKind::UnterminatedString,
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
                    TokenKind::Literal(LiteralKind::Integer),
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
                    kind: LexErrorKind::UnexpectedCharacter('/'),
                    span: Span::new(start, start + 1),
                }
            );
        }
    }

    #[test]
    fn recognizes_boolean_literals_with_source_spans() {
        let tokens = tokenize("true false").unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].kind, TokenKind::Literal(LiteralKind::Boolean));
        assert_eq!(tokens[0].span, Span::new(0, 4));
        assert_eq!(tokens[1].kind, TokenKind::Literal(LiteralKind::Boolean));
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
                TokenKind::Literal(LiteralKind::Integer),
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
        assert_eq!(tokens[3].kind, TokenKind::Literal(LiteralKind::Integer));
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

    #[test]
    fn appends_eof_after_ignored_input() {
        for source in [" \t", "//", " \t// héᮯ", "// first\r\n \t// last"] {
            let tokens = tokenize(source).unwrap();
            let expected_len = if source.contains('\r') { 2 } else { 1 };

            assert_eq!(tokens.len(), expected_len);
            let eof = tokens.last().unwrap();
            assert_eq!(eof.kind, TokenKind::Eof);
            assert_eq!(eof.span, Span::new(source.len(), source.len()));
        }
    }

    #[test]
    fn reports_full_unexpected_unicode_character_span() {
        for prefix in ["", "\"héᮯ\" ", "// héᮯ\r\n \t"] {
            let source = format!("{prefix}λ");

            assert_eq!(
                tokenize(&source).unwrap_err(),
                LexError {
                    kind: LexErrorKind::UnexpectedCharacter('λ'),
                    span: Span::new(prefix.len(), source.len()),
                }
            );
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

    #[test]
    fn recognizes_all_supported_operators() {
        for (source, operator) in [
            ("=", Operator::Assign),
            ("==", Operator::Equal),
            ("!=", Operator::NotEqual),
            (">", Operator::GreaterThan),
            (">=", Operator::GreaterThanOrEqual),
            ("<", Operator::LessThan),
            ("<=", Operator::LessThanOrEqual),
        ] {
            let tokens = tokenize(source).unwrap();
            assert_eq!(tokens.len(), 2);
            assert_eq!(tokens[0].kind, TokenKind::Operator(operator));
            assert_eq!(tokens[0].span, Span::new(0, source.len()));
            assert_eq!(tokens[1].kind, TokenKind::Eof);
            assert_eq!(tokens[1].span, Span::new(source.len(), source.len()));
        }
    }

    #[test]
    fn preserves_diagnostic_wording() {
        for (source, expected) in [
            ("!", "unexpected character '!'"),
            ("λ", "unexpected character 'λ'"),
            ("\"hello", "unterminated string"),
            ("1.", "expected a digit after '.'"),
        ] {
            assert_eq!(tokenize(source).unwrap_err().to_string(), expected);
        }
    }

    // Check invariants without assuming public field names on Span or
    // introducing a property-testing dependency into the parent crate.
    fn assert_lexer_invariants(source: &str) {
        let boundaries: Vec<_> = source
            .char_indices()
            .map(|(offset, _)| offset)
            .chain(std::iter::once(source.len()))
            .collect();

        match tokenize(source) {
            Ok(tokens) => {
                let (eof, body) = tokens.split_last().expect("missing EOF");
                assert_eq!(eof.kind, TokenKind::Eof, "{source:?}");
                assert_eq!(
                    eof.span,
                    Span::new(source.len(), source.len()),
                    "{source:?}"
                );
                let mut previous_end = 0;
                for token in body {
                    assert_ne!(token.kind, TokenKind::Eof, "{source:?}");
                    let mut found = None;
                    'ranges: for &start in &boundaries {
                        if start < previous_end {
                            continue;
                        }
                        for &end in &boundaries {
                            if end > start && token.span == Span::new(start, end) {
                                found = Some(end);
                                break 'ranges;
                            }
                        }
                    }
                    previous_end = found.unwrap_or_else(|| {
                        panic!("invalid or overlapping token span for {source:?}")
                    });
                }
            }
            Err(error) => {
                assert!(
                    boundaries.iter().any(|&start| {
                        boundaries
                            .iter()
                            .any(|&end| start < end && error.span == Span::new(start, end))
                    }),
                    "invalid error span for {source:?}"
                );
            }
        }
    }

    #[test]
    fn checks_invariants_for_generated_utf8_inputs() {
        const ALPHABET: &[char] = &[
            'a', '_', '0', '9', '-', '.', '"', '/', '=', '!', '>', '<', '(', ')', ' ', '\t', '\n',
            '\r', 'é', 'λ', 'ᮯ', '\\',
        ];

        fn visit(source: &mut String, remaining: usize) {
            assert_lexer_invariants(source);
            if remaining == 0 {
                return;
            }
            for &ch in ALPHABET {
                source.push(ch);
                visit(source, remaining - 1);
                source.pop();
            }
        }

        visit(&mut String::new(), 4);
        for source in ["let x = -42.5", "print(\"héᮯ\")", "// λ\r\nlet x = 0"] {
            assert_lexer_invariants(source);
        }
    }
}
