//! Synthesized inputs satisfy the invariants over their nested members, and guards over nested
//! input paths are witnessed (beyond10x/ess#234).
//!
//! One model per gap the issue names:
//!
//! 1. an entity invariant over the members of a struct input the creating branch copies whole;
//! 2. a struct type with invariants used as a command input;
//! 3. a `when_subject` over a member of a stored optional struct;
//! 4. a wrong-state scenario whose sibling guard compares a stored field with a nested input path;
//! 5. a `when:` equality between an input and a member of another input.
//!
//! Each is synthesized, and the suite is run against the interpreter of the same model: a suite
//! that sends an input the model's own invariants refuse fails there.
#![allow(clippy::needless_raw_string_hashes, clippy::missing_panics_doc)]

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, synthesize::Synthesis,
    AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{}: {}: {}",
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause.code(),
                refusal.cause
            )
        })
        .collect()
}

/// The scenarios the interpreter of `model` fails or cannot execute. `Unsupported` is left out: the
/// interpreter reads no view yet, and a scenario that reaches one has already run its commands.
fn failing(result: &Synthesis, model: &str) -> BTreeSet<String> {
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
    let report = Runner::for_suite(&result.suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir(model)))
        .into_report();
    report
        .scenarios
        .iter()
        .filter(|scenario| matches!(scenario.status, Status::Failed | Status::Error))
        .inspect(|scenario| eprintln!("{scenario:#?}"))
        .map(|scenario| scenario.scenario.to_string())
        .collect()
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || panic!("no scenario {id}; refusals: {:#?}", refusals(result)),
            |(_, scenario)| scenario,
        )
}

/// The input of every invocation of `command` in the scenario, in step order.
fn inputs(scenario: &ConformanceScenario, command: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .collect()
}

fn literal(value: &ScenarioValue) -> &Node {
    match value {
        ScenarioValue::Literal { value } => value,
        other => panic!("a literal input, not {other:?}"),
    }
}

fn member<'a>(node: &'a Node, name: &str) -> &'a Node {
    match node {
        Node::Map(members) => members
            .get(name)
            .unwrap_or_else(|| panic!("no member {name} in {node:?}")),
        other => panic!("a mapping, not {other:?}"),
    }
}

/// Everything synthesized, nothing refused, and every scenario passes against the model itself.
fn runs_clean(model: &str) -> Synthesis {
    let result = synthesize(&ir(model));
    assert_eq!(
        refusals(&result),
        Vec::<String>::new(),
        "nothing is refused"
    );
    assert_eq!(
        failing(&result, model),
        BTreeSet::new(),
        "every scenario passes against the interpreter of its own model"
    );
    result
}

// ---- 1. an entity invariant over the members of a struct input --------------------------------

/// `Request` stores the `fingerprint` input whole, and its invariants pin one member to a literal,
/// couple two members, and order two Integer members (whose plain witnesses are equal).
const FINGERPRINT: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.req
types:
  - {name: demo.req.RequestId, kind: newtype, of: Uuid}
  - name: demo.req.Fingerprint
    kind: struct
    fields:
      - {name: version, type: String}
      - {name: digest, type: String}
      - {name: origin, type: String}
      - {name: route, type: String}
      - {name: low, type: Integer}
      - {name: high, type: Integer}
entities:
  - name: demo.req.Request
    identity: {name: id, type: demo.req.RequestId}
    fields:
      - {name: fingerprint, type: demo.req.Fingerprint}
      - {name: label, type: String}
    invariants:
      - 'fingerprint.version == "canonical-v1"'
      - 'fingerprint.origin == fingerprint.route'
      - 'fingerprint.low < fingerprint.high'
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
actors:
  - {name: demo.req.Clerk, may: [demo.req.Submit, demo.req.Close]}
errors:
  - {name: demo.req.NotOpen, summary: Not open., fields: []}
events:
  - name: demo.req.Submitted
    fields:
      - {name: id, type: demo.req.RequestId}
      - {name: fingerprint, type: demo.req.Fingerprint}
  - name: demo.req.Closed
    fields: [{name: id, type: demo.req.RequestId}]
commands:
  - name: demo.req.Submit
    input:
      - {name: fingerprint, type: demo.req.Fingerprint}
      - {name: label, type: String}
    outcomes:
      - name: submitted
        creates: demo.req.Request
        instance: id
        sets: {fingerprint: input.fingerprint, label: input.label}
        emits: [demo.req.Submitted]
        payload:
          demo.req.Submitted: {id: {generated: true}, fingerprint: input.fingerprint}
  - name: demo.req.Close
    input: [{name: id, type: demo.req.RequestId}]
    outcomes:
      - name: closed
        moves: demo.req.Request.close
        instance: id
        emits: [demo.req.Closed]
        payload:
          demo.req.Closed: {id: input.id}
      - name: not-open
        wrong_state: true
        error: demo.req.NotOpen
