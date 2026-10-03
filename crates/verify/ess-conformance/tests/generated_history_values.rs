//! Outcome-only histories never observe a generated stored value merely by minting a witness.
//! Native generation stays concrete; history decisions need their own observation authority.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    history::{self, History, Verdict},
    linearize::{self, CheckRefusal},
    scenario::SuiteProvenance,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use std::fmt::Write as _;

const GENERATED_TIMESTAMP: &str = r#"
format: ess/18
system: demo
version: v1
domain: demo.sessions
entities:
  - name: demo.sessions.Session
    identity: {name: session_id, type: Uuid}
    fields:
      - {name: paused, type: Boolean}
      - {name: login_at, type: Timestamp}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
events:
  - name: demo.sessions.Started
    fields: [{name: session_id, type: Uuid}]
commands:
  - name: demo.sessions.Start
    input: []
    outcomes:
      - name: started
        creates: demo.sessions.Session
        instance: session_id
        sets: {paused: 'false', login_at: {generated: true}}
        emits: [demo.sessions.Started]
        payload: {demo.sessions.Started: {session_id: {generated: true}}}
  - name: demo.sessions.Inspect
    input: [{name: session_id, type: Uuid}]
    outcomes:
      - name: early
        when_subject: {predicate: 'login_at < "2021-01-01T00:00:00Z"'}
        preserves: demo.sessions.Session
        instance: session_id
      - name: late
        preserves: demo.sessions.Session
        instance: session_id
"#;

const GENERATED_NARROW_TO_WIDE: &str = r"
format: ess/18
system: demo
version: v1
domain: demo.counter
types:
  - name: demo.counter.Narrow
    kind: newtype
    of: Integer
    invariants: ['value >= 0', 'value <= 10']
  - name: demo.counter.Wide
    kind: newtype
    of: Integer
conversions:
  - from: demo.counter.Narrow
    to: demo.counter.Wide
    because: Preserve the integer representation while allowing subsequent arithmetic.
entities:
  - name: demo.counter.Counter
    identity: {name: counter_id, type: Uuid}
    fields:
      - {name: sample, type: demo.counter.Narrow}
      - {name: count, type: demo.counter.Wide}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
events:
  - name: demo.counter.Started
    fields: [{name: counter_id, type: Uuid}]
  - name: demo.counter.Changed
    fields: [{name: counter_id, type: Uuid}]
commands:
  - name: demo.counter.Start
    input: []
    outcomes:
      - name: started
        creates: demo.counter.Counter
        instance: counter_id
        sets: {sample: {generated: true}, count: 0}
        emits: [demo.counter.Started]
        payload: {demo.counter.Started: {counter_id: {generated: true}}}
  - name: demo.counter.Copy
    input: [{name: counter_id, type: Uuid}]
    outcomes:
      - name: copied
        updates: demo.counter.Counter
        instance: counter_id
        sets: {count: {subject: sample}}
        emits: [demo.counter.Changed]
        payload: {demo.counter.Changed: {counter_id: input.counter_id}}
  - name: demo.counter.Advance
    input: [{name: counter_id, type: Uuid}]
    outcomes:
      - name: advanced
        updates: demo.counter.Counter
        instance: counter_id
        sets: {count: {increment: 1}}
        emits: [demo.counter.Changed]
        payload: {demo.counter.Changed: {counter_id: input.counter_id}}
  - name: demo.counter.Inspect
    input: [{name: counter_id, type: Uuid}]
    outcomes:
      - name: bounded
        when_subject: {predicate: {all: ['count >= 1', 'count <= 12']}}
        preserves: demo.counter.Counter
        instance: counter_id
      - name: other
        preserves: demo.counter.Counter
        instance: counter_id
";

