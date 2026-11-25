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
            Self::TypeMismatch => write!(f, "invalid operand types for comparison"),
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

    /// Return the current operand stack, from bottom to top.
    pub fn stack(&self) -> &[Value] {
        &self.stack
    }

    /// Return the current local slots, including uninitialized slots.
    pub fn locals(&self) -> &[Option<Value>] {
        &self.locals
    }

    /// Return the index of the next instruction to execute.
    pub fn instruction_pointer(&self) -> usize {
        self.instruction_pointer
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
                Instruction::LessThan => {
                    self.binary(|left, right| Ok(Value::Boolean(less_than(left, right)?)))?;
                }
                Instruction::GreaterThan => {
                    self.binary(|left, right| Ok(Value::Boolean(greater_than(left, right)?)))?;
                }
                Instruction::GreaterThanOrEqual => {
                    self.binary(|left, right| {
                        Ok(Value::Boolean(greater_than_or_equal(left, right)?))
                    })?;
                }
                Instruction::LessThanOrEqual => {
                    self.binary(|left, right| {
                        Ok(Value::Boolean(less_than_or_equal(left, right)?))
                    })?;
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
        (Value::Float(a), Value::Integer(b)) => Ok(*a == *b as f64),
        (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) == *b),
        (Value::Boolean(a), Value::Boolean(b)) => Ok(a == b),
        (Value::String(a), Value::String(b)) => Ok(a == b),
        _ => Err(VmError::TypeMismatch),
    }
}

fn less_than(left: &Value, right: &Value) -> Result<bool, VmError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(a < b),
        (Value::Float(a), Value::Float(b)) => Ok(a < b),
        (Value::Float(a), Value::Integer(b)) => Ok(*a < *b as f64),
        (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) < *b),
        _ => Err(VmError::TypeMismatch),
    }
}

fn greater_than(left: &Value, right: &Value) -> Result<bool, VmError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(a > b),
        (Value::Float(a), Value::Float(b)) => Ok(a > b),
        (Value::Float(a), Value::Integer(b)) => Ok(*a > *b as f64),
        (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) > *b),
        _ => Err(VmError::TypeMismatch),
    }
}

fn greater_than_or_equal(left: &Value, right: &Value) -> Result<bool, VmError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(a >= b),
        (Value::Float(a), Value::Float(b)) => Ok(a >= b),
        (Value::Float(a), Value::Integer(b)) => Ok(*a >= *b as f64),
        (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) >= *b),
        _ => Err(VmError::TypeMismatch),
    }
}

fn less_than_or_equal(left: &Value, right: &Value) -> Result<bool, VmError> {
    match (left, right) {
        (Value::Integer(a), Value::Integer(b)) => Ok(a <= b),
        (Value::Float(a), Value::Float(b)) => Ok(a <= b),
        (Value::Float(a), Value::Integer(b)) => Ok(*a <= *b as f64),
        (Value::Integer(a), Value::Float(b)) => Ok((*a as f64) <= *b),
        _ => Err(VmError::TypeMismatch),
    }
}
