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
tar -xzf rill-v0.1.0-alpha.6-macos-arm64.tar.gz
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

## Documentation

- [Architecture](docs/ARCHITECTURE.md) describes the crates and how a source file becomes a running program.
- [Language grammar](docs/GRAMMAR.md) covers syntax, types, and variable declarations.
- [Bytecode guide](docs/BYTECODE.md) explains the VM instructions with a complete example.

## License

Rill is licensed under either the [MIT license](LICENSE-MIT) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in Rill by you shall be dual licensed as above, without any
additional terms or conditions.
