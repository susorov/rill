//! Tests for virtual machine execution and runtime invariants.

use rill_vm::bytecode::Instruction::{
    LoadLocal, PushBoolean, PushFloat, PushInteger, PushString, StoreLocal,
};
use rill_vm::bytecode::{Bytecode, Instruction};
use rill_vm::value::Value;
use rill_vm::vm::{Vm, VmError};

#[test]
fn mixed_numeric_comparisons_promote_integers_to_floats() {
    // Expected results are ordered as ==, !=, <, >, <=, >=.
    let equal = [true, false, false, false, true, true];
    let less = [false, true, true, false, true, false];
    let greater = [false, true, false, true, false, true];
    let unordered = [false, true, false, false, false, false];
    for (left, right, expected) in [
        (PushInteger(1), PushFloat(1.0), equal),
        (PushFloat(1.0), PushInteger(1), equal),
        (PushInteger(1), PushFloat(1.5), less),
        (PushFloat(1.5), PushInteger(1), greater),
        (PushInteger(2), PushFloat(1.5), greater),
        (PushFloat(1.5), PushInteger(2), less),
        (PushInteger(-2), PushFloat(-1.5), less),
        (PushFloat(-1.5), PushInteger(-2), greater),
        (PushInteger(0), PushFloat(-0.0), equal),
        (PushFloat(-0.0), PushInteger(0), equal),
        (PushInteger(1), PushFloat(f64::NAN), unordered),
        (PushFloat(f64::NAN), PushInteger(1), unordered),
        (PushInteger(i64::MAX), PushFloat(f64::INFINITY), less),
        (PushFloat(f64::NEG_INFINITY), PushInteger(i64::MIN), less),
        // Promotion rounds integers that f64 cannot represent exactly.
        (
            PushInteger(9_007_199_254_740_993),
            PushFloat(9_007_199_254_740_992.0),
            equal,
        ),
        (
            PushFloat(9_007_199_254_740_992.0),
            PushInteger(9_007_199_254_740_993),
            equal,
        ),
        (
            PushInteger(i64::MAX),
            PushFloat(9_223_372_036_854_775_808.0),
            equal,
        ),
        (
            PushFloat(-9_223_372_036_854_775_808.0),
            PushInteger(i64::MIN),
            equal,
        ),
        // Integer-only comparisons must retain all 64 bits of precision.
        (
            PushInteger(9_007_199_254_740_993),
            PushInteger(9_007_199_254_740_992),
            greater,
        ),
        (PushInteger(i64::MAX - 1), PushInteger(i64::MAX), less),
    ] {
        for (instruction, expected) in [
            Instruction::Equal,
            Instruction::NotEqual,
            Instruction::LessThan,
            Instruction::GreaterThan,
            Instruction::LessThanOrEqual,
            Instruction::GreaterThanOrEqual,
        ]
        .into_iter()
        .zip(expected)
        {
            let code = Bytecode {
                instructions: vec![left.clone(), right.clone(), instruction],
                local_count: 0,
            };
            let mut vm = Vm::new(&code);
            vm.run().unwrap();
            assert_eq!(vm.stack(), &[Value::Boolean(expected)], "{code:?}");
        }
    }
}

#[test]
fn less_than_compares_numbers_in_operand_order() {
    for (left, right, expected) in [
        (PushInteger(2), PushInteger(1), true),
        (PushInteger(1), PushInteger(1), false),
        (PushInteger(1), PushInteger(2), false),
        (PushInteger(-2), PushInteger(-1), false),
        (PushInteger(i64::MAX), PushInteger(i64::MIN), true),
        (PushFloat(2.5), PushFloat(1.5), true),
        (PushFloat(1.5), PushFloat(2.5), false),
        (PushFloat(1.5), PushFloat(1.5), false),
        (PushFloat(0.0), PushFloat(-0.0), false),
        (PushFloat(f64::NAN), PushFloat(1.0), false),
    ] {
        let code = Bytecode {
            instructions: vec![right, left, Instruction::LessThan],
            local_count: 0,
        };
        let mut vm = Vm::new(&code);
        vm.run().unwrap();
        assert_eq!(vm.stack(), &[Value::Boolean(expected)]);
    }
    assert_eq!(Instruction::LessThan.to_string(), "LESS_THAN");
}

#[test]
fn greater_than_compares_numbers_in_operand_order() {
    for (left, right, expected) in [
        (PushInteger(2), PushInteger(1), true),
        (PushInteger(1), PushInteger(1), false),
        (PushInteger(1), PushInteger(2), false),
        (PushInteger(-2), PushInteger(-1), false),
        (PushInteger(i64::MAX), PushInteger(i64::MIN), true),
        (PushFloat(2.5), PushFloat(1.5), true),
        (PushFloat(1.5), PushFloat(2.5), false),
        (PushFloat(1.5), PushFloat(1.5), false),
        (PushFloat(0.0), PushFloat(-0.0), false),
        (PushFloat(f64::NAN), PushFloat(1.0), false),
    ] {
        let code = Bytecode {
            instructions: vec![left, right, Instruction::GreaterThan],
            local_count: 0,
        };
        let mut vm = Vm::new(&code);
        vm.run().unwrap();
        assert_eq!(vm.stack(), &[Value::Boolean(expected)]);
    }
    assert_eq!(Instruction::GreaterThan.to_string(), "GREATER_THAN");
}