fn model(source: &str) -> EssIr {
    let raw = RawSpecFile::parse(source).expect("the source parses");
    let spec = Specification::assemble([(Source::new("history-values.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the source is admitted: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("history-values.yaml", source);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("the source compiles: {errors}"))
}

/// Exactly the history envelope a recorder supplies: outcomes and subject identity, no timestamps
/// masquerading as observed input or generated-field authority.
fn recorded(ir: &EssIr, outcome: Option<&str>) -> History {
    let mut operations = vec![serde_json::json!({
        "operation_id": "00000000-0000-4000-8000-000000000201",
        "client": 0, "command": "demo.sessions.Start",
        "subject_key": "00000000-0000-4000-8000-00000000c0de",
        "invoked_at": 1, "returned_at": 2, "completion": "Returned", "outcome": "started"
    })];
    if let Some(outcome) = outcome {
        operations.push(serde_json::json!({
            "operation_id": "00000000-0000-4000-8000-000000000202",
            "client": 0, "command": "demo.sessions.Inspect",
            "subject_key": "00000000-0000-4000-8000-00000000c0de",
            "invoked_at": 3, "returned_at": 4, "completion": "Returned", "outcome": outcome
        }));
    }
    let digest = SuiteProvenance::of(ir).spec_digest;
    let document = serde_json::json!({
        "format": "ess-history/1", "history_id": "00000000-0000-4000-8000-00000000f292",
        "spec_digest": digest, "seed": 0, "clients": 1, "operations": operations
    });
    history::read(
        &serde_json::to_vec(&document).expect("JSON history"),
        &digest,
    )
    .unwrap_or_else(|error| panic!("the actual history reader admits it: {error}"))
}

fn widening_history(ir: &EssIr, advances: usize, inspection: Option<&str>) -> History {
    let subject = "00000000-0000-4000-8000-00000000c292";
    let mut operations = vec![
        serde_json::json!({
            "operation_id":"00000000-0000-4000-8000-000000000291", "client":0,
            "command":"demo.counter.Start", "subject_key":subject, "invoked_at":1,
            "returned_at":2, "completion":"Returned", "outcome":"started"
        }),
        serde_json::json!({
            "operation_id":"00000000-0000-4000-8000-000000000292", "client":0,
            "command":"demo.counter.Copy", "subject_key":subject, "invoked_at":3,
            "returned_at":4, "completion":"Returned", "outcome":"copied"
        }),
    ];
    let mut instant = 5_u64;
    for index in 0..advances {
        operations.push(serde_json::json!({
            "operation_id":format!("00000000-0000-4000-8000-{:012x}", 0x293 + index),
            "client":0, "command":"demo.counter.Advance", "subject_key":subject,
            "invoked_at":instant, "returned_at":instant + 1,
            "completion":"Returned", "outcome":"advanced"
        }));
        instant += 2;
    }
    if let Some(outcome) = inspection {
        operations.push(serde_json::json!({
            "operation_id":"00000000-0000-4000-8000-000000000299", "client":0,
            "command":"demo.counter.Inspect", "subject_key":subject,
            "invoked_at":instant, "returned_at":instant + 1,
            "completion":"Returned", "outcome":outcome
        }));
    }
    let digest = SuiteProvenance::of(ir).spec_digest;
    let document = serde_json::json!({
        "format":"ess-history/1", "history_id":"00000000-0000-4000-8000-00000000f293",
        "spec_digest":digest, "seed":0, "clients":1, "operations":operations
    });
    history::read(&serde_json::to_vec(&document).unwrap(), &digest)
        .unwrap_or_else(|error| panic!("the widening history is admitted: {error}"))
}

#[test]
fn mechanical_widening_and_safe_nonzero_increment_retain_the_complete_domain() {
    let ir = model(GENERATED_NARROW_TO_WIDE);
    for (advances, inspection) in [(0, None), (1, Some("bounded")), (2, Some("bounded"))] {
        let result = linearize::check(
            &ir,
            &widening_history(&ir, advances, inspection),
            linearize::DEFAULT_BUDGET,
        );
        assert_eq!(
            result.unwrap().verdict,
            Verdict::Linearizable,
            "{advances} safe increments and {inspection:?}"
        );
    }

    let uncertain = GENERATED_NARROW_TO_WIDE
        .replace("name: bounded", "name: lower")
        .replace("{all: ['count >= 1', 'count <= 12']}", "'count < 5'");
    let ir = model(&uncertain);
    for outcome in ["lower", "other"] {
        assert!(matches!(
            linearize::check(
                &ir,
                &widening_history(&ir, 1, Some(outcome)),
                linearize::DEFAULT_BUDGET
            ),
            Err(CheckRefusal::Model { .. })
        ));
    }

    let partially_narrowing = GENERATED_NARROW_TO_WIDE.replace(
        "  - name: demo.counter.Wide\n    kind: newtype\n    of: Integer\n",
        "  - name: demo.counter.Wide\n    kind: newtype\n    of: Integer\n    invariants: ['value <= 5']\n",
    );
    let ir = model(&partially_narrowing);
    assert!(matches!(
        linearize::check(
            &ir,
            &widening_history(&ir, 0, None),
            linearize::DEFAULT_BUDGET
        ),
        Err(CheckRefusal::Model { .. })
    ));
}

fn assert_generated_decision_unresolved(outcome: &str) {
    let ir = model(GENERATED_TIMESTAMP);
    let checked = linearize::check(
        &ir,
        &recorded(&ir, Some(outcome)),
        linearize::DEFAULT_BUDGET,
    );
    assert!(
        matches!(&checked, Err(CheckRefusal::Model { operation_id, .. })
            if operation_id == "00000000-0000-4000-8000-000000000202"),
        "an unrecorded generated timestamp must not decide `{outcome}`: {checked:?}"
    );
}

#[test]
fn inert_generated_timestamp_creation_remains_checkable() {
    let ir = model(GENERATED_TIMESTAMP);
    let checked = linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET)
        .expect("an inhabited generated field independent of the outcome stays checkable");
    assert_eq!(checked.verdict, Verdict::Linearizable);
}

#[test]
fn generated_timestamp_cannot_prove_the_early_outcome() {
    assert_generated_decision_unresolved("early");
}

#[test]
fn generated_timestamp_cannot_disprove_the_late_outcome() {
    assert_generated_decision_unresolved("late");
}

#[test]
fn source_known_integer_still_decides_both_outcomes() {
    // Timestamp sets literals are not source-admitted. A source-known Integer supplies the
    // concrete-value control without expanding the source language to suit the test.
    let source = GENERATED_TIMESTAMP
        .replace("type: Timestamp", "type: Integer")
        .replace("login_at: {generated: true}", "login_at: 0")
        .replace("login_at < \"2021-01-01T00:00:00Z\"", "login_at < 1");
    let ir = model(&source);
    for (outcome, verdict) in [
        ("early", Verdict::Linearizable),
        ("late", Verdict::Violation),
    ] {
        let checked = linearize::check(
            &ir,
            &recorded(&ir, Some(outcome)),
            linearize::DEFAULT_BUDGET,
        )
        .expect("the literal is actual source authority");
        assert_eq!(checked.verdict, verdict, "the known `{outcome}` outcome");
    }
}
// These fixtures passed source admission in expanded-source-valid-red.log. Their named baseline
// results distinguish existing safety controls from the missing abstract/shared-history behavior.

fn domain_source(declarations: &str, field_type: &str) -> String {
    let creation = GENERATED_TIMESTAMP
        .split("  - name: demo.sessions.Inspect")
        .next()
        .expect("creation prefix");
    creation.replace("type: Timestamp", &format!("type: {field_type}"))
        + "\ntypes:\n"
        + declarations
}

fn assert_required_generation_cannot_complete(declarations: &str) {
    let ir = model(&domain_source(declarations, "demo.sessions.Value"));
    let result = linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET);
    assert!(
        matches!(&result, Err(CheckRefusal::Model { .. }))
            || matches!(&result, Ok(checked) if checked.verdict == Verdict::Violation),
        "an empty required generated domain cannot prove an executable operation: {result:?}"
    );
}

#[test]
fn required_generated_boolean_with_contradictory_constraints_cannot_complete() {
    assert_required_generation_cannot_complete(
        "  - {name: demo.sessions.Value, kind: newtype, of: Boolean, invariants: ['value == true', 'value == false']}\n",
    );
}

#[test]
fn required_generated_enum_with_every_member_excluded_cannot_complete() {
    assert_required_generation_cannot_complete(
        "  - {name: demo.sessions.Choice, kind: enum, variants: [One, Two]}\n  - {name: demo.sessions.Value, kind: newtype, of: demo.sessions.Choice, invariants: ['value != One', 'value != Two']}\n",
    );
}

#[test]
fn required_generated_struct_must_satisfy_joint_constraints() {
    assert_required_generation_cannot_complete(
        "  - name: demo.sessions.Pair\n    kind: struct\n    fields: [{name: left, type: Integer}, {name: right, type: Integer}]\n  - name: demo.sessions.Value\n    kind: struct\n    fields: [{name: window, type: demo.sessions.Pair}]\n    invariants: ['window.left < window.right', 'window.left >= window.right']\n",
    );
}

#[test]
fn optional_empty_generated_domain_has_a_valid_absent_value() {
    let source = domain_source(
        "  - {name: demo.sessions.Value, kind: newtype, of: Boolean, invariants: ['value == true', 'value == false']}\n",
        "Optional<demo.sessions.Value>",
    );
    let ir = model(&source);
    let result = linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET)
        .expect("Optional of an empty domain still admits absence");
    assert_eq!(result.verdict, Verdict::Linearizable);
}

#[test]
fn required_inhabited_generated_constraint_remains_checkable_when_inert() {
    let ir = model(&domain_source(
        "  - {name: demo.sessions.Value, kind: newtype, of: Boolean, invariants: ['value == true']}\n",
        "demo.sessions.Value",
    ));
    let result = linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET)
        .expect("a checked witness proves existence, never observed value authority");
    assert_eq!(result.verdict, Verdict::Linearizable);
}

const CROSS_ROW_SESSION: &str = r#"
format: ess/20
system: demo
version: v1
domain: demo.cross
types:
  - {name: demo.cross.AId, kind: newtype, of: String, invariants: ['value == "a"']}
  - {name: demo.cross.BId, kind: newtype, of: String, invariants: ['value == "b"']}
entities:
  - name: demo.cross.A
    identity: {name: a_id, type: demo.cross.AId}
    fields: [{name: touched, type: Boolean}]
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.cross.B
    identity: {name: b_id, type: demo.cross.BId}
    fields: [{name: touched, type: Boolean}]
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
events:
  - {name: demo.cross.ACreated, fields: [{name: a_id, type: demo.cross.AId}]}
  - {name: demo.cross.BCreated, fields: [{name: b_id, type: demo.cross.BId}]}
  - {name: demo.cross.Touched, fields: [{name: b_id, type: demo.cross.BId}]}
errors:
  - {name: demo.cross.NoB, fields: []}
  - {name: demo.cross.NotActive, fields: []}
commands:
  - name: demo.cross.CreateA
    outcomes:
      - name: created
        creates: demo.cross.A
        instance: a_id
        sets: {touched: false}
        emits: [demo.cross.ACreated]
        payload: {demo.cross.ACreated: {a_id: {generated: true}}}
  - name: demo.cross.CreateB
    outcomes:
      - name: created
        creates: demo.cross.B
        instance: b_id
        sets: {touched: false}
        emits: [demo.cross.BCreated]
        payload: {demo.cross.BCreated: {b_id: {generated: true}}}
  - name: demo.cross.TouchThroughA
    input: [{name: a_id, type: demo.cross.AId}, {name: b_id, type: demo.cross.BId}]
    outcomes:
      - {name: no-b, when_related: {via: input.b_id, exists: false}, error: demo.cross.NoB}
      - name: touched
        when_related: {via: input.b_id, predicate: state == Active}
        emits: [demo.cross.Touched]
        payload: {demo.cross.Touched: {b_id: input.b_id}}
        updates: demo.cross.A
        instance: a_id
        sets: {touched: true}
        affects:
          - entity: demo.cross.B
            where: touched == false
            sets: {touched: true}
      - {name: not-active, error: demo.cross.NotActive}
  - name: demo.cross.TouchAllB
    input: [{name: b_id, type: demo.cross.BId}]
    outcomes:
      - name: touched
        emits: [demo.cross.Touched]
        payload: {demo.cross.Touched: {b_id: input.b_id}}
        updates: demo.cross.B
        instances: {where: touched == false}
        sets: {touched: true}
