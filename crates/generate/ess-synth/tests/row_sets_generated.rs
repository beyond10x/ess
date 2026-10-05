//! Generated Rust and Go behaviour for a command reading a row set (`docs/design/filtered-related-reads.md`;
//! beyond10x/ess#228, #299): generated storage enumerates no rows by a selector in this cut, so
//! such a command stays an obligation, and the plan names the construct that keeps it owed — the
//! row-set guard, or the read of the one row a selector selects — rather than generating a
//! behaviour that drops the guard or copies some row. A command of the same model that reads no
//! row set is generated as before.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{CapabilityKind, SynthesisDisposition, SynthesisPlan};

const READS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/filtered-related-reads.yaml");
const UNIQUE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/unique-within-scope.yaml");

fn compile_text(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("model.yaml".to_owned(), text.to_owned());
    compile_locating(&specification, &sources, &["model.yaml".to_owned()])
        .unwrap_or_else(|diagnostics| panic!("resolves:\n{diagnostics}"))
}

/// Why the plan keeps `command` owed, or a panic where it does not.
fn owed(plan: &SynthesisPlan, command: &str) -> String {
    match plan.disposition_of(CapabilityKind::CommandBehavior, command) {
        Some(SynthesisDisposition::Obligation(obligation)) => obligation.reason.describes(),
        other => panic!("`{command}` stays an obligation, not {other:?}"),
    }
}

#[test]
fn a_row_set_command_stays_an_obligation_naming_the_row_set() {
    let reads = SynthesisPlan::of(&compile_text(READS));
    for (command, names) in [
        (
            "demo.jobs.Retry",
            "a guard over the rows a selector selects",
        ),
        (
            "demo.jobs.CheckLimit",
            "a guard over the rows a selector selects",
        ),
        (
            "demo.jobs.Close",
            "a guard over the rows a selector selects",
        ),
    ] {
        let why = owed(&reads, command);
        assert!(
            why.contains(names),
            "`{command}` is owed for {names}, not: {why}"
        );
    }
    let markdown = reads.to_markdown();
    assert!(
        markdown.contains("a guard over the rows a selector selects"),
        "{markdown}"
    );
    // The command that reads no row set is generated.
    assert_eq!(
        reads.disposition_of(CapabilityKind::CommandBehavior, "demo.jobs.Record"),
        Some(&SynthesisDisposition::Generated)
    );
    let unique = SynthesisPlan::of(&compile_text(UNIQUE));
    let why = owed(&unique, "demo.binding.BindIdentity");
    assert!(
        why.contains("a guard over the rows a selector selects"),
        "{why}"
    );
}

#[test]
fn a_filtered_read_alone_keeps_its_command_owed_by_name() {
    // `Retry` with its guards taken away: only the read of the selected row is left.
    let (head, tail) = READS.split_once("      - name: ambiguous").unwrap();
    let (_, rest) = tail.split_once("      - name: retried").unwrap();
    let plan = SynthesisPlan::of(&compile_text(&format!("{head}      - name: retried{rest}")));
    let why = owed(&plan, "demo.jobs.Retry");
    assert!(
        why.contains("a read of the one row a selector selects"),
        "the read is named: {why}"
    );
}
