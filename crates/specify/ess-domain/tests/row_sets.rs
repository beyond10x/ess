//! Row sets and filtered related reads (`docs/design/filtered-related-reads.md`; beyond10x/ess#228,
//! beyond10x/ess#299; unit U8 of `docs/design/expression-family-source22.md`).
//!
//! From `ess/22` a branch is guarded by the rows of an entity a `where:` predicate selects —
//! `when_related: {entity, where, exists | count | forall}` — and a value reads one field of the
//! one row such a selector selects: `{related: {entity, where, field}}`. Below `ess/22` the guard is
//! refused naming `ess/22`, and the value keeps the nested-mapping meaning the shape had.
use ess_domain::command::{OutcomeCondition, PayloadSource};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_primitives::predicate::{Operand, Predicate};

const READS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/filtered-related-reads.yaml");
const UNIQUE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/unique-within-scope.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("rows.yaml"), raw)])
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn validates(text: &str) -> Specification {
    match assemble(text) {
        Ok(spec) => spec,
        Err(errors) => panic!("{}\n---\n{text}", listed(&errors)),
    }
}

/// Every refusal of `text` with `code` whose message holds `needle`, as `location message` lines.
fn refusals(text: &str, code: ValidationCode, needle: &str) -> Vec<String> {
    let errors = assemble(text).expect_err("refused");
    let found: Vec<String> = errors
        .as_slice()
        .iter()
        .filter(|error| error.code == code && error.message.contains(needle))
        .map(|error| format!("{} {}", error.location, error.message))
        .collect();
    assert_ne!(
        found.len(),
        0,
        "expected {code:?} containing {needle:?}:\n{}",
        listed(&errors)
    );
    found
}

/// One `Attempt` entity and a command `Probe` whose first branch carries `guard` (indented as an
/// outcome key) and `extra` further outcome lines, under `format`.
fn probe(format: &str, guard: &str, extra: &str) -> String {
    format!(
        "format: {format}
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
      - {{name: batch_id, type: String}}
      - {{name: delay, type: Integer}}
      - {{name: note, type: Optional<String>}}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {{name: close, from: [Open], to: Closed}}
errors:
  - {{name: demo.jobs.Refused, summary: Refused., fields: []}}
events:
  - name: demo.jobs.Probed
    fields:
      - {{name: worker_id, type: String}}
      - {{name: delay, type: Integer}}
      - {{name: holder, type: demo.jobs.Holder}}
  - name: demo.jobs.AttemptRecorded
    fields:
      - {{name: attempt_id, type: demo.jobs.AttemptId}}
commands:
  - name: demo.jobs.Record
    input:
      - {{name: worker_id, type: String}}
      - {{name: batch_id, type: String}}
      - {{name: delay, type: Integer}}
    outcomes:
      - name: recorded
        creates: demo.jobs.Attempt
        instance: attempt_id
        sets: {{worker_id: input.worker_id, batch_id: input.batch_id, delay: input.delay}}
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
        emits: [demo.jobs.AttemptRecorded]
        payload:
          demo.jobs.AttemptRecorded: {{attempt_id: input.attempt_id}}
      - {{name: shut-already, wrong_state: true, error: demo.jobs.Refused}}
  - name: demo.jobs.Probe
    input:
      - {{name: worker_id, type: String}}
      - {{name: batch_id, type: String}}
      - {{name: attempt_id, type: demo.jobs.AttemptId}}
    outcomes:
      - name: refused
{guard}
        error: demo.jobs.Refused
{extra}
"
    )
}

const PROBED: &str = "      - name: probed
        emits: [demo.jobs.Probed]
        payload:
          demo.jobs.Probed: {worker_id: input.worker_id, delay: 0, holder: {generated: true}}";

const SCOPED: &str = "{all: [worker_id == input.worker_id, batch_id == input.batch_id]}";

fn guard(test: &str) -> String {
    format!(
        "        when_related:
          entity: demo.jobs.Attempt
          where: {SCOPED}
          {test}"
    )
}

