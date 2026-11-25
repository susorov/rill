//! Bytecode compiler for the Rill programming language.

mod compiler;
mod error;

pub use compiler::compile;
pub use error::CompileError;
