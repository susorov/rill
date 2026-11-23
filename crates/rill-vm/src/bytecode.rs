use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub local_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    /// Push an integer onto the operand stack.
    PushInteger(i64),
    /// Push a boolean onto the operand stack.
    PushBoolean(bool),
    /// Push a float onto the operand stack.
    PushFloat(f64),
    /// Push a string onto the operand stack.
    PushString(String),
    /// Pop a value from the operand stack into a local slot.
    StoreLocal(u32),
    /// Copy a local slot's value onto the operand stack.
    LoadLocal(u32),
    /// Pop and print a value followed by a newline.
    Print,
    /// Print an empty line.
    PrintNewline,
    /// Compare two values of the same type and push a boolean.
    Equal,
    /// Compare two same-type values for inequality.
    NotEqual,
    /// Compare two same-type numbers, with the left operand below the right on the stack.
    GreaterThan,
    /// Compare two same-type numbers, with the left operand below the right on the stack.
    GreaterThanOrEqual,
    /// Compare two same-type numbers, with the left operand below the right on the stack.
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
            Self::LessThanOrEqual => write!(f, "LESS_THAN_OR_EQUAL"),
            Self::Print => write!(f, "PRINT"),
            Self::PrintNewline => write!(f, "PRINT_NEWLINE"),
        }
    }
}
