use std::fmt;

use crate::bytecode::{Bytecode, Instruction};
use crate::value::Value;

#[derive(Debug, PartialEq, Eq)]
pub enum VmError {
    InvalidLocal(u32),
    UninitializedLocal(u32),
    StackUnderflow,
    TypeMismatch,
}

impl fmt::Display for VmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLocal(slot) => write!(f, "invalid local slot {slot}"),
            Self::UninitializedLocal(slot) => {
                write!(f, "uninitialized local slot {slot}")
            }
            Self::TypeMismatch => write!(f, "equality operands must have the same type"),
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
        while let Some(instruction) = self.bytecode.instructions.get(self.instruction_pointer) {
            match instruction {
                Instruction::PushInteger(value) => {
                    self.stack.push(Value::Integer(*value));
                }
                Instruction::PushFloat(value) => {
                    self.stack.push(Value::Float(*value));
                }
                Instruction::PushBoolean(value) => {
                    self.stack.push(Value::Boolean(*value));
                }
                Instruction::PushString(value) => {
                    self.stack.push(Value::String(value.clone()));
                }
                Instruction::StoreLocal(slot) => {
                    let index = self.local_index(*slot)?;
                    let value = self.stack.pop().ok_or(VmError::StackUnderflow)?;
                    self.locals[index] = Some(value);
                }
                Instruction::LoadLocal(slot) => {
                    let index = self.local_index(*slot)?;
                    let value = self.locals[index]
                        .clone()
                        .ok_or(VmError::UninitializedLocal(*slot))?;
                    self.stack.push(value);
                }
                Instruction::Equal => {
                    self.binary(|left, right| Ok(Value::Boolean(equal(left, right)?)))?;
                }
                Instruction::NotEqual => {
                    self.binary(|left, right| Ok(Value::Boolean(!equal(left, right)?)))?;
                }
                Instruction::Print => {
                    let value = self.stack.pop().ok_or(VmError::StackUnderflow)?;

                    match value {
                        Value::Integer(value) => println!("{value}"),
                        Value::Float(value) => println!("{value}"),
                        Value::Boolean(value) => println!("{value}"),
                        Value::String(value) => println!("{value}"),
                    }
                }
                Instruction::PrintNewline => {
                    println!();
                }
            }

            self.instruction_pointer += 1;
        }

        Ok(())
    }

    fn binary(
        &mut self,
        operation: impl FnOnce(&Value, &Value) -> Result<Value, VmError>,
    ) -> Result<(), VmError> {
        let len = self.stack.len();
        if len < 2 {
            return Err(VmError::StackUnderflow);
        }

        let result = operation(&self.stack[len - 2], &self.stack[len - 1])?;

        self.stack.truncate(len - 2);
        self.stack.push(result);
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

fn equal(left: &Value, right: &Value) -> Result<bool, VmError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(a == b),
        (Value::Float(a), Value::Float(b)) => Ok(a == b),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(a == b),
        (Value::String(a), Value::String(b)) => Ok(a == b),
        _ => Err(VmError::TypeMismatch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Instruction::{LoadLocal, PushBoolean, PushFloat, PushInteger, PushString, StoreLocal};

    #[test]
    fn equality_compares_all_value_types() {
        for (left, right, expected) in [
            (PushInteger(10), PushInteger(10), true),
            (PushInteger(10), PushInteger(20), false),
            (PushFloat(0.0), PushFloat(-0.0), true),
            (PushFloat(1.0), PushFloat(2.0), false),
            (PushBoolean(true), PushBoolean(false), false),
            (PushString("hello".into()), PushString("hello".into()), true),
            (
                PushString("hello".into()),
                PushString("world".into()),
                false,
            ),
        ] {
            for (operator, result) in [
                (Instruction::Equal, expected),
                (Instruction::NotEqual, !expected),
            ] {
                let code = Bytecode {
                    instructions: vec![left.clone(), right.clone(), operator],
                    local_count: 0,
                };
                let mut vm = Vm::new(&code);
                vm.run().unwrap();
                assert_eq!(vm.stack, vec![Value::Boolean(result)]);
            }
        }
    }

    #[test]
    fn equality_errors_preserve_operands() {
        for (instructions, expected) in [
            (vec![Instruction::NotEqual], VmError::StackUnderflow),
            (
                vec![PushInteger(1), Instruction::NotEqual],
                VmError::StackUnderflow,
            ),
            (
                vec![PushInteger(1), PushBoolean(true), Instruction::NotEqual],
                VmError::TypeMismatch,
            ),
            (vec![Instruction::Equal], VmError::StackUnderflow),
            (
                vec![PushInteger(1), Instruction::Equal],
                VmError::StackUnderflow,
            ),
            (
                vec![PushInteger(1), PushBoolean(true), Instruction::Equal],
                VmError::TypeMismatch,
            ),
        ] {
            let count = instructions.len() - 1;
            let code = Bytecode {
                instructions,
                local_count: 0,
            };
            let mut vm = Vm::new(&code);
            assert_eq!(vm.run(), Err(expected));
            assert_eq!(vm.stack.len(), count);
            assert_eq!(vm.instruction_pointer, count);
        }
    }

    #[test]
    fn preserves_float_values_through_locals_and_print() {
        for expected in [
            1.25,
            -1.25,
            0.0,
            -0.0,
            f64::MIN_POSITIVE,
            f64::from_bits(1),
            f64::MAX,
        ] {
            let bytecode = Bytecode {
                instructions: vec![
                    PushFloat(expected),
                    StoreLocal(0),
                    LoadLocal(0),
                    StoreLocal(1),
                    LoadLocal(0),
                    LoadLocal(1),
                    Instruction::Print,
                ],
                local_count: 2,
            };
            let mut vm = Vm::new(&bytecode);

            vm.run().unwrap();

            for local in &vm.locals {
                let Some(Value::Float(actual)) = local else {
                    panic!("expected float local");
                };
                assert_eq!(actual.to_bits(), expected.to_bits());
            }
            let [Value::Float(actual)] = vm.stack.as_slice() else {
                panic!("expected one float after printing the other");
            };
            assert_eq!(actual.to_bits(), expected.to_bits());
            assert_eq!(vm.instruction_pointer, bytecode.instructions.len());
        }
    }

    #[test]
    fn stores_and_copies_string_values() {
        let bytecode = Bytecode {
            instructions: vec![
                PushString("hello".to_owned()),
                StoreLocal(0),
                LoadLocal(0),
                StoreLocal(1),
                PushString("changed".to_owned()),
                StoreLocal(0),
                LoadLocal(1),
                LoadLocal(0),
            ],
            local_count: 2,
        };
        let mut vm = Vm::new(&bytecode);

        vm.run().unwrap();

        assert_eq!(
            vm.locals,
            vec![
                Some(Value::String("changed".to_owned())),
                Some(Value::String("hello".to_owned())),
            ]
        );
        assert_eq!(
            vm.stack,
            vec![
                Value::String("hello".to_owned()),
                Value::String("changed".to_owned()),
            ]
        );
    }

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
                PushString("hello".to_owned()),
                Instruction::Print,
                PushString(String::new()),
                Instruction::Print,
                Instruction::PrintNewline,
            ],
            local_count: 0,
        };

        let mut vm = Vm::new(&bytecode);

        assert_eq!(vm.run(), Ok(()));
        assert!(vm.stack.is_empty());
        assert_eq!(vm.instruction_pointer, bytecode.instructions.len());
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
