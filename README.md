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
tar -xzf rill-v0.1.0-alpha.1-macos-arm64.tar.gz
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

The initial CLI supports version reporting:

```bash
rill --version
```

Output:

```text
rill 0.1.0-alpha.1
```
