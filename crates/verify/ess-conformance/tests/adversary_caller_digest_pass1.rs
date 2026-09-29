//! Adversary pass 1 for beyond10x/ess#216: every entry point that synthesizes a suite from a model
//! reading the caller stamps that model's digests, two models differing only in caller data never
//! share a suite digest, and the suite still tells them apart by content.
#![allow(clippy::too_many_lines, clippy::missing_panics_doc)]

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::coverage::{Origins, Scope};
use ess_conformance::mutate::{self, MutantClass, MODEL_FILE};
use ess_conformance::{ConformanceSuite, SuiteProvenance};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// A caller-reading model; `{ATTRIBUTE}` is replaced by the attribute the created event's
/// `account_id` is filled from, and `{EXTRA}` by further actor attributes.
const TEMPLATE: &str = "format: ess/16
system: demo
version: v1
summary: Minimal repro.
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}
  - {name: demo.notes.AgentId, kind: newtype, of: Uuid}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.notes.AccountUser
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: billing_account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
{EXTRA}    may: [demo.notes.CreateNote, demo.notes.EditNote]
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: {caller: account_id}, agent_id: {caller: agent_id}, text: input.text}
        emits: [demo.notes.NoteCreated]
        payload:
          demo.notes.NoteCreated: {note_id: {generated: true}, account_id: {caller: {ATTRIBUTE}}, text: input.text}
  - name: demo.notes.EditNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: forbidden
        when_subject: {predicate: agent_id != caller.agent_id}
        error: demo.notes.NotYourNote
      - name: edited
        updates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
        emits: [demo.notes.NoteEdited]
        payload:
          demo.notes.NoteEdited: {note_id: input.note_id, text: input.text}
errors:
  - name: demo.notes.NotYourNote
    summary: The caller is not the note's agent.
events:
  - name: demo.notes.NoteCreated
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
      - {name: text, type: String}
  - name: demo.notes.NoteEdited
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
views:
  - name: demo.notes.NoteDetails
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
      - {name: state, type: demo.notes.Note.State}
";

fn text(attribute: &str, extra: &str) -> String {
    TEMPLATE
        .replace("{ATTRIBUTE}", attribute)
        .replace("{EXTRA}", extra)
}

/// The base model: the created event records the caller's `account_id`.
fn base() -> String {
    text("account_id", "")
}

/// Differs from [`base`] only in which same-typed caller attribute the event reads.
fn other_source() -> String {
    text("billing_account_id", "")
}

/// Differs from [`base`] only in one more actor attribute nothing reads.
fn other_attribute() -> String {
    text(
        "account_id",
        "      - {name: region_id, type: demo.notes.AccountId}\n",
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn sha256(text: &str) -> String {
    use sha2::{Digest as _, Sha256};
    use std::fmt::Write as _;
    let mut out = String::new();
    for byte in Sha256::digest(text.as_bytes()) {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// The suite's scenarios with its provenance replaced by `model`'s, so two suites compare by content.
fn content(suite: &ConformanceSuite, model: &EssIr) -> String {
    let mut suite = suite.clone();
    suite.provenance = SuiteProvenance::of(model);
    serde_json::to_string(&suite.scenarios).expect("the scenarios serialize")
}

/// The coverage inventory (`--suite-format 5`, `--report-format 2`), the component-free replay
/// admission and the mutation emission all read the suite synthesis stamps.
#[test]
fn every_entry_point_stamps_the_model_digests_on_a_caller_suite() {
    let text = base();
    let model = ir(&text);
    let expected = SuiteProvenance::of(&model);

    let synthesized = ess_conformance::synthesize::synthesize(&model).suite;
    assert_eq!(synthesized.provenance.spec_digest, expected.spec_digest);
    assert_eq!(
        synthesized.provenance.contract_digest,
        expected.contract_digest
    );
    assert_eq!(synthesized.provenance.system, expected.system);
    assert_eq!(
        synthesized.provenance.specification_version,
        expected.specification_version
    );

    let input =
        ess_conformance::coverage_build::build(&model, &[], Scope::System, Origins::Generated)
            .expect("the coverage inventory of a caller model is admitted");
    let covered = &input.selected().suite().provenance;
    assert_eq!(covered.spec_digest, expected.spec_digest, "input/1 suite");
    assert_eq!(
        covered.contract_digest, expected.contract_digest,
        "input/1 suite"
    );
    ess_conformance::web_replay::AdmittedReplay::new(&model, &input)
        .expect("the replay admits the caller model's own inventory");

    let mut texts = SourceMap::new();
    texts.insert("notes.yaml", text.as_str());
    let documents = vec![(
        Source::new("notes.yaml"),
        RawSpecFile::parse(&text).expect("parses"),
    )];
    let emission = mutate::emit(&documents, &texts, MutantClass::ALL).expect("emits");
    assert_eq!(
        emission.manifest.spec_digest,
        expected.spec_digest.to_string(),
        "the manifest names the model"
    );
    assert_eq!(
        emission.manifest.baseline.spec_digest,
        expected.spec_digest.to_string(),
        "the baseline suite names the model"
    );
    // This model's only mutants (emit-drop) are stillborn; any live one is checked.
    for mutant in &emission.manifest.mutants {
        let (Some(dir), Some(digest)) = (&mutant.dir, &mutant.spec_digest) else {
            continue;
        };
        let written = &emission.files[&format!("{dir}/{MODEL_FILE}")];
        assert_eq!(
            digest,
            &sha256(written.trim_end_matches('\n')),
            "mutant `{}`: its suite names the model written beside it",
            mutant.id
        );
    }
}

/// Models that differ only in caller data never share a suite digest, and each suite names its own.
#[test]
fn models_differing_only_in_caller_data_carry_different_suite_digests() {
    let texts = [base(), other_source(), other_attribute()];
    let mut digests = Vec::new();
    for text in &texts {
        let model = ir(text);
        let suite = ess_conformance::synthesize::synthesize(&model).suite;
        assert_eq!(
            suite.provenance.spec_digest,
            SuiteProvenance::of(&model).spec_digest
        );
        digests.push(suite.provenance.spec_digest.to_string());
    }
    assert_ne!(digests[0], digests[1], "a different caller source");
    assert_ne!(digests[0], digests[2], "a different actor attribute");
    assert_ne!(digests[1], digests[2]);
}

/// Class 1 (equal witness values): a suite synthesized from a model whose event records the
/// caller's `account_id` must not be the suite of one that records the same-typed
/// `billing_account_id` — otherwise an implementation reading the wrong attribute passes the suite
/// that now carries the first model's digest.
#[test]
fn a_different_same_typed_caller_source_changes_the_suite_contents() {
    let first = ir(&base());
    let second = ir(&other_source());
    let one = ess_conformance::synthesize::synthesize(&first).suite;
    let other = ess_conformance::synthesize::synthesize(&second).suite;
    assert_ne!(
        content(&one, &first),
        content(&other, &first),
        "the two suites are the same scenarios, so a target that records `billing_account_id` \
         where the specification reads `account_id` passes"
    );
}

/// Synthesis of a caller model is a function of the model: the same bytes twice.
#[test]
fn a_caller_suite_is_synthesized_to_the_same_bytes_twice() {
    let model = ir(&base());
    let one = ess_conformance::synthesize::synthesize(&model)
        .suite
        .to_canonical_json()
        .expect("admitted");
    let two = ess_conformance::synthesize::synthesize(&ir(&base()))
        .suite
        .to_canonical_json()
        .expect("admitted");
    assert_eq!(one, two);
}
