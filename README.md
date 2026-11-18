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
tar -xzf rill-v0.1.0-alpha.2-macos-arm64.tar.gz
chmod +x rill
./rill --version
```

To install Rill into your user-level binary directory:

```bash
mkdir -p ~/.local/bin
mv rill ~/.local/bin/rill
```

Ensure `~/.local/bin` is included in your `PATH` to run `rill` from anywhere.

## Usage

```bash
rill hello.rill
```

Only files with the .rill extension are accepted.

The command compiles the source and prints the bytecode instructions. It does not execute the program yet. Syntax and compilation errors include the file name, line and column. Errors return a nonzero exit status.

Variables must be declared before they are referenced. A later declaration with the same name creates a new binding. Its initializer can reference the earlier binding.
