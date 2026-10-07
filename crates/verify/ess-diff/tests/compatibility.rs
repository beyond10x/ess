//! beyond10x/ess#290: whether each change breaks someone, on which side, and a gate that fails on it.
//!
//! The issue's reproduction is a variant removed from a type that is both a command input and a
//! stored field; `ess verify diff` called it `narrowed` and exited 0, and the relation alone could
//! not say whether a caller or a reader is the one it breaks. Each case below builds the same small
//! `demo.calls` system with `demo.calls.Channel` placed in a different role, so the answer is read
//! off the role rather than off the relation.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::struct_excessive_bools
)]

mod support;

use std::collections::BTreeSet;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::compatibility::{
    AcknowledgementRefusal, Acknowledgements, ChangeCompatibility, Compatibility, Dimension,
    FailOn, Gate, TypeUse,
};
use ess_diff::{EssDelta, RawEssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationCode;

/// Where `demo.calls.Channel` appears.
#[derive(Clone, Copy, Default)]
struct Roles {
    /// A command input field of type `Channel`.
    input: bool,
    /// A command input field of a struct that holds a `Channel`.
    nested_input: bool,
    /// An entity field.
    entity: bool,
    /// An event field.
    event: bool,
    /// An error field.
    error: bool,
}

fn field(on: bool, name: &str, of: &str) -> String {
    if on {
        format!("      - name: {name}\n        type: {of}\n")
    } else {
        String::new()
    }
}

/// The `demo.calls` system with `Channel`'s variants and roles as given.
fn text(variants: &str, roles: Roles) -> String {
    format!(
        "format: ess/5
system: demo
version: v1
domains: [demo.calls]
domain: demo.calls
types:
  - name: demo.calls.CallId
    kind: newtype
    of: Uuid
  - name: demo.calls.Channel
    kind: enum
    variants: [{variants}]
  - name: demo.calls.CallSpec
    kind: struct
    fields:
      - name: channel
        type: demo.calls.Channel
entities:
  - name: demo.calls.Call
    identity:
      name: call_id
      type: demo.calls.CallId
    fields:
      - name: note
        type: String
{entity}    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - name: close
          from: [Open]
          to: Closed
errors:
  - name: demo.calls.Refused
    fields:
      - name: reason
        type: String
{error}commands:
  - name: demo.calls.OpenCall
    input:
      - name: note
        type: String
{input}{nested}    outcomes:
      - name: opened
        when: note != \"\"
        creates: demo.calls.Call
        instance: call_id
        emits:
          - demo.calls.CallOpened
        payload:
          demo.calls.CallOpened:
            call_id: {{generated: true}}
{event_payload}      - name: refused
        error: demo.calls.Refused
  - name: demo.calls.CloseCall
    input:
      - name: call_id
        type: demo.calls.CallId
    outcomes:
      - name: closed
        moves: demo.calls.Call.close
        instance: call_id
        emits:
          - demo.calls.CallClosed
        payload:
          demo.calls.CallClosed:
            call_id: input.call_id
events:
  - name: demo.calls.CallClosed
    fields:
      - name: call_id
        type: demo.calls.CallId
  - name: demo.calls.CallOpened
    fields:
      - name: call_id
        type: demo.calls.CallId
{event}",
        entity = field(roles.entity, "channel", "demo.calls.Channel"),
        error = field(roles.error, "channel", "demo.calls.Channel"),
        input = field(roles.input, "channel", "demo.calls.Channel"),
        nested = field(roles.nested_input, "spec", "demo.calls.CallSpec"),
        event = field(roles.event, "channel", "demo.calls.Channel"),
        event_payload = if roles.event {
            "            channel: {generated: true}\n"
        } else {
            ""
        },
    )
}

fn model(text: &str) -> EssIr {
    let specification =
        Specification::assemble([(Source::new("calls.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("calls.yaml", text);
    compile(&specification, &sources).unwrap_or_else(|errors| panic!("{errors}"))
}

const THREE: &str = "Voice, Video, Chat";
const TWO: &str = "Voice, Video";

/// The classified delta between `before` and `after` variants, `Channel` in `roles` on both sides.
fn classified(before: &str, after: &str, roles: Roles) -> EssDelta {
    ess_diff::classified(&model(&text(before, roles)), &model(&text(after, roles))).unwrap()
}

/// The one change's id and classification.
fn only(delta: &EssDelta) -> (String, ChangeCompatibility) {
    assert_eq!(delta.len(), 1, "{:#?}", delta.changes());
    let change = &delta.changes()[0];
    let compatibility = delta
        .compatibility_of(&change.id())
        .expect("a classified delta classifies every change")
        .clone();
    (change.id().to_string(), compatibility)
}

fn uses(of: &[TypeUse]) -> BTreeSet<TypeUse> {
    of.iter().copied().collect()
}

/// The reproduction in the issue: `Chat` removed from a type a caller sends and the system stores.
#[test]
fn the_issue_reproduction_is_breaking_for_callers_and_history_and_not_for_readers() {
    let roles = Roles {
        input: true,
        entity: true,
        ..Roles::default()
    };
    let delta = classified(THREE, TWO, roles);
    let (id, compatibility) = only(&delta);

    assert_eq!(id, "type/demo.calls.Channel/variant-removed/Chat");
    assert_eq!(compatibility.verdict(), Compatibility::Breaking);
    assert_eq!(compatibility.callers(), Compatibility::Breaking);
    assert_eq!(compatibility.readers(), Compatibility::Compatible);
    assert_eq!(compatibility.history(), Compatibility::Breaking);
    assert_eq!(
        compatibility.uses(),
        Some(&uses(&[TypeUse::Input, TypeUse::Stored]))
    );
}

#[test]
fn a_narrowing_of_a_type_only_the_system_writes_breaks_no_reader() {
    let roles = Roles {
        error: true,
        ..Roles::default()
    };
    let (_, compatibility) = only(&classified(THREE, TWO, roles));
    assert_eq!(compatibility.uses(), Some(&uses(&[TypeUse::Output])));
    assert_eq!(compatibility.readers(), Compatibility::Compatible);
    assert_eq!(compatibility.verdict(), Compatibility::Compatible);
}

#[test]
fn a_widening_of_a_type_only_the_system_writes_breaks_its_readers() {
    let roles = Roles {
        error: true,
        ..Roles::default()
    };
    let (id, compatibility) = only(&classified(TWO, THREE, roles));
    assert_eq!(id, "type/demo.calls.Channel/variant-added/Chat");
    assert_eq!(compatibility.readers(), Compatibility::Breaking);
    assert_eq!(compatibility.callers(), Compatibility::Compatible);
    assert_eq!(compatibility.verdict(), Compatibility::Breaking);
}

#[test]
fn a_widening_of_a_type_only_callers_send_breaks_nobody_and_its_narrowing_breaks_callers() {
    let roles = Roles {
        input: true,
        ..Roles::default()
    };
    let (_, widened) = only(&classified(TWO, THREE, roles));
    assert_eq!(widened.uses(), Some(&uses(&[TypeUse::Input])));
    assert_eq!(widened.verdict(), Compatibility::Compatible);

    let (_, narrowed) = only(&classified(THREE, TWO, roles));
    assert_eq!(narrowed.callers(), Compatibility::Breaking);
    assert_eq!(narrowed.readers(), Compatibility::Compatible);
    assert_eq!(narrowed.history(), Compatibility::Compatible);
    assert_eq!(narrowed.verdict(), Compatibility::Breaking);
}

#[test]
fn a_type_reached_only_through_a_struct_a_caller_sends_is_a_caller_input() {
    let roles = Roles {
        nested_input: true,
        ..Roles::default()
    };
    let (_, compatibility) = only(&classified(THREE, TWO, roles));
    assert_eq!(compatibility.uses(), Some(&uses(&[TypeUse::Input])));
    assert_eq!(compatibility.callers(), Compatibility::Breaking);
}

#[test]
fn an_event_field_is_read_by_readers_and_by_whoever_replays_history() {
    let roles = Roles {
        event: true,
        ..Roles::default()
    };
    let (_, narrowed) = only(&classified(THREE, TWO, roles));
    assert_eq!(
        narrowed.uses(),
        Some(&uses(&[TypeUse::Output, TypeUse::Stored]))
    );
    assert_eq!(narrowed.readers(), Compatibility::Compatible);
    assert_eq!(narrowed.history(), Compatibility::Breaking);

    let (_, widened) = only(&classified(TWO, THREE, roles));
    assert_eq!(widened.readers(), Compatibility::Breaking);
    assert_eq!(widened.history(), Compatibility::Compatible);
}

#[test]
fn a_type_nothing_uses_breaks_nobody_either_way() {
    for (before, after) in [(THREE, TWO), (TWO, THREE)] {
        let (_, compatibility) = only(&classified(before, after, Roles::default()));
        assert_eq!(compatibility.uses(), Some(&BTreeSet::new()));
        assert_eq!(compatibility.verdict(), Compatibility::Compatible);
    }
}

/// A wire rename has no direction and needs a consumer assumption the model does not hold.
#[test]
fn a_wire_change_on_a_type_a_caller_sends_is_unknown_not_compatible() {
    let roles = Roles {
        input: true,
        ..Roles::default()
    };
    let delta = classified(
        "Voice, Video, Chat",
        "Voice, Video, {name: Chat, wire: text}",
        roles,
    );
    let (id, compatibility) = only(&delta);
    assert_eq!(id, "type/demo.calls.Channel/variant-wire-name-changed/Chat");
    assert_eq!(compatibility.callers(), Compatibility::Unknown);
    assert_eq!(compatibility.readers(), Compatibility::Compatible);
    assert_eq!(compatibility.verdict(), Compatibility::Unknown);
}

/// Grants, a predicate rewrite and a nested variant change, on the committed revision pair.
#[test]
fn the_revision_pair_classifies_grants_variants_and_predicates() {
    let before = support::compiled("examples/revision-pair/before");
    let after = support::compiled("examples/revision-pair/after");
    let delta = ess_diff::classified(&before, &after).unwrap();
    let verdict = |id: &str| {
        let change = delta
            .changes()
            .iter()
            .find(|change| change.id().to_string() == id)
            .unwrap_or_else(|| panic!("{id} is in the delta"));
        delta.compatibility_of(&change.id()).unwrap().clone()
    };

    let removed =
        verdict("actor/catalog.pricing.Auditor/grant-removed/catalog.pricing.RetirePriceList");
    assert_eq!(removed.callers(), Compatibility::Breaking);
    assert_eq!(removed.uses(), None, "only a type change records uses");
    let added =
        verdict("actor/catalog.pricing.PricingManager/grant-added/catalog.pricing.RetirePriceList");
    assert_eq!(added.verdict(), Compatibility::Compatible);

    // `Currency` is reached through `Money` (a command input, an error field and an entity field).
    let gbp = verdict("type/catalog.pricing.Currency/variant-removed/GBP");
    assert_eq!(
        gbp.uses(),
        Some(&uses(&[TypeUse::Input, TypeUse::Output, TypeUse::Stored]))
    );
    assert_eq!(gbp.callers(), Compatibility::Breaking);
    assert_eq!(gbp.history(), Compatibility::Breaking);
    let chf = verdict("type/catalog.pricing.Currency/variant-added/CHF");
    assert_eq!(chf.readers(), Compatibility::Breaking);
    assert_eq!(chf.callers(), Compatibility::Compatible);

    let invariant = verdict("entity/catalog.pricing.PriceList/invariants-changed");
    assert_eq!(invariant.verdict(), Compatibility::Unknown);
    let condition =
        verdict("command/catalog.pricing.CreatePriceList/outcome-condition-changed/created");
    assert_eq!(condition.verdict(), Compatibility::Unknown);
}

#[test]
fn a_documentation_only_change_is_compatible() {
    let roles = Roles {
        input: true,
        ..Roles::default()
    };
    let delta = classified(
        THREE,
        "Voice, Video, {name: Chat, summary: Text messages instead of a voice line}",
        roles,
    );
    let (id, compatibility) = only(&delta);
    assert_eq!(id, "type/demo.calls.Channel/variant-summary-changed/Chat");
    assert_eq!(compatibility.verdict(), Compatibility::Compatible);
}

/// Classification is opt-in: the delta every existing caller gets keeps its format and bytes.
#[test]
fn an_unclassified_delta_keeps_its_format_and_carries_no_compatibility() {
    let roles = Roles {
        input: true,
        entity: true,
        ..Roles::default()
    };
    let before = model(&text(THREE, roles));
    let after = model(&text(TWO, roles));
    let plain = ess_diff::diff(&before, &after).unwrap();
    assert_eq!(plain.format.to_string(), "ess-diff/2");
    assert!(plain.compatibility().is_none());
    let json = plain.to_canonical_json();
    assert!(!json.contains("compatibility"), "{json}");

    let classified = ess_diff::classified(&before, &after).unwrap();
    assert_eq!(classified.format.to_string(), "ess-diff/14");
    assert_eq!(classified.changes(), plain.changes());
}

fn read(json: &str) -> Result<EssDelta, Vec<ValidationCode>> {
    let raw: RawEssDelta = serde_json::from_str(json).unwrap();
    EssDelta::try_from(raw).map_err(|errors| errors.as_slice().iter().map(|e| e.code).collect())
}

#[test]
fn a_classified_delta_is_ess_diff_14_and_reads_back_byte_identical() {
    let roles = Roles {
        input: true,
        entity: true,
        ..Roles::default()
    };
    let delta = classified(THREE, TWO, roles);
    let json = delta.to_canonical_json();
    assert!(json.contains("\"format\": \"ess-diff/14\""), "{json}");
    assert!(json.contains("\"verdict\": \"breaking\""), "{json}");
    assert!(json.contains("\"uses\": [\n"), "{json}");

    let read = read(&json).unwrap();
    assert_eq!(read, delta);
    assert_eq!(read.to_canonical_json(), json);
}

#[test]
fn a_reader_refuses_a_classification_its_own_content_does_not_derive() {
    let roles = Roles {
        input: true,
        entity: true,
        ..Roles::default()
    };
    let json = classified(THREE, TWO, roles).to_canonical_json();

    let softened = json.replacen(
        "\"verdict\": \"breaking\"",
        "\"verdict\": \"compatible\"",
        1,
    );
    assert_eq!(
        read(&softened).unwrap_err(),
        vec![ValidationCode::ConflictingDeclaration]
    );

    let callers = json.replacen(
        "\"callers\": \"breaking\"",
        "\"callers\": \"compatible\"",
        1,
    );
    assert!(read(&callers)
        .unwrap_err()
        .contains(&ValidationCode::ConflictingDeclaration));

    let unused = json.replacen("\"input\",\n", "", 1);
    assert!(read(&unused)
        .unwrap_err()
        .contains(&ValidationCode::ConflictingDeclaration));
}

#[test]
fn a_classification_is_refused_below_ess_diff_14_and_required_from_it() {
    let roles = Roles {
        input: true,
        entity: true,
        ..Roles::default()
    };
    let classified = classified(THREE, TWO, roles);
    let json = classified.to_canonical_json();

    let relabelled = json.replace("ess-diff/14", "ess-diff/13");
    assert_eq!(
        read(&relabelled).unwrap_err(),
        vec![ValidationCode::UnsupportedFormatVersion]
    );

    let before = model(&text(THREE, roles));
    let after = model(&text(TWO, roles));
    let plain = ess_diff::diff(&before, &after)
        .unwrap()
        .to_canonical_json()
        .replace("ess-diff/2", "ess-diff/14");
    assert_eq!(
        read(&plain).unwrap_err(),
        vec![ValidationCode::MissingDeclaration]
    );

    let refused = classified.to_canonical_json_for("ess-diff/13".parse().unwrap());
    assert!(
        matches!(
            refused,
            Err(ess_diff::DeltaWriteRefusal::Unclassifiable { .. })
        ),
        "{refused:?}"
    );
}

fn acknowledgements(delta: &EssDelta, ids: &[&str]) -> String {
    serde_json::json!({
        "format": "ess-diff-acknowledgements/1",
        "before": delta.before.spec_digest.as_str(),
        "after": delta.after.spec_digest.as_str(),
        "acknowledged": ids,
    })
    .to_string()
}

const CHAT: &str = "type/demo.calls.Channel/variant-removed/Chat";

#[test]
fn the_gate_fails_on_the_issue_reproduction_and_passes_once_it_is_acknowledged() {
    let roles = Roles {
        input: true,
        entity: true,
        ..Roles::default()
    };
    let delta = classified(THREE, TWO, roles);
    let gate = Gate::new(FailOn::Breaking);

    let outcome = gate.judge(&delta, None).unwrap();
    assert!(!outcome.passed());
    assert_eq!(
        outcome
            .failing()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec![CHAT.to_owned()]
    );

    let acknowledged =
        Acknowledgements::from_json(&acknowledgements(&delta, &[CHAT]), &delta).unwrap();
    let outcome = gate.judge(&delta, Some(&acknowledged)).unwrap();
    assert!(outcome.passed());
    assert_eq!(outcome.acknowledged().len(), 1);
}

#[test]
fn the_gate_considers_only_the_dimensions_it_is_given() {
    let roles = Roles {
        input: true,
        entity: true,
        ..Roles::default()
    };
    let delta = classified(THREE, TWO, roles);
    let readers_only = Gate::new(FailOn::Breaking).dimensions([Dimension::Readers]);
    assert!(readers_only.judge(&delta, None).unwrap().passed());
    let callers = Gate::new(FailOn::Breaking).dimensions([Dimension::Callers]);
    assert!(!callers.judge(&delta, None).unwrap().passed());
}

#[test]
fn an_unknown_fails_only_the_stricter_gate() {
    let roles = Roles {
        input: true,
        ..Roles::default()
    };
    let delta = classified(THREE, "Voice, Video, {name: Chat, wire: text}", roles);
    assert!(Gate::new(FailOn::Breaking)
        .judge(&delta, None)
        .unwrap()
        .passed());
    assert!(!Gate::new(FailOn::BreakingOrUnknown)
        .judge(&delta, None)
        .unwrap()
        .passed());
}

#[test]
fn the_gate_refuses_an_unclassified_delta() {
    let roles = Roles {
        input: true,
        ..Roles::default()
    };
    let before = model(&text(THREE, roles));
    let after = model(&text(TWO, roles));
    let plain = ess_diff::diff(&before, &after).unwrap();
    assert!(Gate::new(FailOn::Breaking).judge(&plain, None).is_err());
}

#[test]
fn an_acknowledgement_is_bound_to_the_compared_pair_and_to_ids_the_delta_holds() {
    let roles = Roles {
        input: true,
        entity: true,
        ..Roles::default()
    };
    let delta = classified(THREE, TWO, roles);
    let codes = |json: &str| -> Vec<ValidationCode> {
        match Acknowledgements::from_json(json, &delta) {
            Err(AcknowledgementRefusal::Invalid(errors)) => {
                errors.as_slice().iter().map(|e| e.code).collect()
            }
            other => panic!("{other:?}"),
        }
    };

    // The same id, acknowledged for another pair: the digests swapped.
    let other_pair = serde_json::json!({
        "format": "ess-diff-acknowledgements/1",
        "before": delta.after.spec_digest.as_str(),
        "after": delta.before.spec_digest.as_str(),
        "acknowledged": [CHAT],
    })
    .to_string();
    assert_eq!(
        codes(&other_pair),
        vec![
            ValidationCode::ConflictingDeclaration,
            ValidationCode::ConflictingDeclaration
        ]
    );

    let stale = acknowledgements(&delta, &["type/demo.calls.Channel/variant-removed/Fax"]);
    assert_eq!(codes(&stale), vec![ValidationCode::UndeclaredReference]);

    let twice = acknowledgements(&delta, &[CHAT, CHAT]);
    assert_eq!(codes(&twice), vec![ValidationCode::DuplicateDeclaration]);

    let future = acknowledgements(&delta, &[CHAT])
        .replace("ess-diff-acknowledgements/1", "ess-diff-acknowledgements/2");
    assert_eq!(
        codes(&future),
        vec![ValidationCode::UnsupportedFormatVersion]
    );

    let unknown_field = acknowledgements(&delta, &[CHAT]).replacen('{', "{\"note\":1,", 1);
    assert!(matches!(
        Acknowledgements::from_json(&unknown_field, &delta),
        Err(AcknowledgementRefusal::Malformed(_))
    ));
}

// ---- correction round 1: uses outside the graph, and failing closed -------------------------

/// `Channel` held only by a component setting, which the deployer supplies: a caller input.
fn setting_only(variants: &str) -> String {
    format!(
        "format: ess/5
system: demo
version: v1
domain: demo.calls
types:
  - name: demo.calls.Channel
    kind: enum
    variants: [{variants}]
components:
  - component: calls-service
    owns:
      domains: [demo.calls]
    settings:
      - name: default-channel
        type: demo.calls.Channel
        required: true
"
    )
}

#[test]
fn a_component_setting_type_is_a_caller_input() {
    let narrowed =
        ess_diff::classified(&model(&setting_only(THREE)), &model(&setting_only(TWO))).unwrap();
    let (_, narrowed) = only(&narrowed);
    assert_eq!(narrowed.uses(), Some(&uses(&[TypeUse::Input])));
    assert_eq!(narrowed.callers(), Compatibility::Breaking);
    assert_eq!(narrowed.verdict(), Compatibility::Breaking);

    let widened =
        ess_diff::classified(&model(&setting_only(TWO)), &model(&setting_only(THREE))).unwrap();
    let (_, widened) = only(&widened);
    assert_eq!(widened.verdict(), Compatibility::Compatible);
}

/// `Channel` held only by an actor's credential attributes, a role this classifier does not read.
fn attribute_only(variants: &str) -> String {
    format!(
        "format: ess/16
system: demo
version: v1
domain: demo.calls
types:
  - name: demo.calls.Channel
    kind: enum
    variants: [{variants}]
events:
  - name: demo.calls.Pinged
    fields:
      - {{name: note, type: String}}
commands:
  - name: demo.calls.Ping
    input:
      - {{name: note, type: String}}
    outcomes:
      - name: pinged
        emits: [demo.calls.Pinged]
        payload:
          demo.calls.Pinged: {{note: input.note}}
actors:
  - name: demo.calls.Agent
    may: [demo.calls.Ping]
    attributes:
      - {{name: channel, type: demo.calls.Channel}}
"
    )
}

#[test]
fn a_use_the_classifier_does_not_model_is_unknown_in_every_dimension_never_compatible() {
    for (before, after) in [(THREE, TWO), (TWO, THREE)] {
        let delta = ess_diff::classified(
            &model(&attribute_only(before)),
            &model(&attribute_only(after)),
        )
        .unwrap();
        let (id, compatibility) = only(&delta);
        assert_eq!(
            compatibility.uses(),
            Some(&uses(&[TypeUse::Unmodelled])),
            "{id}"
        );
        assert_eq!(compatibility.callers(), Compatibility::Unknown, "{id}");
        assert_eq!(compatibility.readers(), Compatibility::Unknown, "{id}");
        assert_eq!(compatibility.history(), Compatibility::Unknown, "{id}");
    }

    // A documentation change to the same type is still compatible: nobody reads a summary.
    let delta = ess_diff::classified(
        &model(&attribute_only(THREE)),
        &model(&attribute_only(
            "Voice, Video, {name: Chat, summary: Text messages instead of a voice line}",
        )),
    )
    .unwrap();
    let (_, documentation) = only(&delta);
    assert_eq!(documentation.verdict(), Compatibility::Compatible);
}
