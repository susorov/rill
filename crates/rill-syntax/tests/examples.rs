use std::fs;
use std::path::Path;

use rill_syntax::parser::parse;

fn check_examples(dir: &Path) -> usize {
    let mut count = 0;

    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();

        if path.is_dir() {
            count += check_examples(&path);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("rill") {
            continue;
        }

        let source = fs::read_to_string(&path).unwrap();

        if let Err(error) = parse(&source) {
            panic!(
                "{}: bytes {}..{}: {}",
                path.display(),
                error.span.start,
                error.span.end,
                error.message
            );
        }

        count += 1;
    }

    count
}

#[test]
fn all_examples_parse_successfully() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");

    let count = check_examples(&examples);

    assert!(count > 0, "no rill examples found");
}
