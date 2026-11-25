//! Compiler diagnostics.

use rill_syntax::span::Span;

#[derive(Debug, PartialEq, Eq)]
pub struct CompileError {
    pub message: String,
    pub span: Span,
}
