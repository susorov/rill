//! Stack-based bytecode execution for Rill.
//!
//! Create a [`Vm`] from [`bytecode::Bytecode`] and call [`Vm::run`].
//! Inspect its stack, locals, and instruction pointer to observe execution state.

pub mod bytecode;
pub mod value;
pub mod vm;

pub use value::Value;
pub use vm::Vm;
