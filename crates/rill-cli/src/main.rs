//! Command-line runner for UTF-8 `.rill` source files.
//!
//! `rill <file.rill>` compiles and executes; `--bytecode <file.rill>` prints
//! instructions without running. `--help` and `--version` show usage and version.
//! Normal output goes to stdout. Reported errors go to stderr with exit status 1;
//! success returns 0. Syntax and compilation errors include `path:line:column`.

use std::path::Path;
use std::process::ExitCode;

use rill_vm::bytecode::Bytecode;
use rill_vm::vm::Vm;

const RILL_VERSION: &str = env!("CARGO_PKG_VERSION");
const HELP: &str = "\
Usage:
  rill <file.rill>
  rill --bytecode <file.rill>
  rill --version
  rill --help

Options:
  --bytecode  Print bytecode without executing
  --version   Print version information
  --help      Print this help message";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("rill: {message}");
            ExitCode::FAILURE
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Run,
    Bytecode,
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    let (path, mode) = match args.as_slice() {
        [option] if option == "--help" => {
            println!("{HELP}");
            return Ok(());
        }
        [option] if option == "--version" => {
            println!("rill {RILL_VERSION}");
            return Ok(());
        }
        [option, path] if option == "--bytecode" => (path.as_str(), Mode::Bytecode),
        [path] if !path.starts_with('-') => (path.as_str(), Mode::Run),
        [option, ..] if option.starts_with('-') => {
            return Err(format!("unknown option '{option}'. Try 'rill --help'"));
        }
        _ => {
            return Err("usage: rill [--bytecode] <file.rill> | --version | --help".into());
        }
    };

    let bytecode = compile_file(path)?;

    match mode {
        Mode::Run => {
            let mut vm = Vm::new(&bytecode);
            vm.run().map_err(|error| error.to_string())?;
        }
        Mode::Bytecode => {
            for instruction in &bytecode.instructions {
                println!("{instruction}");
            }
        }
    }

    Ok(())
}

fn compile_file(path: &str) -> Result<Bytecode, String> {
    if Path::new(path).extension().and_then(|ext| ext.to_str()) != Some("rill") {
        return Err(format!("expected a '.rill' source file: '{path}'"));
    }

    let source =
        std::fs::read_to_string(path).map_err(|error| format!("cannot read '{path}': {error}"))?;

    let diagnostic = |message: &str, start: usize| {
        let offset = start.min(source.len());
        let prefix = &source[..offset];

        let line = prefix.bytes().filter(|&byte| byte == b'\n').count() + 1;
        let column = prefix.rsplit('\n').next().unwrap_or("").chars().count() + 1;

        format!("{path}:{line}:{column}: {message}")
    };

    let program = rill_syntax::parser::parse(&source)
        .map_err(|error| diagnostic(&error.message, error.span.start))?;

    rill_compiler::compile(&program).map_err(|error| diagnostic(&error.message, error.span.start))
}
