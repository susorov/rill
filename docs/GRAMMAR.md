# Rill Grammar

## Syntax

```rill
let name = "Rill"
let version = 1
let active = true

print(name)
print(version)
print()
```

Each statement must be on a separate line. Blank lines are allowed, and the final newline is optional.

## Data Types

Rill supports four data types:

- `int` - 64-bit signed integers (`i64`)
- `float` - 64-bit floating point numbers (`f64`)
- `bool` - `true` or `false`
- `string` - text enclosed in double quotes

Numeric overflow is rejected. Numbers cannot contain spaces.

## Variables

Variables are declared using `let` and must be initialized.

Names may contain letters, digits, and underscores, but cannot start with a digit. Names are case sensitive.

Variables must be declared before use. Redeclaration and reassignment are not allowed.

Reserved words: `let`, `int`, `float`, `bool`, `true`, `false`, `print`.

## Printing

`print(value)` outputs a value followed by a newline.

`print()` outputs an empty line.

Values can be literals, previously declared variables, or comparisons.

## Comments and Strings

Comments start with `//` and continue to the end of the line.

Spaces and tabs are ignored outside strings.

Strings use double quotes. They cannot contain double quotes or newlines. Backslashes are treated as normal characters; escape sequences are not supported.

## Comparisons

`==` tests equality and `!=` tests inequality: `let b = 10 != 20`.
Both return a boolean and require operands of the same type. Mixed types,
including integers versus floats, are compile time errors. Strings compare by
content; floats use exact numeric equality, so `0.0 != -0.0` is false.
`>=` tests whether the left number is greater than or equal to the right:
`print(10 >= 5)`. Both operands must be integers or both must be floats.
The result is a boolean. Strings, booleans, and mixed numeric types are rejected
at compile time. `0.0 >= -0.0` is true.

`<=` tests whether the left number is less than or equal to the right:
`print(5 <= 10)`. It uses the same numeric type rules as `>=` and returns a
boolean. `0.0 <= -0.0` is true.

`>` tests whether the left number is strictly greater than the right:
`print(10 > 5)`. It uses the same numeric type rules as `>=` and returns a
boolean. Equal values return false, including `0.0 > -0.0`.

`<` tests whether the left number is strictly less than the right:
`print(5 < 10)`. It uses the same numeric type rules as `>` and returns a
boolean. Equal values return false, including `0.0 < -0.0`.

Comparisons cannot be chained.

## Limitations

Rill currently does not support other operators, expression grouping, type annotations, control flow, or variable reassignment.

Line endings can be LF, CRLF, or CR.