views:
  - name: demo.req.Requests
    source: demo.req.Request
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.req.RequestId}
      - {name: fingerprint, type: demo.req.Fingerprint}
      - {name: state, type: demo.req.Request.State}
"#;

#[test]
fn issue_234_a_struct_input_satisfies_the_entity_invariants_over_its_members() {
    let result = runs_clean(FINGERPRINT);
    let sent = inputs(
        scenario(&result, "demo.req.Submit/outcome/submitted"),
        "demo.req.Submit",
    );
    assert_ne!(sent.len(), 0);
    for input in &sent {
        let fingerprint = literal(&input["fingerprint"]);
        assert_eq!(
            member(fingerprint, "version"),
            &Node::Text("canonical-v1".to_owned())
        );
        assert_eq!(member(fingerprint, "origin"), member(fingerprint, "route"));
        // Defect class 1: members the invariants do not tie keep their own path witnesses.
        assert_ne!(member(fingerprint, "digest"), member(fingerprint, "origin"));
        assert_ne!(member(fingerprint, "low"), member(fingerprint, "high"));
    }
}

/// Every creation of the entity is arranged with a valid struct, the further instances too:
/// `Close` needs a row, so its scenarios create one before closing it.
#[test]
fn issue_234_every_arranged_creation_satisfies_the_entity_invariants() {
    let result = runs_clean(FINGERPRINT);
    let mut creations = 0;
    for scenario in result.suite.scenarios.values() {
        for input in inputs(scenario, "demo.req.Submit") {
            let fingerprint = literal(&input["fingerprint"]);
            assert_eq!(
                member(fingerprint, "version"),
                &Node::Text("canonical-v1".to_owned())
            );
            assert_eq!(member(fingerprint, "origin"), member(fingerprint, "route"));
            creations += 1;
        }
    }
    assert!(creations > 1, "more than one creation is arranged");
}

/// A further instance is a further witness (rule 5) with its invariants satisfied too: every leaf
/// the invariants move still differs between two instances, except where an invariant pins it to
/// one literal.
#[test]
fn issue_234_a_further_instance_satisfies_the_invariants_and_differs_from_the_first() {
    use ess_conformance::witness::{candidates, Distinction};
    let ir = ir(FINGERPRINT);
    let command = &ir.commands()[&"demo.req.Submit".parse().unwrap()];
    let base = |distinction| {
        candidates(&ir, command, &[], distinction)
            .unwrap()
            .into_iter()
            .next()
            .expect("a candidate is admitted")
    };
    let first = base(Distinction::PLAIN);
    let second = base(Distinction::further(1));
    for input in [&first, &second] {
        let fingerprint = &input["fingerprint"];
        assert_eq!(
            member(fingerprint, "version"),
            &Node::Text("canonical-v1".to_owned())
        );
        assert_eq!(member(fingerprint, "origin"), member(fingerprint, "route"));
        let (Node::Number(low), Node::Number(high)) =
            (member(fingerprint, "low"), member(fingerprint, "high"))
        else {
            panic!("numbers: {fingerprint:?}");
        };
        assert!(low < high, "{fingerprint:?}");
    }
    for leaf in ["origin", "digest", "low", "high"] {
        assert_ne!(
            member(&first["fingerprint"], leaf),
            member(&second["fingerprint"], leaf),
            "{leaf} differs between two instances"
        );
    }
    assert_ne!(first["label"], second["label"]);
}

// ---- 2. a struct type with invariants used as a command input -----------------------------------

const WINDOW: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.win
types:
  - {name: demo.win.SlotId, kind: newtype, of: Uuid}
  - name: demo.win.Window
    kind: struct
    fields:
      - {name: start, type: Integer}
      - {name: end, type: Integer}
      - {name: zone, type: String}
      - {name: note, type: String}
    invariants:
      - 'start >= 5'
      - 'zone == "utc"'
entities:
  - name: demo.win.Slot
    identity: {name: id, type: demo.win.SlotId}
    fields:
      - {name: window, type: demo.win.Window}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - {name: demo.win.Clerk, may: [demo.win.Book]}
events:
  - name: demo.win.Booked
    fields: [{name: id, type: demo.win.SlotId}]
commands:
  - name: demo.win.Book
    input:
      - {name: window, type: demo.win.Window}
    outcomes:
      - name: booked
        creates: demo.win.Slot
        instance: id
        sets: {window: input.window}
        emits: [demo.win.Booked]
        payload:
          demo.win.Booked: {id: {generated: true}}
views:
  - name: demo.win.Slots
    source: demo.win.Slot
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.win.SlotId}
      - {name: window, type: demo.win.Window}
"#;

#[test]
fn issue_234_a_struct_type_with_invariants_is_synthesizable_as_an_input() {
    let result = runs_clean(WINDOW);
    let sent = inputs(
        scenario(&result, "demo.win.Book/outcome/booked"),
        "demo.win.Book",
    );
    for input in &sent {
        let window = literal(&input["window"]);
        assert_eq!(member(window, "zone"), &Node::Text("utc".to_owned()));
        assert_ne!(member(window, "zone"), member(window, "note"));
    }
}

