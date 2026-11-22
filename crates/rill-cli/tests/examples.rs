use std::{fs, path::Path, process::Command};

#[test]
fn all_examples_run_successfully() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut dirs = vec![root];
    let mut count = 0;

    while let Some(dir) = dirs.pop() {
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();

            if path.is_dir() {
                dirs.push(path);
                continue;
            }

            if path.extension().and_then(|s| s.to_str()) != Some("rill") {
                continue;
            }

            let output = Command::new(env!("CARGO_BIN_EXE_rill"))
                .arg(&path)
                .output()
                .unwrap();

            assert!(
                output.status.success(),
                "{}: {}",
                path.display(),
                String::from_utf8_lossy(&output.stderr)
            );

            let expected = match path.file_name().unwrap().to_str().unwrap() {
                "inequality.rill" => "true\nfalse\ntrue\nfalse\ntrue\nfalse\ntrue\nfalse\n",
                "equality.rill" => "true\ntrue\nfalse\ntrue\ntrue\ntrue\n",
                "floats.rill" => "19.95\n-4.5\n0\n42\n",
                "float_references.rill" => "1.25\n1.25\n1.25\n",
                "strings.rill" => "Hello, Rill!\n\n// this is text\n",
                "string_references.rill" => "Hello, Rill!\nHello, Rill!\nHello, Rill!\n",
                "print.rill" => concat!(
                    "42\n42\n\ntrue\ntrue\n19.95\n19.95\nHello, Rill!\n",
                    "42\n-42.01\ntrue\nfalse\nNo variable needed\n\n",
                ),
                "variables.rill"
                | "variable_references.rill"
                | "comments.rill"
                | "booleans.rill"
                | "boolean_references.rill" => "",
                name => panic!("missing expected output for {name}"),
            };
            assert_eq!(
                String::from_utf8(output.stdout).unwrap(),
                expected,
                "{}",
                path.display()
            );
            count += 1;
        }
    }

    assert!(count > 0, "no Rill examples found");
}
