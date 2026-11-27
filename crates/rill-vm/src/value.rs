//! Values stored on the operand stack and in local slots.

/// A runtime value. Cloning a string copies its contents.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A signed 64-bit integer.
    Integer(i64),
    /// A 64-bit float, including infinities and NaN.
    Float(f64),
    /// A boolean value.
    Boolean(bool),
    /// Owned UTF-8 text.
    String(String),
}
