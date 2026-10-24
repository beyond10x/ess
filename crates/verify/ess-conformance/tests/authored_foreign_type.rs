//! A scenario document of a foreign `type:` is refused for its type, not for its first unknown key.
//!
//! An adopter's own trace format (`type: mcp-outbound-lifecycle/1`, given/when/then prose with keys
//! such as `story` and `then`) handed to the authored-scenario reader was refused as ESS-AUTHOR-001
//! "unknown field `story`", because the reader admitted every key before it looked at `type:`. A
//! document that is not an ESS scenario at all read as an ESS scenario with a misspelt key. The
//! format refusal, ESS-AUTHOR-002, comes first now; an `ess-scenario/N` document with a misspelt key
//! keeps the unknown-key refusal it always had.

use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Cause, Refusal, Source};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;

/// `examples/billing`, compiled from the files it lives in.
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

/// The one refusal a document produced.
fn refusal(ir: &EssIr, text: &str) -> Refusal {
    let authoring = compile_authored(ir, &[Source::new("scenario.yaml", text)]);
    assert!(
        authoring.scenarios.is_empty(),
        "a refused document compiled"
    );
    let mut refusals = authoring.refusals;
    assert_eq!(
        refusals.len(),
        1,
        "{}",
        refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
    refusals.remove(0)
}

/// An adopter's trace document, shaped as the one reported: its own `type:`, then its own keys.
const FOREIGN: &str = "\
type: mcp-outbound-lifecycle/1
story: cancellation closes the response stream
document: outbound
domain: billing.invoice
scenario: cancellation-closes-the-response-stream
summary: A cancelled call closes its stream.
state: open
outcome: closed
wire_code: 499
given: an open stream
when: the caller cancels
then: the stream closes
never: a late frame arrives
";

/// The same keys after `type:` and before it: the refusal does not depend on where `type:` sits.
const FOREIGN_TYPE_LAST: &str = "\
story: cancellation closes the response stream
given: an open stream
then: the stream closes
type: mcp-outbound-lifecycle/1
";

#[test]
fn authored_foreign_type_is_refused_by_format_before_unknown_keys() {
    let ir = billing();
    for text in [FOREIGN, FOREIGN_TYPE_LAST] {
        let refused = refusal(&ir, text);
        assert_eq!(refused.code().to_string(), "ESS-AUTHOR-002", "{refused}");
        assert!(
            matches!(&refused.cause, Cause::UnsupportedFormat { found } if found == "mcp-outbound-lifecycle/1"),
            "{refused}"
        );
        let shown = refused.to_string();
        assert!(!shown.contains("unknown field"), "{shown}");
        for accepted in [
            "ess-scenario/1",
            "ess-scenario/2",
            "ess-scenario/3",
            "ess-scenario/4",
            "ess-scenario/5",
        ] {
            assert!(shown.contains(accepted), "names `{accepted}`:\n{shown}");
        }
    }
}

#[test]
fn authored_foreign_type_help_says_a_trace_format_is_not_a_scenario_input() {
    let refused = refusal(&billing(), FOREIGN);
    let help = refused.hint();
    assert!(help.contains("--scenarios"), "{help}");
    assert!(help.contains("ess-inputs.yaml"), "{help}");
    assert!(!help.starts_with("write `type:"), "{help}");
}

#[test]
fn authored_ess_scenario_with_unknown_key_still_refused() {
    let text = "\
type: ess-scenario/1
domain: billing.invoice
scenario: a-scenario
summary: What this scenario proves, in one line.
timelime: []
";
    let refused = refusal(&billing(), text);
    assert_eq!(
        refused.to_string(),
        "refusal[ESS-AUTHOR-001]: scenario.yaml\n  \
         unreadable: unknown field `timelime`, expected one of `type`, `domain`, `scenario`, \
         `summary`, `fixtures`, `arrange`, `timeline`, `assert` at line 5 column 1\n  \
         help: the document is YAML with the keys `type`, `domain`, `scenario` and `summary`; a \
         key it does not know is refused rather than ignored"
    );
}

#[test]
fn authored_missing_type_is_refused_by_name() {
    let ir = billing();
    let with_foreign_keys = "\
story: cancellation closes the response stream
domain: billing.invoice
scenario: a-scenario
summary: What this scenario proves, in one line.
";
    let refused = refusal(&ir, with_foreign_keys);
    assert_eq!(refused.code().to_string(), "ESS-AUTHOR-001", "{refused}");
    let shown = refused.to_string();
    assert!(shown.contains("missing field `type`"), "{shown}");
    assert!(!shown.contains("unknown field"), "{shown}");
}

#[test]
fn a_missing_type_without_foreign_keys_keeps_its_refusal() {
    let ir = billing();
    let only_ess_keys = "\
domain: billing.invoice
scenario: a-scenario
summary: What this scenario proves, in one line.
";
    assert_eq!(
        refusal(&ir, only_ess_keys).to_string(),
        "refusal[ESS-AUTHOR-001]: scenario.yaml\n  \
         unreadable: missing field `type`\n  \
         help: the document is YAML with the keys `type`, `domain`, `scenario` and `summary`; a \
         key it does not know is refused rather than ignored"
    );
}

/// A `type:` that is not a string keeps the refusal it always had, byte for byte: a scalar reads as
/// its text and is refused for its format, anything else is refused as unreadable.
#[test]
fn a_non_string_type_keeps_its_refusal() {
    let ir = billing();
    let header = "\
domain: billing.invoice
scenario: a-scenario
summary: What this scenario proves, in one line.
";
    let scalar = refusal(&ir, &format!("type: 7\n{header}"));
    assert_eq!(
        scalar.cause,
        Cause::UnsupportedFormat { found: "7".into() },
        "{scalar}"
    );
    assert_eq!(scalar.cause.to_string(), "`7` is not `ess-scenario/1`");
    let sequence = refusal(&ir, &format!("type: [a, b]\nstory: x\n{header}"));
    assert_eq!(
        sequence.to_string(),
        "refusal[ESS-AUTHOR-001]: scenario.yaml\n  \
         unreadable: type: invalid type: sequence, expected a string at line 1 column 7\n  \
         help: the document is YAML with the keys `type`, `domain`, `scenario` and `summary`; a \
         key it does not know is refused rather than ignored"
    );
}