#[test]
fn row_sets_admit_the_guard_and_the_filtered_read_from_ess_22() {
    validates(READS);
    validates(UNIQUE);
    for test in [
        "exists: true",
        "exists: false",
        "count: {eq: 0}",
        "count: {ne: 1}",
        "count: {lt: 2}",
        "count: {lte: 2}",
        "count: {gt: 1}",
        "count: {gte: 3}",
        "forall: delay > 3",
        "forall: {all: [delay <= 10, defined(note)]}",
    ] {
        validates(&probe("ess/22", &guard(test), PROBED));
    }
}

#[test]
fn row_set_guard_below_ess_22_names_ess_22() {
    for format in ["ess/18", "ess/20", "ess/21"] {
        let text = UNIQUE.replacen("format: ess/22", &format!("format: {format}"), 1);
        let found = refusals(&text, ValidationCode::UnsupportedFormatVersion, "ess/22");
        assert!(
            found
                .iter()
                .any(|line| line.contains("claims-taken") && line.contains("when_related")),
            "{format}: {found:#?}"
        );
    }
}

#[test]
fn row_set_guard_refuses_malformed_selectors_at_when_related() {
    let cases: [(&str, ValidationCode, &str); 9] = [
        (
            "        when_related: {via: input.attempt_id, entity: demo.jobs.Attempt, where: delay > 1, exists: true}",
            ValidationCode::ConflictingDeclaration,
            "`via`",
        ),
        (
            "        when_related: {entity: demo.jobs.Attempt, exists: true}",
            ValidationCode::MissingDeclaration,
            "`where`",
        ),
        (
            "        when_related: {entity: demo.jobs.Attempt, where: delay > 1}",
            ValidationCode::EmptyDeclaration,
            "`exists`, `count` or `forall`",
        ),
        (
            "        when_related: {entity: demo.jobs.Attempt, where: delay > 1, exists: true, count: {eq: 1}}",
            ValidationCode::ConflictingDeclaration,
            "exactly one",
        ),
        (
            "        when_related: {entity: demo.jobs.Attempt, where: delay > 1, count: {eq: 1, gt: 0}}",
            ValidationCode::TypeMismatch,
            "one comparison",
        ),
        (
            "        when_related: {entity: demo.jobs.Attempt, where: delay > 1, count: {between: 1}}",
            ValidationCode::TypeMismatch,
            "`eq`, `ne`, `lt`, `lte`, `gt` or `gte`",
        ),
        (
            "        when_related: {entity: demo.jobs.Attempt, where: delay > 1, count: {eq: -1}}",
            ValidationCode::TypeMismatch,
            "nonnegative",
        ),
        (
            "        when_related: {entity: demo.jobs.Missing, where: delay > 1, exists: true}",
            ValidationCode::UndeclaredReference,
            "demo.jobs.Missing",
        ),
        (
            "        when_related: {entity: demo.jobs.Attempt, where: always, exists: true}",
            ValidationCode::EmptyDeclaration,
            "selects every row",
        ),
    ];
    for (written, code, needle) in cases {
        let found = refusals(&probe("ess/22", written, PROBED), code, needle);
        assert!(
            found.iter().any(|line| line.contains("when_related")),
            "{written}: {found:#?}"
        );
    }
}

