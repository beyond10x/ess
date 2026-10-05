//! Adversary cases for unit E-U8 (row sets and filtered reads, beyond10x/ess#228, #299).
//!
//! `docs/design/filtered-related-reads.md`, "Compatibility and targets": below `ess/22` the shape
//! `{related: {entity, where, field}}` keeps the nested-mapping meaning it had wherever it was
//! valid, a refusal the old reader made is replayed, and "Parsing precedes source-version
//! admission". The joint guard partition of a row-set command is checked ("Shared row-set guard
//! contract").
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("parsed: {error}"));
    Specification::assemble([(Source::new("legacy.yaml"), raw)])
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
        .collect::<Vec<_>>()
        .join("\n")
}

/// An `ess/21` model whose entity holds a struct with a field `related` of fields `entity`,
/// `where` and `field`; `Record` fills it through `sets:`, and `Shut` through the `sets:` of an
/// `affects:` entry (ess/16) — each with `holder`.
fn legacy(holder: &str) -> String {
    format!(
        "format: ess/21
system: demo
version: v1
domain: demo.jobs
types:
  - {{name: demo.jobs.AttemptId, kind: newtype, of: Uuid}}
  - name: demo.jobs.Wrapped
    kind: struct
    fields:
      - {{name: entity, type: String}}
      - {{name: where, type: String}}
      - {{name: field, type: String}}
  - name: demo.jobs.Holder
    kind: struct
    fields:
      - {{name: related, type: demo.jobs.Wrapped}}
entities:
  - name: demo.jobs.Attempt
    identity: {{name: attempt_id, type: demo.jobs.AttemptId}}
    fields:
      - {{name: worker_id, type: String}}
      - {{name: holder, type: demo.jobs.Holder}}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {{name: close, from: [Open], to: Closed}}
errors:
  - {{name: demo.jobs.Refused, summary: Refused., fields: []}}
events:
  - name: demo.jobs.AttemptRecorded
    fields:
      - {{name: attempt_id, type: demo.jobs.AttemptId}}
commands:
  - name: demo.jobs.Record
    input:
      - {{name: worker_id, type: String}}
    outcomes:
      - name: recorded
        creates: demo.jobs.Attempt
        instance: attempt_id
        sets:
          worker_id: input.worker_id
          holder:
            related:
              entity: demo.jobs.Attempt
              where: worker_id
              field: delay
        emits: [demo.jobs.AttemptRecorded]
        payload:
          demo.jobs.AttemptRecorded: {{attempt_id: {{generated: true}}}}
  - name: demo.jobs.Shut
    input:
      - {{name: attempt_id, type: demo.jobs.AttemptId}}
    outcomes:
      - name: shut
        moves: demo.jobs.Attempt.close
        instance: attempt_id
        affects:
          - entity: demo.jobs.Attempt
            where: worker_id == subject.worker_id
            sets:
              holder:
{holder}
        emits: [demo.jobs.AttemptRecorded]
        payload:
          demo.jobs.AttemptRecorded: {{attempt_id: input.attempt_id}}
      - {{name: shut-already, wrong_state: true, error: demo.jobs.Refused}}
"
    )
}

const STRUCT_HOLDER: &str = "                related:
                  entity: demo.jobs.Attempt
                  where: worker_id
                  field: delay";

/// Below `ess/22` the struct-shaped `related:` keeps its nested-mapping meaning in every place a
/// nested mapping was valid — the `sets:` of an `affects:` entry (ess/16) as much as an outcome's
/// own `sets:` and `payload:`. The same document validates on the base `f53579c78`.
#[test]
fn below_ess_22_an_affect_struct_named_related_keeps_its_meaning() {
    let text = legacy(STRUCT_HOLDER);
    if let Err(errors) = assemble(&text) {
        panic!(
            "an ess/21 model valid before row sets is refused:\n{}",
            listed(&errors)
        );
    }
}

/// Below `ess/22` a `where:` leaf the nested reader refuses is refused by that reader, at parse,
/// as it was: parsing precedes source-version admission, and the old reader's acceptance is not
/// changed by recognising the new shape first (`filtered-related-reads.md:132-142`).
#[test]
fn below_ess_22_a_malformed_where_leaf_is_refused_at_parse_as_before() {
    let text = legacy(STRUCT_HOLDER).replacen(
        "              where: worker_id\n",
        "              where: {generated: maybe}\n",
        1,
    );
    assert!(text.contains("where: {generated: maybe}"));
    if let Err(error) = RawSpecFile::parse(&text) {
        assert!(
            error.to_string().contains("`generated` takes `true`"),
            "the old reader's refusal: {error}"
        );
    } else {
        let assembled = Specification::assemble([(
            Source::new("legacy.yaml"),
            RawSpecFile::parse(&text).expect("parsed"),
        )]);
        panic!(
            "an ess/21 document the old reader refused now parses; assembly reports: {}",
            assembled
                .err()
                .map_or("nothing".to_owned(), |errors| listed(&errors))
        );
    }
}

