# Rill bytecode

The Rill compiler assigns each variable a numbered local slot, starting at zero.
The virtual machine executes instructions in order and uses a stack to pass
values between them.

Save this program as `example.rill` and run `rill --bytecode example.rill`.
The compiler emits 11 instructions that use eight of the ten opcodes.

```rill
let count = 42
print(count)
print(-1.25)
print(true)
print("Hello, Rill!")
print()
```

```text
PUSH_INTEGER 42
STORE_LOCAL 0
LOAD_LOCAL 0
PRINT
PUSH_FLOAT -1.25
PRINT
PUSH_BOOLEAN true
PRINT
PUSH_STRING "Hello, Rill!"
PRINT
PRINT_NEWLINE
```

`PUSH_INTEGER 42` puts the value on the stack. `STORE_LOCAL 0` pops it into the
slot for `count`. `LOAD_LOCAL 0` pushes a copy, which `PRINT` pops and writes
followed by a newline.

Float, boolean, and string literals use `PUSH_FLOAT`, `PUSH_BOOLEAN`, and
`PUSH_STRING`. An empty `print()` emits `PRINT_NEWLINE`, which writes a blank
line and leaves the stack unchanged.

Equality emits `EQUAL` after its two operands and pushes their boolean equality
result. Operands must have the same type; insufficient operands or mismatched
types produce a VM error. The compiler rejects type mismatches before execution.

Inequality emits `NOT_EQUAL` after its operands. It applies the same type and
stack checks as `EQUAL`, but pushes the inverse boolean result.