#[test]
fn row_set_predicates_are_typed_over_the_candidate_row_input_and_subject() {
    // A field the entity does not declare, an input the command does not take, and a comparison
    // across types are each refused where the predicate is written.
    for (written, needle) in [
        (
            guard("exists: true").replace("batch_id == input.batch_id", "shard == input.batch_id"),
            "shard",
        ),
        (
            guard("exists: true").replace("input.batch_id", "input.shard"),
            "shard",
        ),
        (guard("forall: delay == input.worker_id"), "delay"),
    ] {
        let errors = assemble(&probe("ess/22", &written, PROBED)).expect_err("refused");
        let text = listed(&errors);
        assert!(
            text.contains("when_related") && text.contains(needle),
            "{written}\n{text}"
        );
    }
    // `subject.` reads the subject the command addresses: refused where the command creates its
    // only subject, admitted where it moves an existing one.
    let on_creation = probe(
        "ess/22",
        "        when_related:
          entity: demo.jobs.Attempt
          where: worker_id == subject.worker_id
          exists: true",
        "      - name: started
        creates: demo.jobs.Attempt
        instance: attempt_id
        sets: {worker_id: input.worker_id, batch_id: input.batch_id, delay: 0}
        emits: [demo.jobs.AttemptRecorded]
        payload:
          demo.jobs.AttemptRecorded: {attempt_id: {generated: true}}",
    );
    let errors = assemble(&on_creation).expect_err("refused");
    assert!(
        listed(&errors).contains("subject"),
        "a subject read on a creation is refused:\n{}",
        listed(&errors)
    );
    let on_move = probe(
        "ess/22",
        "        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == subject.worker_id, attempt_id != subject.attempt_id]}
          exists: true",
        "      - name: closed
        moves: demo.jobs.Attempt.close
        instance: attempt_id
        emits: [demo.jobs.Probed]
        payload:
          demo.jobs.Probed: {worker_id: input.worker_id, delay: 0, holder: {generated: true}}
      - {name: gone, unknown_instance: true, error: demo.jobs.Refused}
      - {name: shut, wrong_state: true, error: demo.jobs.Refused}",
    );
    validates(&on_move);
}

#[test]
fn row_set_guards_admit_now_in_where_and_forall_from_ess_22() {
    let timed = |test: &str| {
        probe("ess/22", &guard(test), PROBED)
            .replace(
                "      - {name: note, type: Optional<String>}",
                "      - {name: note, type: Optional<String>}\n      - {name: due_at, type: Timestamp}",
            )
            .replace(
                "sets: {worker_id: input.worker_id, batch_id: input.batch_id, delay: input.delay}",
                "sets: {worker_id: input.worker_id, batch_id: input.batch_id, delay: input.delay, due_at: input.due_at}",
            )
            .replace(
                "      - {name: delay, type: Integer}\n    outcomes:\n      - name: recorded",
                "      - {name: delay, type: Integer}\n      - {name: due_at, type: Timestamp}\n    outcomes:\n      - name: recorded",
            )
    };
    validates(&timed("forall: due_at > now - 5m"));
    validates(&timed("exists: true").replace(
        "batch_id == input.batch_id]}",
        "batch_id == input.batch_id, due_at <= now + 1h]}",
    ));
    let found = refusals(
        &timed("forall: due_at == now"),
        ValidationCode::TypeMismatch,
        "never equated",
    );
    assert_eq!(found.len(), 1, "{found:#?}");
}

#[test]
fn filtered_value_reads_one_field_of_the_selected_row_at_its_type() {
    let reading = |field: &str, target: &str| {
        probe(
            "ess/22",
            &guard("count: {ne: 1}"),
            &format!(
                "      - name: probed
        emits: [demo.jobs.Probed]
        payload:
          demo.jobs.Probed:
            worker_id: input.worker_id
            holder: {{generated: true}}
            {target}:
              related:
                entity: demo.jobs.Attempt
                where: {SCOPED}
                field: {field}"
            ),
        )
    };
    validates(&reading("delay", "delay"));
    // The selected field must be one the entity declares, at a type the target takes.
    let missing = assemble(&reading("latency", "delay")).expect_err("refused");
    assert!(listed(&missing).contains("latency"), "{}", listed(&missing));
    let mismatched = assemble(&reading("worker_id", "delay")).expect_err("refused");
    assert!(
        listed(&mismatched).contains("worker_id"),
        "{}",
        listed(&mismatched)
    );
    // `via` beside a selector, and a selector missing `where`, are refused.
    let mixed = reading("delay", "delay").replace(
        "                entity: demo.jobs.Attempt\n",
        "                entity: demo.jobs.Attempt\n                via: input.attempt_id\n",
    );
    assert!(RawSpecFile::parse(&mixed).is_err() || assemble(&mixed).is_err());
}