views:
  - name: demo.cross.AllB
    source: demo.cross.B
    consistency: read_your_writes
    fields: [{name: b_id, type: demo.cross.BId}]
"#;

#[test]
fn native_cross_row_control_really_updates_a_and_b() {
    use ess_conformance::interpret::execute::{
        execute, execute_generating, Externals, Generated, GeneratedSlot, Store,
    };
    use ess_primitives::node::Node;
    use std::collections::BTreeMap;

    let ir = model(CROSS_ROW_SESSION);
    let mut store = Store::default();
    for (command, event, field, identity) in [
        ("CreateA", "ACreated", "a_id", "a"),
        ("CreateB", "BCreated", "b_id", "b"),
    ] {
        let mut steps = execute_generating(
            &ir,
            &store,
            &format!("demo.cross.{command}").parse().unwrap(),
            &BTreeMap::new(),
            &Externals::Withheld,
            &Generated::Given(BTreeMap::from([(
                GeneratedSlot::new(format!("demo.cross.{event}").parse().unwrap(), field),
                Node::Text(identity.into()),
            )])),
        )
        .expect("native concrete creation accepts the actual published identity");
        assert_eq!(steps.len(), 1);
        store = steps.remove(0).next;
    }
    let input = BTreeMap::from([
        ("a_id".into(), Node::Text("a".into())),
        ("b_id".into(), Node::Text("b".into())),
    ]);
    let mut steps = execute(
        &ir,
        &store,
        &"demo.cross.TouchThroughA".parse().unwrap(),
        &input,
        &Externals::Withheld,
    )
    .expect("the admitted related guard and affects compose in native execution");
    assert_eq!(steps.len(), 1);
    let step = steps.remove(0);
    assert_eq!(
        step.outcome.unwrap().to_string(),
        "demo.cross.TouchThroughA/touched"
    );
    for (entity, identity) in [("A", "a"), ("B", "b")] {
        let row = step
            .next
            .instance(&format!("demo.cross.{entity}").parse().unwrap(), identity)
            .expect("the actual row is still held");
        assert_eq!(row.fields["touched"], Node::Bool(true));
    }
}

fn cross_row_history(ir: &EssIr, reader: u64, rows: &[&str], addressed: bool) -> History {
    let digest = SuiteProvenance::of(ir).spec_digest;
    let document = serde_json::json!({
        "format": "ess-history/1", "history_id": "00000000-0000-4000-8000-00000000f293",
        "spec_digest": digest, "seed": 0, "clients": 3,
        "operations": [
            {"operation_id":"00000000-0000-4000-8000-000000000301", "client":1,
             "command":"demo.cross.CreateA", "subject_key":"a", "invoked_at":1,
             "returned_at":2, "completion":"Returned", "outcome":"created"},
            {"operation_id":"00000000-0000-4000-8000-000000000302", "client":1,
             "command":"demo.cross.CreateB", "subject_key":"b", "invoked_at":3,
             "returned_at":4, "completion":"Returned", "outcome":"created"},
            {"operation_id":"00000000-0000-4000-8000-000000000303", "client":0,
             "command": if addressed {"demo.cross.TouchThroughA"} else {"demo.cross.TouchAllB"},
             "subject_key": if addressed {"a"} else {""}, "invoked_at":5,
             "returned_at":6, "completion":"Returned", "outcome":"touched"},
            {"operation_id":"00000000-0000-4000-8000-000000000304", "client":reader,
             "command":"demo.cross.AllB", "subject_key":"", "invoked_at":7,
             "returned_at":8, "completion":"Returned", "outcome":"rows", "rows": rows}
        ]
    });
    history::read(&serde_json::to_vec(&document).unwrap(), &digest)
        .expect("outcomes and row identities only, through the actual history reader")
}

#[test]
fn acknowledged_a_addressed_write_requires_related_b_in_its_clients_view() {
    let ir = model(CROSS_ROW_SESSION);
    let result = linearize::check(
        &ir,
        &cross_row_history(&ir, 0, &[], true),
        linearize::DEFAULT_BUDGET,
    )
    .expect("the supported cross-row history is judged");
    assert_eq!(result.verdict, Verdict::Violation, "{result:?}");
    let read = result
        .read
        .expect("a stale read, not a command partition failure");
    assert_eq!(read.anomaly, linearize::Anomaly::StaleRead);
    assert_eq!(read.subject_key, "b");
    assert_eq!(read.operation_id, "00000000-0000-4000-8000-000000000304");
}

#[test]
fn acknowledged_a_addressed_write_and_current_b_view_are_consistent() {
    let ir = model(CROSS_ROW_SESSION);
    let result = linearize::check(
        &ir,
        &cross_row_history(&ir, 0, &["b"], true),
        linearize::DEFAULT_BUDGET,
    )
    .expect("cross-row command selection sees the actual B row");
    assert_eq!(result.verdict, Verdict::Linearizable, "{result:?}");
    assert_eq!(result.judged, 1);
}

#[test]
fn another_clients_b_view_may_precede_the_a_addressed_write() {
    let ir = model(CROSS_ROW_SESSION);
    let result = linearize::check(
        &ir,
        &cross_row_history(&ir, 2, &[], true),
        linearize::DEFAULT_BUDGET,
    )
    .expect("another client's acknowledged writes are not this reader's session barrier");
    assert_eq!(result.verdict, Verdict::Linearizable, "{result:?}");
    assert_eq!(result.judged, 1);
}

#[test]
fn acknowledged_subjectless_set_write_also_requires_b_in_its_clients_view() {
    let ir = model(CROSS_ROW_SESSION);
    let result = linearize::check(
        &ir,
        &cross_row_history(&ir, 0, &[], false),
        linearize::DEFAULT_BUDGET,
    )
    .expect("a subjectless set command participates in shared history state");
    assert_eq!(result.verdict, Verdict::Violation, "{result:?}");
    let read = result
        .read
        .expect("a stale read, not a command partition failure");
    assert_eq!(read.anomaly, linearize::Anomaly::StaleRead);
    assert_eq!(read.subject_key, "b");
}

fn assert_history_outcome(source: &str, outcome: &str, expected: Verdict) {
    let ir = model(source);
    let checked = linearize::check(
        &ir,
        &recorded(&ir, Some(outcome)),
        linearize::DEFAULT_BUDGET,
    )
    .expect("only necessary unknown decisions refuse");
    assert_eq!(checked.verdict, expected, "{checked:?}");
}

#[test]
fn false_input_conjunct_resolves_an_unknown_stored_predicate() {
    let source = GENERATED_TIMESTAMP.replace(
        "      - name: early\n",
        "      - name: early\n        when: false\n",
    );
    assert_history_outcome(&source, "late", Verdict::Linearizable);
}

#[test]
fn required_generation_has_known_presence_without_known_content() {
    let source =
        GENERATED_TIMESTAMP.replace("login_at < \"2021-01-01T00:00:00Z\"", "defined(login_at)");
    assert_history_outcome(&source, "early", Verdict::Linearizable);
}

#[test]
fn optional_generated_presence_is_not_fabricated_as_absence() {
    let source = GENERATED_TIMESTAMP
        .replace("type: Timestamp", "type: Optional<Timestamp>")
        .replace("login_at < \"2021-01-01T00:00:00Z\"", "defined(login_at)");
    let ir = model(&source);
    for outcome in ["early", "late"] {
        let result = linearize::check(
            &ir,
            &recorded(&ir, Some(outcome)),
            linearize::DEFAULT_BUDGET,
        );
        assert!(
            matches!(result, Err(CheckRefusal::Model { .. })),
            "{outcome}: {result:?}"
        );
    }
}

