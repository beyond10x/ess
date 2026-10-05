//! Adversary pass 1 against unit W1-1 (beyond10x/ess#429 re-keying updates, #458
//! `{subject: state}`).
//!
//! Each case drives the unit's own fixtures in a shape its tests do not: a rename beside views that
//! do not project the identity or are parameterised by another field, a struct identity, an
//! `instance:` input not named as the identity; `{subject: state}` on an `updates:`, a `deletes:` and
//! a held-state refusal, and below `ess/23` in `sets:` and an event payload alone. The interpreter
//! stands in for an honest target: a valid model whose synthesized suite the reference semantics
//! fails is a suite an honest implementation fails too.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_conformance::interpret::Interpreted;
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_primitives::node::Node;

const RENAME: &str = include_str!("fixtures/identity-changing-updates.yaml");
const STATE: &str = include_str!("fixtures/subject-state-source.yaml");

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replacen(from, to, 1);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("model.yaml"), raw)])
}

fn ir_of(text: &str) -> EssIr {
    let spec = assemble(text).unwrap_or_else(|errors| panic!("admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

/// The suite of `ir`, which must synthesize without a refusal.
fn suite_of(ir: &EssIr) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(ir);
    assert_eq!(synthesis.refusals.len(), 0, "{:#?}", synthesis.refusals);
    synthesis.suite
}

/// The interpreter, as the honest target, passes every scenario `text` synthesizes.
fn interpreter_passes(text: &str) -> ConformanceSuite {
    let ir = ir_of(text);
    let suite = suite_of(&ir);
    let verdicts = support_go::rust_outcomes(&suite, &Interpreted::for_model(ir));
    assert_eq!(
        support_go::not_passed(&verdicts),
        Vec::<&str>::new(),
        "{verdicts:#?}"
    );
    suite
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

// ---- #429: renames the unit's tests do not exercise ----------------------------------------------

/// A second stored field, a view projecting only `value`, and a view filtered by `owner`.
fn rename_beside_other_views() -> String {
    let text = replaced(
        RENAME,
        "      - {name: value, type: String}\n    lifecycle:",
        "      - {name: value, type: String}\n      - {name: owner, type: String}\n    lifecycle:",
    );
    let text = replaced(
        &text,
        "      - {name: value, type: String}\n    outcomes:",
        "      - {name: value, type: String}\n      - {name: owner, type: String}\n    outcomes:",
    );
    let text = replaced(
        &text,
        "sets: {value: input.value}",
        "sets: {value: input.value, owner: input.owner}",
    );
    replaced(
        &text,
        "components:\n",
        "  - name: demo.vault.SecretValues\n    source: demo.vault.Secret\n    consistency: read_your_writes\n    fields:\n      - {name: value, type: String}\n  - name: demo.vault.SecretsByOwner\n    source: demo.vault.Secret\n    consistency: read_your_writes\n    params: [{name: owner, type: String}]\n    filter: owner == param.owner\n    fields:\n      - {name: name, type: demo.vault.SecretName}\n      - {name: owner, type: String}\ncomponents:\n",
    )
}

#[test]
fn adv_rename_beside_unprojected_and_parameterised_views_passes_the_reference() {
    interpreter_passes(&rename_beside_other_views());
}

/// The identity a struct, written whole.
fn rename_struct_identity() -> String {
    replaced(
        RENAME,
        "  - {name: demo.vault.SecretName, kind: newtype, of: String}\n",
        "  - name: demo.vault.SecretName\n    kind: struct\n    fields:\n      - {name: shelf, type: String}\n      - {name: label, type: String}\n",
    )
}

/// Rewritten in correction round 1 at the coordinator's decision: a struct-identity rename is out of
/// scope for 0.54.0, so it is refused by name at the identity write (`unsupported_construct`, naming
/// struct identities) rather than by the incidental `==` over an aggregate in its collision answer.
#[test]
fn adv_rename_of_a_struct_identity_is_refused_by_name() {
    let errors = assemble(&rename_struct_identity())
        .err()
        .map(|errors| errors.as_slice().to_vec())
        .unwrap_or_default();
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::UnsupportedConstruct
                && error.location == "command.demo.vault.RenameSecret.outcomes.renamed.sets.name"
                && error.message.contains("struct")),
        "a struct identity's rename is refused by name: {errors:#?}"
    );
}

/// `instance:` reads an input not named as the identity.
fn rename_from_other_input() -> String {
    let text = replaced(
        RENAME,
        "    input:\n      - {name: name, type: demo.vault.SecretName}\n      - {name: new_name, type: demo.vault.SecretName}\n",
        "    input:\n      - {name: current, type: demo.vault.SecretName}\n      - {name: new_name, type: demo.vault.SecretName}\n",
    );
    let text = replaced(
        &text,
        "        instance: name\n        sets: {name: input.new_name}",
        "        instance: current\n        sets: {name: input.new_name}",
    );
    replaced(
        &text,
        "{name: input.name, new_name: input.new_name}",
        "{name: input.current, new_name: input.new_name}",
    )
}

#[test]
fn adv_rename_from_an_instance_input_named_otherwise_passes_the_reference() {
    interpreter_passes(&rename_from_other_input());
}

/// The collision answer declared after the rename: the design note gives it the row-set refusal's
/// own precedence, before every accepting branch, whatever the declaration order.
#[test]
fn adv_rename_with_the_collision_answer_declared_last_passes_the_reference() {
    let taken = "      - name: taken\n        when_related:\n          entity: demo.vault.Secret\n          where: name == input.new_name\n          exists: true\n        error: demo.vault.NameTaken\n";
    let text = replaced(RENAME, taken, "");
    let text = replaced(
        &text,
        "      - {name: no-such-secret, unknown_instance: true, error: demo.vault.NoSuchSecret}\n",
        &format!(
            "      - {{name: no-such-secret, unknown_instance: true, error: demo.vault.NoSuchSecret}}\n{taken}"
        ),
    );
    interpreter_passes(&text);
}

#[test]
fn adv_rename_back_to_a_previous_name_restores_the_row() {
    let ir = ir_of(RENAME);
    let send = |store: &Store, command: &str, input: &[(&str, &str)]| {
        let input: BTreeMap<String, Node> = input
            .iter()
            .map(|(field, value)| ((*field).to_owned(), text(value)))
            .collect();
        let mut steps = execute(
            &ir,
            store,
            &command.parse().unwrap(),
            &input,
            &Externals::Withheld,
        )
        .unwrap_or_else(|undetermined| panic!("{command}: {undetermined:?}"));
        assert_eq!(steps.len(), 1, "{steps:#?}");
        steps.remove(0)
    };
    let stored = send(
        &Store::default(),
        "demo.vault.StoreSecret",
        &[("name", "alpha"), ("value", "a")],
    );
    let there = send(
        &stored.next,
        "demo.vault.RenameSecret",
        &[("name", "alpha"), ("new_name", "gamma")],
    );
    let back = send(
        &there.next,
        "demo.vault.RenameSecret",
        &[("name", "gamma"), ("new_name", "alpha")],
    );
    assert_eq!(
        back.outcome.as_ref().map(ToString::to_string).as_deref(),
        Some("demo.vault.RenameSecret/renamed")
    );
    let entity = "demo.vault.Secret".parse().unwrap();
    assert!(back.next.instance_typed(&entity, &text("gamma")).is_none());
    assert_eq!(
        back.next
            .instance_typed(&entity, &text("alpha"))
            .map(|row| row.fields["value"].clone()),
        Some(text("a"))
    );
}

// ---- #458: `{subject: state}` in positions the unit's tests do not exercise ---------------------

/// `TouchDoc` updates without a move, reading the held state into `sets:` and an event, and
/// refuses an archived document with the held state in its error.
/// (`{subject: …}` on a `deletes:` is refused today, as before this unit, so none is declared.)
fn state_everywhere() -> String {
    let text = replaced(
        STATE,
        "  - name: demo.docs.DocArchived\n",
        "  - name: demo.docs.DocTouched\n    fields:\n      - {name: doc_id, type: demo.docs.DocId}\n      - {name: at, type: demo.docs.Doc.State}\n  - name: demo.docs.DocArchived\n",
    );
    let text = replaced(
        &text,
        "may: [demo.docs.CreateDoc, demo.docs.PublishDoc, demo.docs.ArchiveDoc]",
        "may: [demo.docs.CreateDoc, demo.docs.PublishDoc, demo.docs.ArchiveDoc, demo.docs.TouchDoc]",
    );
    let text = replaced(
        &text,
        "accepts: {commands: [demo.docs.CreateDoc, demo.docs.PublishDoc, demo.docs.ArchiveDoc]}",
        "accepts: {commands: [demo.docs.CreateDoc, demo.docs.PublishDoc, demo.docs.ArchiveDoc, demo.docs.TouchDoc]}",
    );
    let text = replaced(
        &text,
        "publishes: {events: [demo.docs.DocCreated, demo.docs.DocPublished, demo.docs.DocArchived]}",
        "publishes: {events: [demo.docs.DocCreated, demo.docs.DocPublished, demo.docs.DocArchived, demo.docs.DocTouched]}",
    );
    replaced(
        &text,
        "views:\n",
        "  - name: demo.docs.TouchDoc\n    input:\n      - {name: doc_id, type: demo.docs.DocId}\n    outcomes:\n      - name: frozen\n        when_subject_state: [Archived]\n        error: demo.docs.StateConflict\n        payload:\n          demo.docs.StateConflict: {doc_id: input.doc_id, current: {subject: state}, requested: Draft}\n      - name: touched\n        updates: demo.docs.Doc\n        instance: doc_id\n        sets: {previous: {subject: state}}\n        emits: [demo.docs.DocTouched]\n        payload:\n          demo.docs.DocTouched: {doc_id: input.doc_id, at: {subject: state}}\nviews:\n",
    )
}

/// Every `ExpectEvent` / `ExpectEventValues` of `event` in `suite`, by scenario, with the literal
/// payload values each compares.
fn expected_events(suite: &ConformanceSuite, event: &str) -> Vec<(String, BTreeMap<String, Node>)> {
    suite
        .scenarios
        .iter()
        .flat_map(|(id, scenario)| {
            scenario.steps.iter().filter_map(move |step| match step {
                ScenarioStep::ExpectEvent {
                    event: name,
                    payload,
                    ..
                } if name.to_string() == event => Some((id.to_string(), payload.clone())),
                ScenarioStep::ExpectEventValues {
                    event: name,
                    payload,
                    ..
                } if name.to_string() == event => Some((
                    id.to_string(),
                    payload
                        .iter()
                        .filter_map(|(field, value)| {
                            value.as_literal().map(|node| (field.clone(), node.clone()))
                        })
                        .collect(),
                )),
                _ => None,
            })
        })
        .collect()
}

#[test]
fn adv_subject_state_on_an_update_passes_the_reference() {
    interpreter_passes(&state_everywhere());
}

#[test]
fn adv_subject_state_on_an_update_event_is_compared() {
    let suite = suite_of(&ir_of(&state_everywhere()));
    for (event, field) in [("demo.docs.DocTouched", "at")] {
        let expected = expected_events(&suite, event);
        assert!(!expected.is_empty(), "{event} is expected somewhere");
        for (id, payload) in expected {
            assert!(
                matches!(payload.get(field), Some(Node::Text(state)) if state == "Draft" || state == "Published"),
                "{id}: `{event}.{field}` reads the held state and is not compared with it: {payload:?}"
            );
        }
    }
}

#[test]
fn adv_subject_state_on_a_held_state_refusal_is_compared() {
    let suite = suite_of(&ir_of(&state_everywhere()));
    let mut seen = 0;
    for (id, scenario) in &suite.scenarios {
        for step in &scenario.steps {
            if let ScenarioStep::ExpectError { error, fields } = step {
                if error.to_string() != "demo.docs.StateConflict"
                    || fields.get("requested") != Some(&text("Draft"))
                {
                    continue;
                }
                seen += 1;
                assert_eq!(
                    fields.get("current"),
                    Some(&text("Archived")),
                    "{id}: the frozen refusal answers the held state"
                );
            }
        }
    }
    assert!(seen > 0, "the frozen refusal is exercised");
}

/// The model below `ess/23` with only `source` left reading `{subject: state}`; the one error.
fn below_23_with_only(keep: &str) -> ess_primitives::error::ValidationError {
    let mut text = replaced(STATE, "format: ess/23\n", "format: ess/22\n");
    for (from, to) in [
        (
            "          demo.docs.DocPublished: {doc_id: input.doc_id, from: {subject: state}}\n",
            "          demo.docs.DocPublished: {doc_id: input.doc_id, from: Draft}\n",
        ),
        (
            "        sets: {previous: {subject: state}}\n",
            "        sets: {previous: Published}\n",
        ),
        (
            "{doc_id: input.doc_id, current: {subject: state}, requested: Published}",
            "{doc_id: input.doc_id, current: Published, requested: Published}",
        ),
        (
            "{doc_id: input.doc_id, current: {subject: state}, requested: Archived}",
            "{doc_id: input.doc_id, current: Published, requested: Archived}",
        ),
    ] {
        if from != keep {
            text = replaced(&text, from, to);
        }
    }
    let errors = assemble(&text).expect_err("refused").as_slice().to_vec();
    assert_eq!(errors.len(), 1, "{errors:#?}");
    errors.into_iter().next().unwrap()
}

#[test]
fn adv_subject_state_in_sets_alone_is_refused_below_ess23() {
    let error = below_23_with_only("        sets: {previous: {subject: state}}\n");
    assert_eq!(
        error.code,
        ValidationCode::UnsupportedFormatVersion,
        "{error:#?}"
    );
    assert_eq!(
        error.location,
        "command.demo.docs.ArchiveDoc.outcomes.archived.sets.previous"
    );
    assert!(error.message.contains("ess/23"), "{error:#?}");
}

#[test]
fn adv_subject_state_in_an_event_payload_alone_is_refused_below_ess23() {
    let error = below_23_with_only(
        "          demo.docs.DocPublished: {doc_id: input.doc_id, from: {subject: state}}\n",
    );
    assert_eq!(
        error.code,
        ValidationCode::UnsupportedFormatVersion,
        "{error:#?}"
    );
    assert_eq!(
        error.location,
        "command.demo.docs.PublishDoc.outcomes.published.payload.demo.docs.DocPublished.from"
    );
    assert!(error.message.contains("ess/23"), "{error:#?}");
}

// ---- ess/22: an old document whose row-set selector reads the identity ---------------------------

/// An `ess/22` document, no identity write and no `{subject: state}`: `NameSuccessor` refuses a
/// successor identity a team already carries (`when_related … where: team_id ==
/// input.successor_id, exists: true`). Nothing in it is new in `ess/23`.
const ESS22_IDENTITY_SELECTOR: &str =
    include_str!("fixtures/adversary-w1-1-identity-selector-ess22.yaml");

/// The canonical suite and refusals, exactly as `adversary_w3_266_267_dump.rs` writes them for a
/// single-file model labelled by its file name.
fn dumped(label: &str, text: &str) -> String {
    use std::fmt::Write as _;
    let mut sources = SourceMap::new();
    sources.insert(label.to_owned(), text.to_owned());
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new(label.to_owned()), raw)]).unwrap();
    let ir = compile(&spec, &sources).unwrap();
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let mut out = synthesis.suite.to_canonical_json().unwrap();
    out.push_str("\n---- refusals\n");
    for refusal in &synthesis.refusals {
        writeln!(out, "{refusal}").unwrap();
    }
    out
}