#[test]
fn below_ess_22_a_related_mapping_keeps_its_struct_meaning() {
    // A struct whose field `related` holds `{entity, where, field}` texts: a nested mapping before
    // ess/22, the same shape the filtered read has from it.
    let old = probe(
        "ess/21",
        "        when: worker_id == \"x\"",
        "      - name: probed
        emits: [demo.jobs.Probed]
        payload:
          demo.jobs.Probed:
            worker_id: input.worker_id
            delay: 0
            holder:
              related:
                entity: demo.jobs.Attempt
                where: worker_id
                field: delay",
    );
    let spec = validates(&old);
    let probe = spec
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.jobs.Probe")
        .expect("declared");
    let probed = probe
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "probed")
        .expect("declared");
    let rendered = format!("{:?}", probed.payload);
    assert!(
        rendered.contains("Struct") && !rendered.contains("RelatedSelection"),
        "{rendered}"
    );
    // The same over a scalar target below ess/22 is refused as the nested mapping it reads as.
    let scalar = old.replace(
        "            delay: 0\n            holder:\n",
        "            holder: {generated: true}\n            delay:\n",
    );
    assert!(assemble(&scalar).is_err());
}

#[test]
fn a_bare_word_in_a_selector_names_a_field_of_the_candidate_row() {
    // From ess/22 an unquoted word on the right names a field of the place it is written in: the
    // candidate row of the selector, in the guard and in a filtered read alike.
    let same = "{all: [worker_id == input.worker_id, batch_id == worker_id]}";
    let text = probe(
        "ess/22",
        &format!(
            "        when_related:\n          entity: demo.jobs.Attempt\n          where: {same}\n          exists: true"
        ),
        &format!(
            "      - name: probed
        emits: [demo.jobs.Probed]
        payload:
          demo.jobs.Probed:
            worker_id: input.worker_id
            holder: {{generated: true}}
            delay:
              related:
                entity: demo.jobs.Attempt
                where: {same}
                field: delay"
        ),
    );
    let spec = validates(&text);
    let probe = spec
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.jobs.Probe")
        .expect("declared");
    // Both read the field, never the text `"worker_id"`.
    let reads_field = |filter: &Predicate| match filter {
        Predicate::All(children) => matches!(
            &children[1],
            Predicate::Compare { right: Operand::Fact(path), .. } if path.to_string() == "worker_id"
        ),
        _ => false,
    };
    let guard = probe
        .outcomes
        .iter()
        .find_map(|outcome| match &outcome.condition {
            OutcomeCondition::RelatedSet { selection, .. } => Some(&selection.filter),
            _ => None,
        })
        .expect("a row-set guard");
    assert!(reads_field(guard), "{guard:?}");
    let read = probe
        .outcomes
        .iter()
        .flat_map(|outcome| outcome.payload.values())
        .find_map(|table| match table.get("delay") {
            Some(PayloadSource::RelatedSelection { selection, .. }) => Some(&selection.filter),
            _ => None,
        })
        .expect("a filtered read");
    assert!(reads_field(read), "{read:?}");
}

/// An `ess/21` model whose struct field `related` holds `{entity, where, field}` texts in an
/// outcome's `sets:` and in the `sets:` of an `affects:` entry (ess/16).
const AFFECTED: &str = "format: ess/21
system: demo
version: v1
domain: demo.jobs
types:
  - {name: demo.jobs.AttemptId, kind: newtype, of: Uuid}
  - name: demo.jobs.Wrapped
    kind: struct
    fields:
      - {name: entity, type: String}
      - {name: where, type: String}
      - {name: field, type: String}
  - name: demo.jobs.Holder
    kind: struct
    fields:
      - {name: related, type: demo.jobs.Wrapped}