// ---- 3. a `when_subject` over a member of a stored optional struct ------------------------------

/// `Touch` declares an input refusal first, then a `when_subject` over `meta.tier`, where `meta`
/// is an optional struct `Make` leaves absent, then the default. `Annotate` sets `meta` from the
/// `ANNOTATION` it is given.
const META: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.meta
types:
  - {name: demo.meta.ItemId, kind: newtype, of: Uuid}
  - {name: demo.meta.Tier, kind: enum, variants: [Basic, Gold]}
  - name: demo.meta.Meta
    kind: struct
    fields:
      - {name: tier, type: demo.meta.Tier}
      - {name: note, type: String}
entities:
  - name: demo.meta.Item
    identity: {name: id, type: demo.meta.ItemId}
    fields:
      - {name: label, type: String}
      - {name: meta, type: Optional<demo.meta.Meta>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - {name: demo.meta.Clerk, may: [demo.meta.Make, demo.meta.Annotate, demo.meta.Touch]}
errors:
  - {name: demo.meta.ReasonRequired, summary: No reason., fields: []}
  - {name: demo.meta.GoldLocked, summary: Gold items are locked., fields: []}
events:
  - name: demo.meta.Made
    fields: [{name: id, type: demo.meta.ItemId}]
  - name: demo.meta.Annotated
    fields: [{name: id, type: demo.meta.ItemId}]
commands:
  - name: demo.meta.Make
    input: [{name: label, type: String}]
    outcomes:
      - name: made
        creates: demo.meta.Item
        instance: id
        sets: {label: input.label}
        emits: [demo.meta.Made]
        payload:
          demo.meta.Made: {id: {generated: true}}
  - name: demo.meta.Annotate
    input:
      - {name: id, type: demo.meta.ItemId}
      - {name: meta, type: demo.meta.Meta}
    outcomes:
      - name: annotated
        updates: demo.meta.Item
        instance: id
        sets: {meta: ANNOTATION}
        emits: [demo.meta.Annotated]
        payload:
          demo.meta.Annotated: {id: input.id}
  - name: demo.meta.Touch
    input:
      - {name: id, type: demo.meta.ItemId}
      - {name: reason, type: String}
    outcomes:
      - name: reason-required
        when: reason == ""
        error: demo.meta.ReasonRequired
      - name: gold-locked
        when_subject: {predicate: 'meta.tier == Gold'}
        error: demo.meta.GoldLocked
      - name: touched
        preserves: demo.meta.Item
        instance: id
views:
  - name: demo.meta.Items
    source: demo.meta.Item
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.meta.ItemId}
      - {name: label, type: String}
      - {name: meta, type: Optional<demo.meta.Meta>}
      - {name: state, type: demo.meta.Item.State}
"#;

/// `meta` copied whole from the input: every row the stored guard reads is one an arrangement
/// determined.
const COPIED: &str = "input.meta";

/// `meta.note` left to the implementation: no arrangement determines `meta`, so the stored guard is
/// undecided on every row.
const UNDETERMINED: &str = "{tier: Basic, note: {generated: true}}";

fn meta(annotation: &str) -> String {
    META.replace("ANNOTATION", annotation)
}

#[test]
fn issue_234_a_when_subject_over_a_stored_optional_struct_member_is_witnessed() {
    let model = meta(COPIED);
    let result = runs_clean(&model);
    for id in [
        "demo.meta.Touch/outcome/reason-required",
        "demo.meta.Touch/outcome/gold-locked",
        "demo.meta.Touch/outcome/touched",
    ] {
        scenario(&result, id);
    }
}

/// The input-guarded refusal declared before the stored guard answers before the row is read, so a
/// row the stored guard cannot be decided on refuses at most the branches that read it — each
/// with the named cause — and never the refusal.
#[test]
fn issue_234_an_undecided_stored_struct_member_never_refuses_the_input_refusal_before_it() {
    let model = meta(UNDETERMINED);
    let result = synthesize(&ir(&model));
    let refused: Vec<String> = refusals(&result)
        .into_iter()
        .filter(|refusal| refusal.starts_with("demo.meta.Touch/"))
        .collect();
    assert!(
        refused
            .iter()
            .all(|refusal| !refusal.starts_with("demo.meta.Touch/outcome/reason-required")),
        "the refusal declared first is witnessed: {refused:#?}"
    );
    let sent = inputs(
        scenario(&result, "demo.meta.Touch/outcome/reason-required"),
        "demo.meta.Touch",
    );
    assert_eq!(
        literal(&sent.last().expect("Touch is sent")["reason"]),
        &Node::Text(String::new())
    );
    // The guard no row decides true is refused naming the stored field — not dropped. The default
    // is witnessed on the row `Make` leaves: `meta` absent there, so `meta.tier == Gold` is the
    // specification's unknown, which takes no branch (story acceptance line 3).
    assert!(
        refused.iter().any(|refusal| {
            refusal.starts_with("demo.meta.Touch/outcome/gold-locked")
                && refusal.contains("stored meta")
        }),
        "gold-locked is refused with its cause: {refused:#?}"
    );
    assert!(
        refused
            .iter()
            .all(|refusal| !refusal.starts_with("demo.meta.Touch/outcome/touched")),
        "the default beside the guard is witnessed: {refused:#?}"
    );
    scenario(&result, "demo.meta.Touch/outcome/touched");
    assert_eq!(failing(&result, &model), BTreeSet::new());
}

