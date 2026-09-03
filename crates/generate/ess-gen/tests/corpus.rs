//! The bytes `ess generate --kind docs` writes, pinned.
//!
//! It was written for the port from writing markdown directly to producing an `ess-docs/1`
//! document that a renderer writes: two thousand lines across thirty functions, where the only
//! claim worth making was that the output did not move. The port is done and the pin stays, doing
//! the same job for every change after it.
//!
//! So the corpus is committed and compared. A change here is not a test to update — it is a change
//! to what every adopter's committed pages say, and it belongs in the commit that meant to make
//! it, with the reason in the message.
//!
//! # Why a whole tree and not assertions
//!
//! `tests/docs.rs` asserts individual sentences and is the right shape for *what the projection
//! should say*. It cannot catch a blank line that moved, a table separator that gained a space, or
//! a link that resolved one directory differently — which is exactly the class of defect a port
//! introduces and exactly what an adopter would see in `git diff`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_compiler::{compile, EssIr};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// The repository root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

/// The documentation of one example, keyed by path.
fn documentation(example: &str) -> BTreeMap<String, String> {
    let directory = root().join("examples").join(example);
    let ir = compiled(&directory);
    ess_gen::artifact::run(&ess_gen::docs::Docs, &ir)
        .expect("the projection claims no path twice")
        .into_iter()
        .map(|(path, artifact)| (path, artifact.contents))
        .collect()
}

/// One example, compiled.
///
/// Files are discovered rather than listed, for the reason `tests/docs.rs` gives: a file added to
/// an example would otherwise be compiled by the CLI and ignored by the test meant to keep the
/// example honest.
fn compiled(directory: &Path) -> EssIr {
    let base = directory
        .canonicalize()
        .unwrap_or_else(|error| panic!("{} exists: {error}", directory.display()));

    let mut found = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(next) = pending.pop() {
        for entry in std::fs::read_dir(&next).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();

    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside the example")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{label} is readable: {error}"));
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("{} validates:\n{errors}", base.display()));
    compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("{} resolves:\n{diagnostics}", base.display()))
}

/// Where the pinned bytes live.
fn corpus(example: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/corpus")
        .join(example)
}

/// Compares one example's pages against the corpus, naming the first file that moved.
fn assert_pinned(example: &str) {
    let generated = documentation(example);
    let pinned = corpus(example);

    for (path, contents) in &generated {
        let file = pinned.join(path);
        let recorded = std::fs::read_to_string(&file).unwrap_or_else(|error| {
            panic!(
                "{} is generated and not pinned ({error}). If the projection gained a page, add it \
                 to the corpus in the commit that added the page",
                file.display()
            )
        });
        assert_eq!(
            *contents, recorded,
            "`{path}` of `{example}` is not what is pinned. This is what every adopter's \
             committed pages would gain in `git diff`; if the change is deliberate, regenerate \
             the corpus in the commit that meant to make it and say why in the message"
        );
    }

    // The other direction, so a page that stops being generated fails rather than lingering.
    let mut walked = Vec::new();
    walk(&pinned, &pinned, &mut walked);
    for path in walked {
        assert!(
            generated.contains_key(&path),
            "`{path}` of `{example}` is pinned and is no longer generated"
        );
    }
}

/// Every file under `directory`, as a path relative to `base`.
fn walk(base: &Path, directory: &Path, out: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(base, &path, out);
        } else if let Ok(relative) = path.strip_prefix(base) {
            out.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
}

#[test]
fn the_billing_documentation_is_byte_for_byte_what_is_pinned() {
    assert_pinned("billing");
}

#[test]
fn the_gatepass_documentation_is_byte_for_byte_what_is_pinned() {
    assert_pinned("gatepass");
}

#[test]
fn the_oracle_fixture_documentation_is_byte_for_byte_what_is_pinned() {
    assert_pinned("oracle-fixture");
}
