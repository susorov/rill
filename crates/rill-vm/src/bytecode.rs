//! Bytecode instructions and local slot allocation.

use std::fmt;

/// Instructions and local slot count for a program.
#[derive(Debug, Clone, PartialEq)]
pub struct Bytecode {
    /// Instructions in execution order.
    pub instructions: Vec<Instruction>,
    /// Number of local slots, addressed from zero.
    pub local_count: u32,
}

/// An operation on the VM's stack or local slots.
///
/// Comparisons consume `[left, right]` and push a boolean.
/// Equality accepts matching types or numeric pairs; ordering requires numbers.
/// Mixed numeric pairs convert the integer to `f64`, which may round large values.
/// With NaN, only `!=` returns true; other numeric comparisons return false.
#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Push an integer onto the operand stack.
    PushInteger(i64),
    /// Push a boolean onto the operand stack.
    PushBoolean(bool),
    /// Push a float onto the operand stack.
    PushFloat(f64),
    /// Push a copy of a string onto the operand stack.
    PushString(String),
    /// Pop a value from the operand stack into a local slot.
    StoreLocal(u32),
    /// Copy a local slot's value onto the operand stack.
    LoadLocal(u32),
    /// Pop and print a value followed by a newline.
    Print,
    /// Print an empty line.
    PrintNewline,
    /// Compare `left == right`.
    Equal,
    /// Compare `left != right`.
    NotEqual,
    /// Compare `left > right`.
    GreaterThan,
    /// Compare `left >= right`.
    GreaterThanOrEqual,
    /// Compare `left < right`.
    LessThan,
    /// Compare `left <= right`.
    LessThanOrEqual,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PushInteger(value) => write!(f, "PUSH_INTEGER {value}"),
            Self::PushBoolean(value) => write!(f, "PUSH_BOOLEAN {value}"),
            Self::PushFloat(value) => write!(f, "PUSH_FLOAT {value}"),
            Self::PushString(value) => write!(f, "PUSH_STRING {value:?}"),
            Self::StoreLocal(slot) => write!(f, "STORE_LOCAL {slot}"),
            Self::LoadLocal(slot) => write!(f, "LOAD_LOCAL {slot}"),
            Self::NotEqual => write!(f, "NOT_EQUAL"),
            Self::Equal => write!(f, "EQUAL"),
            Self::GreaterThan => write!(f, "GREATER_THAN"),
            Self::GreaterThanOrEqual => write!(f, "GREATER_THAN_OR_EQUAL"),
            Self::LessThan => write!(f, "LESS_THAN"),
            Self::LessThanOrEqual => write!(f, "LESS_THAN_OR_EQUAL"),
            Self::Print => write!(f, "PRINT"),
            Self::PrintNewline => write!(f, "PRINT_NEWLINE"),
        }
    }
}
