//! Family F part C at the source (beyond10x/ess#233): from `ess/22`, `.utf8_bytes` after a `String`
//! — through newtypes and `Optional` — is the UTF-8 byte length of that text, an `Integer`. It is a
//! comparison operand and nothing else (final review decision 11): the resolved predicate carries
//! `{utf8_bytes: <parent>}`, and every other position refuses it. A declared struct member named
//! `utf8_bytes` stays that member in every format, and below `ess/22` the selector is refused by
//! format. `docs/design/expression-family-source22.md`, "String `.utf8_bytes`", is the design.

use ess_domain::command::OutcomeCondition;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::predicate::Predicate;

/// A note filing command guarded by `when`, a renaming command guarded over the stored row by
/// `subject`, an entity holding `invariant`, and a view filtered by `filter`.
fn model(format: u32, when: &str, subject: &str, invariant: &str, filter: &str) -> String {
    format!(
        r"format: ess/{format}
system: notes
version: v1
domain: notes.core
types:
  - {{name: notes.core.Title, kind: newtype, of: String}}
  - {{name: notes.core.Kind, kind: enum, variants: [Plain, Rich]}}
  - name: notes.core.Record
    kind: struct
    fields:
      - {{name: utf8_bytes, type: Integer}}
      - {{name: label, type: String}}
entities:
  - name: notes.core.Note
    identity: {{name: note_id, type: Uuid}}
    fields:
      - {{name: label, type: String}}
      - {{name: title, type: notes.core.Title}}
    invariants:
      - {invariant}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
errors:
  - {{name: notes.core.Refused, summary: Refused.}}
events:
  - name: notes.core.Filed
    fields:
      - {{name: note_id, type: Uuid}}
commands:
  - name: notes.core.File
    input:
      - {{name: label, type: String}}
      - {{name: title, type: notes.core.Title}}
      - {{name: note, type: Optional<String>}}
      - {{name: id, type: Uuid}}
      - {{name: at, type: Timestamp}}
      - {{name: blob, type: Bytes}}
      - {{name: kind, type: notes.core.Kind}}
      - {{name: tags, type: List<String>}}
      - {{name: record, type: notes.core.Record}}
      - {{name: limit, type: Integer}}
    outcomes:
      - name: refused
        when: {when}
        error: notes.core.Refused
      - name: filed
        creates: notes.core.Note
        instance: note_id
        emits: [notes.core.Filed]
        payload:
          notes.core.Filed: {{note_id: {{generated: true}}}}
        sets: {{label: input.label, title: input.title}}
  - name: notes.core.Rename
    input:
      - {{name: note_id, type: Uuid}}
      - {{name: label, type: String}}
    outcomes:
      - name: too-long
        when_subject: {{predicate: {subject}}}
        error: notes.core.Refused
      - name: renamed
        updates: notes.core.Note
        instance: note_id
        sets: {{label: input.label}}
        emits: [notes.core.Filed]
        payload: {{notes.core.Filed: {{note_id: input.note_id}}}}
views:
  - name: notes.core.Short
    source: notes.core.Note
    consistency: read_your_writes
    filter: {filter}
    fields:
      - {{name: note_id, type: Uuid}}
      - {{name: label, type: String}}
"
    )
}

/// Every site plain, so a variant of one site is the only thing that moves.
fn guarded(format: u32, when: &str) -> String {
    model(
        format,
        when,
        "label == input.label",
        "label != \"\"",
        "label != \"\"",
    )
}

