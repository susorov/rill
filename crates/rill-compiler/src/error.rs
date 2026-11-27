//! Compiler diagnostics.

use rill_syntax::span::Span;

/// A compilation failure and its source location.
#[derive(Debug, PartialEq, Eq)]
pub struct CompileError {
    /// Human-readable description of the failure.
    pub message: String,
    /// Half-open byte range in the original source, copied from the AST.
    /// The compiler does not validate these offsets.
    pub span: Span,
}