/// Every guard that reads a member of the absent `meta` — alone, negated, or beside a second guard
/// over another member — leaves the default witnessed, and a correct target passes every
/// scenario. Under `not` the unknown stays unknown: `{not: meta.tier == Gold}` is not taken on the
/// absent row either, so a scenario sending that refusal there would fail the model.
#[test]
fn issue_234_every_guard_over_an_absent_optional_struct_member_leaves_the_default_witnessed() {
    for (name, guards) in [
        ("plain", vec!["'meta.tier == Gold'"]),
        ("negated", vec!["{not: 'meta.tier == Gold'}"]),
        (
            "two members",
            vec!["'meta.tier == Gold'", "'meta.note == \"frozen\"'"],
        ),
    ] {
        let outcomes = guards
            .iter()
            .enumerate()
            .map(|(at, guard)| {
                format!(
                    "      - name: locked-{at}\n        when_subject: {{predicate: {guard}}}\n        \
                     error: demo.meta.GoldLocked\n"
                )
            })
            .collect::<Vec<_>>()
            .concat();
        let model = meta(UNDETERMINED).replace(
            "      - name: gold-locked\n        when_subject: {predicate: 'meta.tier == Gold'}\n        \
             error: demo.meta.GoldLocked\n",
            &outcomes,
        );
        assert_ne!(model, meta(UNDETERMINED), "{name}: the guard was replaced");
        let result = synthesize(&ir(&model));
        let refused: Vec<String> = refusals(&result)
            .into_iter()
            .filter(|refusal| refusal.starts_with("demo.meta.Touch/"))
            .collect();
        assert!(
            refused
                .iter()
                .all(|refusal| !refusal.starts_with("demo.meta.Touch/outcome/touched")),
            "{name}: the default is witnessed: {refused:#?}"
        );
        scenario(&result, "demo.meta.Touch/outcome/touched");
        for at in 0..guards.len() {
            let id = format!("demo.meta.Touch/outcome/locked-{at}");
            assert!(
                refused
                    .iter()
                    .any(|refusal| refusal.starts_with(&id) && refusal.contains("stored meta")),
                "{name}: {id}, which no row takes, is refused naming its cause: {refused:#?}"
            );
        }
        assert_eq!(failing(&result, &model), BTreeSet::new(), "{name}");
    }
}

/// An `Optional` field whose only later writer generates it is still arrangeable: the row `Make`
/// leaves holds it absent, so a guard over it is decided there and the default is witnessed rather
/// than every branch being refused as unarrangeable.
#[test]
fn issue_234_a_field_the_creator_leaves_absent_is_arrangeable_though_only_generated_later() {
    let model = meta(COPIED)
        .replace(
            "      - {name: meta, type: Optional<demo.meta.Meta>}\n    lifecycle",
            "      - {name: meta, type: Optional<demo.meta.Meta>}\n      - {name: memo, type: \
             Optional<String>}\n    lifecycle",
        )
        .replace(
            "sets: {meta: input.meta}",
            "sets: {meta: input.meta, memo: {generated: true}}",
        )
        .replace(
            "when_subject: {predicate: 'meta.tier == Gold'}",
            "when_subject: {predicate: 'memo == \"frozen\"'}",
        )
        .replace(
            "      - {name: meta, type: Optional<demo.meta.Meta>}\n      - {name: state",
            "      - {name: meta, type: Optional<demo.meta.Meta>}\n      - {name: memo, type: \
             Optional<String>}\n      - {name: state",
        );
    assert_eq!(model.matches("memo").count(), 4, "every edit applied");
    let result = synthesize(&ir(&model));
    let refused = refusals(&result);
    assert!(
        refused
            .iter()
            .all(|refusal| !refusal.starts_with("demo.meta.Touch/outcome/touched")),
        "the default is witnessed: {refused:#?}"
    );
    scenario(&result, "demo.meta.Touch/outcome/touched");
    assert!(
        refused.iter().any(|refusal| {
            refusal.starts_with("demo.meta.Touch/outcome/gold-locked") && refusal.contains("memo")
        }),
        "the guard no row takes is refused naming the field: {refused:#?}"
    );
    assert_eq!(failing(&result, &model), BTreeSet::new());
}

