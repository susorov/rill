const RILL_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(String::as_str) {
        Some("--version") if args.len() == 2 => {
            println!("rill {RILL_VERSION}");
        }
        Some("--version") => {
            eprintln!("rill: '--version' does not accept arguments");
            std::process::exit(1);
        }
        Some(command) => {
            eprintln!("rill: unknown command '{command}'");
            std::process::exit(1);
        }
        None => {
            eprintln!("rill: no command provided");
            std::process::exit(1);
        }
    }
}