#[test]
fn a_known_struct_sibling_survives_an_unknown_generated_leaf() {
    let source = GENERATED_TIMESTAMP
        .replace("type: Timestamp", "type: demo.sessions.Packet")
        .replace("login_at: {generated: true}", "login_at: {stamp: {generated: true}, ready: true}")
        .replace("login_at < \"2021-01-01T00:00:00Z\"", "login_at.ready == true")
        + "\ntypes:\n  - name: demo.sessions.Packet\n    kind: struct\n    fields: [{name: stamp, type: Optional<Timestamp>}, {name: ready, type: Boolean}]\n";
    assert_history_outcome(&source, "early", Verdict::Linearizable);
}

#[test]
fn generated_dependent_entity_invariant_is_unresolved() {
    let source = GENERATED_TIMESTAMP.replace(
        "    lifecycle:",
        "    invariants: ['login_at < \"2021-01-01T00:00:00Z\"']\n    lifecycle:",
    );
    let ir = model(&source);
    let result = linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET);
    assert!(
        matches!(result, Err(CheckRefusal::Model { .. })),
        "{result:?}"
    );
}

#[test]
fn known_true_disjunction_resolves_a_generated_dependent_invariant() {
    let source = GENERATED_TIMESTAMP.replace(
        "    lifecycle:",
        "    invariants: [{any: ['paused == false', 'login_at < \"2021-01-01T00:00:00Z\"']}]\n    lifecycle:",
    );
    let ir = model(&source);
    let result = linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET)
        .expect("a true disjunct resolves the invariant");
    assert_eq!(result.verdict, Verdict::Linearizable);
}

#[test]
fn a_resolved_generated_subpredicate_does_not_taint_an_unwritten_legacy_fact() {
    let source = GENERATED_TIMESTAMP
        .replace("      - {name: paused, type: Boolean}", "      - {name: paused, type: Boolean}\n      - {name: unwritten, type: Optional<Integer>}")
        .replace("    lifecycle:", "    invariants: [{all: [{any: ['paused == false', 'login_at < \"2021-01-01T00:00:00Z\"']}, 'unwritten > 0']}]\n    lifecycle:");
    let ir = model(&source);
    let result = linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET)
        .expect("the remaining unknown has no unrecorded generated dependency");
    assert_eq!(result.verdict, Verdict::Linearizable);
}

fn mutation_source(field_type: &str, sets: &str, predicate: &str) -> String {
    let inserted = format!(
        "  - name: demo.sessions.Change\n    input: [{{name: session_id, type: Uuid}}]\n    outcomes:\n      - name: changed\n        updates: demo.sessions.Session\n        instance: session_id\n        sets: {sets}\n        emits: [demo.sessions.Started]\n        payload: {{demo.sessions.Started: {{session_id: input.session_id}}}}\n"
    );
    GENERATED_TIMESTAMP
        .replace("type: Timestamp", &format!("type: {field_type}"))
        .replace("login_at < \"2021-01-01T00:00:00Z\"", predicate)
        .replace(
            "  - name: demo.sessions.Inspect",
            &(inserted + "  - name: demo.sessions.Inspect"),
        )
}

fn changed_history(ir: &EssIr, outcome: &str) -> History {
    let mut written = serde_json::to_value(recorded(ir, Some(outcome))).unwrap();
    let operations = written["operations"].as_array_mut().unwrap();
    operations[1]["invoked_at"] = serde_json::json!(5);
    operations[1]["returned_at"] = serde_json::json!(6);
    operations.insert(
        1,
        serde_json::json!({
            "operation_id":"00000000-0000-4000-8000-000000000203", "client":0,
            "command":"demo.sessions.Change", "subject_key":"00000000-0000-4000-8000-00000000c0de",
            "invoked_at":3, "returned_at":4, "completion":"Returned", "outcome":"changed"
        }),
    );
    history::read(
        &serde_json::to_vec(&written).unwrap(),
        &SuiteProvenance::of(ir).spec_digest,
    )
    .unwrap()
}

#[test]
fn a_known_overwrite_clears_the_generated_value_dependency() {
    let ir = model(&mutation_source("Integer", "{login_at: 0}", "login_at < 1"));
    let result = linearize::check(
        &ir,
        &changed_history(&ir, "early"),
        linearize::DEFAULT_BUDGET,
    )
    .expect("the source-written value replaces its old unknown origin");
    assert_eq!(result.verdict, Verdict::Linearizable);
}

#[test]
fn a_known_clear_recovers_optional_absence() {
    let ir = model(&mutation_source(
        "Optional<Integer>",
        "{login_at: {cleared: true}}",
        "defined(login_at)",
    ));
    let result = linearize::check(
        &ir,
        &changed_history(&ir, "late"),
        linearize::DEFAULT_BUDGET,
    )
    .expect("clearing establishes known absence");
    assert_eq!(result.verdict, Verdict::Linearizable);
}

#[test]
fn same_outcome_copy_retains_the_unknown_pre_overwrite_value() {
    let source = mutation_source(
        "Integer",
        "{login_at: 0, copied: {subject: login_at}}",
        "copied < 1",
    )
    .replace(
        "      - {name: paused, type: Boolean}",
        "      - {name: paused, type: Boolean}\n      - {name: copied, type: Integer}",
    )
    .replace(
        "sets: {paused: 'false', login_at: {generated: true}}",
        "sets: {paused: 'false', login_at: {generated: true}, copied: 0}",
    );
    let ir = model(&source);
    let result = linearize::check(
        &ir,
        &changed_history(&ir, "early"),
        linearize::DEFAULT_BUDGET,
    );
    assert!(
        matches!(result, Err(CheckRefusal::Model { .. })),
        "{result:?}"
    );
}

fn provider_before_unknown() -> String {
    GENERATED_TIMESTAMP.replace("      - name: early", "      - name: offered\n        external: The provider answers before stored fallback selection.\n        preserves: demo.sessions.Session\n        instance: session_id\n      - name: early")
}

#[test]
fn a_proven_provider_alternative_survives_an_unresolved_fallback() {
    assert_history_outcome(&provider_before_unknown(), "offered", Verdict::Linearizable);
}

#[test]
fn a_dead_provider_alternative_does_not_erase_an_unresolved_fallback() {
    let ir = model(&provider_before_unknown());
    let result = linearize::check(
        &ir,
        &recorded(&ir, Some("early")),
        linearize::DEFAULT_BUDGET,
    );
    assert!(
        matches!(result, Err(CheckRefusal::Model { .. })),
        "{result:?}"
    );
}

#[test]
fn an_indeterminate_unresolved_call_may_not_have_happened() {
    let ir = model(GENERATED_TIMESTAMP);
    let mut document = serde_json::to_value(recorded(&ir, Some("early"))).unwrap();
    document["operations"][1]["completion"] = serde_json::json!("Indeterminate");
    document["operations"][1]
        .as_object_mut()
        .unwrap()
        .remove("returned_at");
    document["operations"][1]
        .as_object_mut()
        .unwrap()
        .remove("outcome");
    let recorded = history::read(
        &serde_json::to_vec(&document).unwrap(),
        &SuiteProvenance::of(&ir).spec_digest,
    )
    .unwrap();
    let result = linearize::check(&ir, &recorded, linearize::DEFAULT_BUDGET).unwrap();
    assert_eq!(result.verdict, Verdict::Linearizable);
}

#[test]
fn exhausting_the_actual_budget_remains_unknown() {
    let ir = model(GENERATED_TIMESTAMP);
    let result = linearize::check(&ir, &recorded(&ir, Some("early")), 1).unwrap();
    assert_eq!(result.verdict, Verdict::Unknown);
}

