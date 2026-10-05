//! Family F part A1 at the source (beyond10x/ess#225, #233): from `ess/22` a bare word on the right
//! of a comparison names a root of the place it is written in, and a plain `when:` reads
//! `input.<path>` as the input it names. `docs/design/expression-family-source22.md` is the design;
//! the resolver order is: binder, enum variant of the left side, root, text.

use ess_domain::command::OutcomeCondition;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{CompareKind, CompareOp, Operand, Predicate};

/// One command over two identity inputs, a few more scalar inputs, and an entity whose invariant is
/// `invariant`. `{when}` is the guard of the `refused` branch.
fn model(format: u32, when: &str, invariant: &str, extra_input: &str) -> String {
    format!(
        r"format: ess/{format}
system: graph
version: v1
domain: graph.tasks
types:
  - {{name: graph.tasks.TaskId, kind: newtype, of: Uuid}}
  - {{name: graph.tasks.Phase, kind: enum, variants: [draft, final]}}
  - name: graph.tasks.Envelope
    kind: struct
    fields:
      - {{name: task_id, type: graph.tasks.TaskId}}
entities:
  - name: graph.tasks.Edge
    identity: {{name: edge_id, type: Uuid}}
    fields:
      - {{name: valid_from, type: Timestamp}}
      - {{name: valid_until, type: Timestamp}}
      - {{name: tags, type: List<String>}}
      - {{name: banned, type: List<String>}}
    invariants:
      - {invariant}
    lifecycle: {{initial: Linked, states: [Linked], terminal: [Linked]}}
errors:
  - name: graph.tasks.Refused
    summary: The link is refused.
events:
  - name: graph.tasks.Linked
    fields:
      - {{name: edge_id, type: Uuid}}
commands:
  - name: graph.tasks.Link
    input:
      - {{name: task_id, type: graph.tasks.TaskId}}
      - {{name: depends_on, type: graph.tasks.TaskId}}
      - {{name: phase, type: graph.tasks.Phase}}
      - {{name: note, type: String}}
      - {{name: deadline, type: Timestamp}}
      - {{name: valid_from, type: Timestamp}}
      - {{name: valid_until, type: Timestamp}}
      - {{name: tags, type: List<String>}}
      - {{name: banned, type: List<String>}}
{extra_input}    outcomes:
      - name: refused
        when: {when}
        error: graph.tasks.Refused
      - name: linked
        creates: graph.tasks.Edge
        instance: edge_id
        emits: [graph.tasks.Linked]
        payload:
          graph.tasks.Linked: {{edge_id: {{generated: true}}}}
        sets: {{valid_from: input.valid_from, valid_until: input.valid_until, tags: input.tags, banned: input.banned}}
"
    )
}

const PLAIN: &str = "valid_until >= valid_from";