/// A row-set command whose branches leave a request answered by none of them is refused as
/// non-exhaustive: here the row set answers zero rows only, and the input-guarded accepting branch
/// answers `fast` only, so one stored row and `fast: false` is a request the specification says
/// nothing about ("Validation checks each predicate's types and the joint guard partition").
#[test]
fn a_row_set_command_leaving_a_count_and_input_unanswered_is_refused() {
    let text = "format: ess/22
system: demo
version: v1
domain: demo.jobs
types:
  - {name: demo.jobs.AttemptId, kind: newtype, of: Uuid}
entities:
  - name: demo.jobs.Attempt
    identity: {name: attempt_id, type: demo.jobs.AttemptId}
    fields:
      - {name: worker_id, type: String}
      - {name: batch_id, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
events:
  - name: demo.jobs.Started
    fields:
      - {name: worker_id, type: String}
  - name: demo.jobs.Recorded
    fields:
      - {name: attempt_id, type: demo.jobs.AttemptId}
commands:
  - name: demo.jobs.Record
    input:
      - {name: worker_id, type: String}
      - {name: batch_id, type: String}
    outcomes:
      - name: recorded
        creates: demo.jobs.Attempt
        instance: attempt_id
        sets: {worker_id: input.worker_id, batch_id: input.batch_id}
        emits: [demo.jobs.Recorded]
        payload:
          demo.jobs.Recorded: {attempt_id: {generated: true}}
  - name: demo.jobs.Start
    input:
      - {name: worker_id, type: String}
      - {name: batch_id, type: String}
      - {name: fast, type: Boolean}
    outcomes:
      - name: fresh
        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}
          count: {eq: 0}
        emits: [demo.jobs.Started]
        payload:
          demo.jobs.Started: {worker_id: input.worker_id}
      - name: quick
        when: fast == true
        emits: [demo.jobs.Started]
        payload:
          demo.jobs.Started: {worker_id: input.worker_id}
";
    match assemble(text) {
        Ok(_) => {
            panic!("`Start` with one stored row and `fast: false` selects no branch, and validates")
        }
        Err(errors) => assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::NonExhaustiveBranches),
            "{}",
            listed(&errors)
        ),
    }
}

/// A `count:` bound is any nonnegative Integer ("Bounds above the existing 10,000-live-cursor
/// synthesis cap are valid declarations"), and checking the branches over it terminates: two
/// complementary branches at the largest bound validate in well under the time limit.
#[test]
fn checking_a_row_set_partition_at_a_large_count_bound_terminates() {
    let text = "format: ess/22
system: demo
version: v1
domain: demo.jobs
types:
  - {name: demo.jobs.AttemptId, kind: newtype, of: Uuid}
entities:
  - name: demo.jobs.Attempt
    identity: {name: attempt_id, type: demo.jobs.AttemptId}
    fields:
      - {name: worker_id, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
errors:
  - {name: demo.jobs.Crowded, summary: Crowded., fields: []}
events:
  - name: demo.jobs.Recorded
    fields:
      - {name: attempt_id, type: demo.jobs.AttemptId}
  - name: demo.jobs.Started
    fields:
      - {name: worker_id, type: String}
commands:
  - name: demo.jobs.Record
    input:
      - {name: worker_id, type: String}
    outcomes:
      - name: recorded
        creates: demo.jobs.Attempt
        instance: attempt_id
        sets: {worker_id: input.worker_id}
        emits: [demo.jobs.Recorded]
        payload:
          demo.jobs.Recorded: {attempt_id: {generated: true}}
  - name: demo.jobs.Start
    input:
      - {name: worker_id, type: String}
    outcomes:
      - name: crowded
        when_related:
          entity: demo.jobs.Attempt
          where: worker_id == input.worker_id
          count: {gt: 9223372036854775807}
        error: demo.jobs.Crowded
      - name: started
        when_related:
          entity: demo.jobs.Attempt
          where: worker_id == input.worker_id
          count: {lte: 9223372036854775807}
        emits: [demo.jobs.Started]
        payload:
          demo.jobs.Started: {worker_id: input.worker_id}
";
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("parsed: {error}"));
    let (done, finished) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let assembled = Specification::assemble([(Source::new("bound.yaml"), raw)]);
        let _ = done.send(assembled.err().map(|errors| listed(&errors)));
    });
    match finished.recv_timeout(std::time::Duration::from_secs(20)) {
        Ok(errors) => assert_eq!(errors, None, "the complementary branches validate"),
        Err(error) => panic!("validating `Start` did not finish within 20 s: {error}"),
    }
}