#[test]
fn abstract_copy_retains_unknown_and_admitted_increment_requires_overflow_proof() {
    let zero = mutation_source("Integer", "{login_at: {increment: 0}}", "login_at < 1");
    let rejected =
        Specification::assemble([(Source::new("zero.yaml"), RawSpecFile::parse(&zero).unwrap())])
            .unwrap_err();
    assert!(rejected.to_string().contains("not a non-zero amount"));
    // Zero increment is not admitted. An actual subject copy supplies the unchanged-value
    // control, while the admitted +1 must prove overflow/constraints rather than choose a value.
    for (sets, unchanged) in [
        ("{login_at: {subject: login_at}}", true),
        ("{login_at: {increment: 1}}", false),
    ] {
        let ir = model(&mutation_source("Integer", sets, "login_at < 1"));
        let result = linearize::check(
            &ir,
            &changed_history(&ir, "early"),
            linearize::DEFAULT_BUDGET,
        );
        assert!(
            matches!(result, Err(CheckRefusal::Model { .. })),
            "{result:?}"
        );
        let mut creation_and_change = changed_history(&ir, "early");
        creation_and_change.operations.pop();
        let result = linearize::check(&ir, &creation_and_change, linearize::DEFAULT_BUDGET);
        if unchanged {
            assert_eq!(result.unwrap().verdict, Verdict::Linearizable);
        } else {
            assert!(
                matches!(result, Err(CheckRefusal::Model { .. })),
                "{result:?}"
            );
        }
    }
}

#[test]
fn known_increment_keeps_adjacent_integers_above_binary64_precision_distinct() {
    let source = mutation_source(
        "Integer",
        "{login_at: {increment: 1}}",
        "login_at == 9007199254740993",
    )
    .replace(
        "login_at: {generated: true}",
        "login_at: '9007199254740992'",
    );
    let ir = model(&source);
    let result = linearize::check(
        &ir,
        &changed_history(&ir, "early"),
        linearize::DEFAULT_BUDGET,
    )
    .unwrap();
    assert_eq!(result.verdict, Verdict::Linearizable);
    assert_eq!(
        linearize::check(
            &ir,
            &changed_history(&ir, "late"),
            linearize::DEFAULT_BUDGET
        )
        .unwrap()
        .verdict,
        Verdict::Violation
    );
}

#[test]
fn contradictory_integer_domain_is_empty_and_its_optional_form_is_known_absent() {
    let declaration = "  - {name: demo.sessions.Value, kind: newtype, of: Integer, invariants: ['value > 0', 'value < 0']}\n";
    let required = model(&domain_source(declaration, "demo.sessions.Value"));
    assert_eq!(
        linearize::check(
            &required,
            &recorded(&required, None),
            linearize::DEFAULT_BUDGET
        )
        .unwrap()
        .verdict,
        Verdict::Violation
    );
    let source = GENERATED_TIMESTAMP
        .replace("type: Timestamp", "type: Optional<demo.sessions.Value>")
        .replace("login_at < \"2021-01-01T00:00:00Z\"", "defined(login_at)")
        + "\ntypes:\n"
        + declaration;
    assert_history_outcome(&source, "late", Verdict::Linearizable);
    assert_history_outcome(&source, "early", Verdict::Violation);
}

#[test]
fn a_unique_proved_domain_value_is_a_guarantee_not_a_selected_witness() {
    for (declaration, predicate) in [
        ("  - {name: demo.sessions.Value, kind: newtype, of: Boolean, invariants: ['value == true']}\n", "login_at == true"),
        ("  - {name: demo.sessions.Value, kind: newtype, of: Integer, invariants: ['value == 9007199254740993']}\n", "login_at == 9007199254740993"),
    ] {
        let source = GENERATED_TIMESTAMP.replace("type: Timestamp", "type: demo.sessions.Value")
            .replace("login_at < \"2021-01-01T00:00:00Z\"", predicate)
            + "\ntypes:\n" + declaration;
        assert_history_outcome(&source, "early", Verdict::Linearizable);
        assert_history_outcome(&source, "late", Verdict::Violation);
    }
}

#[test]
fn required_generated_event_payloads_need_an_inhabited_domain_too() {
    for (invariants, expected) in [
        ("['value == true']", Verdict::Linearizable),
        ("['value == true', 'value == false']", Verdict::Violation),
    ] {
        let source = domain_source(&format!("  - {{name: demo.sessions.Value, kind: newtype, of: Boolean, invariants: {invariants}}}\n"), "Timestamp")
            .replace("fields: [{name: session_id, type: Uuid}]", "fields: [{name: session_id, type: Uuid}, {name: result, type: demo.sessions.Value}]")
            .replace("{session_id: {generated: true}}}", "{session_id: {generated: true}, result: {generated: true}}}");
        let ir = model(&source);
        let result =
            linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET).unwrap();
        assert_eq!(result.verdict, expected);
    }
}

fn uncertain_view(ir: &EssIr, rows: &[&str]) -> History {
    let mut document = serde_json::to_value(recorded(ir, Some("early"))).unwrap();
    document["operations"][1]["completion"] = serde_json::json!("Indeterminate");
    document["operations"][1]
        .as_object_mut()
        .unwrap()
        .remove("returned_at");
    document["operations"][1]
        .as_object_mut()
        .unwrap()
        .remove("outcome");
    document["operations"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!({
            "operation_id":"00000000-0000-4000-8000-000000000204", "client":0,
            "command":"demo.sessions.All", "subject_key":"", "invoked_at":5,
            "returned_at":6, "completion":"Returned", "outcome":"rows", "rows":rows
        }));
    history::read(
        &serde_json::to_vec(&document).unwrap(),
        &SuiteProvenance::of(ir).spec_digest,
    )
    .unwrap()
}

#[test]
fn incomplete_view_reachability_cannot_prove_a_stale_read_but_known_explanations_pass() {
    let source = GENERATED_TIMESTAMP.replace("predicate: 'login_at < \"2021-01-01T00:00:00Z\"'}\n        preserves:", "predicate: 'login_at < \"2021-01-01T00:00:00Z\"'}\n        deletes:")
        + "\nviews:\n  - name: demo.sessions.All\n    source: demo.sessions.Session\n    consistency: read_your_writes\n    fields: [{name: session_id, type: Uuid}]\n";
    let ir = model(&source);
    let identity = "00000000-0000-4000-8000-00000000c0de";
    let missing = linearize::check(&ir, &uncertain_view(&ir, &[]), linearize::DEFAULT_BUDGET);
    assert!(
        matches!(missing, Err(CheckRefusal::Model { .. })),
        "{missing:?}"
    );
    assert_eq!(
        linearize::check(
            &ir,
            &uncertain_view(&ir, &[identity]),
            linearize::DEFAULT_BUDGET
        )
        .unwrap()
        .verdict,
        Verdict::Linearizable
    );
    let duplicate = linearize::check(
        &ir,
        &uncertain_view(&ir, &[identity, identity]),
        linearize::DEFAULT_BUDGET,
    )
    .unwrap();
    assert_eq!(
        duplicate.read.unwrap().anomaly,
        linearize::Anomaly::DuplicateRow
    );
}

#[test]
fn generated_declared_error_payloads_are_checked_without_observed_error_values() {
    for (invariants, expected) in [
        ("['value == true']", Verdict::Linearizable),
        ("['value == true', 'value == false']", Verdict::Violation),
    ] {
        let prefix = GENERATED_TIMESTAMP
            .split("  - name: demo.sessions.Inspect")
            .next()
            .unwrap();
        let source = prefix.replace("format: ess/18", "format: ess/19")
            + &format!(
                r"
  - name: demo.sessions.Inspect
    input: [{{name: session_id, type: Uuid}}]
    outcomes:
      - name: denied
        error: demo.sessions.Denied
        payload: {{demo.sessions.Denied: {{result: {{generated: true}}}}}}
errors:
  - {{name: demo.sessions.Denied, fields: [{{name: result, type: demo.sessions.Value}}]}}
types:
  - {{name: demo.sessions.Value, kind: newtype, of: Boolean, invariants: {invariants}}}
"
            );
        assert_history_outcome(&source, "denied", expected);
    }
}

