//! Lexer, parser, and syntax tree for Rill.
//!
//! Use [`parser::parse`] to build a syntax tree from source text.
//! Use [`lexer::tokenize`] to read tokens directly.
//! Spans use byte offsets in the original source.

pub mod ast;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;
