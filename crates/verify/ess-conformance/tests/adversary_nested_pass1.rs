//! Adversary pass 1 for beyond10x/ess#234 (synthesized inputs satisfy invariants over nested
//! members). Each case states what a correct synthesis must do; none reads the implementation.
//!
//! The central property: a suite synthesized from a model never contains a scenario the
//! interpreter of that same model fails. Refusing at synthesis with a named cause is acceptable;
//! emitting a scenario a correct target fails is not.
#![allow(
    clippy::needless_raw_string_hashes,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::uninlined_format_args
)]

use std::collections::BTreeMap;

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

/// Scenario id -> failure detail, for every scenario the interpreter of `model` fails or errors.
fn failing(result: &Synthesis, model: &str) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
    let report = Runner::for_suite(&result.suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir(model)))
        .into_report();
    report
        .scenarios
        .iter()
        .filter(|scenario| matches!(scenario.status, Status::Failed | Status::Error))
        .map(|scenario| {
            let detail = format!("{scenario:?}");
            let from = detail.find("diagnostic: Some").unwrap_or(0);
            let short: String = detail[from..].chars().take(500).collect();
            (scenario.scenario.to_string(), short)
        })
        .collect()
}

/// No scenario of the suite fails against the model's own interpreter.
fn no_false_red(model: &str) -> Synthesis {
    let result = synthesize(&ir(model));
    let failed = failing(&result, model);
    assert!(
        failed.is_empty(),
        "a correct target fails {} synthesized scenario(s): {failed:#?}\nrefusals: {:#?}",
        failed.len(),
        refusals(&result)
    );
    result
}

fn has(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .iter()
        .any(|(key, _)| key.to_string() == id)
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

// ---- a request model, parameterised -------------------------------------------------------------

/// `Request` stores the `fingerprint` input whole. `FIELDS` are the struct's members, `INVARIANTS`
/// the entity's, `OUTCOMES` the outcomes of `Submit`.
const REQ: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.req
types:
  - {name: demo.req.RequestId, kind: newtype, of: Uuid}
  - {name: demo.req.Kind, kind: enum, variants: [Primary, Legacy]}
  - name: demo.req.Fingerprint
    kind: struct
    fields:
FIELDS
entities:
  - name: demo.req.Request
    identity: {name: id, type: demo.req.RequestId}
    fields:
      - {name: fingerprint, type: demo.req.Fingerprint}
    invariants:
INVARIANTS
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - {name: demo.req.Clerk, may: [demo.req.Submit]}
errors:
  - {name: demo.req.Rejected, summary: Rejected., fields: []}
events:
  - name: demo.req.Submitted
    fields: [{name: id, type: demo.req.RequestId}]
commands:
  - name: demo.req.Submit
    input:
      - {name: fingerprint, type: demo.req.Fingerprint}
    outcomes:
OUTCOMES
views:
  - name: demo.req.Requests
    source: demo.req.Request
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.req.RequestId}
      - {name: fingerprint, type: demo.req.Fingerprint}
"#;

const SUBMITTED: &str = r#"      - name: submitted
        creates: demo.req.Request
        instance: id
        sets: {fingerprint: input.fingerprint}
        emits: [demo.req.Submitted]
        payload:
          demo.req.Submitted: {id: {generated: true}}"#;

fn req(fields: &[&str], invariants: &[&str], outcomes: &str) -> String {
    let fields: Vec<String> = fields
        .iter()
        .map(|field| format!("      - {field}"))
        .collect();
    let invariants: Vec<String> = invariants
        .iter()
        .map(|invariant| format!("      - {invariant}"))
        .collect();
    REQ.replace("FIELDS", &fields.join("\n"))
        .replace("INVARIANTS", &invariants.join("\n"))
        .replace("OUTCOMES", outcomes)
}

const TEXTS: &[&str] = &[
    "{name: version, type: String}",
    "{name: digest, type: String}",
    "{name: origin, type: String}",
    "{name: route, type: String}",
];

// ---- gap (a): an entity invariant the bounded search cannot satisfy -----------------------------

