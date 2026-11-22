//! Every hint that asks for a specification format names where the `format:` key goes: on the
//! source that declares the system
//! (`story:every-format-version-hint-names-the-source-that-declares-the-system`).
//!
//! `FormatVersion::declaration_hint` and the `declaration_hint!` macro in `src/system.rs` are
//! the one place that spells the instruction. This reads the production source of `ess-domain`
//! and `ess-conformance` and fails, naming the file and the line, at any other string literal
//! that spells `` `format: ess/`` or `` `format: {``.

use std::path::{Path, PathBuf};

/// Files a later unit of the same change converts. It only shrinks, and is empty when the change
/// is complete.
const PENDING: &[&str] = &[];

/// The two literals that are the instruction itself, each with the file that owns it.
const EXEMPT: &[(&str, &str)] = &[
    (
        "crates/specify/ess-domain/src/system.rs",
        "\"write `format: {self}` on the source that declares the system\"",
    ),
    (
        "crates/specify/ess-domain/src/system.rs",
        "\"write `format: ess/\",",
    ),
];

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn rust_files(directory: &Path, found: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("{}: {error}", directory.display()))
        .map(|entry| entry.expect("a directory entry").path())
        .collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            rust_files(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// Whether the first line after an attribute that is neither blank nor another attribute opens a
/// module.
fn opens_module(rest: &[&str]) -> bool {
    rest.iter()
        .map(|line| line.trim())
        .find(|line| !line.is_empty() && !line.starts_with("#["))
        .is_some_and(|line| {
            line.starts_with("mod ")
                || line.starts_with("pub mod ")
                || line.starts_with("pub(crate) mod ")
        })
}

/// The production lines of one file, numbered by the line each starts on. A line ending in `\`
/// is joined to the next with that line's leading whitespace dropped, as Rust reads a continued
/// string literal. Reading stops at the first `#[cfg(test)]` that opens a module: every test
/// module of these crates sits at the end of its file.
fn production_lines(text: &str) -> Vec<(usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut joined = Vec::new();
    let mut open: Option<(usize, String)> = None;
    for (index, line) in lines.iter().enumerate() {
        if open.is_none() && line.trim() == "#[cfg(test)]" && opens_module(&lines[index + 1..]) {
            break;
        }
        let piece = if open.is_some() {
            line.trim_start()
        } else {
            line
        };
        let (number, mut current) = open.take().unwrap_or((index + 1, String::new()));
        current.push_str(piece);
        match current.strip_suffix('\\') {
            Some(continued) => open = Some((number, continued.to_owned())),
            None => joined.push((number, current)),
        }
    }
    joined.extend(open);
    joined
}

#[test]
fn no_literal_outside_the_function_spells_a_format_to_declare() {
    let root = repository();
    let mut files = Vec::new();
    rust_files(&root.join("crates/specify/ess-domain/src"), &mut files);
    rust_files(&root.join("crates/verify/ess-conformance/src"), &mut files);
    let mut found = Vec::new();
    for path in &files {
        let relative = path
            .strip_prefix(&root)
            .expect("under the repository")
            .to_string_lossy()
            .replace('\\', "/");
        if PENDING.contains(&relative.as_str()) {
            continue;
        }
        let text =
            std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{relative}: {error}"));
        for (number, line) in production_lines(&text) {
            if line.trim_start().starts_with("//") {
                continue;
            }
            if !(line.contains("`format: ess/") || line.contains("`format: {")) {
                continue;
            }
            if EXEMPT
                .iter()
                .any(|(file, needle)| relative == *file && line.contains(needle))
            {
                continue;
            }
            found.push(format!("{relative}:{number}: {}", line.trim()));
        }
    }
    assert!(
        found.is_empty(),
        "build these hints with `FormatVersion::declaration_hint`, or `declaration_hint!` where \
         a `&'static str` is needed:\n{}",
        found.join("\n")
    );
}

#[test]
fn a_continued_literal_is_read_whole_and_a_test_module_is_not_read() {
    let text = r#"    .with_hint("declare \
                `format: ess/18`")
#[cfg(test)]
mod tests {
    const T: &str = "`format: ess/1`";
}
"#;
    let lines = production_lines(text);
    assert_eq!(lines.len(), 1, "{lines:?}");
    assert_eq!(lines[0].0, 1);
    assert!(lines[0].1.contains("declare `format: ess/18`"), "{lines:?}");
}

#[test]
fn every_pending_file_exists() {
    let root = repository();
    for file in PENDING {
        assert!(
            root.join(file).is_file(),
            "{file} is not a file: fix PENDING"
        );
    }
}

/// Every single-file fixture of the conformance suite, read at `ess/1`, where every format
/// gate it uses fires: each `unsupported_format_version` refusal carries a hint naming the
/// source that declares the system, and none spells the instruction in its message. On
/// 0.97.1 this corpus draws 802 such refusals from 197 files.
#[test]
fn every_format_refusal_of_the_fixture_corpus_carries_the_hint() {
    use ess_domain::spec::{RawSpecFile, Specification};
    use ess_domain::system::Source;
    use ess_primitives::error::ValidationCode;

    let directory = repository().join("crates/verify/ess-conformance/tests/fixtures");
    let mut names: Vec<PathBuf> = std::fs::read_dir(&directory)
        .expect("the fixture directory")
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "yaml")
        })
        .collect();
    names.sort();
    let mut seen = 0;
    let mut missing = Vec::new();
    for path in &names {
        let text = std::fs::read_to_string(path).expect("a fixture");
        let Some(start) = text
            .find("\nformat: ess/")
            .map(|at| at + 1)
            .or_else(|| text.starts_with("format: ess/").then_some(0))
        else {
            continue;
        };
        if !text.lines().any(|line| line.starts_with("system:")) {
            continue;
        }
        let end = text[start..].find('\n').map_or(text.len(), |at| start + at);
        let lowered = format!("{}format: ess/1{}", &text[..start], &text[end..]);
        let Ok(raw) = RawSpecFile::parse(&lowered) else {
            continue;
        };
        let name = path
            .file_name()
            .expect("a name")
            .to_string_lossy()
            .into_owned();
        let Err(errors) = Specification::assemble([(Source::new(name.clone()), raw)]) else {
            continue;
        };
        for error in errors.as_slice() {
            if error.code != ValidationCode::UnsupportedFormatVersion {
                continue;
            }
            seen += 1;
            let carries_hint = error
                .hint
                .as_deref()
                .is_some_and(|hint| hint.contains("on the source that declares the system"));
            if !carries_hint || error.message.contains("`format:") {
                missing.push(format!("{name}: {} — {:?}", error.message, error.hint));
            }
        }
    }
    assert!(
        seen >= 500,
        "the corpus draws {seen} format refusals; expected about 800"
    );
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}