/// A wrong-state scenario must see the sibling guard over the absent member refuted by the row:
/// `Ship` on a shipped item whose `meta` no step wrote answers `NotOpen`, never `GoldLocked`.
const SHIP: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.meta
types:
  - {name: demo.meta.ItemId, kind: newtype, of: Uuid}
  - {name: demo.meta.Tier, kind: enum, variants: [Basic, Gold]}
  - name: demo.meta.Meta
    kind: struct
    fields:
      - {name: tier, type: demo.meta.Tier}
      - {name: note, type: String}
entities:
  - name: demo.meta.Item
    identity: {name: id, type: demo.meta.ItemId}
    fields:
      - {name: label, type: String}
      - {name: meta, type: Optional<demo.meta.Meta>}
    lifecycle:
      initial: Open
      states: [Open, Shipped]
      terminal: [Shipped]
      transitions:
        - {name: ship, from: [Open], to: Shipped}
actors:
  - {name: demo.meta.Clerk, may: [demo.meta.Make, demo.meta.Annotate, demo.meta.Ship]}
errors:
  - {name: demo.meta.GoldLocked, summary: Gold items are locked., fields: []}
  - {name: demo.meta.NotOpen, summary: Not open., fields: []}
events:
  - name: demo.meta.Made
    fields: [{name: id, type: demo.meta.ItemId}]
  - name: demo.meta.Annotated
    fields: [{name: id, type: demo.meta.ItemId}]
  - name: demo.meta.Shipped
    fields: [{name: id, type: demo.meta.ItemId}]
commands:
  - name: demo.meta.Make
    input: [{name: label, type: String}]
    outcomes:
      - name: made
        creates: demo.meta.Item
        instance: id
        sets: {label: input.label}
        emits: [demo.meta.Made]
        payload:
          demo.meta.Made: {id: {generated: true}}
  - name: demo.meta.Annotate
    input:
      - {name: id, type: demo.meta.ItemId}
    outcomes:
      - name: annotated
        updates: demo.meta.Item
        instance: id
        sets: {meta: {tier: Basic, note: {generated: true}}}
        emits: [demo.meta.Annotated]
        payload:
          demo.meta.Annotated: {id: input.id}
  - name: demo.meta.Ship
    input:
      - {name: id, type: demo.meta.ItemId}
    outcomes:
      - name: gold-locked
        when_subject: {predicate: 'meta.tier == Gold'}
        error: demo.meta.GoldLocked
      - name: shipped
        moves: demo.meta.Item.ship
        instance: id
        emits: [demo.meta.Shipped]
        payload:
          demo.meta.Shipped: {id: input.id}
      - name: not-open
        wrong_state: true
        error: demo.meta.NotOpen
views:
  - name: demo.meta.Items
    source: demo.meta.Item
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.meta.ItemId}
      - {name: label, type: String}
      - {name: meta, type: Optional<demo.meta.Meta>}
      - {name: state, type: demo.meta.Item.State}
"#;

#[test]
fn issue_234_a_wrong_state_row_refutes_a_sibling_guard_over_an_absent_struct_member() {
    let result = synthesize(&ir(SHIP));
    let refused: Vec<String> = refusals(&result)
        .into_iter()
        .filter(|refusal| !refusal.starts_with("demo.meta.Ship/outcome/gold-locked"))
        .collect();
    assert!(
        refused
            .iter()
            .all(|refusal| !refusal.contains("demo.meta.Ship")),
        "only the guard no row takes is refused: {refused:#?}"
    );
    scenario(&result, "demo.meta.Ship/outcome/shipped");
    scenario(
        &result,
        "demo.meta.Item/state/Shipped/refuses/demo.meta.Ship",
    );
    assert_eq!(failing(&result, SHIP), BTreeSet::new());
}

/// The same rule for a `when_related` guard: the related row `Onboard` creates holds `profile`
/// absent, so `profile.tier == Gold` takes no branch there and the default is witnessed.
const PROFILE: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.sso
types:
  - {name: demo.sso.TenantId, kind: newtype, of: Uuid}
  - {name: demo.sso.SignInId, kind: newtype, of: Uuid}
  - {name: demo.sso.Tier, kind: enum, variants: [Basic, Gold]}
  - name: demo.sso.Profile
    kind: struct
    fields:
      - {name: tier, type: demo.sso.Tier}
      - {name: note, type: String}
entities:
  - name: demo.sso.Tenant
    identity: {name: tenant, type: demo.sso.TenantId}
    fields:
      - {name: profile, type: Optional<demo.sso.Profile>}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.sso.SignIn
    identity: {name: sign_in_id, type: demo.sso.SignInId}
    fields:
      - {name: tenant, type: demo.sso.TenantId}
    lifecycle: {initial: Initiated, states: [Initiated], terminal: [Initiated]}
