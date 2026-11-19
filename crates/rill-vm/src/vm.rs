use std::fmt;

use crate::bytecode::{Bytecode, Instruction};
use crate::value::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum VmError {
    InvalidLocal(u32),
    UninitializedLocal(u32),
    StackUnderflow,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLocal(slot) => write!(f, "invalid local slot {slot}"),
            Self::UninitializedLocal(slot) => {
                write!(f, "uninitialized local slot {slot}")
            }
            Self::StackUnderflow => write!(f, "operand stack underflow"),
        }
    }
}

impl std::error::Error for VmError {}

#[derive(Debug)]
pub struct Vm<'a> {
    bytecode: &'a Bytecode,
    instruction_pointer: usize,
    stack: Vec<Value>,
    locals: Vec<Option<Value>>,
}

impl<'a> Vm<'a> {
    pub fn new(bytecode: &'a Bytecode) -> Self {
        Self {
            bytecode,
            instruction_pointer: 0,
            stack: Vec::new(),
            locals: vec![None; bytecode.local_count as usize],
        }
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        while let Some(&instruction) = self.bytecode.instructions.get(self.instruction_pointer) {
            match instruction {
                Instruction::PushInteger(value) => {
                    self.stack.push(Value::Integer(value));
                }
                Instruction::PushBoolean(value) => {
                    self.stack.push(Value::Boolean(value));
                }
                Instruction::StoreLocal(slot) => {
                    let index = self.local_index(slot)?;
                    let value = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    self.locals[index] = Some(value);
                }
                Instruction::LoadLocal(slot) => {
                    let index = self.local_index(slot)?;
                    let value = self.locals[index]
                        .clone()
                        .ok_or(VmError::UninitializedLocal(slot))?;
                    self.stack.push(value);
                }
                Instruction::Print => {
                    let value = self.stack.pop().ok_or(VmError::StackUnderflow)?;

                    match value {
                        Value::Integer(value) => println!("{value}"),
                        Value::Boolean(value) => println!("{value}"),
                    }
                }
            }

            self.instruction_pointer += 1;
        }

        Ok(())
    }

    fn local_index(&self, slot: u32) -> Result<usize, VmError> {
        let index = slot as usize;

        if index >= self.locals.len() {
            return Err(VmError::InvalidLocal(slot));
        }

        Ok(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Instruction::{LoadLocal, PushBoolean, PushInteger, StoreLocal};

    #[test]
    fn executes_instructions_and_does_not_restart_after_completion() {
        let bytecode = Bytecode {
            instructions: vec![
                PushInteger(42),
                PushBoolean(true),
                StoreLocal(0),
                StoreLocal(1),
                PushBoolean(false),
                StoreLocal(2),
                LoadLocal(1),
                LoadLocal(0),
                LoadLocal(2),
            ],
            local_count: 3,
        };
        let mut vm = Vm::new(&bytecode);

        for _ in 0..2 {
            vm.run().unwrap();
            assert_eq!(
                vm.locals,
                vec![
                    Some(Value::Boolean(true)),
                    Some(Value::Integer(42)),
                    Some(Value::Boolean(false)),
                ]
            );
            assert_eq!(
                vm.stack,
                vec![
                    Value::Integer(42),
                    Value::Boolean(true),
                    Value::Boolean(false)
                ]
            );
            assert_eq!(vm.instruction_pointer, bytecode.instructions.len());
        }
    }

    #[test]
    fn accepts_empty_program() {
        let bytecode = Bytecode {
            instructions: vec![],
            local_count: 0,
        };
        let mut vm = Vm::new(&bytecode);

        assert_eq!(vm.run(), Ok(()));
        assert_eq!(vm.instruction_pointer, 0);
        assert!(vm.stack.is_empty());
        assert!(vm.locals.is_empty());
    }

    #[test]
    fn rejects_invalid_slots_without_consuming_stack_values() {
        for slot in [1, u32::MAX] {
            for instruction in [LoadLocal(slot), StoreLocal(slot)] {
                let bytecode = Bytecode {
                    instructions: vec![PushInteger(42), instruction],
                    local_count: 1,
                };
                let mut vm = Vm::new(&bytecode);

                assert_eq!(vm.run(), Err(VmError::InvalidLocal(slot)));
                assert_eq!(vm.instruction_pointer, 1);
                assert_eq!(vm.stack, vec![Value::Integer(42)]);
                assert_eq!(vm.locals, vec![None]);
            }
        }
    }

    #[test]
    fn rejects_uninitialized_reads_without_advancing() {
        let bytecode = Bytecode {
            instructions: vec![LoadLocal(0)],
            local_count: 1,
        };
        let mut vm = Vm::new(&bytecode);

        for _ in 0..2 {
            assert_eq!(vm.run(), Err(VmError::UninitializedLocal(0)));
            assert_eq!(vm.instruction_pointer, 0);
            assert!(vm.stack.is_empty());
            assert_eq!(vm.locals, vec![None]);
        }
    }

    #[test]
    fn stack_underflow_does_not_overwrite_local() {
        let bytecode = Bytecode {
            instructions: vec![PushInteger(42), StoreLocal(0), StoreLocal(0)],
            local_count: 1,
        };
        let mut vm = Vm::new(&bytecode);

        assert_eq!(vm.run(), Err(VmError::StackUnderflow));
        assert_eq!(vm.instruction_pointer, 2);
        assert!(vm.stack.is_empty());
        assert_eq!(vm.locals, vec![Some(Value::Integer(42))]);
    }

    #[test]
    fn print_consumes_stack_value() {
        let bytecode = Bytecode {
            instructions: vec![
                PushInteger(42),
                Instruction::Print,
                PushBoolean(true),
                Instruction::Print,
            ],
            local_count: 0,
        };

        let mut vm = Vm::new(&bytecode);

        assert_eq!(vm.run(), Ok(()));
        assert!(vm.stack.is_empty());
        assert_eq!(vm.instruction_pointer, 4);
    }

    #[test]
    fn print_rejects_stack_underflow() {
        let bytecode = Bytecode {
            instructions: vec![Instruction::Print],
            local_count: 0,
        };

        let mut vm = Vm::new(&bytecode);

        assert_eq!(vm.run(), Err(VmError::StackUnderflow));
        assert_eq!(vm.instruction_pointer, 0);
    }
}
