# Rill architecture

The CLI reads a `.rill` file and passes its contents through the syntax,
compiler, and VM crates.

```text
Source -> Tokens -> Syntax tree -> Bytecode -> Execution
```

- **rill-syntax** turns source text into tokens and parses them into a syntax tree.
  Source spans record byte offsets for diagnostics.
- **rill-compiler** resolves variable names to typed local slots, checks comparison operand types, and translates the
  syntax tree into bytecode.
- **rill-vm** defines the bytecode instructions and runtime values. It executes
  instructions in order using an operand stack and local slots.
- **rill-cli** connects these stages and reports errors. By default it runs
  the bytecode. With `--bytecode`, it prints the instruction listing.

See the [language grammar](GRAMMAR.md) and [bytecode guide](BYTECODE.md) for details.
