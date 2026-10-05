//! Adversary pass 1 on unit E-U1 (resolver, A1, decision 9).
//!
//! Two claims under attack:
//! - below `ess/22` every model's refusal bytes are unchanged against base `4e6180546`;
//! - decision 9: an ordering between identity tokens in an `ess/22` command guard is
//!   `type_mismatch`.
//!
//! The expected refusal strings below were read from base `4e6180546` with this same model.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn model(format: u32, when: &str) -> String {
    format!(
        r"format: ess/{format}
system: graph
version: v1
domain: graph.tasks
types:
  - {{name: graph.tasks.TaskId, kind: newtype, of: Uuid}}
entities:
  - name: graph.tasks.Edge
    identity: {{name: edge_id, type: Uuid}}
    fields:
      - {{name: valid_from, type: Timestamp}}
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
      - {{name: note, type: String}}
      - {{name: valid_from, type: Timestamp}}
    outcomes:
      - name: refused
        when: {when}
        error: graph.tasks.Refused
      - name: linked
        creates: graph.tasks.Edge
        instance: edge_id
        emits: [graph.tasks.Linked]
        payload:
          graph.tasks.Linked: {{edge_id: {{generated: true}}}}
        sets: {{valid_from: input.valid_from}}
"
    )
}

fn assemble(text: &str) -> String {
    match RawSpecFile::parse(text) {
        Err(error) => format!("PARSE: {error}"),
        Ok(raw) => match Specification::assemble([(Source::new("model.yaml"), raw)]) {
            Ok(_) => "ADMITTED".to_owned(),
            Err(errors) => format!("REFUSED: {errors}"),
        },
    }
}

/// An `ess/21` guard reading `input.<field>`: base refuses it as `unobservable_fact` at `when`,
/// naming the declared input. The unit skips that structural check for every format and leaves the
/// refusal to the registry-aware checker.
#[test]
fn adversary_ess21_input_namespace_refusal_bytes_match_base() {
    let base = "REFUSED: [unobservable_fact] command.graph.tasks.Link.outcomes.refused.when: \
                `input.depends_on` reads `input`, which `graph.tasks.Link` does not declare as \
                input; a condition on something the caller never supplied cannot be decided when \
                the command arrives (hint: declared input: `depends_on`, `note`, `task_id`, \
                `valid_from`)";
    assert_eq!(assemble(&model(21, "task_id == input.depends_on")), base);
}

/// An `ess/21` comparison whose operand is a mapping that is not `{fact: …}`: base refuses it with
/// "a comparison operand must be a scalar". Since beyond10x/ess#448 the same sentence is reported
/// at the guard that wrote it rather than ending the document.
#[test]
fn adversary_ess21_mapping_operand_refusal_bytes_match_base() {
    let base = "REFUSED: [unparsable_predicate] command.graph.tasks.Link.outcomes.refused.when: \
                cannot parse predicate \"note: {eq: {x: 1}}\": a comparison operand must be a \
                scalar";
    assert_eq!(assemble(&model(21, "{note: {eq: {x: 1}}}")), base);
}

/// The model of `expression_identity.rs` reduced to the guard: `Task` is identified by `TaskId`, and
/// `Link` takes one task and a list of blockers, all identity tokens.
fn identities(when: &str) -> String {
    format!(
        r"format: ess/22
system: graph
version: v1
domain: graph.deps
types:
  - {{name: graph.deps.TaskId, kind: newtype, of: Uuid}}
entities:
  - name: graph.deps.Task
    identity: {{name: task_id, type: graph.deps.TaskId}}
    fields:
      - {{name: title, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
errors:
  - {{name: graph.deps.Refused, summary: Refused., fields: []}}
events:
  - name: graph.deps.Linked
    fields: [{{name: task_id, type: graph.deps.TaskId}}]
commands:
  - name: graph.deps.Link
    input:
      - {{name: task_id, type: graph.deps.TaskId}}
      - {{name: depends_on, type: graph.deps.TaskId}}
      - {{name: blockers, type: List<graph.deps.TaskId>}}
    outcomes:
      - name: refused
        when: {when}
        error: graph.deps.Refused
      - name: linked
        emits: [graph.deps.Linked]
        payload: {{graph.deps.Linked: {{task_id: input.task_id}}}}
"
    )
}

/// The identity model with a stored identity on the row, and a `when_subject` guard over it.
fn stored_identities(predicate: &str) -> String {
    format!(
        r"format: ess/22
system: graph
version: v1
domain: graph.deps
types:
  - {{name: graph.deps.TaskId, kind: newtype, of: Uuid}}
entities:
  - name: graph.deps.Task
    identity: {{name: task_id, type: graph.deps.TaskId}}
    fields:
      - {{name: depends_on, type: graph.deps.TaskId}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
errors:
  - {{name: graph.deps.Refused, summary: Refused., fields: []}}
events:
  - name: graph.deps.Blocked
    fields: [{{name: task_id, type: graph.deps.TaskId}}]
commands:
  - name: graph.deps.Block
    input:
      - {{name: task_id, type: graph.deps.TaskId}}
      - {{name: other, type: graph.deps.TaskId}}
    outcomes:
      - name: refused
        when_subject: {{predicate: {predicate}}}
        error: graph.deps.Refused
      - name: blocked
        updates: graph.deps.Task
        instance: task_id
        sets: {{depends_on: input.other}}
        emits: [graph.deps.Blocked]
        payload: {{graph.deps.Blocked: {{task_id: input.task_id}}}}
"
    )
}

/// Decision 9 on a `when_subject` guard of an `ess/22` command: ordering the stored identity
/// against the input identity orders two identity tokens. Control: `!=` is admitted.
#[test]
fn adversary_identity_ordering_in_a_when_subject_guard_is_refused() {
    assert_eq!(
        assemble(&stored_identities("depends_on != input.other")),
        "ADMITTED"
    );
    let refused = assemble(&stored_identities("depends_on < input.other"));
    assert!(
        refused.contains("type_mismatch") && refused.contains("only by `==` and `!=`"),
        "{refused}"
    );
}

/// Control: the unit's own refusal, outside a quantifier.
#[test]
fn adversary_identity_ordering_control_is_refused() {
    let refused = assemble(&identities("task_id < depends_on"));
    assert!(
        refused.contains("type_mismatch") && refused.contains("only by `==` and `!=`"),
        "{refused}"
    );
}

/// Decision 9: identity tokens admit only `==` and `!=`. The same ordering between the same two
/// kinds of token, written inside a quantifier over a list of them, orders identity tokens just
/// the same, and is admitted.
#[test]
fn adversary_identity_ordering_inside_a_quantifier_is_refused() {
    for when in [
        "{forall: {in: blockers, as: b, that: b < task_id}}",
        "{exists: {in: blockers, as: b, that: task_id >= b}}",
    ] {
        let refused = assemble(&identities(when));
        assert!(
            refused.contains("type_mismatch") && refused.contains("only by `==` and `!=`"),
            "{when}: {refused}"
        );
    }
}
