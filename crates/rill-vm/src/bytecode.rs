#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub local_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instruction {
    /// Push an integer onto the operand stack.
    PushInteger(i64),
    /// Pop a value from the operand stack into a local slot.
    StoreLocal(u32),
    /// Copy a local slot's value onto the operand stack.
    LoadLocal(u32),
}