#[test]
fn bounded_feasibility_failure_is_not_an_empty_domain_proof() {
    use ess_conformance::interpret::execute::{
        execute_generating, Externals, Generated, GeneratedSlot, Store,
    };
    use ess_primitives::node::Node;
    use std::collections::BTreeMap;
    let declaration = "  - {name: demo.sessions.Value, kind: newtype, of: String, invariants: ['value.count > 4096']}\n";
    let source = domain_source(declaration, "demo.sessions.Value");
    let ir = model(&source);
    let result = linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET);
    assert!(
        matches!(result, Err(CheckRefusal::Model { .. })),
        "a failed bounded witness is unresolved: {result:?}"
    );
    // Demonstrate an actual inhabitant through the native declared-input validator. This sample
    // proves domain existence in the test; it is never supplied to the outcome-only history.
    let concrete_source = source
        .replace(
            "input: []",
            "input: [{name: known, type: demo.sessions.Value}]",
        )
        .replace("login_at: {generated: true}", "login_at: input.known");
    let concrete = model(&concrete_source);
    let result = execute_generating(
        &concrete,
        &Store::default(),
        &"demo.sessions.Start".parse().unwrap(),
        &BTreeMap::from([("known".into(), Node::Text("a".repeat(4097)))]),
        &Externals::Withheld,
        &Generated::Given(BTreeMap::from([(
            GeneratedSlot::new("demo.sessions.Started".parse().unwrap(), "session_id"),
            Node::Text("00000000-0000-4000-8000-00000000c0de".into()),
        )])),
    )
    .unwrap();
    assert_eq!(result.len(), 1);
}

fn generated_cross_rows() -> String {
    CROSS_ROW_SESSION
        .replace(
            "fields: [{name: touched, type: Boolean}]",
            "fields: [{name: touched, type: Boolean}, {name: stamp, type: Optional<Timestamp>}]",
        )
        .replace(
            "sets: {touched: false}",
            "sets: {touched: false, stamp: {generated: true}}",
        )
}

#[test]
fn actual_related_known_leaves_remain_readable_beside_generated_values() {
    let ir = model(&generated_cross_rows());
    let result = linearize::check(
        &ir,
        &cross_row_history(&ir, 0, &["b"], true),
        linearize::DEFAULT_BUDGET,
    )
    .unwrap();
    assert_eq!(result.verdict, Verdict::Linearizable);
}

#[test]
fn actual_related_generated_presence_remains_unresolved() {
    let source =
        generated_cross_rows().replace("predicate: state == Active", "predicate: defined(stamp)");
    let ir = model(&source);
    let result = linearize::check(
        &ir,
        &cross_row_history(&ir, 0, &["b"], true),
        linearize::DEFAULT_BUDGET,
    );
    assert!(
        matches!(result, Err(CheckRefusal::Model { .. })),
        "{result:?}"
    );
}

#[test]
fn set_filter_false_conjuncts_resolve_unknown_leaves_in_either_order() {
    for filter in [
        "{all: ['touched == true', 'defined(stamp)']}",
        "{all: ['defined(stamp)', 'touched == true']}",
    ] {
        let source =
            generated_cross_rows().replace("where: touched == false", &format!("where: {filter}"));
        let ir = model(&source);
        let result = linearize::check(
            &ir,
            &cross_row_history(&ir, 0, &["b"], false),
            linearize::DEFAULT_BUDGET,
        )
        .unwrap();
        assert_eq!(result.verdict, Verdict::Linearizable);
    }
}

#[test]
fn set_filter_unknown_presence_is_not_a_zero_match() {
    let source = generated_cross_rows().replace("where: touched == false", "where: defined(stamp)");
    let ir = model(&source);
    let result = linearize::check(
        &ir,
        &cross_row_history(&ir, 0, &["b"], false),
        linearize::DEFAULT_BUDGET,
    );
    assert!(
        matches!(result, Err(CheckRefusal::Model { .. })),
        "{result:?}"
    );
}

#[test]
fn set_known_clear_replaces_unknown_presence_for_a_later_actual_related_guard() {
    let source = generated_cross_rows()
        .replace(
            "sets: {touched: true}",
            "sets: {touched: true, stamp: {cleared: true}}",
        )
        .replace(
            "predicate: state == Active",
            "predicate: not defined(stamp)",
        );
    let ir = model(&source);
    let mut document = serde_json::to_value(cross_row_history(&ir, 0, &["b"], true)).unwrap();
    let operations = document["operations"].as_array_mut().unwrap();
    for operation in operations.iter_mut().skip(2) {
        operation["invoked_at"] = serde_json::json!(operation["invoked_at"].as_u64().unwrap() + 2);
        operation["returned_at"] =
            serde_json::json!(operation["returned_at"].as_u64().unwrap() + 2);
    }
    operations.insert(
        2,
        serde_json::json!({
            "operation_id":"00000000-0000-4000-8000-000000000305", "client":0,
            "command":"demo.cross.TouchAllB", "subject_key":"", "invoked_at":5,
            "returned_at":6, "completion":"Returned", "outcome":"touched"
        }),
    );
    let history = history::read(
        &serde_json::to_vec(&document).unwrap(),
        &SuiteProvenance::of(&ir).spec_digest,
    )
    .unwrap();
    assert_eq!(
        linearize::check(&ir, &history, linearize::DEFAULT_BUDGET)
            .unwrap()
            .verdict,
        Verdict::Linearizable
    );
}

#[test]
fn known_overflow_and_proved_impossible_constrained_increment_have_no_successor() {
    for source in [
        mutation_source("Integer", "{login_at: {increment: 1}}", "login_at < 1")
            .replace("login_at: {generated: true}", "login_at: '9223372036854775807'"),
        mutation_source("demo.sessions.Value", "{login_at: {increment: 1}}", "login_at < 1")
            + "\ntypes:\n  - {name: demo.sessions.Value, kind: newtype, of: Integer, invariants: ['value == 0']}\n",
    ] {
        let ir = model(&source);
        let mut history = changed_history(&ir, "early");
        history.operations.pop();
        assert_eq!(linearize::check(&ir, &history, linearize::DEFAULT_BUDGET).unwrap().verdict, Verdict::Violation);
    }
}

