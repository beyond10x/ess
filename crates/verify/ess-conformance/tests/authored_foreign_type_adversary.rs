//! Adversary cases for the `type:`-first read of authored scenario documents.
//!
//! The pre-read parses the document as a YAML `Value` and falls back to the full typed read when
//! that parse fails. A `Value` mapping refuses a duplicate key at any depth, where the typed read
//! stops at the first unknown key first, so a foreign document with one repeated key is refused for
//! that key again (ESS-AUTHOR-001), not for its format.

use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Document, Refusal, Source};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;

fn billing() -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
        .canonicalize()
        .expect("the billing example exists");
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
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
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((SpecSource::new(label), raw));
    }
    let specification =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("validates:\n{errors}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
}

fn refusal(ir: &EssIr, text: &str) -> Refusal {
    let authoring = compile_authored(ir, &[Source::new("scenario.yaml", text)]);
    assert!(
        authoring.scenarios.is_empty(),
        "a refused document compiled"
    );
    let mut refusals = authoring.refusals;
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    refusals.remove(0)
}

/// The adopter's trace shape with one prose key written twice, as a hand-edited trace can be.
const FOREIGN_WITH_A_REPEATED_KEY: &str = "\
type: mcp-outbound-lifecycle/1
story: cancellation closes the response stream
given: an open stream
when: the caller cancels
then: the stream closes
then: no late frame arrives
";

/// The same, with the repeated key nested under a foreign key.
const FOREIGN_WITH_A_NESTED_REPEATED_KEY: &str = "\
type: mcp-outbound-lifecycle/1
story: cancellation closes the response stream
wire:
  code: 499
  code: 499
";

#[test]
fn adversary_foreign_type_with_a_repeated_key_is_refused_by_format() {
    let ir = billing();
    for text in [
        FOREIGN_WITH_A_REPEATED_KEY,
        FOREIGN_WITH_A_NESTED_REPEATED_KEY,
    ] {
        let refused = refusal(&ir, text);
        assert_eq!(refused.code().to_string(), "ESS-AUTHOR-002", "{refused}");
        assert!(!refused.to_string().contains("unknown field"), "{refused}");
    }
}

/// The full typed read's own refusal of a document without `type:` and with only ESS keys: the
/// bytes the change says such a document keeps.
#[test]
fn adversary_missing_type_without_foreign_keys_keeps_the_full_read_bytes() {
    let only_ess_keys = "\
domain: billing.invoice
scenario: a-scenario
summary: What this scenario proves, in one line.
";
    let old = ess_yaml::from_str::<Document>(only_ess_keys)
        .expect_err("a document without `type:` is refused")
        .to_string();
    let refused = refusal(&billing(), only_ess_keys);
    assert_eq!(refused.cause.to_string(), format!("unreadable: {old}"));
}