fn assemble(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| format!("parse: {error}"))?;
    Specification::assemble([(Source::new("model.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn admitted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("admitted:\n{errors}\n---\n{text}"))
}

fn json(predicate: &Predicate) -> String {
    serde_json::to_string(predicate).expect("serialises")
}

fn guard(spec: &Specification) -> String {
    let command = &spec.commands()[&"notes.core.File".parse().unwrap()];
    match &command.outcomes[0].condition {
        OutcomeCondition::When(predicate) => json(predicate),
        other => panic!("a plain when, not {other:?}"),
    }
}

fn subject(spec: &Specification) -> String {
    let command = &spec.commands()[&"notes.core.Rename".parse().unwrap()];
    match &command.outcomes[0].condition {
        OutcomeCondition::SubjectPredicate { predicate, .. } => json(predicate),
        other => panic!("a when_subject, not {other:?}"),
    }
}

fn invariant(spec: &Specification) -> String {
    json(&spec.entities()[&"notes.core.Note".parse().unwrap()].invariants[0].predicate)
}

fn filter(spec: &Specification) -> String {
    json(
        spec.views()[&"notes.core.Short".parse().unwrap()]
            .filter
            .as_ref()
            .expect("a filter"),
    )
}

#[test]
fn utf8_a_guard_resolves_to_the_derived_selector() {
    for (written, resolved) in [
        (
            "label.utf8_bytes <= 8",
            r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"lte","right":8.0}}"#,
        ),
        (
            "title.utf8_bytes < 10",
            r#"{"compare":{"left":{"utf8_bytes":"title"},"op":"lt","right":10.0}}"#,
        ),
        (
            "note.utf8_bytes > 2",
            r#"{"compare":{"left":{"utf8_bytes":"note"},"op":"gt","right":2.0}}"#,
        ),
        (
            "record.label.utf8_bytes == 3",
            r#"{"compare":{"left":{"utf8_bytes":"record.label"},"op":"eq","right":3.0}}"#,
        ),
        (
            "input.label.utf8_bytes >= 1",
            r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"gte","right":1.0}}"#,
        ),
        (
            "limit >= label.utf8_bytes",
            r#"{"limit":{"gte":{"utf8_bytes":"label"}}}"#,
        ),
        (
            "label.utf8_bytes != title.utf8_bytes",
            r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"ne","right":{"utf8_bytes":"title"}}}"#,
        ),
        (
            "label.utf8_bytes <= limit",
            r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"lte","right":{"fact":"limit"}}}"#,
        ),
        (
            "{forall: {in: tags, as: tag, that: tag.utf8_bytes <= 4}}",
            r#"{"forall":{"as":"tag","in":"tags","that":{"compare":{"left":{"utf8_bytes":"tag"},"op":"lte","right":4.0}}}}"#,
        ),
    ] {
        let spec = admitted(&guarded(22, written));
        assert_eq!(guard(&spec), resolved, "{written}");
    }
}

#[test]
fn utf8_the_canonical_form_reads_back_as_the_compact_spelling_resolves() {
    let compact = admitted(&guarded(22, "label.utf8_bytes <= 8"));
    let canonical = admitted(&guarded(
        22,
        "{compare: {left: {utf8_bytes: label}, op: lte, right: 8}}",
    ));
    assert_eq!(guard(&compact), guard(&canonical));
    let right = admitted(&guarded(22, "{limit: {gte: {utf8_bytes: label}}}"));
    assert_eq!(guard(&right), r#"{"limit":{"gte":{"utf8_bytes":"label"}}}"#);
}

#[test]
fn utf8_every_predicate_site_resolves_it() {
    let spec = admitted(&model(
        22,
        "label == \"x\"",
        "label.utf8_bytes > input.label.utf8_bytes",
        "label.utf8_bytes <= 255",
        "label.utf8_bytes <= 16",
    ));
    assert_eq!(
        subject(&spec),
        r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"gt","right":{"utf8_bytes":"input.label"}}}"#
    );
    assert_eq!(
        invariant(&spec),
        r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"lte","right":255.0}}"#
    );
    assert_eq!(
        filter(&spec),
        r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"lte","right":16.0}}"#
    );
}