fn related_creation(via: &str, known_absent: bool) -> String {
    let source = generated_cross_rows();
    let (before_b, from_b) = source.split_once("  - name: demo.cross.CreateB").unwrap();
    let before_b = before_b
        .replace("  - name: demo.cross.CreateA\n", "  - name: demo.cross.CreateA\n    input: [{name: b_id, type: demo.cross.BId}]\n")
        .replace("fields: [{name: touched, type: Boolean}, {name: stamp, type: Optional<Timestamp>}]\n    lifecycle:",
            "fields: [{name: touched, type: Boolean}, {name: stamp, type: Optional<Timestamp>}, {name: linked, type: demo.cross.BId} ]\n    lifecycle:")
        .replacen("    lifecycle:", "    relations: [{name: linked_b, kind: references, target: demo.cross.B, cardinality: one, via: linked}]\n    lifecycle:", 1)
        .replace("sets: {touched: false, stamp: {generated: true}}", &format!("sets: {{touched: false, linked: input.b_id, stamp: {{related: {{via: {via}, field: stamp}}}}}}"));
    let from_b = if known_absent {
        from_b.replacen(
            "sets: {touched: false, stamp: {generated: true}}",
            "sets: {touched: false}",
            1,
        )
    } else {
        from_b.to_owned()
    };
    (before_b + "  - name: demo.cross.CreateB" + &from_b).replace("views:\n", r"  - name: demo.cross.InspectA
    input: [{name: a_id, type: demo.cross.AId}]
    outcomes:
      - {name: absent, when_subject: {predicate: not defined(stamp)}, preserves: demo.cross.A, instance: a_id}
      - {name: present, preserves: demo.cross.A, instance: a_id}
views:
")
}

#[test]
fn both_related_creation_mappings_preserve_unknown_and_absent_values() {
    for via in ["input.b_id", "linked"] {
        for known_absent in [false, true] {
            let ir = model(&related_creation(via, known_absent));
            let mut document =
                serde_json::to_value(cross_row_history(&ir, 0, &["b"], true)).unwrap();
            let operations = document["operations"].as_array_mut().unwrap();
            operations.swap(0, 1);
            operations[0]["invoked_at"] = serde_json::json!(1);
            operations[0]["returned_at"] = serde_json::json!(2);
            operations[1]["invoked_at"] = serde_json::json!(3);
            operations[1]["returned_at"] = serde_json::json!(4);
            operations[2]["command"] = serde_json::json!("demo.cross.InspectA");
            operations[2]["outcome"] = serde_json::json!("absent");
            operations.pop();
            let history = history::read(
                &serde_json::to_vec(&document).unwrap(),
                &SuiteProvenance::of(&ir).spec_digest,
            )
            .unwrap();
            let result = linearize::check(&ir, &history, linearize::DEFAULT_BUDGET);
            if known_absent {
                assert_eq!(result.unwrap().verdict, Verdict::Linearizable);
            } else {
                assert!(
                    matches!(result, Err(CheckRefusal::Model { .. })),
                    "{via}: {result:?}"
                );
            }
        }
    }
}

#[test]
fn lifecycle_ineligible_set_rows_do_not_require_unknown_membership() {
    for unresolved in [false, true] {
        let base = generated_cross_rows();
        let (before, after) = base
            .rsplit_once("lifecycle: {initial: Active, states: [Active], terminal: [Active]}")
            .unwrap();
        let source = format!("{before}lifecycle: {{initial: Active, states: [Active, Closed], terminal: [Closed], transitions: [{{name: close, from: [Active], to: Closed}}]}}{after}")
            .replace("updates: demo.cross.B\n        instances: {where: touched == false}",
                "moves: demo.cross.B.close\n        instances: {where: defined(stamp)}")
            .replace("views:\n", "  - name: demo.cross.CloseB\n    input: [{name: b_id, type: demo.cross.BId}]\n    outcomes:\n      - name: closed\n        moves: demo.cross.B.close\n        instance: b_id\n        emits: [demo.cross.Touched]\n        payload: {demo.cross.Touched: {b_id: input.b_id}}\nviews:\n");
        let ir = model(&source);
        let mut document = serde_json::to_value(cross_row_history(&ir, 0, &["b"], false)).unwrap();
        if !unresolved {
            let operations = document["operations"].as_array_mut().unwrap();
            for operation in operations.iter_mut().skip(2) {
                operation["invoked_at"] =
                    serde_json::json!(operation["invoked_at"].as_u64().unwrap() + 2);
                operation["returned_at"] =
                    serde_json::json!(operation["returned_at"].as_u64().unwrap() + 2);
            }
            operations.insert(2, serde_json::json!({"operation_id":"00000000-0000-4000-8000-000000000305","client":0,"command":"demo.cross.CloseB","subject_key":"b","invoked_at":5,"returned_at":6,"completion":"Returned","outcome":"closed"}));
        }
        let history = history::read(
            &serde_json::to_vec(&document).unwrap(),
            &SuiteProvenance::of(&ir).spec_digest,
        )
        .unwrap();
        let result = linearize::check(&ir, &history, linearize::DEFAULT_BUDGET);
        if unresolved {
            assert!(
                matches!(result, Err(CheckRefusal::Model { .. })),
                "{result:?}"
            );
        } else {
            assert_eq!(result.unwrap().verdict, Verdict::Linearizable);
        }
    }
}

#[test]
fn stored_caller_namespace_preserves_unknown_optional_presence_and_known_siblings() {
    let source = GENERATED_TIMESTAMP
        .replace("{name: login_at, type: Timestamp}", "{name: caller, type: demo.sessions.Bundle}")
        .replace("login_at: {generated: true}", "caller: {key: {generated: true}, known: false}")
        + "\ntypes:\n  - {name: demo.sessions.Bundle, kind: struct, fields: [{name: key, type: Optional<Timestamp>}, {name: known, type: Boolean}]}\n";
    for (predicate, expected) in [
        ("'defined(caller.key)'", None),
        (
            "{all: ['caller.known == true', 'defined(caller.key)']}",
            Some(Verdict::Linearizable),
        ),
        (
            "{all: ['defined(caller.key)', 'caller.known == true']}",
            Some(Verdict::Linearizable),
        ),
    ] {
        let source = source.replace("'login_at < \"2021-01-01T00:00:00Z\"'", predicate);
        let ir = model(&source);
        let result = linearize::check(&ir, &recorded(&ir, Some("late")), linearize::DEFAULT_BUDGET);
        if let Some(expected) = expected {
            assert_eq!(result.unwrap().verdict, expected);
        } else {
            assert!(
                matches!(result, Err(CheckRefusal::Model { .. })),
                "{result:?}"
            );
        }
    }
}

#[test]
fn inert_generated_fields_preserve_retry_chains_and_indeterminate_origin_authority() {
    let source = include_str!("fixtures/explore-retry/retry.yaml")
        .replace("format: ess/7", "format: ess/20")
        .replace(
            "      - {name: value, type: String}",
            "      - {name: value, type: String}\n      - {name: stamp, type: Optional<Timestamp>}",
        )
        .replace(
            "sets: {value: input.document}",
            "sets: {value: input.document, stamp: {generated: true}}",
        );
    let ir = model(&source);
    let id = |n: u64| format!("00000000-0000-4000-8000-{n:012}");
    let returned = |n: u64, client: u64, key: &str, outcome: &str, retry: Option<u64>| {
        let mut operation = serde_json::json!({"operation_id":id(n),"client":client,"command":"retry.core.Seed","subject_key":key,"invoked_at":n*2,"returned_at":n*2+1,"completion":"Returned","outcome":outcome});
        if let Some(retry) = retry {
            operation["retry_of"] = serde_json::json!(id(retry));
        }
        operation
    };
    let check = |operations: Vec<serde_json::Value>| {
        let digest = SuiteProvenance::of(&ir).spec_digest;
        let document = serde_json::json!({"format":"ess-history/1","history_id":id(99),"spec_digest":digest,"seed":0,"clients":2,"operations":operations});
        let history = history::read(&serde_json::to_vec(&document).unwrap(), &digest).unwrap();
        linearize::check(&ir, &history, linearize::DEFAULT_BUDGET)
            .unwrap()
            .verdict
    };
    let x = id(101);
    let y = id(102);
    assert_eq!(
        check(vec![
            returned(1, 0, &x, "seeded", None),
            returned(2, 0, "", "replayed", Some(1)),
            returned(3, 0, "", "replayed", Some(2))
        ]),
        Verdict::Linearizable
    );
    assert_eq!(
        check(vec![
            returned(1, 0, &x, "seeded", None),
            returned(2, 0, "", "replayed", Some(1)),
            returned(3, 0, &y, "seeded", Some(2))
        ]),
        Verdict::Violation
    );
    let unanswered = serde_json::json!({"operation_id":id(1),"client":0,"command":"retry.core.Seed","subject_key":x,"invoked_at":2,"completion":"Indeterminate"});
    assert_eq!(
        check(vec![unanswered.clone(), returned(3, 1, &x, "seeded", None)]),
        Verdict::Linearizable
    );
    assert_eq!(
        check(vec![
            unanswered,
            returned(2, 0, "", "replayed", Some(1)),
            returned(3, 1, &x, "seeded", None)
        ]),
        Verdict::Violation
    );
}

fn required_struct_chain(length: usize) -> String {
    let mut declarations = String::new();
    for index in 0..length {
        let child = if index + 1 == length {
            "Boolean".to_owned()
        } else {
            format!("demo.sessions.S{}", index + 1)
        };
        writeln!(&mut declarations, "  - {{name: demo.sessions.S{index}, kind: struct, fields: [{{name: next, type: {child}}}]}}")
            .expect("writing a String is infallible");
    }
    declarations
}

#[test]
fn sixteen_required_structs_have_a_valid_generated_inhabitant() {
    let ir = model(&domain_source(
        &required_struct_chain(16),
        "demo.sessions.S0",
    ));
    let result = linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET).unwrap();
    assert_eq!(result.verdict, Verdict::Linearizable);
}

#[test]
fn seventeen_required_structs_are_unresolved_not_proven_empty_at_validator_depth_limit() {
    let ir = model(&domain_source(
        &required_struct_chain(17),
        "demo.sessions.S0",
    ));
    let result = linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET);
    assert!(
        matches!(result, Err(CheckRefusal::Model { .. })),
        "validation depth exhaustion is not an empty generated domain: {result:?}"
    );
}

