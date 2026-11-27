//! Compiles syntax trees into bytecode.
//!
//! Statements are compiled in source order. Local slots are numbered from
//! zero and each declaration becomes visible after its initializer.
//! Equality accepts operands of the same type or two numeric operands.
//! Compilation stops at the first error and discards partial bytecode.

use std::collections::HashMap;

use rill_syntax::ast::{BinaryOperator, Expression, ExpressionKind, Program, Statement};
use rill_vm::bytecode::{Bytecode, Instruction};

use crate::error::CompileError;

/// Compile a syntax tree into owned bytecode, resolving names and checking types.
///
/// Variables must be declared before use and cannot be redeclared.
/// Each call starts with empty locals and preserves statement order.
///
/// # Errors
///
/// Returns the first [`CompileError`] for undefined or duplicate variables,
/// invalid comparisons or exhausted local slots. Partial bytecode is discarded.
pub fn compile(program: &Program) -> Result<Bytecode, CompileError> {
    let mut compiler = Compiler {
        locals: HashMap::new(),
        bytecode: Bytecode {
            instructions: Vec::new(),
            local_count: 0,
        },
    };

    for statement in &program.statements {
        compiler.compile_statement(statement)?;
    }

    Ok(compiler.bytecode)
}

/// Types used by the compiler to check expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExpressionType {
    Integer,
    Float,
    Boolean,
    String,
}

struct Local {
    slot: u32,
    value_type: ExpressionType,
}

struct Compiler {
    locals: HashMap<String, Local>,
    bytecode: Bytecode,
}

impl Compiler {
    fn compile_statement(&mut self, statement: &Statement) -> Result<(), CompileError> {
        match statement {
            Statement::Let { name, value, span } => {
                if self.locals.contains_key(name) {
                    return Err(CompileError {
                        message: format!("variable '{name}' is already declared"),
                        span: *span,
                    });
                }

                let value_type = self.compile_expression(value)?;

                let slot = self.bytecode.local_count;

                self.bytecode.local_count = slot.checked_add(1).ok_or_else(|| CompileError {
                    message: "too many local variables".to_owned(),
                    span: *span,
                })?;

                self.bytecode
                    .instructions
                    .push(Instruction::StoreLocal(slot));

                self.locals.insert(name.clone(), Local { slot, value_type });
            }

            Statement::Print { value, .. } => {
                if let Some(value) = value {
                    self.compile_expression(value)?;
                    self.bytecode.instructions.push(Instruction::Print);
                } else {
                    self.bytecode.instructions.push(Instruction::PrintNewline);
                }
            }
        }

        Ok(())
    }

    fn compile_expression(
        &mut self,
        expression: &Expression,
    ) -> Result<ExpressionType, CompileError> {
        let (instruction, value_type) = match &expression.kind {
            ExpressionKind::Integer(value) => {
                (Instruction::PushInteger(*value), ExpressionType::Integer)
            }

            ExpressionKind::Boolean(value) => {
                (Instruction::PushBoolean(*value), ExpressionType::Boolean)
            }

            ExpressionKind::Float(value) => (Instruction::PushFloat(*value), ExpressionType::Float),

            ExpressionKind::String(value) => (
                Instruction::PushString(value.clone()),
                ExpressionType::String,
            ),

            ExpressionKind::Identifier(name) => {
                let local = self.locals.get(name).ok_or_else(|| CompileError {
                    message: format!("undefined variable '{name}'"),
                    span: expression.span,
                })?;

                (Instruction::LoadLocal(local.slot), local.value_type)
            }

            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                let instruction = match operator {
                    BinaryOperator::Equal => Instruction::Equal,
                    BinaryOperator::NotEqual => Instruction::NotEqual,
                    BinaryOperator::GreaterThan => Instruction::GreaterThan,
                    BinaryOperator::GreaterThanOrEqual => Instruction::GreaterThanOrEqual,
                    BinaryOperator::LessThan => Instruction::LessThan,
                    BinaryOperator::LessThanOrEqual => Instruction::LessThanOrEqual,
                };

                let left_type = self.compile_expression(left)?;
                let right_type = self.compile_expression(right)?;

                let is_numeric = [left_type, right_type].iter().all(|operand_type| {
                    matches!(
                        operand_type,
                        ExpressionType::Integer | ExpressionType::Float
                    )
                });

                if left_type != right_type && !is_numeric {
                    return Err(CompileError {
                        message: format!(
                            "cannot compare {left_type:?} with {right_type:?}: \
                             operands must have the same type or both be numeric"
                        ),
                        span: expression.span,
                    });
                }

                if *operator == BinaryOperator::LessThan && !is_numeric {
                    return Err(CompileError {
                        message: "'<' requires integer or float operands".to_owned(),
                        span: expression.span,
                    });
                }

                if *operator == BinaryOperator::GreaterThan && !is_numeric {
                    return Err(CompileError {
                        message: "'>' requires integer or float operands".to_owned(),
                        span: expression.span,
                    });
                }

                if *operator == BinaryOperator::GreaterThanOrEqual && !is_numeric {
                    return Err(CompileError {
                        message: "'>=' requires integer or float operands".to_owned(),
                        span: expression.span,
                    });
                }

                if *operator == BinaryOperator::LessThanOrEqual && !is_numeric {
                    return Err(CompileError {
                        message: "'<=' requires integer or float operands".to_owned(),
                        span: expression.span,
                    });
                }

                (instruction, ExpressionType::Boolean)
            }
        };

        self.bytecode.instructions.push(instruction);

        Ok(value_type)
    }
}