errors:
  - {name: demo.sso.NoTenant, summary: No tenant., fields: []}
  - {name: demo.sso.GoldOnly, summary: Gold tenants sign in elsewhere., fields: []}
events:
  - name: demo.sso.Onboarded
    fields: [{name: tenant, type: demo.sso.TenantId}]
  - name: demo.sso.Profiled
    fields: [{name: tenant, type: demo.sso.TenantId}]
  - name: demo.sso.SignInInitiated
    fields: [{name: sign_in_id, type: demo.sso.SignInId}]
actors:
  - name: demo.sso.Operator
    may: [demo.sso.Onboard, demo.sso.SetProfile, demo.sso.InitiateSignIn]
commands:
  - name: demo.sso.Onboard
    input: []
    outcomes:
      - name: onboarded
        creates: demo.sso.Tenant
        instance: tenant
        emits: [demo.sso.Onboarded]
        payload: {demo.sso.Onboarded: {tenant: {generated: true}}}
  - name: demo.sso.SetProfile
    input:
      - {name: tenant, type: demo.sso.TenantId}
    outcomes:
      - name: profiled
        updates: demo.sso.Tenant
        instance: tenant
        sets: {profile: {tier: Basic, note: {generated: true}}}
        emits: [demo.sso.Profiled]
        payload: {demo.sso.Profiled: {tenant: input.tenant}}
  - name: demo.sso.InitiateSignIn
    input:
      - {name: tenant, type: demo.sso.TenantId}
    outcomes:
      - name: no-tenant
        when_related: {via: input.tenant, exists: false}
        error: demo.sso.NoTenant
      - name: gold-only
        when_related: {via: input.tenant, predicate: 'profile.tier == Gold'}
        error: demo.sso.GoldOnly
      - name: initiated
        creates: demo.sso.SignIn
        instance: sign_in_id
        emits: [demo.sso.SignInInitiated]
        payload: {demo.sso.SignInInitiated: {sign_in_id: {generated: true}}}
        sets: {tenant: input.tenant}
views:
  - name: demo.sso.Tenants
    source: demo.sso.Tenant
    consistency: read_your_writes
    fields:
      - {name: tenant, type: demo.sso.TenantId}
      - {name: state, type: demo.sso.Tenant.State}
      - {name: profile, type: Optional<demo.sso.Profile>}
  - name: demo.sso.SignIns
    source: demo.sso.SignIn
    consistency: read_your_writes
    fields:
      - {name: sign_in_id, type: demo.sso.SignInId}
      - {name: tenant, type: demo.sso.TenantId}
"#;

#[test]
fn issue_234_a_when_related_guard_over_an_absent_struct_member_leaves_the_default_witnessed() {
    let result = synthesize(&ir(PROFILE));
    let refused = refusals(&result);
    assert!(
        refused
            .iter()
            .all(|refusal| !refusal.starts_with("demo.sso.InitiateSignIn/outcome/initiated")),
        "the default beside the related guard is witnessed: {refused:#?}"
    );
    scenario(&result, "demo.sso.InitiateSignIn/outcome/initiated");
    scenario(&result, "demo.sso.InitiateSignIn/outcome/no-tenant");
    assert_eq!(failing(&result, PROFILE), BTreeSet::new());
}

// ---- 4. a wrong-state scenario beside a guard over a nested input path --------------------------

const FENCE: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.doc
types:
  - {name: demo.doc.DocId, kind: newtype, of: Uuid}
  - name: demo.doc.Publication
    kind: struct
    fields:
      - {name: expected_fence, type: String}
      - {name: channel, type: String}
entities:
  - name: demo.doc.Doc
    identity: {name: id, type: demo.doc.DocId}
    fields:
      - {name: fence, type: String}
    lifecycle:
      initial: Draft
      states: [Draft, Published, Archived]
      terminal: [Archived]
      transitions:
        - {name: publish, from: [Draft], to: Published}
        - {name: archive, from: [Draft, Published], to: Archived}
actors:
  - {name: demo.doc.Editor, may: [demo.doc.Write, demo.doc.Publish, demo.doc.Archive]}
errors:
  - {name: demo.doc.StaleFence, summary: The fence moved., fields: []}
  - {name: demo.doc.NotDraft, summary: Not a draft., fields: []}
events:
  - name: demo.doc.Written
    fields: [{name: id, type: demo.doc.DocId}]
  - name: demo.doc.Published
    fields: [{name: id, type: demo.doc.DocId}]
  - name: demo.doc.Archived
    fields: [{name: id, type: demo.doc.DocId}]
