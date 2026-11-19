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
                if self.locals.contains_key(name) {
                    return Err(CompileError {
                        message: format!("variable '{name}' is already declared"),
                        span: span.clone(),
                    });
                }
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
            Statement::Print { name, span } => {
                let slot = self.locals.get(name).ok_or_else(|| CompileError {
                    message: format!("undefined variable '{name}'"),
                    span: span.clone(),
                })?;

                self.bytecode
                    .instructions
                    .push(Instruction::LoadLocal(*slot));

                self.bytecode.instructions.push(Instruction::Print);
            }
        }

        Ok(())
    }

    fn compile_expression(&mut self, expression: &Expression) -> Result<(), CompileError> {
        let instruction = match &expression.kind {
            ExpressionKind::Integer(value) => Instruction::PushInteger(*value),
            ExpressionKind::Boolean(value) => Instruction::PushBoolean(*value),
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
    use Instruction::{LoadLocal, PushBoolean, PushInteger, StoreLocal};
    use rill_syntax::parser::parse;

    #[test]
    fn compiles_boolean_literals_and_references() {
        for (literal, value) in [("true", true), ("false", false)] {
            let source = format!("let enabled = {literal}\nlet copy = enabled\nlet count = 42");

            assert_eq!(
                compile(&parse(&source).unwrap()).unwrap(),
                Bytecode {
                    instructions: vec![
                        PushBoolean(value),
                        StoreLocal(0),
                        LoadLocal(0),
                        StoreLocal(1),
                        PushInteger(42),
                        StoreLocal(2),
                    ],
                    local_count: 3,
                }
            );
        }
    }

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
    fn rejects_duplicate_declarations() {
        for (source, start, end) in [
            ("let x = 42\nlet x = 10", 11, 21),
            ("let x = 42\nlet x = x", 11, 20),
            ("let x = 42\nlet y = x\nlet x = 10", 21, 31),
        ] {
            assert_eq!(
                compile(&parse(source).unwrap()).unwrap_err(),
                CompileError {
                    message: "variable 'x' is already declared".to_owned(),
                    span: Span::new(start, end),
                },
                "source: {source:?}"
            );
        }
    }

    #[test]
    fn rejects_duplicate_before_resolving_initializer() {
        let program = parse("let x = 42\nlet x = missing").unwrap();

        assert_eq!(
            compile(&program).unwrap_err(),
            CompileError {
                message: "variable 'x' is already declared".to_owned(),
                span: Span::new(11, 26),
            }
        );
    }

    #[test]
    fn compilation_does_not_share_bindings_between_programs() {
        compile(&parse("let x = 42").unwrap()).unwrap();
        assert!(compile(&parse("let y = x").unwrap()).is_err());
    }

    #[test]
    fn compiles_print_statement() {
        let program = parse("let count = 42\nprint(count)").unwrap();

        assert_eq!(
            compile(&program).unwrap(),
            Bytecode {
                instructions: vec![
                    PushInteger(42),
                    StoreLocal(0),
                    LoadLocal(0),
                    Instruction::Print,
                ],
                local_count: 1,
            }
        );
    }

    #[test]
    fn rejects_printing_undefined_variable() {
        let program = parse("print(missing)").unwrap();

        assert_eq!(
            compile(&program).unwrap_err(),
            CompileError {
                message: "undefined variable 'missing'".to_owned(),
                span: Span::new(0, 14),
            }
        );
    }
}