entities:
  - name: demo.jobs.Attempt
    identity: {name: attempt_id, type: demo.jobs.AttemptId}
    fields:
      - {name: worker_id, type: String}
      - {name: holder, type: demo.jobs.Holder}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - {name: demo.jobs.Refused, summary: Refused., fields: []}
events:
  - name: demo.jobs.AttemptRecorded
    fields:
      - {name: attempt_id, type: demo.jobs.AttemptId}
commands:
  - name: demo.jobs.Record
    input:
      - {name: worker_id, type: String}
    outcomes:
      - name: recorded
        creates: demo.jobs.Attempt
        instance: attempt_id
        sets:
          worker_id: input.worker_id
          holder: {related: {entity: demo.jobs.Attempt, where: worker_id, field: delay}}
        emits: [demo.jobs.AttemptRecorded]
        payload:
          demo.jobs.AttemptRecorded: {attempt_id: {generated: true}}
  - name: demo.jobs.Shut
    input:
      - {name: attempt_id, type: demo.jobs.AttemptId}
    outcomes:
      - name: shut
        moves: demo.jobs.Attempt.close
        instance: attempt_id
        affects:
          - entity: demo.jobs.Attempt
            where: worker_id == subject.worker_id
            sets:
              holder: {related: {entity: demo.jobs.Attempt, where: worker_id, field: delay}}
        emits: [demo.jobs.AttemptRecorded]
        payload:
          demo.jobs.AttemptRecorded: {attempt_id: input.attempt_id}
      - {name: shut-already, wrong_state: true, error: demo.jobs.Refused}
";

/// The `affects:` entries' `sets:` of `command`, rendered.
fn affected(spec: &Specification, command: &str) -> String {
    let command = spec
        .commands()
        .values()
        .find(|candidate| candidate.name.to_string() == command)
        .expect("declared");
    command
        .outcomes
        .iter()
        .map(|outcome| format!("{:?}", outcome.set_effects.affects))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn below_ess_22_an_affect_s_related_mapping_keeps_its_struct_meaning() {
    // Read by a parser that knows the format: the nested reader, as before ess/22.
    let parsed = validates(AFFECTED);
    // Deserialised directly, the reader knows no format and recognises the shape; assembly reads
    // it back as the nested mapping wherever it stands, an `affects:` entry included.
    let raw: RawSpecFile =
        ess_yaml::from_str(AFFECTED).unwrap_or_else(|error| panic!("deserialised: {error}"));
    let direct = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates:\n{}", listed(&errors)));
    for spec in [&parsed, &direct] {
        let rendered = affected(spec, "demo.jobs.Shut");
        assert!(
            rendered.contains("Struct") && !rendered.contains("RelatedSelection"),
            "{rendered}"
        );
    }
    assert_eq!(
        affected(&parsed, "demo.jobs.Shut"),
        affected(&direct, "demo.jobs.Shut")
    );
}

#[test]
fn below_ess_22_a_where_leaf_the_nested_reader_refuses_is_refused_at_parse() {
    let text = AFFECTED.replacen(
        "holder: {related: {entity: demo.jobs.Attempt, where: worker_id, field: delay}}",
        "holder: {related: {entity: demo.jobs.Attempt, where: {generated: maybe}, field: delay}}",
        1,
    );
    assert_ne!(text, AFFECTED);
    let error = RawSpecFile::parse(&text)
        .err()
        .unwrap_or_else(|| panic!("an ess/21 `where:` the nested reader refuses parses"));
    assert!(
        error.to_string().contains("`generated` takes `true`"),
        "{error}"
    );
    // From ess/22 the same mapping is a filtered read, and its selector is refused at assembly.
    let source22 = text.replacen("format: ess/21", "format: ess/22", 1);
    assert!(RawSpecFile::parse(&source22).is_ok());
}

/// The reproduction of beyond10x/ess#554, at `ess/32`: a wire joins an output port to an
/// input port, each port named by the input and read through `when_related: {via: …}`, beside a
/// row-set guard refusing a source port a wire already leaves.
const REPRO: &str = include_str!("fixtures/identity-beside-row-set.yaml");

/// The reproduction under `format`.
fn identity_beside(format: &str) -> String {
    let text = REPRO.replacen("format: ess/32\n", &format!("format: {format}\n"), 1);
    assert!(text.starts_with(&format!("format: {format}\n")), "{text}");
    text
}

#[test]
fn from_ess_32_an_identity_guard_beside_a_row_set_validates() {
    let source = identity_beside("ess/32");
    let spec = validates(&source);
    let connect = spec
        .commands()
        .values()
        .find(|command| command.name.to_string() == "wiring.board.Connect")
        .expect("Connect");
    assert!(connect
        .outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, OutcomeCondition::RelatedSet { .. })));
    assert!(connect
        .outcomes
        .iter()
        .any(|outcome| matches!(outcome.condition, OutcomeCondition::Related { .. })));
    // The row set is credited with no case of the named rows' partition: without the default,
    // the pair is refused as before rather than read as covered.
    let without_default = source.replacen(
        "      - name: connected\n        creates",
        "      - name: connected\n        when: input.from_port != input.to_port\n        creates",
        1,
    );
    assert_ne!(without_default, source);
    refusals(&without_default, ValidationCode::NonExhaustiveBranches, "");
    // A row-set refusal declared above a named row's `exists: false` validates: a marker declared
    // below a guarded branch is not refused (ess/29), and the missing row still answers first
    // (`precedence_plan_answers.rs`, `identity_and_row_set_guards_answer_in_step_4_order`).
    let taken = "      - name: source-taken
        when_related:
          entity: wiring.board.Wire
          where: from_port == input.from_port
          exists: true
        error: wiring.board.SourceTaken
