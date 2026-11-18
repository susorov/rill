use std::path::Path;
use std::process::ExitCode;

const RILL_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("rill: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.as_slice() {
        [option] if option == "--version" => {
            println!("rill {RILL_VERSION}");
            Ok(())
        }

        [option, ..] if option == "--version" => {
            Err("'--version' does not accept arguments".into())
        }

        [option, ..] if option.starts_with('-') => Err(format!("unknown option '{option}'")),

        [path] => compile_file(path),

        [] => Err("usage: rill <file.rill> | --version".into()),

        _ => Err("expected one source file; usage: rill <file.rill>".into()),
    }
}

fn compile_file(path: &str) -> Result<(), String> {
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
    let bytecode = rill_compiler::compile(&program)
        .map_err(|error| diagnostic(&error.message, error.span.start))?;

    for instruction in &bytecode.instructions {
        println!("{instruction}");
    }

    Ok(())
}
