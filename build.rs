use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_LINES: usize = 1200;
const SKIP_ENV_VAR: &str = "VIBECRAFT_SKIP_LINE_CHECK";
const DECOMPILED_SOURCE_ROOT_ENV: &str = "VIBECRAFT_DECOMPILED_SOURCE_ROOT";

// Test files that contain a single very large `#[test]` function whose body
// shares state across hundreds of assertions. Splitting the function into
// smaller `#[test]`s would change test names and require careful state
// reconstruction, and macro_rules!/include!() workarounds either don't share
// hygiene context (macros) or require single-expression contents (include!).
// These files are exempt from the per-file line cap.
const ALLOWED_OVERSIZED_FILES: &[&str] = &[
    "src/network/play/tests/entity_movement_test.rs",
    "src/network/play/tests/small_play_packets_test.rs",
    // handle_login_connection is a single 1041-line function whose body
    // carries dozens of mutable bindings across the login handshake;
    // refactoring it into helpers would mean threading a wide context
    // struct through every sub-call.
    "src/network/status/chunk_a.rs",
];

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed={}", SKIP_ENV_VAR);
    println!("cargo:rerun-if-env-changed={}", DECOMPILED_SOURCE_ROOT_ENV);
    println!("cargo:rustc-check-cfg=cfg(vibecraft_has_decompiled_sources)");
    println!("cargo:rustc-check-cfg=cfg(vibecraft_has_sound_events_source)");

    let manifest_dir = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(dir) => PathBuf::from(dir),
        None => panic!("CARGO_MANIFEST_DIR must be set when running build script"),
    };
    configure_optional_decompiled_sources(&manifest_dir);
    configure_optional_sound_events_source(&manifest_dir);

    if env::var_os(SKIP_ENV_VAR).is_some() {
        println!(
            "cargo:warning={} is set; skipping per-file line-count enforcement.",
            SKIP_ENV_VAR
        );
        return;
    }

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
        message.push_str(&format!("  {:>7} lines  {}\n", lines, rel.display()));
    }
    message.push_str(&format!(
        "\nSplit the offending files into smaller modules. To bypass this \
         check temporarily (e.g. while running tests mid-refactor), set \
         {}=1.\n",
        SKIP_ENV_VAR
    ));

    panic!("{}", message);
}

fn configure_optional_decompiled_sources(manifest_dir: &Path) {
    let requested_source_root = optional_decompiled_source_root(manifest_dir);
    println!("cargo:rerun-if-changed={}", requested_source_root.display());

    if requested_source_root.is_dir() {
        let source_root = fs::canonicalize(&requested_source_root).unwrap_or_else(|err| {
            panic!(
                "failed to canonicalize optional Java decompilation root {}: {}",
                requested_source_root.display(),
                err
            )
        });
        println!(
            "cargo:rustc-env=VIBECRAFT_DECOMPILED_SOURCE_ROOT={}",
            source_root.display()
        );
        println!("cargo:rustc-cfg=vibecraft_has_decompiled_sources");
    } else {
        println!(
            "cargo:warning=optional Java decompilation root not found at {}; \
             source-backed parity test modules will be skipped",
            requested_source_root.display()
        );
    }
}

fn configure_optional_sound_events_source(manifest_dir: &Path) {
    let source = optional_decompiled_source_root(manifest_dir)
        .join("net")
        .join("minecraft")
        .join("sounds")
        .join("SoundEvents.java");
    println!("cargo:rerun-if-changed={}", source.display());

    if source.is_file() {
        let source = fs::canonicalize(&source).unwrap_or_else(|err| {
            panic!(
                "failed to canonicalize optional Java parity source {}: {}",
                source.display(),
                err
            )
        });
        println!(
            "cargo:rustc-env=VIBECRAFT_SOUND_EVENTS_SOURCE={}",
            source.display()
        );
        println!("cargo:rustc-cfg=vibecraft_has_sound_events_source");
    } else {
        println!(
            "cargo:warning=optional Java parity source not found at {}; \
             sound-event parity checks will be skipped",
            source.display()
        );
    }
}

fn optional_decompiled_source_root(manifest_dir: &Path) -> PathBuf {
    env::var_os(DECOMPILED_SOURCE_ROOT_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join(".missing-optional-decompiled-source-root"))
}

fn collect_offenders(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, usize)>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => panic!("failed to read {}: {}", dir.display(), err),
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => panic!(
                "failed to read directory entry in {}: {}",
                dir.display(),
                err
            ),
        };
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
            let rel = path
                .strip_prefix(root.parent().unwrap_or(root))
                .map(Path::to_path_buf)
                .unwrap_or_else(|_| path.clone());
            let rel_str = rel.to_string_lossy();
            if !ALLOWED_OVERSIZED_FILES
                .iter()
                .any(|allowed| rel_str == *allowed)
            {
                out.push((path, line_count));
            }
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