fn assemble(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| format!("parse: {error}"))?;
    Specification::assemble([(Source::new("model.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn admitted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("admitted:\n{errors}\n---\n{text}"))
}

fn guard(spec: &Specification) -> Predicate {
    let command = &spec.commands()[&"graph.tasks.Link".parse().unwrap()];
    match &command.outcomes[0].condition {
        OutcomeCondition::When(predicate) => predicate.clone(),
        other => panic!("a plain when, not {other:?}"),
    }
}

fn invariant(spec: &Specification) -> Predicate {
    spec.entities()[&"graph.tasks.Edge".parse().unwrap()].invariants[0]
        .predicate
        .clone()
}

fn path(text: &str) -> FactPath {
    text.parse().expect("a path")
}

fn compare(left: &str, op: CompareOp, right: Operand) -> Predicate {
    Predicate::Compare {
        kind: ess_primitives::predicate::CompareKind::Value,
        left: Operand::Fact(path(left)),
        op,
        right,
    }
}

/// A comparison of two facts tagged to compare instants (decision 2).
fn instants(left: &str, op: CompareOp, right: &str) -> Predicate {
    Predicate::Compare {
        left: Operand::Fact(path(left)),
        op,
        right: Operand::Fact(path(right)),
        kind: CompareKind::Instant,
    }
}

fn fact(text: &str) -> Operand {
    Operand::Fact(path(text))
}

fn text(word: &str) -> Operand {
    Operand::Literal(FactValue::text(word))
}

fn json(predicate: &Predicate) -> String {
    serde_json::to_string(predicate).expect("serialises")
}

#[test]
fn a1_self_edge_guard_reads_both_inputs() {
    for (written, op) in [
        ("task_id == depends_on", CompareOp::Eq),
        ("task_id != depends_on", CompareOp::Ne),
    ] {
        let spec = admitted(&model(22, written, PLAIN, ""));
        let resolved = guard(&spec);
        assert_eq!(
            resolved,
            compare("task_id", op, fact("depends_on")),
            "{written}"
        );
        assert_eq!(
            json(&resolved),
            format!(
                r#"{{"task_id":{{"{}":{{"fact":"depends_on"}}}}}}"#,
                op.keyword()
            )
        );
    }
}

#[test]
fn a1_input_namespace_plain_when() {
    let spec = admitted(&model(22, "task_id == input.depends_on", PLAIN, ""));
    assert_eq!(
        guard(&spec),
        compare("task_id", CompareOp::Eq, fact("depends_on"))
    );
    let spec = admitted(&model(22, "input.task_id != depends_on", PLAIN, ""));
    assert_eq!(
        guard(&spec),
        compare("task_id", CompareOp::Ne, fact("depends_on"))
    );
    let spec = admitted(&model(22, "\"input.note == 'x'\"", PLAIN, ""));
    assert_eq!(guard(&spec), compare("note", CompareOp::Eq, text("x")));
    // Below ess/22 the namespace is what it was: nothing the command declares.
    let refused = assemble(&model(21, "task_id == input.depends_on", PLAIN, ""))
        .expect_err("ess/21 has no input namespace in a plain when");
    assert!(refused.contains("input"), "{refused}");
}

#[test]
fn a1_declared_input_root_keeps_field() {
    let extra = "      - {name: input, type: graph.tasks.Envelope}\n";
    let spec = admitted(&model(22, "input.task_id == depends_on", PLAIN, extra));
    assert_eq!(
        guard(&spec),
        compare("input.task_id", CompareOp::Eq, fact("depends_on")),
        "a declared `input` struct keeps being read as itself"
    );
}

#[test]
fn a1_enum_variant_beats_root() {
    let extra = "      - {name: draft, type: graph.tasks.Phase}\n";
    let spec = admitted(&model(22, "phase == draft", PLAIN, extra));
    assert_eq!(guard(&spec), compare("phase", CompareOp::Eq, text("draft")));
    // The same word on the left of a comparison whose left is no enum is the root.
    let spec = admitted(&model(22, "phase == draft", PLAIN, ""));
    assert_eq!(guard(&spec), compare("phase", CompareOp::Eq, text("draft")));
}

#[test]
fn a1_root_named_now_is_fact() {
    let extra = "      - {name: now, type: Timestamp}\n";
    let spec = admitted(&model(22, "deadline < now", PLAIN, extra));
    assert_eq!(guard(&spec), instants("deadline", CompareOp::Lt, "now"));
    // Without such a root the word is the current time, as it was.
    let spec = admitted(&model(22, "deadline < now", PLAIN, ""));
    assert_eq!(
        guard(&spec),
        compare("deadline", CompareOp::Lt, text("now"))
    );
}

#[test]
fn a1_missing_root_stays_literal() {
    let spec = admitted(&model(22, "note == pending", PLAIN, ""));
    assert_eq!(
        guard(&spec),
        compare("note", CompareOp::Eq, text("pending"))
    );
    assert_eq!(json(&guard(&spec)), r#""note == pending""#);
}

#[test]
fn a1_quoted_root_refused() {
    for written in [
        r#"'task_id == "depends_on"'"#,
        r#"{task_id: {eq: '"depends_on"'}}"#,
        "{task_id: depends_on}",
    ] {
        let refused = assemble(&model(22, written, PLAIN, ""))
            .expect_err("a quoted word naming a field is refused");
        assert!(
            refused.contains("not the field `depends_on`") && refused.contains("write it unquoted"),
            "{written}: {refused}"
        );
    }
    // The structured operator spelling of the bare word reads the field.
    let spec = admitted(&model(22, "{task_id: {ne: depends_on}}", PLAIN, ""));
    assert_eq!(
        guard(&spec),
        compare("task_id", CompareOp::Ne, fact("depends_on"))
    );
}

#[test]
fn a1_binder_shadowing_compact_bytes() {
    let binder = "{forall: {in: tags, as: a, that: {forall: {in: banned, as: b, that: a != b}}}}";
    let spec = admitted(&model(22, PLAIN, binder, ""));
    let resolved = invariant(&spec);
    assert_eq!(
        json(&resolved),
        r#"{"forall":{"as":"a","in":"tags","that":{"forall":{"as":"b","in":"banned","that":"a != b"}}}}"#,
        "a binder keeps its compact bytes"
    );
    assert!(!resolved.reads_root_fact_operand());
    // A binder named like a field of the entity makes the bare word mean two things: refused.
    let shadow = "{forall: {in: tags, as: valid_from, that: {forall: {in: banned, as: b, that: b != valid_from}}}}";
    let refused = assemble(&model(22, PLAIN, shadow, "")).expect_err("a shadowed root");
    assert!(refused.contains("rename the binder"), "{refused}");
    // The same document below ess/22 keeps meaning the binder (#289).
    assemble(&model(21, "note == pending", shadow, "")).expect("ess/21 keeps the binder");
}

#[test]
fn a1_timestamp_sibling_resolves_in_an_invariant() {
    let spec = admitted(&model(22, PLAIN, "valid_until > valid_from", ""));
    assert_eq!(
        invariant(&spec),
        instants("valid_until", CompareOp::Gt, "valid_from"),
        "two Timestamp facts are tagged to compare instants (decision 2)"
    );
    assert_eq!(
        json(&invariant(&spec)),
        r#"{"compare":{"as":"timestamp","left":"valid_until","op":"gt","right":{"fact":"valid_from"}}}"#
    );
    // A Timestamp against an instant literal is the comparison it always was.
    let spec = admitted(&model(
        22,
        PLAIN,
        "valid_until > \"2020-01-01T00:00:00Z\"",
        "",
    ));
    assert!(!invariant(&spec).compares_instants());
}

#[test]
fn the_instant_tag_is_an_ess22_form_between_two_timestamps() {
    let tagged = "{compare: {left: valid_until, op: gt, right: {fact: valid_from}, as: timestamp}}";
    // Below ess/22 the form is no form: `compare` is read as the fact it always was, and refused
    // in the words it always was (final review, F6), at the invariant that wrote it
    // (beyond10x/ess#448).
    let refused =
        assemble(&model(21, "note == pending", tagged, "")).expect_err("ess/21 refuses the tag");
    assert_eq!(
        refused,
        "[unparsable_predicate] entity graph.tasks.Edge.invariants[0]: cannot parse predicate \
         \"compare: {as: …}\": unknown operator \"as\"; expected one of eq, ne, lt, lte, gt, gte, \
         any_of, none_of, exists, truthy, starts_with, ends_with, contains, equals_ignore_case, \
         in_ignore_case"
    );
    let spec = admitted(&model(22, PLAIN, tagged, ""));
    assert_eq!(
        invariant(&spec),
        instants("valid_until", CompareOp::Gt, "valid_from")
    );
    let wrong = "{compare: {left: valid_until, op: gt, right: {fact: tags}, as: timestamp}}";
    let refused = assemble(&model(22, PLAIN, wrong, "")).expect_err("a list is no Timestamp");
    assert!(
        refused.contains("only two Timestamp facts compare"),
        "{refused}"
    );
}

#[test]
fn a1_source21_bytes_unchanged() {
    // The words keep their old meaning and their old refusal below ess/22.
    let refused = assemble(&model(21, "task_id == depends_on", PLAIN, ""))
        .expect_err("ess/21 reads `depends_on` as text");
    assert!(
        refused.contains("a right-hand side without a dot is a literal"),
        "{refused}"
    );
    let spec = admitted(&model(
        21,
        "note == pending",
        "valid_until > \"2020-01-01T00:00:00Z\"",
        "",
    ));
    assert_eq!(json(&guard(&spec)), r#""note == pending""#);
    assert_eq!(
        json(&invariant(&spec)),
        r#""valid_until > 2020-01-01T00:00:00Z""#
    );
    // The canonical fact operand is an ess/22 form: below it the mapping is refused in the words
    // it always was (final review, F6), at the guard that wrote it, and no longer hides the
    // invariant's own refusal beside it (beyond10x/ess#448).
    let refused = assemble(&model(21, "{task_id: {eq: {fact: depends_on}}}", PLAIN, ""))
        .expect_err("ess/21 refuses the explicit fact operand");
    assert_eq!(
        refused,
        "2 validation errors:\n  - [unparsable_predicate] command.graph.tasks.Link.outcomes.\
         refused.when: cannot parse predicate \"task_id: {eq: {fact: depends_on}}\": a comparison \
         operand must be a scalar\n  - [type_mismatch] entity graph.tasks.Edge.invariants[0]: \
         `valid_until >= valid_from` reads `valid_from` as the text literal \"valid_from\", not \
         the field `valid_from`: a right-hand side without a dot is a literal. To compare two \
         fields, declare them in one struct and compare its members, such as \
         `window.valid_until >= window.valid_from`\n"
    );
    // ... which ess/22 reads back as the fact it writes.
    let spec = admitted(&model(22, "{task_id: {eq: {fact: depends_on}}}", PLAIN, ""));
    assert_eq!(
        guard(&spec),
        compare("task_id", CompareOp::Eq, fact("depends_on"))
    );
}

#[test]
fn the_ordinary_type_rules_hold_between_two_facts() {
    // Text against Timestamp is no comparison two facts can make.
    let refused = assemble(&model(22, "note < deadline", PLAIN, ""))
        .expect_err("an ordering of text against a Timestamp");
    assert!(refused.contains("deadline"), "{refused}");
    // A list is no scalar.
    let refused =
        assemble(&model(22, "note == tags", PLAIN, "")).expect_err("a list compared with text");
    assert!(refused.contains("tags"), "{refused}");
}

// ---- the other predicate sites ---------------------------------------------------------------

const PARCELS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/stored-field-guards.yaml");
const TASKS: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/related-guard-stored-reference.yaml"
);
const DESK: &str = include_str!("../../ess-compiler/tests/fixtures/set-effects.yaml");

/// The parcel model at `format`, with a stored `limit_kg` beside `weight_kg`, a guard, a view filter
/// and a struct invariant each comparing two stored siblings by a bare word.
fn parcels(format: u32) -> String {
    PARCELS
        .replace("format: ess/9", &format!("format: ess/{format}"))
        .replace(
            "      - {name: weight_kg, type: Integer}\n    lifecycle:",
            "      - {name: weight_kg, type: Integer}\n      - {name: limit_kg, type: Integer}\n    lifecycle:",
        )
        .replace(
            "sets: {service: input.service, weight_kg: input.weight_kg}",
            "sets: {service: input.service, weight_kg: input.weight_kg, limit_kg: input.weight_kg}",
        )
        .replace("              - weight_kg > 20", "              - weight_kg > limit_kg")
        .replace(
            "    consistency: read_your_writes\n    fields:\n      - {name: parcel_id, type: Uuid}",
            "    consistency: read_your_writes\n    filter: weight_kg <= limit_kg\n    fields:\n      - {name: parcel_id, type: Uuid}",
        )
        .replace(
            "    variants: [Standard, Express]\n",
            "    variants: [Standard, Express]\n  - name: shipping.parcel.Band\n    kind: struct\n    fields:\n      - {name: low, type: Integer}\n      - {name: high, type: Integer}\n    invariants:\n      - high >= low\n",
        )
}

#[test]
fn a1_reaches_when_subject_view_filters_and_struct_invariants() {
    let spec = admitted(&parcels(22));
    let dispatch = &spec.commands()[&"shipping.parcel.Dispatch".parse().unwrap()];
    let OutcomeCondition::SubjectPredicate { predicate, .. } = &dispatch.outcomes[0].condition
    else {
        panic!("a stored-field guard");
    };
    assert!(
        predicate
            .to_string()
            .contains("weight_kg > {fact: limit_kg}"),
        "{predicate}"
    );
    let view = &spec.views()[&"shipping.parcel.Parcels".parse().unwrap()];
    assert_eq!(
        view.filter.as_ref().map(ToString::to_string).as_deref(),
        Some("weight_kg <= {fact: limit_kg}")
    );
    let band = spec
        .system()
        .types
        .get(&"shipping.parcel.Band".parse().unwrap())
        .expect("the struct");
    let ess_domain::TypeBody::Struct { invariants, .. } = &band.body else {
        panic!("a struct");
    };
    assert_eq!(invariants[0].predicate.to_string(), "high >= {fact: low}");
    // Below ess/22 each of the three is the text it was, and refused as one.
    let refused = assemble(&parcels(21)).expect_err("ess/21 reads the words as text");
    for site in ["when_subject", "filter", "Band.invariants"] {
        assert!(refused.contains(site), "{site}: {refused}");
    }
}

#[test]
fn a1_reaches_when_related_and_set_effect_filters() {
    let tasks = TASKS.replace(
        "predicate: state != Done}",
        "predicate: {all: [state != Done, blocked_by == blocked_by]}}",
    );
    let spec = admitted(&tasks);
    let complete = &spec.commands()[&"demo.tasks.CompleteTask".parse().unwrap()];
    let OutcomeCondition::Related { test, .. } = &complete.outcomes[1].condition else {
        panic!("a related guard");
    };
    let rendered = test.predicate().expect("a predicate").to_string();
    assert!(
        rendered.contains("state != Done") && rendered.contains("blocked_by == {fact: blocked_by}"),
        "the variant stays a literal and the field is read: {rendered}"
    );
    let desk = DESK
        .replace("format: ess/16", "format: ess/22")
        .replace(
            "instances: {where: team == input.team}\n        emits: [demo.desk.TeamNoted]",
            "instances: {where: {all: [team == input.team, note == team]}}\n        emits: [demo.desk.TeamNoted]",
        )
        .replace(
            "where: team == subject.team",
            "where: {all: [team == subject.team, note == team]}",
        );
    let spec = admitted(&desk);
    let note = &spec.commands()[&"demo.desk.NoteTeam".parse().unwrap()];
    let instances = note.outcomes[0]
        .set_effects
        .instances
        .as_ref()
        .expect("instances");
    assert!(
        instances
            .filter
            .to_string()
            .contains("note == {fact: team}"),
        "{}",
        instances.filter
    );
    let invite = &spec.commands()[&"demo.desk.Invite".parse().unwrap()];
    let affect = &invite.outcomes[0].set_effects.affects[0];
    assert!(
        affect.filter.to_string().contains("note == {fact: team}"),
        "{}",
        affect.filter
    );
}

/// A `when_related` guard of an `ess/22` command, over the row an input identity addresses: the
/// row stores another task's identity in `parent`.
fn related_identities(predicate: &str) -> String {
    format!(
        r"format: ess/22
system: graph
version: v1
domain: graph.deps
types:
  - {{name: graph.deps.TaskId, kind: newtype, of: Uuid}}
  - {{name: graph.deps.EdgeId, kind: newtype, of: Uuid}}
entities:
  - name: graph.deps.Task
    identity: {{name: task_id, type: graph.deps.TaskId}}
    fields:
      - {{name: parent, type: graph.deps.TaskId}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
  - name: graph.deps.Edge
    identity: {{name: edge_id, type: graph.deps.EdgeId}}
    fields: []
    lifecycle: {{initial: Linked, states: [Linked], terminal: [Linked]}}
errors:
  - {{name: graph.deps.NoSuchTask, summary: No task carries that identity., fields: []}}
  - {{name: graph.deps.Refused, summary: Refused., fields: []}}
events:
  - name: graph.deps.Opened
    fields: [{{name: task_id, type: graph.deps.TaskId}}]
  - name: graph.deps.Linked
    fields: [{{name: edge_id, type: graph.deps.EdgeId}}]
commands:
  - name: graph.deps.Open
    input:
      - {{name: parent, type: graph.deps.TaskId}}
    outcomes:
      - name: opened
        creates: graph.deps.Task
        instance: task_id
        sets: {{parent: input.parent}}
        emits: [graph.deps.Opened]
        payload: {{graph.deps.Opened: {{task_id: {{generated: true}}}}}}
  - name: graph.deps.Link
    input:
      - {{name: task_id, type: graph.deps.TaskId}}
      - {{name: other, type: graph.deps.TaskId}}
    outcomes:
      - name: no-such-task
        when_related: {{via: input.task_id, exists: false}}
        error: graph.deps.NoSuchTask
      - name: refused
        when_related: {{via: input.task_id, predicate: {predicate}}}
        error: graph.deps.Refused
      - name: linked
        creates: graph.deps.Edge
        instance: edge_id
        emits: [graph.deps.Linked]
        payload: {{graph.deps.Linked: {{edge_id: {{generated: true}}}}}}
"
    )
}

/// Decision 9 on a `when_related` guard (final review, F5): the related row's stored identity
/// compares with an input identity by `==` and `!=`, and an ordering of the two is refused as
/// `type_mismatch` at that guard.
#[test]
fn decision9_identity_ordering_in_a_when_related_guard_is_refused() {
    for admitted_predicate in ["parent != input.other", "parent == input.other"] {
        admitted(&related_identities(admitted_predicate));
    }
    for ordering in ["parent < input.other", "input.other >= parent"] {
        let refused = assemble(&related_identities(ordering)).expect_err(ordering);
        assert!(
            refused.contains(
                "[type_mismatch] command.graph.deps.Link.outcomes.refused.when_related: "
            ) && refused
                .contains("orders an identity: an identity token compares only by `==` and `!=`"),
            "{ordering}: {refused}"
        );
    }
}
