//! A guard mutant that leaves its branch selected on no row is scored `equivalent`
//! (`ESS-MUTATE-005`) beside a `when_subject:` guard too: where every input its `when:` admits
//! leaves out a field the stored guard can hold only by comparing, that guard is `Unknown`, never
//! `True`, on every row (story `negated-defined-optional-input-witnessed-beside-when-subject`).
//!
//! `stale-version: when: defined(expected_version)` beside
//! `when_subject: state == Open and version != input.expected_version` is negated into
//! `not defined(expected_version)`: only an absent `expected_version` satisfies it, and
//! `version != input.expected_version` over an absent operand is `Unknown`. The mutant's suite
//! refuses the branch (`ESS-SYNTH-003`); it was `unwitnessed`. A stored guard holding through a
//! test that reads no input, or through another disjunct, keeps its scoring.
#![allow(clippy::missing_panics_doc)]

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, MutationReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::Value;

const ORDER: &str = include_str!("fixtures/negated-defined-beside-subject.yaml");
const NEGATE: &str = "guard-negate/demo.order.Fulfil/stale-version";
const COMPARED: &str = "              - version != input.expected_version\n";

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let label = "spec.yaml".to_owned();
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text.to_owned());
    (vec![(Source::new(label), raw)], texts)
}

fn audit(text: &str) -> MutationReport {
    let (files, texts) = documents(text);
    let ir = mutate::compile(files.clone(), &texts).expect("the specification compiles");
    mutate::audit(&files, &texts, &[MutantClass::GuardNegate], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn negate(text: &str) -> (Value, String) {
    let report = audit(text);
    let rendered = report.render_text();
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    let entry = json["mutants"]
        .as_array()
        .expect("mutants")
        .iter()
        .find(|it| it["id"] == NEGATE)
        .unwrap_or_else(|| panic!("no {NEGATE} in {json:#}"))
        .clone();
    (entry, rendered)
}

fn edited(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "`{from}` occurs once");
    text.replace(from, to)
}

#[test]
fn the_negated_defined_guard_beside_the_comparison_is_equivalent_naming_the_dead_guard() {
    let (entry, text) = negate(ORDER);
    assert_eq!(entry["verdict"], "equivalent", "{text}");
    let dead = entry["unsatisfiable_guard"]
        .as_str()
        .unwrap_or_else(|| panic!("an unsatisfiable guard is named: {entry:#}"));
    assert!(
        dead.contains("not (defined(expected_version))") && dead.contains("input.expected_version"),
        "the guard named is the mutant's `when:` beside its stored guard: {dead}"
    );
    assert!(
        text.lines()
            .any(|line| line.starts_with(&format!("equivalent {NEGATE}: "))
                && line.contains("ESS-MUTATE-005")),
        "{text}"
    );
}

/// A stored guard that reads no input holds with `expected_version` absent: the mutant's branch is
/// taken there, and the baseline's scenario sending it absent tells the two apart.
#[test]
fn beside_a_stored_guard_reading_no_input_the_mutant_keeps_its_scoring() {
    let (entry, text) = negate(&edited(ORDER, COMPARED, "              - version != 0\n"));
    assert!(entry.get("unsatisfiable_guard").is_none(), "{entry:#}");
    assert_eq!(entry["verdict"], "killed", "{text}");
}

/// The comparison is one disjunct: the other can hold on a row whatever the input carries, so the
/// mutant's branch is not dead and no `unsatisfiable_guard` is named.
#[test]
fn a_comparison_beside_another_disjunct_is_not_dead() {
    let text = edited(
        ORDER,
        "            all:\n              - state == Open\n              - version != input.expected_version\n",
        "            any:\n              - version != input.expected_version\n              - version == 7\n",
    );
    let (entry, rendered) = negate(&text);
    assert!(entry.get("unsatisfiable_guard").is_none(), "{entry:#}");
    assert_ne!(entry["verdict"], "equivalent", "{rendered}");
}

const SIGN_IN: &str = include_str!("fixtures/related-guard-sign-in.yaml");
const RELATED_NEGATE: &str = "guard-negate/demo.signin.InitiateSignIn/no-redirect-entry";

/// The sign-in fixture with the related row compared with an `Optional` input, the branch guarded
/// by that input being present: `when:` beside `when_related:`, the same evaluation as beside
/// `when_subject:` (the row's predicate and the input guard, joined by Kleene `and`).
fn related(predicate: &str) -> String {
    let text = edited(SIGN_IN, "format: ess/18\n", "format: ess/23\n");
    let text = edited(
        &text,
        "      - {name: client, type: demo.signin.ClientId}\n    outcomes:",
        "      - {name: client, type: demo.signin.ClientId}\n      - {name: expected_client, type: Optional<demo.signin.ClientId>}\n    outcomes:",
    );
    edited(
        &text,
        "        when_related: {via: input.tenant, predicate: redirect_client != input.client}\n",
        &format!(
            "        when: defined(expected_client)\n        when_related: {{via: input.tenant, predicate: \"{predicate}\"}}\n"
        ),
    )
}

fn negate_related(text: &str) -> (Value, String) {
    let report = audit(text);
    let rendered = report.render_text();
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    let entry = json["mutants"]
        .as_array()
        .expect("mutants")
        .iter()
        .find(|it| it["id"] == RELATED_NEGATE)
        .unwrap_or_else(|| panic!("no {RELATED_NEGATE} in {json:#}"))
        .clone();
    (entry, rendered)
}

/// Beside `when_related:` the same negation is already told apart: the mutant's suite answers its
/// accepting branch where the baseline refuses, so it is `killed`, and the rule for `when_subject:`
/// names no `unsatisfiable_guard` here and does not turn that verdict into `equivalent`.
#[test]
fn the_negated_defined_guard_beside_a_related_comparison_stays_killed() {
    let (entry, text) = negate_related(&related("redirect_client != input.expected_client"));
    assert!(entry.get("unsatisfiable_guard").is_none(), "{entry:#}");
    assert_eq!(entry["verdict"], "killed", "{text}");
}

/// A related predicate reading the other, required input holds with `expected_client` absent: the
/// mutant's branch is reachable and no `unsatisfiable_guard` is named.
#[test]
fn beside_a_related_predicate_reading_a_present_input_the_mutant_is_not_dead() {
    let (entry, text) = negate_related(&related("redirect_client != input.client"));
    assert!(entry.get("unsatisfiable_guard").is_none(), "{entry:#}");
    assert_ne!(entry["verdict"], "equivalent", "{text}");
}

/// `not (version == input.expected_version)` is `Unknown` with the operand absent, as the
/// comparison itself is: negation does not make an undecided comparison hold.
#[test]
fn a_negated_comparison_of_the_absent_input_is_dead_too() {
    let (entry, text) = negate(&edited(
        ORDER,
        COMPARED,
        "              - {not: \"version == input.expected_version\"}\n",
    ));
    assert_eq!(entry["verdict"], "equivalent", "{text}");
    assert!(entry["unsatisfiable_guard"].is_string(), "{entry:#}");
}
