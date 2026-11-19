use crate::value::Value;

#[derive(Debug, Default)]
pub struct Vm {
    instruction_pointer: usize,
    stack: Vec<Value>,
    // None represents a slot that has not been initialized.
    locals: Vec<Option<Value>>,
}

impl Vm {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn instruction_pointer(&self) -> usize {
        self.instruction_pointer
    }

    pub fn stack(&self) -> &[Value] {
        &self.stack
    }

    pub fn locals(&self) -> &[Option<Value>] {
        &self.locals
    }
}