";
    let reordered = source.replacen(taken, "", 1).replacen(
        "      - name: no-source\n",
        &format!("{taken}      - name: no-source\n"),
        1,
    );
    assert_ne!(reordered, source);
    validates(&reordered);
}

#[test]
fn below_ess_32_an_identity_guard_beside_a_row_set_names_ess_32() {
    for format in ["ess/31", "ess/25"] {
        let text = identity_beside(format);
        let errors = assemble(&text).expect_err("refused below ess/32");
        let refused: Vec<_> = errors
            .as_slice()
            .iter()
            .filter(|error| error.code == ValidationCode::UnsupportedConstruct)
            .collect();
        assert_eq!(refused.len(), 1, "{format}:\n{}", listed(&errors));
        assert_eq!(
            refused[0].message,
            "`wiring.board.Connect` selects on a row set (`when_related: {entity, where, …}`) and \
             on an identity-addressed `when_related`; one command reads one kind of related row \
             in this cut",
            "{format}"
        );
        let hint = refused[0].hint.as_deref().unwrap_or_default();
        assert!(hint.contains("`format: ess/32`"), "{format}: {hint}");
        // The refusal is the only one: nothing partitions the pair under it.
        assert_eq!(errors.as_slice().len(), 1, "{format}:\n{}", listed(&errors));
    }
    // Any other guard beside a row set keeps its refusal, and its hint, at ess/32.
    let subject = probe(
        "ess/32",
        "        when_related: {entity: demo.jobs.Attempt, where: worker_id == input.worker_id, exists: true}",
        "      - name: probed\n        when_subject:\n          predicate: worker_id == input.worker_id\n        error: demo.jobs.Refused",
    );
    let errors = assemble(&subject).expect_err("a `when_subject` beside a row set is refused");
    let refused = errors
        .as_slice()
        .iter()
        .find(|error| {
            error.code == ValidationCode::UnsupportedConstruct
                && error.message.contains("a `when_subject` guard")
        })
        .unwrap_or_else(|| panic!("{}", listed(&errors)));
    assert!(
        !refused
            .hint
            .as_deref()
            .unwrap_or_default()
            .contains("ess/32"),
        "{refused:?}"
    );
}
