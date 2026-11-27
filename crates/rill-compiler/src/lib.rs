//! Bytecode compiler for Rill.
//!
//! Use [`compile`] to resolve names, check types, and emit bytecode.
//! Errors carry byte spans from the syntax tree.

mod compiler;
mod error;

pub use compiler::compile;
pub use error::CompileError;
