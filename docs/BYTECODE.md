# Rill bytecode

Run `rill --bytecode example.rill` to inspect a program without executing it.
The VM executes instructions in order, using a stack and numbered local slots
starting at zero.

## Instructions

| Instruction | Stack effect | Behavior |
| --- | --- | --- |
| `PUSH_INTEGER value` | `[] → [value]` | Push a signed 64-bit integer. |
| `PUSH_FLOAT value` | `[] → [value]` | Push a 64-bit float. |
| `PUSH_BOOLEAN value` | `[] → [value]` | Push `true` or `false`. |
| `PUSH_STRING value` | `[] → [value]` | Push a string. |
| `STORE_LOCAL slot` | `[value] → []` | Store the top value in a local slot. |
| `LOAD_LOCAL slot` | `[] → [value]` | Push a copy of a local slot's value. |
| `EQUAL` | `[left, right] → [bool]` | Compare `left == right`. |
| `NOT_EQUAL` | `[left, right] → [bool]` | Compare `left != right`. |
| `GREATER_THAN` | `[left, right] → [bool]` | Compare `left > right`. |
| `GREATER_THAN_OR_EQUAL` | `[left, right] → [bool]` | Compare `left >= right`. |
| `LESS_THAN` | `[left, right] → [bool]` | Compare `left < right`. |
| `LESS_THAN_OR_EQUAL` | `[left, right] → [bool]` | Compare `left <= right`. |
| `PRINT` | `[value] → []` | Print the value followed by a newline. |
| `PRINT_NEWLINE` | `[] → []` | Print an empty line. |

Equality accepts any two values of the same type or two numeric values. Ordering
requires two numeric values. When one operand is an integer and the other is a
float, the VM converts the integer to `f64` before comparing. This may round
large integers; integer-only comparisons retain full `i64` precision. Other
mixed types are rejected at compile time and checked by the VM. Strings compare
by content; floats use exact numeric comparisons. With NaN, `!=` returns true
and the other comparison operators return false.

The VM reports invalid or uninitialized local slots, insufficient stack operands,
and invalid comparison types. These errors preserve the failing instruction's
stack and locals and leave the instruction pointer unchanged.

## Example

```rill
let count = 42
print(count >= 40)
```

```text
PUSH_INTEGER 42
STORE_LOCAL 0
LOAD_LOCAL 0
PUSH_INTEGER 40
GREATER_THAN_OR_EQUAL
PRINT
```

The comparison consumes `42` and `40`, pushes `true`, and `PRINT` consumes and
prints that result.