#[test]
fn greater_than_or_equal_compares_numbers_in_operand_order() {
    for (left, right, expected) in [
        (PushInteger(2), PushInteger(1), true),
        (PushInteger(1), PushInteger(1), true),
        (PushInteger(1), PushInteger(2), false),
        (PushInteger(-2), PushInteger(-1), false),
        (PushInteger(i64::MAX), PushInteger(i64::MIN), true),
        (PushFloat(2.5), PushFloat(1.5), true),
        (PushFloat(1.5), PushFloat(2.5), false),
        (PushFloat(1.5), PushFloat(1.5), true),
        (PushFloat(0.0), PushFloat(-0.0), true),
        (PushFloat(f64::NAN), PushFloat(1.0), false),
    ] {
        let code = Bytecode {
            instructions: vec![left, right, Instruction::GreaterThanOrEqual],
            local_count: 0,
        };
        let mut vm = Vm::new(&code);
        vm.run().unwrap();
        assert_eq!(vm.stack(), &[Value::Boolean(expected)]);
    }
    assert_eq!(
        Instruction::GreaterThanOrEqual.to_string(),
        "GREATER_THAN_OR_EQUAL"
    );
}

#[test]
fn less_than_or_equal_compares_numbers_in_operand_order() {
    for (left, right, expected) in [
        (PushInteger(2), PushInteger(1), true),
        (PushInteger(1), PushInteger(1), true),
        (PushInteger(1), PushInteger(2), false),
        (PushInteger(-2), PushInteger(-1), false),
        (PushInteger(i64::MAX), PushInteger(i64::MIN), true),
        (PushFloat(2.5), PushFloat(1.5), true),
        (PushFloat(1.5), PushFloat(2.5), false),
        (PushFloat(1.5), PushFloat(1.5), true),
        (PushFloat(0.0), PushFloat(-0.0), true),
        (PushFloat(f64::NAN), PushFloat(1.0), false),
    ] {
        let code = Bytecode {
            instructions: vec![right, left, Instruction::LessThanOrEqual],
            local_count: 0,
        };
        let mut vm = Vm::new(&code);
        vm.run().unwrap();
        assert_eq!(vm.stack(), &[Value::Boolean(expected)]);
    }
    assert_eq!(
        Instruction::LessThanOrEqual.to_string(),
        "LESS_THAN_OR_EQUAL"
    );
}

#[test]
fn less_than_errors_preserve_operands_and_instruction_pointer() {
    for (operands, expected) in [
        (vec![], VmError::StackUnderflow),
        (vec![PushInteger(1)], VmError::StackUnderflow),
        (
            vec![PushInteger(1), PushBoolean(true)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushBoolean(true), PushFloat(1.0)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushBoolean(true), PushBoolean(false)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushString("a".into()), PushString("b".into())],
            VmError::TypeMismatch,
        ),
    ] {
        let valid = Bytecode {
            instructions: operands.clone(),
            local_count: 0,
        };
        let mut baseline = Vm::new(&valid);
        baseline.run().unwrap();
        let mut instructions = operands;
        instructions.push(Instruction::LessThan);
        let code = Bytecode {
            instructions,
            local_count: 0,
        };
        let mut vm = Vm::new(&code);
        assert_eq!(vm.run(), Err(expected));
        assert_eq!(vm.stack(), baseline.stack());
        assert_eq!(vm.instruction_pointer(), baseline.instruction_pointer());
    }
}

#[test]
fn greater_than_errors_preserve_operands_and_instruction_pointer() {
    for (operands, expected) in [
        (vec![], VmError::StackUnderflow),
        (vec![PushInteger(1)], VmError::StackUnderflow),
        (
            vec![PushInteger(1), PushBoolean(true)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushBoolean(true), PushFloat(1.0)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushBoolean(true), PushBoolean(false)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushString("a".into()), PushString("b".into())],
            VmError::TypeMismatch,
        ),
    ] {
        let valid = Bytecode {
            instructions: operands.clone(),
            local_count: 0,
        };
        let mut baseline = Vm::new(&valid);
        baseline.run().unwrap();
        let mut instructions = operands;
        instructions.push(Instruction::GreaterThan);
        let code = Bytecode {
            instructions,
            local_count: 0,
        };
        let mut vm = Vm::new(&code);
        assert_eq!(vm.run(), Err(expected));
        assert_eq!(vm.stack(), baseline.stack());
        assert_eq!(vm.instruction_pointer(), baseline.instruction_pointer());
    }
}