#[test]
fn utf8_a_declared_member_named_utf8_bytes_keeps_its_meaning_in_every_format() {
    for format in [21, 22] {
        let spec = admitted(&guarded(format, "record.utf8_bytes <= 3"));
        assert_eq!(guard(&spec), r#""record.utf8_bytes <= 3""#, "ess/{format}");
    }
    // The member is an Integer like any other: it is compared, not measured.
    let refused = assemble(&guarded(22, "record.utf8_bytes == \"abc\"")).expect_err("Integer");
    assert!(refused.contains("type_mismatch"), "{refused}");
}

#[test]
fn utf8_every_other_position_refuses_it() {
    for written in [
        "defined(label.utf8_bytes)",
        "label.utf8_bytes",
        "{label.utf8_bytes: {any_of: [1, 2]}}",
        "{label.utf8_bytes: {none_of: [1]}}",
        "{label.utf8_bytes: {starts_with: \"a\"}}",
        "{label.utf8_bytes: {equals_ignore_case: \"a\"}}",
        "{forall: {in: label.utf8_bytes, as: byte, that: byte == 1}}",
    ] {
        let refused = assemble(&guarded(22, written)).expect_err(written);
        assert!(
            refused.contains("type_mismatch") && refused.contains("comparison operand"),
            "{written}: {refused}"
        );
    }
}

#[test]
fn utf8_only_a_string_is_measured_and_nothing_follows_it() {
    for written in [
        "id.utf8_bytes <= 4",
        "at.utf8_bytes <= 4",
        "blob.utf8_bytes <= 4",
        "kind.utf8_bytes <= 4",
        "tags.utf8_bytes <= 4",
        "limit.utf8_bytes <= 4",
        "tags.count.utf8_bytes <= 4",
        "label.count.utf8_bytes <= 4",
        "label.utf8_bytes.count <= 4",
        "label.utf8_bytes.x <= 4",
        "record.utf8_bytes.utf8_bytes <= 4",
        "{compare: {left: {utf8_bytes: id}, op: lte, right: 4}}",
        "{compare: {left: {utf8_bytes: record}, op: lte, right: 4}}",
        "{limit: {gte: {utf8_bytes: tags}}}",
    ] {
        let refused = assemble(&guarded(22, written)).expect_err(written);
        assert!(
            refused.contains("type_mismatch") || refused.contains("unobservable_fact"),
            "{written}: {refused}"
        );
    }
}

#[test]
fn utf8_it_compares_as_an_integer() {
    for written in [
        "label.utf8_bytes == \"abc\"",
        "label.utf8_bytes == true",
        "label.utf8_bytes < at",
        "{compare: {left: {utf8_bytes: label}, op: eq, right: abc}}",
    ] {
        let refused = assemble(&guarded(22, written)).expect_err(written);
        assert!(refused.contains("type_mismatch"), "{written}: {refused}");
    }
    let spec = admitted(&guarded(22, "label.utf8_bytes > note.utf8_bytes"));
    assert_eq!(
        guard(&spec),
        r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"gt","right":{"utf8_bytes":"note"}}}"#
    );
}

#[test]
fn utf8_below_source22_it_is_refused_by_format() {
    let refused = assemble(&guarded(21, "label.utf8_bytes <= 8")).expect_err("ess/21");
    assert!(
        refused.contains("unsupported_format_version") && refused.contains("ess/22"),
        "{refused}"
    );
    for written in [
        "{compare: {left: {utf8_bytes: label}, op: lte, right: 8}}",
        "{limit: {gte: {utf8_bytes: label}}}",
    ] {
        let refused = assemble(&guarded(21, written)).expect_err(written);
        assert!(refused.starts_with("parse:"), "{written}: {refused}");
    }
    let refused = assemble(&model(
        21,
        "label == \"x\"",
        "label == input.label",
        "label.utf8_bytes <= 255",
        "label != \"\"",
    ))
    .expect_err("an ess/21 invariant");
    assert!(refused.contains("ess/22"), "{refused}");
}

#[test]
fn utf8_a_selector_assembled_directly_is_rechecked_against_the_format() {
    // A predicate built in code — or read from a canonical document — reaches the checker without a
    // source spelling: the format and the parent's type still decide.
    let raw = RawSpecFile::parse(&guarded(21, "label == \"x\"")).expect("parses");
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)]).expect("assembles");
    let command = &spec.commands()[&"notes.core.File".parse().unwrap()];
    let errors = |format: Option<ess_domain::system::FormatVersion>, predicate: &str| {
        let registry = spec.system().types.clone();
        let registry = match format {
            Some(format) => registry.with_format(format),
            None => registry,
        };
        let environment = ess_domain::expression::DomainEnvironment::new(&registry, &command.input);
        let predicate: Predicate = serde_json::from_str(predicate).expect("reads");
        ess_domain::expression::check_predicate(&environment, &predicate, "test")
            .errors
            .iter()
            .map(|error| error.validation_error().to_string())
            .collect::<Vec<_>>()
    };
    let derived = r#"{"compare":{"left":{"utf8_bytes":"label"},"op":"lte","right":8}}"#;
    let old = errors(Some(spec.system().format), derived);
    assert!(old.iter().any(|error| error.contains("ess/22")), "{old:?}");
    assert_eq!(errors(None, derived), Vec::<String>::new());
    let uuid = r#"{"compare":{"left":{"utf8_bytes":"id"},"op":"lte","right":8}}"#;
    let refused = errors(None, uuid);
    assert!(
        refused.iter().any(|error| error.contains("String")),
        "{refused:?}"
    );
    // The path spelling never stands for the selector: a primitive evaluator would read a field.
    let field = r#""label.utf8_bytes <= 8""#;
    let refused = errors(None, field);
    assert!(
        refused
            .iter()
            .any(|error| error.contains("comparison operand")),
        "{refused:?}"
    );
}
