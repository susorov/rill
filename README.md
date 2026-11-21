# rill_

An experimental, actor-oriented programming language written in Rust.

The project aims to explore concurrency through isolated actors, message passing, bounded mailboxes, and deterministic execution.

> **Project status:** Early alpha. Under active development.

## Installation

Precompiled binaries are available for:

- macOS (Apple Silicon and Intel)
- Linux (x64 and ARM64)
- Windows (x64)

Download the appropriate binary from [GitHub Releases](https://github.com/susorov/rill/releases).

For example, on macOS Apple Silicon:

```bash
tar -xzf rill-v0.1.0-alpha.4-macos-arm64.tar.gz
chmod +x rill
./rill --version
```

To install Rill into your user-level binary directory:

```bash
mkdir -p ~/.local/bin
mv rill ~/.local/bin/rill
```

Ensure `~/.local/bin` is included in your `PATH` to run `rill` from anywhere.

## Editor highlighting

Enable the TextMate Bundles plugin, then import the `editors/textmate` folder. `.rill` files use your editor's colors.

## Usage

Compile and execute a Rill program:
```bash
rill hello.rill
```
Inspect the compiled bytecode without executing:
```bash
rill --bytecode hello.rill
```
Additional commands:
```bash
rill --version
rill --help
```

## Language Features

Rill currently supports integer and boolean values, variable declarations, variable references, single-line comments, and basic output.

### Variables

Variables are declared using `let`. Types are inferred automatically.

```bash
let count = 42
let limit = count
let enabled = true
```

### Comments

Single-line comments begin with `//`.

```bash
// This is a comment
let greeting = "Hello, Rill!" // Inline comment
```

### Output

Use `print()` to display a variable or print an empty line.

```bash
let count = 42
let enabled = true

print(count)
print()
print(enabled)
```

### Bytecode

Rill compiles source code into bytecode executed by its virtual machine.

```bash
let count = 42
print(count)
print()
print("Hello, Rill!")
print(-1.25)
print(true)
```
Running `rill --bytecode hello.rill` produces:
```bash
PUSH_INTEGER 42
STORE_LOCAL 0
LOAD_LOCAL 0
PRINT
PRINT_NEWLINE
PUSH_STRING "Hello, Rill!"
PRINT
PUSH_FLOAT -1.25
PRINT
PUSH_BOOLEAN true
PRINT
```