commands:
  - name: demo.doc.Write
    input: [{name: fence, type: String}]
    outcomes:
      - name: written
        creates: demo.doc.Doc
        instance: id
        sets: {fence: input.fence}
        emits: [demo.doc.Written]
        payload:
          demo.doc.Written: {id: {generated: true}}
  - name: demo.doc.Publish
    input:
      - {name: id, type: demo.doc.DocId}
      - {name: publication, type: demo.doc.Publication}
    outcomes:
      - name: stale
        when_subject: {predicate: 'fence != input.publication.expected_fence'}
        error: demo.doc.StaleFence
      - name: published
        moves: demo.doc.Doc.publish
        instance: id
        emits: [demo.doc.Published]
        payload:
          demo.doc.Published: {id: input.id}
      - name: not-draft
        wrong_state: true
        error: demo.doc.NotDraft
  - name: demo.doc.Archive
    input: [{name: id, type: demo.doc.DocId}]
    outcomes:
      - name: archived
        moves: demo.doc.Doc.archive
        instance: id
        emits: [demo.doc.Archived]
        payload:
          demo.doc.Archived: {id: input.id}
      - name: gone
        wrong_state: true
        error: demo.doc.NotDraft
views:
  - name: demo.doc.Docs
    source: demo.doc.Doc
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.doc.DocId}
      - {name: fence, type: String}
      - {name: state, type: demo.doc.Doc.State}
"#;

#[test]
fn issue_234_a_wrong_state_scenario_beside_a_nested_input_guard_is_witnessed() {
    let result = runs_clean(FENCE);
    for id in [
        "demo.doc.Doc/state/Published/refuses/demo.doc.Publish",
        "demo.doc.Doc/state/Archived/refuses/demo.doc.Publish",
        "demo.doc.Publish/outcome/published",
        "demo.doc.Publish/outcome/stale",
    ] {
        let the = scenario(&result, id);
        let written: Vec<Node> = inputs(the, "demo.doc.Write")
            .iter()
            .map(|input| literal(&input["fence"]).clone())
            .collect();
        let published = inputs(the, "demo.doc.Publish");
        let last = published.last().expect("Publish is sent");
        let expected = member(literal(&last["publication"]), "expected_fence");
        if id.ends_with("/stale") {
            assert!(
                !written.contains(expected),
                "{id}: {written:?} {expected:?}"
            );
        } else {
            assert!(written.contains(expected), "{id}: {written:?} {expected:?}");
        }
        // Defect class 1: the fence the guard compares differs from the member beside it.
        assert_ne!(
            expected,
            member(literal(&last["publication"]), "channel"),
            "{id}"
        );
    }
}

// ---- 5. a `when:` equality between an input and a member of another input -----------------------

const CLAIM: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.claim
types:
  - {name: demo.claim.ClaimId, kind: newtype, of: Uuid}
  - name: demo.claim.Ticket
    kind: struct
    fields:
      - {name: owner, type: String}
      - {name: seat, type: String}
entities:
  - name: demo.claim.Claim
    identity: {name: id, type: demo.claim.ClaimId}
    fields:
      - {name: owner, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - {name: demo.claim.Clerk, may: [demo.claim.Redeem]}
errors:
  - {name: demo.claim.NotYours, summary: The ticket names another owner., fields: []}
events:
  - name: demo.claim.Redeemed
    fields: [{name: id, type: demo.claim.ClaimId}]
commands:
  - name: demo.claim.Redeem
    input:
      - {name: owner, type: String}
      - {name: ticket, type: demo.claim.Ticket}
    outcomes:
      - name: redeemed
        when: owner == ticket.owner
        creates: demo.claim.Claim
        instance: id
        sets: {owner: input.owner}
        emits: [demo.claim.Redeemed]
        payload:
          demo.claim.Redeemed: {id: {generated: true}}
      - name: not-yours
        error: demo.claim.NotYours
views:
  - name: demo.claim.Claims
    source: demo.claim.Claim
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.claim.ClaimId}
      - {name: owner, type: String}
"#;

#[test]
fn issue_234_a_when_equality_with_another_inputs_member_is_satisfied() {
    let result = runs_clean(CLAIM);
    let redeemed = inputs(
        scenario(&result, "demo.claim.Redeem/outcome/redeemed"),
        "demo.claim.Redeem",
    );
    let last = redeemed.last().expect("Redeem is sent");
    let ticket = literal(&last["ticket"]);
    assert_eq!(literal(&last["owner"]), member(ticket, "owner"));
    assert_ne!(member(ticket, "owner"), member(ticket, "seat"));
    let refused = inputs(
        scenario(&result, "demo.claim.Redeem/outcome/not-yours"),
        "demo.claim.Redeem",
    );
    let last = refused.last().expect("Redeem is sent");
    assert_ne!(
        literal(&last["owner"]),
        member(literal(&last["ticket"]), "owner")
    );
}

// ---- correction 1: an invariant the bounded repair cannot meet, and a refusal beside one --------