/// `adversary_w3_266_267_dump.rs` run at the unit's base 9ffa94d66a over this file, labelled
/// `adv-identity-selector-present.yaml`, wrote a document whose SHA-256 is the literal below: the
/// suite synthesized `FormTeam/*` and `NameSuccessor/outcome/no-such-team`, and refused
/// `NameSuccessor/successor-is-a-team` and `NameSuccessor/named` as "its row set is Unknown". The
/// unit's own invariant: an old format keeps its bytes.
#[test]
fn adv_an_ess22_identity_selector_keeps_its_suite_bytes() {
    use sha2::{Digest, Sha256};
    let out = dumped(
        "adv-identity-selector-present.yaml",
        ESS22_IDENTITY_SELECTOR,
    );
    let digest = Sha256::digest(out.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            use std::fmt::Write as _;
            write!(hex, "{byte:02x}").unwrap();
            hex
        });
    assert_eq!(
        digest,
        "c9cc15173d057fbe168d1a35738c4cb58f133d5888d7ef7c7300c8db64fa256c",
        "the ess/22 suite moved:\n{}",
        &out[out.find("---- refusals").unwrap_or(0)..]
    );
}

/// Where the suite of the `ess/22` document did move, the scenarios it gained are ones the
/// reference semantics passes.
#[test]
fn adv_an_ess22_identity_selector_suite_passes_the_reference() {
    let ir = ir_of(ESS22_IDENTITY_SELECTOR);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let verdicts = support_go::rust_outcomes(&synthesis.suite, &Interpreted::for_model(ir));
    assert_eq!(
        support_go::not_passed(&verdicts),
        Vec::<&str>::new(),
        "{verdicts:#?}"
    );
}
