use std::collections::HashMap;

use rill_syntax::ast::{BinaryOperator, Expression, ExpressionKind, Program, Statement};
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
                            "cannot compare {left_type:?} with {right_type:?}: operands must have the same type or both be numeric"
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

#[cfg(test)]
mod tests {
    use super::*;
    use Instruction::{LoadLocal, PushBoolean, PushFloat, PushInteger, PushString, StoreLocal};
    use rill_syntax::parser::parse;

    #[test]
    fn compiles_less_than_and_infers_boolean_result() {
        let code = compile(
            &parse("let a = 10\nlet copy = a\nlet result = copy < 5\nprint(result == true)")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            code.instructions,
            vec![
                PushInteger(10),
                StoreLocal(0),
                LoadLocal(0),
                StoreLocal(1),
                LoadLocal(1),
                PushInteger(5),
                Instruction::LessThan,
                StoreLocal(2),
                LoadLocal(2),
                PushBoolean(true),
                Instruction::Equal,
                Instruction::Print,
            ]
        );
        assert!(compile(&parse("print(1.5 < 2.5)").unwrap()).is_ok());
    }

    #[test]
    fn compiles_greater_than_and_infers_boolean_result() {
        let code = compile(
            &parse("let a = 10\nlet copy = a\nlet result = copy > 5\nprint(result == true)")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            code.instructions,
            vec![
                PushInteger(10),
                StoreLocal(0),
                LoadLocal(0),
                StoreLocal(1),
                LoadLocal(1),
                PushInteger(5),
                Instruction::GreaterThan,
                StoreLocal(2),
                LoadLocal(2),
                PushBoolean(true),
                Instruction::Equal,
                Instruction::Print,
            ]
        );
        assert!(compile(&parse("print(1.5 > 2.5)").unwrap()).is_ok());
    }

    #[test]
    fn compiles_numeric_ordering_and_infers_boolean_result() {
        let code = compile(
            &parse("let a = 10\nlet copy = a\nlet result = copy >= 5\nprint(result == true)")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            code.instructions,
            vec![
                PushInteger(10),
                StoreLocal(0),
                LoadLocal(0),
                StoreLocal(1),
                LoadLocal(1),
                PushInteger(5),
                Instruction::GreaterThanOrEqual,
                StoreLocal(2),
                LoadLocal(2),
                PushBoolean(true),
                Instruction::Equal,
                Instruction::Print,
            ]
        );
        assert!(compile(&parse("print(1.5 >= 2.5)").unwrap()).is_ok());
    }

    #[test]
    fn compiles_numeric_ordering_and_infers_boolean_result_less_than_or_equal() {
        let code = compile(
            &parse("let a = 10\nlet copy = a\nlet result = copy <= 5\nprint(result == true)")
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            code.instructions,
            vec![
                PushInteger(10),
                StoreLocal(0),
                LoadLocal(0),
                StoreLocal(1),
                LoadLocal(1),
                PushInteger(5),
                Instruction::LessThanOrEqual,
                StoreLocal(2),
                LoadLocal(2),
                PushBoolean(true),
                Instruction::Equal,
                Instruction::Print,
            ]
        );
        assert!(compile(&parse("print(1.5 <= 2.5)").unwrap()).is_ok());
    }

    #[test]
    fn compiles_mixed_numeric_comparisons() {
        for (operator, instruction) in [
            ("==", Instruction::Equal),
            ("!=", Instruction::NotEqual),
            ("<", Instruction::LessThan),
            (">", Instruction::GreaterThan),
            ("<=", Instruction::LessThanOrEqual),
            (">=", Instruction::GreaterThanOrEqual),
        ] {
            for (left, right, operands) in [
                ("1", "1.5", vec![PushInteger(1), PushFloat(1.5)]),
                ("1.5", "1", vec![PushFloat(1.5), PushInteger(1)]),
            ] {
                let source = format!("print({left} {operator} {right})");
                let code = compile(&parse(&source).unwrap()).unwrap();
                let mut expected = operands;
                expected.extend([instruction.clone(), Instruction::Print]);
                assert_eq!(code.instructions, expected, "{source}");

                let source = format!(
                    "let a = {left}\nlet copy = a\nlet b = {right}\nlet result = copy {operator} b\nprint(result == true)"
                );
                let code = compile(&parse(&source).unwrap()).unwrap();
                assert!(code.instructions.contains(&instruction), "{source}");
            }
        }
    }

