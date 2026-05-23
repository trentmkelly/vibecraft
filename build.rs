use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_LINES: usize = 1200;
const SKIP_ENV_VAR: &str = "RUSTCRAFT_SKIP_LINE_CHECK";

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed={}", SKIP_ENV_VAR);

    if env::var_os(SKIP_ENV_VAR).is_some() {
        println!(
            "cargo:warning={} is set; skipping per-file line-count enforcement.",
            SKIP_ENV_VAR
        );
        return;
    }

    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .expect("CARGO_MANIFEST_DIR must be set when running build script");
    let src_dir = manifest_dir.join("src");

    let mut offenders: Vec<(PathBuf, usize)> = Vec::new();
    collect_offenders(&src_dir, &src_dir, &mut offenders);
    offenders.sort_by(|a, b| b.1.cmp(&a.1));

    if offenders.is_empty() {
        return;
    }

    let mut message = String::new();
    message.push_str(&format!(
        "\n\n{} source file(s) exceed the {}-line per-file limit:\n",
        offenders.len(),
        MAX_LINES
    ));
    for (path, lines) in &offenders {
        let rel = path.strip_prefix(&manifest_dir).unwrap_or(path);
        message.push_str(&format!(
            "  {:>7} lines  {}\n",
            lines,
            rel.display()
        ));
    }
    message.push_str(&format!(
        "\nSplit the offending files into smaller modules. To bypass this \
         check temporarily (e.g. while running tests mid-refactor), set \
         {}=1.\n",
        SKIP_ENV_VAR
    ));

    panic!("{}", message);
}

fn collect_offenders(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, usize)>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => panic!("failed to read {}: {}", dir.display(), err),
    };

    for entry in entries {
        let entry = entry.expect("failed to read directory entry");
        let path = entry.path();
        let file_type = entry
            .file_type()
            .unwrap_or_else(|err| panic!("failed to stat {}: {}", path.display(), err));

        if file_type.is_dir() {
            collect_offenders(root, &path, out);
            continue;
        }

        if !file_type.is_file() {
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(err) => panic!("failed to read {}: {}", path.display(), err),
        };

        let line_count = count_lines(&contents);
        if line_count > MAX_LINES {
            out.push((path, line_count));
        }
    }
}

fn count_lines(contents: &str) -> usize {
    if contents.is_empty() {
        return 0;
    }
    let newlines = contents.bytes().filter(|b| *b == b'\n').count();
    if contents.ends_with('\n') {
        newlines
    } else {
        newlines + 1
    }
}