/// A strict chain over five Integer members. The bounded ladder offers each member only the
/// values -1, 0, 1 and 2 (its base, 0, -1, and the others' base plus or minus one), four values for
/// five strictly ordered members. A correct synthesis either finds `a < b < c < d < e` or refuses
/// `Submit/outcome/submitted` with a named cause. Sending the base (every member at one value)
/// yields a scenario the model's own interpreter fails: the dangerous direction.
#[test]
fn adv_an_entity_invariant_out_of_bounded_reach_never_yields_a_scenario_a_correct_target_fails() {
    let model = req(
        &[
            "{name: a, type: Integer}",
            "{name: b, type: Integer}",
            "{name: c, type: Integer}",
            "{name: d, type: Integer}",
            "{name: e, type: Integer}",
        ],
        &[
            "'fingerprint.a < fingerprint.b'",
            "'fingerprint.b < fingerprint.c'",
            "'fingerprint.c < fingerprint.d'",
            "'fingerprint.d < fingerprint.e'",
        ],
        SUBMITTED,
    );
    no_false_red(&model);
}

/// `.count` over a list member, the shape the issue names ("`.count` ... on a struct"). The
/// repair drops every length choice ("only values"), so no candidate it tries changes the count.
#[test]
fn adv_an_entity_invariant_over_a_list_members_count_never_yields_a_failing_scenario() {
    let model = req(
        &[
            "{name: version, type: String}",
            "{name: parts, type: 'List<String>'}",
        ],
        &["'fingerprint.parts.count == 2'"],
        SUBMITTED,
    );
    no_false_red(&model);
}

/// A text length pinned by the entity (`digest.count == 64`, the issue's `value.count == 64`).
#[test]
fn adv_an_entity_invariant_over_a_text_members_length_never_yields_a_failing_scenario() {
    let model = req(TEXTS, &["'fingerprint.digest.count == 64'"], SUBMITTED);
    no_false_red(&model);
}

// ---- a guard that moves a leaf the invariant ties -----------------------------------------------

/// The accepting branch is guarded on `origin`, which the entity invariant couples to `route`.
/// The repair makes `origin == route` hold at the base; the guard's ladder then moves `origin` to
/// `"eu"` alone, and the accepted input creates a row the invariant refuses.
#[test]
fn adv_a_guard_moving_a_coupled_member_keeps_the_entity_invariant() {
    let outcomes = format!(
        "{}\n        when: 'fingerprint.origin == \"eu\"'\n      - name: elsewhere\n        error: demo.req.Rejected",
        SUBMITTED
    );
    let model = req(
        TEXTS,
        &["'fingerprint.origin == fingerprint.route'"],
        &outcomes,
    );
    let result = no_false_red(&model);
    assert!(
        has(&result, "demo.req.Submit/outcome/submitted"),
        "origin = route = \"eu\" satisfies both; refusals: {:#?}",
        refusals(&result)
    );
}

/// The same with an order: the guard raises `low` past the `high` the repair chose.
#[test]
fn adv_a_guard_raising_an_ordered_member_keeps_the_entity_invariant() {
    let outcomes = format!(
        "{}\n        when: 'fingerprint.low > 10'\n      - name: small\n        error: demo.req.Rejected",
        SUBMITTED
    );
    let model = req(
        &["{name: low, type: Integer}", "{name: high, type: Integer}"],
        &["'fingerprint.low < fingerprint.high'"],
        &outcomes,
    );
    no_false_red(&model);
}

// ---- invariant shapes ----------------------------------------------------------------------------

#[test]
fn adv_a_disjunctive_entity_invariant_is_met() {
    let model = req(
        TEXTS,
        &["{any: ['fingerprint.version == \"v1\"', 'fingerprint.version == \"v2\"']}"],
        SUBMITTED,
    );
    let result = no_false_red(&model);
    assert!(has(&result, "demo.req.Submit/outcome/submitted"));
}

#[test]
fn adv_a_starts_with_entity_invariant_is_met() {
    let model = req(
        TEXTS,
        &["{fingerprint.digest: {starts_with: \"sha256:\"}}"],
        SUBMITTED,
    );
    let result = no_false_red(&model);
    assert!(has(&result, "demo.req.Submit/outcome/submitted"));
}

#[test]
fn adv_an_entity_invariant_over_an_optional_member_is_met() {
    let model = req(
        &[
            "{name: version, type: 'Optional<String>'}",
            "{name: digest, type: String}",
        ],
        &["'fingerprint.version == \"v1\"'"],
        SUBMITTED,
    );
    let result = no_false_red(&model);
    assert!(has(&result, "demo.req.Submit/outcome/submitted"));
}