#[test]
fn optional_deep_generated_struct_is_not_fabricated_as_definitely_absent() {
    let source = GENERATED_TIMESTAMP
        .replace("type: Timestamp", "type: Optional<demo.sessions.S0>")
        .replace("login_at < \"2021-01-01T00:00:00Z\"", "defined(login_at)")
        + "\ntypes:\n"
        + &required_struct_chain(17);
    let ir = model(&source);
    let result = linearize::check(&ir, &recorded(&ir, Some("late")), linearize::DEFAULT_BUDGET);
    assert!(
        matches!(result, Err(CheckRefusal::Model { .. })),
        "unknown inner feasibility cannot choose optional absence: {result:?}"
    );
}

#[test]
fn productive_recursive_generated_unions_find_a_leaf_after_a_recursive_first_variant() {
    for leaf in ["leaf", "a_leaf"] {
        let declarations = format!("  - name: demo.sessions.Expr\n    kind: union\n    tag: value\n    variants: {{branch: demo.sessions.Pair, {leaf}: Integer}}\n  - name: demo.sessions.Pair\n    kind: struct\n    fields: [{{name: left, type: demo.sessions.Expr}}, {{name: right, type: demo.sessions.Expr}}]\n");
        let ir = model(&domain_source(&declarations, "demo.sessions.Expr"));
        assert_eq!(
            linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET)
                .unwrap()
                .verdict,
            Verdict::Linearizable
        );
    }
}

#[test]
fn recursive_optional_list_and_map_bases_remain_inhabited_without_inner_samples() {
    let declarations = "  - name: demo.sessions.Tree\n    kind: struct\n    fields: [{name: parent, type: Optional<demo.sessions.Tree>}, {name: children, type: List<demo.sessions.Tree>}, {name: indexed, type: 'Map<String, demo.sessions.Tree>'}]\n";
    let ir = model(&domain_source(declarations, "demo.sessions.Tree"));
    assert_eq!(
        linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET)
            .unwrap()
            .verdict,
        Verdict::Linearizable
    );
}

#[test]
fn one_valid_finite_candidate_proves_existence_despite_an_unknown_optional_candidate() {
    let declarations = "  - name: demo.sessions.Bundle\n    kind: struct\n    fields: [{name: flag, type: Optional<Boolean>}]\n    invariants: ['flag == true']\n";
    let ir = model(&domain_source(declarations, "demo.sessions.Bundle"));
    assert_eq!(
        linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET)
            .unwrap()
            .verdict,
        Verdict::Linearizable
    );
}

#[test]
fn definite_false_suffix_dominates_unknown_optional_prefix_in_generated_domain() {
    let declarations = r"  - name: demo.sessions.Value
    kind: struct
    fields: [{name: maybe, type: Optional<Boolean>}, {name: known, type: Boolean}]
    invariants: ['maybe == true', 'known == true', 'known == false']
";
    let ir = model(&domain_source(declarations, "demo.sessions.Value"));
    let checked = linearize::check(&ir, &recorded(&ir, None), linearize::DEFAULT_BUDGET)
        .expect("a definitely empty domain is a resolved absence of a successor");
    assert_eq!(
        checked.verdict,
        Verdict::Violation,
        "an unknown optional prefix cannot mask a later contradictory Boolean suffix"
    );
}

#[test]
fn complete_increment_domains_distinguish_partial_from_total_overflow() {
    for (lower, expected_model_refusal) in [
        ("9223372036854775806", true),
        ("9223372036854775807", false),
    ] {
        let source = GENERATED_NARROW_TO_WIDE.replace(
            "invariants: ['value >= 0', 'value <= 10']",
            &format!("invariants: ['value >= {lower}', 'value <= 9223372036854775807']"),
        );
        let ir = model(&source);
        let result = linearize::check(
            &ir,
            &widening_history(&ir, 1, None),
            linearize::DEFAULT_BUDGET,
        );
        if expected_model_refusal {
            assert!(
                matches!(result, Err(CheckRefusal::Model { .. })),
                "one safe and one overflowing member is unresolved: {result:?}"
            );
        } else {
            assert_eq!(
                result.unwrap().verdict,
                Verdict::Violation,
                "an entirely overflowing complete domain has no successor"
            );
        }
    }
}

fn generated_related_identity_source(generated: bool) -> String {
    let linked = if generated {
        "{generated: true}"
    } else {
        "input.b_id"
    };
    format!(
        r#"format: ess/20
system: demo
version: v1
domain: demo.authority
entities:
  - name: demo.authority.A
    identity: {{name: a_id, type: String}}
    fields: [{{name: linked, type: String}}, {{name: copied, type: Boolean}}]
    relations: [{{name: linked_b, kind: references, target: demo.authority.B, cardinality: one, via: linked}}]
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
  - name: demo.authority.B
    identity: {{name: b_id, type: String}}
    fields: [{{name: marker, type: Boolean}}]
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
events:
  - {{name: demo.authority.ACreated, fields: [{{name: a_id, type: String}}]}}
  - {{name: demo.authority.BCreated, fields: [{{name: b_id, type: String}}]}}
  - {{name: demo.authority.Resolved, fields: []}}
errors:
  - {{name: demo.authority.OtherB, fields: []}}
commands:
  - name: demo.authority.CreateB
    outcomes:
      - name: created
        creates: demo.authority.B
        instance: b_id
        sets: {{marker: true}}
        emits: [demo.authority.BCreated]
        payload: {{demo.authority.BCreated: {{b_id: {{generated: true}}}}}}
  - name: demo.authority.CreateA
    input: [{{name: b_id, type: String}}]
    outcomes:
      - {{name: other-b, when: 'b_id != "b"', error: demo.authority.OtherB}}
      - name: created
        creates: demo.authority.A
        instance: a_id
        sets: {{linked: {linked}, copied: false}}
        emits: [demo.authority.ACreated]
        payload: {{demo.authority.ACreated: {{a_id: {{generated: true}}}}}}
  - name: demo.authority.Resolve
    input: [{{name: a_id, type: String}}]
    outcomes:
      - name: resolved
        updates: demo.authority.A
        instance: a_id
        sets: {{copied: {{related: {{via: linked, field: marker}}}}}}
        emits: [demo.authority.Resolved]
"#
    )
}

fn generated_related_identity_history(ir: &EssIr) -> History {
    let digest = SuiteProvenance::of(ir).spec_digest;
    let document = serde_json::json!({
        "format":"ess-history/1", "history_id":"00000000-0000-4000-8000-00000000f294",
        "spec_digest":digest, "seed":0, "clients":1,
        "operations":[
            {"operation_id":"00000000-0000-4000-8000-000000000401", "client":0,
             "command":"demo.authority.CreateB", "subject_key":"b", "invoked_at":1,
             "returned_at":2, "completion":"Returned", "outcome":"created"},
            {"operation_id":"00000000-0000-4000-8000-000000000402", "client":0,
             "command":"demo.authority.CreateA", "subject_key":"a", "invoked_at":3,
             "returned_at":4, "completion":"Returned", "outcome":"created"},
            {"operation_id":"00000000-0000-4000-8000-000000000403", "client":0,
             "command":"demo.authority.Resolve", "subject_key":"a", "invoked_at":5,
             "returned_at":6, "completion":"Returned", "outcome":"resolved"}
        ]
    });
    history::read(&serde_json::to_vec(&document).unwrap(), &digest)
        .expect("the generated-related-identity history is admitted")
}

#[test]
fn a_generated_identity_is_not_observation_authority_for_a_cross_row_read() {
    let generated = model(&generated_related_identity_source(true));
    let result = linearize::check(
        &generated,
        &generated_related_identity_history(&generated),
        linearize::DEFAULT_BUDGET,
    );
    assert!(
        matches!(&result, Err(CheckRefusal::Model { operation_id, .. })
            if operation_id == "00000000-0000-4000-8000-000000000403"),
        "a generated identity is still unrecorded address authority: {result:?}"
    );

    let supplied = model(&generated_related_identity_source(false));
    assert_eq!(
        linearize::check(
            &supplied,
            &generated_related_identity_history(&supplied),
            linearize::DEFAULT_BUDGET,
        )
        .expect("the exact admitted input can authorize the related address")
        .verdict,
        Verdict::Linearizable
    );
}