#[test]
fn ordering_errors_preserve_operands_and_instruction_pointer() {
    for (operands, expected) in [
        (vec![], VmError::StackUnderflow),
        (vec![PushInteger(1)], VmError::StackUnderflow),
        (
            vec![PushInteger(1), PushBoolean(true)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushBoolean(true), PushFloat(1.0)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushBoolean(true), PushBoolean(false)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushString("a".into()), PushString("b".into())],
            VmError::TypeMismatch,
        ),
    ] {
        let valid = Bytecode {
            instructions: operands.clone(),
            local_count: 0,
        };
        let mut baseline = Vm::new(&valid);
        baseline.run().unwrap();
        let mut instructions = operands;
        instructions.push(Instruction::GreaterThanOrEqual);
        let code = Bytecode {
            instructions,
            local_count: 0,
        };
        let mut vm = Vm::new(&code);
        assert_eq!(vm.run(), Err(expected));
        assert_eq!(vm.stack(), baseline.stack());
        assert_eq!(vm.instruction_pointer(), baseline.instruction_pointer());
    }
}

#[test]
fn less_than_or_equal_errors_preserve_operands_and_instruction_pointer() {
    for (operands, expected) in [
        (vec![], VmError::StackUnderflow),
        (vec![PushInteger(1)], VmError::StackUnderflow),
        (
            vec![PushInteger(1), PushBoolean(true)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushBoolean(true), PushFloat(1.0)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushBoolean(true), PushBoolean(false)],
            VmError::TypeMismatch,
        ),
        (
            vec![PushString("a".into()), PushString("b".into())],
            VmError::TypeMismatch,
        ),
    ] {
        let valid = Bytecode {
            instructions: operands.clone(),
            local_count: 0,
        };
        let mut baseline = Vm::new(&valid);
        baseline.run().unwrap();
        let mut instructions = operands;
        instructions.push(Instruction::LessThanOrEqual);
        let code = Bytecode {
            instructions,
            local_count: 0,
        };
        let mut vm = Vm::new(&code);
        assert_eq!(vm.run(), Err(expected));
        assert_eq!(vm.stack(), baseline.stack());
        assert_eq!(vm.instruction_pointer(), baseline.instruction_pointer());
    }
}

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
            assert_eq!(vm.stack(), &[Value::Boolean(result)]);
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
        assert_eq!(vm.stack().len(), count);
        assert_eq!(vm.instruction_pointer(), count);
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

        for local in vm.locals() {
            let Some(Value::Float(actual)) = local else {
                panic!("expected float local");
            };
            assert_eq!(actual.to_bits(), expected.to_bits());
        }
        let [Value::Float(actual)] = vm.stack() else {
            panic!("expected one float after printing the other");
        };
        assert_eq!(actual.to_bits(), expected.to_bits());
        assert_eq!(vm.instruction_pointer(), bytecode.instructions.len());
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
        vm.locals(),
        &[
            Some(Value::String("changed".to_owned())),
            Some(Value::String("hello".to_owned())),
        ]
    );
    assert_eq!(
        vm.stack(),
        &[
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
            vm.locals(),
            &[
                Some(Value::Boolean(true)),
                Some(Value::Integer(42)),
                Some(Value::Boolean(false)),
            ]
        );
        assert_eq!(
            vm.stack(),
            &[
                Value::Integer(42),
                Value::Boolean(true),
                Value::Boolean(false)
            ]
        );
        assert_eq!(vm.instruction_pointer(), bytecode.instructions.len());
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
    assert_eq!(vm.instruction_pointer(), 0);
    assert!(vm.stack().is_empty());
    assert!(vm.locals().is_empty());
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
            assert_eq!(vm.instruction_pointer(), 1);
            assert_eq!(vm.stack(), &[Value::Integer(42)]);
            assert_eq!(vm.locals(), &[None]);
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
        assert_eq!(vm.instruction_pointer(), 0);
        assert!(vm.stack().is_empty());
        assert_eq!(vm.locals(), &[None]);
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
    assert_eq!(vm.instruction_pointer(), 2);
    assert!(vm.stack().is_empty());
    assert_eq!(vm.locals(), &[Some(Value::Integer(42))]);
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
    assert!(vm.stack().is_empty());
    assert_eq!(vm.instruction_pointer(), bytecode.instructions.len());
}

#[test]
fn print_rejects_stack_underflow() {
    let bytecode = Bytecode {
        instructions: vec![Instruction::Print],
        local_count: 0,
    };

    let mut vm = Vm::new(&bytecode);

    assert_eq!(vm.run(), Err(VmError::StackUnderflow));
    assert_eq!(vm.instruction_pointer(), 0);
}