// ---- class 3: every branch, not only the conjunction of all -------------------------------------

/// Two creating branches copy the same input into two entities whose invariants pin one member to
/// two different literals. Each branch's own invariant is satisfiable; their conjunction is not.
/// Solving the conjunction of every branch's invariants fails, the base stays, and both branches
/// send a fingerprint their own entity refuses.
const TWO_ENTITIES: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.two
types:
  - {name: demo.two.Id, kind: newtype, of: Uuid}
  - {name: demo.two.Kind, kind: enum, variants: [Primary, Legacy]}
  - name: demo.two.Fingerprint
    kind: struct
    fields:
      - {name: version, type: String}
      - {name: digest, type: String}
entities:
  - name: demo.two.Current
    identity: {name: id, type: demo.two.Id}
    fields:
      - {name: fingerprint, type: demo.two.Fingerprint}
    invariants:
      - 'fingerprint.version == "v2"'
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.two.Old
    identity: {name: id, type: demo.two.Id}
    fields:
      - {name: fingerprint, type: demo.two.Fingerprint}
    invariants:
      - 'fingerprint.version == "v1"'
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - {name: demo.two.Clerk, may: [demo.two.Submit]}
events:
  - name: demo.two.Submitted
    fields: [{name: id, type: demo.two.Id}]
commands:
  - name: demo.two.Submit
    input:
      - {name: kind, type: demo.two.Kind}
      - {name: fingerprint, type: demo.two.Fingerprint}
    outcomes:
      - name: current
        when: kind == Primary
        creates: demo.two.Current
        instance: id
        sets: {fingerprint: input.fingerprint}
        emits: [demo.two.Submitted]
        payload:
          demo.two.Submitted: {id: {generated: true}}
      - name: old
        creates: demo.two.Old
        instance: id
        sets: {fingerprint: input.fingerprint}
        emits: [demo.two.Submitted]
        payload:
          demo.two.Submitted: {id: {generated: true}}
views:
  - name: demo.two.Currents
    source: demo.two.Current
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.two.Id}
      - {name: fingerprint, type: demo.two.Fingerprint}
  - name: demo.two.Olds
    source: demo.two.Old
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.two.Id}
      - {name: fingerprint, type: demo.two.Fingerprint}
"#;

#[test]
fn adv_each_branch_meets_its_own_entitys_invariant_when_two_branches_disagree() {
    let result = no_false_red(TWO_ENTITIES);
    for id in [
        "demo.two.Submit/outcome/current",
        "demo.two.Submit/outcome/old",
    ] {
        assert!(has(&result, id), "{id}; refusals: {:#?}", refusals(&result));
    }
}

// ---- class 1: a further instance of a member an invariant bounds, not pins ----------------------

/// `start >= 5` bounds `start`; it does not pin it to one literal. The plain witness and a further
/// instance both repair `start` to the invariant's literal, so two instances carry one value.
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
    invariants:
INVARIANTS
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

fn window(invariants: &[&str]) -> String {
    let lines: Vec<String> = invariants
        .iter()
        .map(|invariant| format!("      - {invariant}"))
        .collect();
    WINDOW.replace("INVARIANTS", &lines.join("\n"))
}

#[test]
fn adv_a_further_instance_differs_on_a_member_an_invariant_bounds_but_does_not_pin() {
    use ess_conformance::witness::{candidates, Distinction};
    let ir = ir(&window(&["'start >= 5'"]));
    let command = &ir.commands()[&"demo.win.Book".parse().unwrap()];
    let base = |distinction| {
        candidates(&ir, command, &[], distinction)
            .unwrap()
            .into_iter()
            .next()
            .expect("a candidate is admitted")
    };
    let first = base(Distinction::PLAIN);
    let second = base(Distinction::further(1));
    assert_ne!(
        member(&first["window"], "start"),
        member(&second["window"], "start"),
        "two instances carry one start: {first:?} / {second:?}"
    );
}

/// Acceptance 2, a struct type whose invariants bound two members (a struct invariant cannot compare two bare members).
#[test]
fn adv_a_struct_type_bounding_two_members_is_synthesizable() {
    let model = window(&["'start >= 5'", "'end >= 20'"]);
    let result = no_false_red(&model);
    assert!(
        has(&result, "demo.win.Book/outcome/booked"),
        "refusals: {:#?}",
        refusals(&result)
    );
}