    #[test]
    fn rejects_numeric_and_nonnumeric_comparisons_in_both_orders() {
        for operator in ["==", "!=", "<", ">", "<=", ">="] {
            for number in ["1", "1.0"] {
                for other in ["true", "\"1\""] {
                    for (left, right) in [(number, other), (other, number)] {
                        let expression = format!("{left} {operator} {right}");
                        let source = format!("print({expression})");
                        let error = compile(&parse(&source).unwrap()).unwrap_err();
                        assert_eq!(error.span, Span::new(6, 6 + expression.len()));
                        assert!(error.message.contains("same type or both be numeric"));
                    }
                }
            }
        }
    }

    #[test]
    fn rejects_invalid_less_than_operands_at_compile_time() {
        for expression in ["true < false", "\"a\" < \"b\"", "1 < true"] {
            let source = format!("print({expression})");
            let error = compile(&parse(&source).unwrap()).unwrap_err();
            assert_eq!(error.span, Span::new(6, 6 + expression.len()));
        }
        for source in [
            "let a = true\nlet copy = a\nprint(copy < false)",
            "let a = \"a\"\nprint(a < \"b\")",
            "let a = 1 < 2\nprint(a < false)",
        ] {
            let error = compile(&parse(source).unwrap()).unwrap_err();
            assert_eq!(error.message, "'<' requires integer or float operands");
        }
    }

    #[test]
    fn rejects_invalid_greater_than_operands_at_compile_time() {
        for expression in ["true > false", "\"a\" > \"b\"", "1 > true"] {
            let source = format!("print({expression})");
            let error = compile(&parse(&source).unwrap()).unwrap_err();
            assert_eq!(error.span, Span::new(6, 6 + expression.len()));
        }
        for source in [
            "let a = true\nlet copy = a\nprint(copy > false)",
            "let a = \"a\"\nprint(a > \"b\")",
            "let a = 1 > 2\nprint(a > false)",
        ] {
            let error = compile(&parse(source).unwrap()).unwrap_err();
            assert_eq!(error.message, "'>' requires integer or float operands");
        }
    }

    #[test]
    fn rejects_invalid_ordering_operands_at_compile_time() {
        for expression in ["true >= false", "\"a\" >= \"b\"", "1 >= true"] {
            let source = format!("print({expression})");
            let error = compile(&parse(&source).unwrap()).unwrap_err();
            assert_eq!(error.span, Span::new(6, 6 + expression.len()));
        }
        for source in [
            "let a = true\nlet copy = a\nprint(copy >= false)",
            "let a = \"a\"\nprint(a >= \"b\")",
            "let a = 1 >= 2\nprint(a >= false)",
        ] {
            let error = compile(&parse(source).unwrap()).unwrap_err();
            assert_eq!(error.message, "'>=' requires integer or float operands");
        }
    }

    #[test]
    fn rejects_invalid_ordering_operands_at_compile_time_less_than_or_equal() {
        for expression in ["true <= false", "\"a\" <= \"b\"", "1 <= true"] {
            let source = format!("print({expression})");
            let error = compile(&parse(&source).unwrap()).unwrap_err();
            assert_eq!(error.span, Span::new(6, 6 + expression.len()));
        }
        for source in [
            "let a = true\nlet copy = a\nprint(copy <= false)",
            "let a = \"a\"\nprint(a <= \"b\")",
            "let a = 1 <= 2\nprint(a <= false)",
        ] {
            let error = compile(&parse(source).unwrap()).unwrap_err();
            assert_eq!(error.message, "'<=' requires integer or float operands");
        }
    }

