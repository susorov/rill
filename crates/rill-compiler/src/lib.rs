use std::collections::HashMap;

use rill_syntax::ast::{Expression, ExpressionKind, Program, Statement};
use rill_syntax::span::Span;
use rill_vm::bytecode::{Bytecode, Instruction};

#[derive(Debug, PartialEq, Eq)]
pub struct CompileError {
    pub message: String,
    pub span: Span,
}

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

struct Compiler {
    locals: HashMap<String, u32>,
    bytecode: Bytecode,
}

impl Compiler {
    fn compile_statement(&mut self, statement: &Statement) -> Result<(), CompileError> {
        match statement {
            Statement::Let { name, value, span } => {
                self.compile_expression(value)?;

                let slot = self.bytecode.local_count;
                self.bytecode.local_count = slot.checked_add(1).ok_or_else(|| CompileError {
                    message: "too many local variables".to_owned(),
                    span: span.clone(),
                })?;
                self.bytecode
                    .instructions
                    .push(Instruction::StoreLocal(slot));
                self.locals.insert(name.clone(), slot);
            }
        }

        Ok(())
    }

    fn compile_expression(&mut self, expression: &Expression) -> Result<(), CompileError> {
        let instruction = match &expression.kind {
            ExpressionKind::Integer(value) => Instruction::PushInteger(*value),
            ExpressionKind::Identifier(name) => {
                let slot = self.locals.get(name).ok_or_else(|| CompileError {
                    message: format!("undefined variable '{name}'"),
                    span: expression.span.clone(),
                })?;
                Instruction::LoadLocal(*slot)
            }
        };

        self.bytecode.instructions.push(instruction);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Instruction::{LoadLocal, PushInteger, StoreLocal};
    use rill_syntax::parser::parse;

    #[test]
    fn compiles_empty_program() {
        assert_eq!(
            compile(&parse("").unwrap()).unwrap(),
            Bytecode {
                instructions: vec![],
                local_count: 0,
            }
        );
    }

    #[test]
    fn compiles_integers_and_chained_references() {
        let program = parse("let count = 42\nlet limit = count\nlet maximum = limit").unwrap();

        assert_eq!(
            compile(&program).unwrap(),
            Bytecode {
                instructions: vec![
                    PushInteger(42),
                    StoreLocal(0),
                    LoadLocal(0),
                    StoreLocal(1),
                    LoadLocal(1),
                    StoreLocal(2),
                ],
                local_count: 3,
            }
        );
    }

    #[test]
    fn rejects_unknown_self_and_forward_references() {
        for (source, name, end) in [
            ("let x = missing", "missing", 15),
            ("let x = x", "x", 9),
            ("let x = y\nlet y = 42", "y", 9),
        ] {
            assert_eq!(
                compile(&parse(source).unwrap()).unwrap_err(),
                CompileError {
                    message: format!("undefined variable '{name}'"),
                    span: Span::new(8, end),
                }
            );
        }
    }

    #[test]
    fn reports_reference_span_in_later_statement() {
        let error = compile(&parse("let x = 42\nlet y = missing").unwrap()).unwrap_err();

        assert_eq!(error.message, "undefined variable 'missing'");
        assert_eq!(error.span, Span::new(19, 26));
    }

    #[test]
    fn shadowing_reads_previous_binding_and_allocates_new_slot() {
        let program = parse("let x = 42\nlet x = x\nlet y = x").unwrap();

        assert_eq!(
            compile(&program).unwrap(),
            Bytecode {
                instructions: vec![
                    PushInteger(42),
                    StoreLocal(0),
                    LoadLocal(0),
                    StoreLocal(1),
                    LoadLocal(1),
                    StoreLocal(2),
                ],
                local_count: 3,
            }
        );
    }

    #[test]
    fn compilation_does_not_share_bindings_between_programs() {
        compile(&parse("let x = 42").unwrap()).unwrap();
        assert!(compile(&parse("let y = x").unwrap()).is_err());
    }
}