// ---- gap 3: the default branch beside a guard over a stored optional struct ---------------------

/// The implementor's META model with `Annotate` setting `meta` to `{tier: Basic, note: generated}`:
/// the tier the guard reads is a literal the arrangement writes, and a row with `meta` absent is
/// reachable from `Make` alone. Acceptance: "a `when_subject` over a member of a stored optional
/// struct refuses at most its own branch, never the command's other outcomes". `touched` is
/// another outcome.
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
        sets: {meta: {tier: Basic, note: {generated: true}}}
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

#[test]
fn adv_the_default_branch_beside_a_guard_over_a_stored_optional_struct_is_witnessed() {
    let result = no_false_red(META);
    assert!(
        has(&result, "demo.meta.Touch/outcome/touched"),
        "touched is another outcome of the command; refusals: {:#?}",
        refusals(&result)
    );
}

// ---- gap 4 and gap 5 with the operands swapped, and through a newtype ---------------------------

const FENCE: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.doc
types:
  - {name: demo.doc.DocId, kind: newtype, of: Uuid}
  - {name: demo.doc.Fence, kind: newtype, of: String}
  - name: demo.doc.Publication
    kind: struct
    fields:
      - {name: expected_fence, type: demo.doc.Fence}
      - {name: channel, type: String}
entities:
  - name: demo.doc.Doc
    identity: {name: id, type: demo.doc.DocId}
    fields:
      - {name: fence, type: demo.doc.Fence}
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
    input: [{name: fence, type: demo.doc.Fence}]
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
      - {name: fence, type: demo.doc.Fence}
      - {name: state, type: demo.doc.Doc.State}
"#;

#[test]
fn adv_gap4_through_a_newtype_is_witnessed() {
    let result = no_false_red(FENCE);
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
        let sent = inputs(the, "demo.doc.Publish");
        let last = sent.last().expect("Publish is sent");
        let expected = member(literal(&last["publication"]), "expected_fence");
        if id.ends_with("/stale") {
            assert!(
                !written.contains(expected),
                "{id}: {written:?} {expected:?}"
            );
        } else {
            assert!(written.contains(expected), "{id}: {written:?} {expected:?}");
        }
    }
}

const CLAIM: &str = r#"format: ess/18
system: demo
version: v1
domain: demo.claim
types:
  - {name: demo.claim.ClaimId, kind: newtype, of: Uuid}
  - {name: demo.claim.Owner, kind: newtype, of: String, invariants: ['value != ""']}
  - name: demo.claim.Ticket
    kind: struct
    fields:
      - {name: owner, type: demo.claim.Owner}
      - {name: seat, type: String}
entities:
  - name: demo.claim.Claim
    identity: {name: id, type: demo.claim.ClaimId}
    fields:
      - {name: owner, type: demo.claim.Owner}
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
      - {name: owner, type: demo.claim.Owner}
      - {name: ticket, type: demo.claim.Ticket}
    outcomes:
      - name: not-yours
        when: owner != ticket.owner
        error: demo.claim.NotYours
      - name: redeemed
        creates: demo.claim.Claim
        instance: id
        sets: {owner: input.owner}
        emits: [demo.claim.Redeemed]
        payload:
          demo.claim.Redeemed: {id: {generated: true}}
views:
  - name: demo.claim.Claims
    source: demo.claim.Claim
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.claim.ClaimId}
      - {name: owner, type: demo.claim.Owner}
"#;

/// Gap 5 with the refusal first and the owner a newtype (a bare right-hand side is a literal, so the operands cannot be swapped): the default branch
/// is taken only where the two are equal.
#[test]
fn adv_gap5_inequality_refusal_first_through_a_newtype_is_witnessed() {
    let result = no_false_red(CLAIM);
    let sent = inputs(
        scenario(&result, "demo.claim.Redeem/outcome/redeemed"),
        "demo.claim.Redeem",
    );
    let last = sent.last().expect("Redeem is sent");
    let ticket = literal(&last["ticket"]);
    assert_eq!(literal(&last["owner"]), member(ticket, "owner"));
    // Class 1: the two members of the ticket differ.
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