    #[test]
    fn infers_equality_types_through_variable_copies() {
        let source = "let a = 10
let copy = a
let b = copy == 10
let c = b
print(c == true)";
        let code = compile(&parse(source).unwrap()).unwrap();
        assert_eq!(
            code.instructions
                .iter()
                .filter(|i| **i == Instruction::Equal)
                .count(),
            2
        );
        assert_eq!(code.local_count, 4);
        for source in [
            "let a = 10
print(a == true)",
            "let b = 1 == 1
print(b == 1)",
        ] {
            let error = compile(&parse(source).unwrap()).unwrap_err();
            assert!(error.message.contains("same type"));
        }
    }

    #[test]
    fn compiles_float_literals_and_references() {
        for (literal, expected) in [
            ("1.25", 1.25_f64),
            ("-1.25", -1.25),
            ("42.0", 42.0),
            ("0.0", 0.0),
            ("-0.0", -0.0),
        ] {
            let source =
                format!("let value = {literal}\nlet copy = value\nprint(copy)\nprint(value)");
            let bytecode = compile(&parse(&source).unwrap()).unwrap();

            assert_eq!(
                bytecode,
                Bytecode {
                    instructions: vec![
                        PushFloat(expected),
                        StoreLocal(0),
                        LoadLocal(0),
                        StoreLocal(1),
                        LoadLocal(1),
                        Instruction::Print,
                        LoadLocal(0),
                        Instruction::Print,
                    ],
                    local_count: 2,
                }
            );
            let PushFloat(actual) = &bytecode.instructions[0] else {
                panic!("expected float instruction");
            };
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
    }

    #[test]
    fn compiles_string_literals_and_references() {
        for text in ["", "hello world", "// not a comment"] {
            let source =
                format!("let text = \"{text}\"\nlet copy = text\nprint(copy)\nprint(text)");

            assert_eq!(
                compile(&parse(&source).unwrap()).unwrap(),
                Bytecode {
                    instructions: vec![
                        PushString(text.to_owned()),
                        StoreLocal(0),
                        LoadLocal(0),
                        StoreLocal(1),
                        LoadLocal(1),
                        Instruction::Print,
                        LoadLocal(0),
                        Instruction::Print,
                    ],
                    local_count: 2,
                }
            );
        }
    }

    #[test]
    fn compiles_literal_and_empty_print_statements() {
        let source = "print(42)\nprint(-1.25)\nprint(true)\nprint(false)\nprint(\"hello\")\nprint(\"\")\nprint()";

        assert_eq!(
            compile(&parse(source).unwrap()).unwrap(),
            Bytecode {
                instructions: vec![
                    PushInteger(42),
                    Instruction::Print,
                    PushFloat(-1.25),
                    Instruction::Print,
                    PushBoolean(true),
                    Instruction::Print,
                    PushBoolean(false),
                    Instruction::Print,
                    PushString("hello".to_owned()),
                    Instruction::Print,
                    PushString(String::new()),
                    Instruction::Print,
                    Instruction::PrintNewline,
                ],
                local_count: 0,
            }
        );
    }

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
                span: Span::new(6, 13),
            }
        );
    }
}

#[test]
fn compiles_not_equal_and_checks_types() {
    use rill_syntax::parser::parse;
    let code = compile(
        &parse(
            "let a = 10
let b = a != 20
print(b != false)",
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        code.instructions
            .iter()
            .filter(|i| **i == Instruction::NotEqual)
            .count(),
        2
    );
    for source in [
        "print(10 != true)",
        "let b = 1 != 2
print(b != 1)",
    ] {
        assert!(
            compile(&parse(source).unwrap())
                .unwrap_err()
                .message
                .contains("same type")
        );
    }
}