/// `Stamp` stores `seal` whole. `INVARIANTS` are the entity's, `OUTCOMES` those of `Seal`.
const SEAL: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.seal
types:
  - {name: demo.seal.StampId, kind: newtype, of: Uuid}
  - name: demo.seal.Seal
    kind: struct
    fields:
      - {name: version, type: String}
      - {name: a, type: Integer}
      - {name: b, type: Integer}
      - {name: c, type: Integer}
      - {name: d, type: Integer}
      - {name: e, type: Integer}
entities:
  - name: demo.seal.Stamp
    identity: {name: id, type: demo.seal.StampId}
    fields:
      - {name: seal, type: demo.seal.Seal}
    invariants:
INVARIANTS
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - {name: demo.seal.Clerk, may: [demo.seal.Apply]}
errors:
  - {name: demo.seal.Stale, summary: Stale seal., fields: []}
events:
  - name: demo.seal.Applied
    fields: [{name: id, type: demo.seal.StampId}]
commands:
  - name: demo.seal.Apply
    input:
      - {name: seal, type: demo.seal.Seal}
    outcomes:
OUTCOMES
      - name: applied
        creates: demo.seal.Stamp
        instance: id
        sets: {seal: input.seal}
        emits: [demo.seal.Applied]
        payload:
          demo.seal.Applied: {id: {generated: true}}
views:
  - name: demo.seal.Stamps
    source: demo.seal.Stamp
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.seal.StampId}
      - {name: seal, type: demo.seal.Seal}
"#;

fn seal(invariants: &[&str], outcomes: &str) -> String {
    let lines: Vec<String> = invariants
        .iter()
        .map(|invariant| format!("      - {invariant}"))
        .collect();
    SEAL.replace("INVARIANTS", &lines.join("\n"))
        .replace("OUTCOMES\n", outcomes)
}

/// A strict chain over five members is past the bounded repair. The branch is refused with
/// `ESS-SYNTH-003` naming the invariants, and no scenario sends an input the entity refuses.
#[test]
fn issue_234_an_invariant_no_bounded_input_meets_refuses_the_branch_naming_it() {
    let model = seal(
        &[
            "'seal.a < seal.b'",
            "'seal.b < seal.c'",
            "'seal.c < seal.d'",
            "'seal.d < seal.e'",
        ],
        "",
    );
    let result = synthesize(&ir(&model));
    assert_eq!(failing(&result, &model), BTreeSet::new());
    let refused = refusals(&result);
    assert!(
        refused.iter().any(|refusal| {
            refusal.starts_with("demo.seal.Apply/outcome/applied: ESS-SYNTH-003:")
                && refusal.contains("`demo.seal.Stamp` invariant `seal.a < seal.b`")
        }),
        "{refused:#?}"
    );
    for scenario in result.suite.scenarios.values() {
        assert_eq!(inputs(scenario, "demo.seal.Apply").len(), 0);
    }
}

/// A refusal whose guard is the negation of the entity invariant is still witnessed: its input
/// breaks the invariant, and no accepting branch answers it.
#[test]
fn issue_234_a_refusal_guarding_the_invariant_keeps_its_witness() {
    let model = seal(
        &["'seal.version == \"v1\"'"],
        "      - name: stale\n        when: 'seal.version != \"v1\"'\n        error: demo.seal.Stale\n",
    );
    let result = runs_clean(&model);
    let applied = inputs(
        scenario(&result, "demo.seal.Apply/outcome/applied"),
        "demo.seal.Apply",
    );
    let last = applied.last().expect("Apply is sent");
    assert_eq!(
        member(literal(&last["seal"]), "version"),
        &Node::Text("v1".to_owned())
    );
    let stale = inputs(
        scenario(&result, "demo.seal.Apply/outcome/stale"),
        "demo.seal.Apply",
    );
    let last = stale.last().expect("Apply is sent");
    assert_ne!(
        member(literal(&last["seal"]), "version"),
        &Node::Text("v1".to_owned())
    );
}

/// A guard that contradicts the invariant of the entity its branch creates selects only inputs
/// that entity refuses. The branch is refused naming both, and never sent.
#[test]
fn issue_234_a_guard_contradicting_the_entitys_invariant_refuses_the_branch_naming_both() {
    let model = seal(&["'seal.version == \"v1\"'"], "")
        .replace(
            "      - name: applied\n",
            "      - name: applied\n        when: 'seal.version == \"v2\"'\n",
        )
        .replace(
            "views:\n",
            "      - name: stale\n        error: demo.seal.Stale\nviews:\n",
        );
    let result = synthesize(&ir(&model));
    assert_eq!(failing(&result, &model), BTreeSet::new());
    let refused = refusals(&result);
    assert!(
        refused.iter().any(|refusal| {
            refusal.starts_with("demo.seal.Apply/outcome/applied: ESS-SYNTH-003:")
                && refusal.contains("`demo.seal.Stamp` invariant `seal.version == \"v1\"`")
                && refusal.contains("the branch's guard")
        }),
        "{refused:#?}"
    );
    assert!(!result
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == "demo.seal.Apply/outcome/applied"));
}
