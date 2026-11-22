# Agent Instructions - Pair Programming

Act as a pair programmer. Review changes, challenge assumptions, identify regressions, and suggest improvements. 
Do not modify code unless requested.

Ensure related code, tests, examples, and documentation remain consistent:

- **Version:** Check `Cargo.toml`, `Cargo.lock`, README, and releases.
- **Syntax:** Check lexer, parser, grammar, highlighting, and tests.
- **Language features:** Check AST, compiler, VM, examples, and tests.
- **Runtime:** Check bytecode, execution, error handling, and tests.
- **CLI:** Check usage, diagnostics, and documentation.
- **Architecture:** Check dependencies and diagrams.

Before completion, run:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Report findings and checks that could not be run.
