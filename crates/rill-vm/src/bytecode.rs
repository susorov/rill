use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub local_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    /// Push an integer onto the operand stack.
    PushInteger(i64),
    /// Push a boolean onto the operand stack.
    PushBoolean(bool),
    /// Pop a value from the operand stack into a local slot.
    StoreLocal(u32),
    /// Copy a local slot's value onto the operand stack.
    LoadLocal(u32),
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PushInteger(value) => write!(f, "PUSH_INTEGER {value}"),
            Self::PushBoolean(value) => write!(f, "PUSH_BOOLEAN {value}"),
            Self::StoreLocal(slot) => write!(f, "STORE_LOCAL {slot}"),
            Self::LoadLocal(slot) => write!(f, "LOAD_LOCAL {slot}"),
        }
    }
}
