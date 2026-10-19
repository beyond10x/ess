//! `shorthand: <field>` on a `kind: struct` type (`ess/27`, beyond10x/ess#594): a value of the
//! struct may be written as that field's scalar alone. Validation admits it only on a struct whose
//! named field is required and scalar on the wire and whose every other field is `Optional`, and
//! refuses each other use by name at `<type>.shorthand`. The IR names the shorthands beside the
//! types, and a model that does not write the key keeps its bytes.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationError};

/// A tracker whose `Run` command takes an `Operation`. `{format}` is the header's major,
/// `{shorthand}` the line written on `Operation`, `{fields}` its fields after `op`, `{op}` the
/// type of `op`, and `{elsewhere}` a key written on the event.
const MODEL: &str = "\
format: ess/{format}
system: tracker
version: v1
domain: tracker.ops
types:
  - name: tracker.ops.Operation
    kind: struct
{shorthand}    fields:
      - name: op
        type: {op}
      - name: paging
        type: Optional<String>
{fields}  - name: tracker.ops.OpName
    kind: newtype
    of: tracker.ops.Verb
  - name: tracker.ops.Verb
    kind: enum
    variants: [getIssue, listIssues]
  - name: tracker.ops.Page
    kind: struct
    fields:
      - name: token
        type: String
  - name: tracker.ops.Label
    kind: newtype
    of: String
  - name: tracker.ops.Choice
    kind: enum
    variants: [one, two]
  - name: tracker.ops.Either
    kind: union
    tag: kind
    variants:
      a: String
      b: Integer
actors:
  - name: tracker.ops.Operator
    may:
      - tracker.ops.Run
commands:
  - name: tracker.ops.Run
    input:
      - name: operation
        type: tracker.ops.Operation
      - name: label
        type: Optional<tracker.ops.Label>
      - name: choice
        type: Optional<tracker.ops.Choice>
      - name: either
        type: Optional<tracker.ops.Either>
      - name: page
        type: Optional<tracker.ops.Page>
    outcomes:
      - name: ran
        emits: [tracker.ops.Ran]
        payload:
          tracker.ops.Ran:
            op: input.operation.op
events:
  - name: tracker.ops.Ran
{elsewhere}    fields:
      - name: op
        type: {op}
";

/// The model with `shorthand: <value>` on `Operation` (none when `None`), `op` of type `op`, and
/// `fields` (each line indented as a field list entry) after `paging`.
fn text(format: u32, shorthand: Option<&str>, op: &str, fields: &str) -> String {
    MODEL
        .replace("{format}", &format.to_string())
        .replace(
            "{shorthand}",
            &shorthand.map_or_else(String::new, |value| format!("    shorthand: {value}\n")),
        )
        .replace("{op}", op)
        .replace("{fields}", fields)
        .replace("{elsewhere}", "")
}

/// The specification, or the refusals of reading, assembling or compiling it.
fn outcome(text: &str) -> Result<EssIr, Vec<ValidationError>> {
    let raw = RawSpecFile::parse(text).map_err(|error| {
        vec![ValidationError::new(
            ValidationCode::UnsupportedConstruct,
            "parse",
            error.to_string(),
        )]
    })?;
    let spec = Specification::assemble([(Source::new("tracker.yaml"), raw)])
        .map_err(|errors| errors.as_slice().to_vec())?;
    // Every refusal of the key is a validation refusal; a model that validates compiles.
    Ok(compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}")))
}

fn ir(text: &str) -> EssIr {
    outcome(text).unwrap_or_else(|errors| panic!("{errors:#?}"))
}

/// The refusals of `text`, which must not validate.
fn refusals(text: &str) -> Vec<ValidationError> {
    match outcome(text) {
        Ok(_) => panic!("the model validates, and should not:\n{text}"),
        Err(errors) => errors,
    }
}

/// The one refusal at `<construct> <name>.shorthand`, with its code.
fn refused_at(text: &str, name: &str, code: ValidationCode) -> ValidationError {
    let errors = refusals(text);
    let located: Vec<&ValidationError> = errors
        .iter()
        .filter(|error| error.location.contains(name) && error.location.ends_with(".shorthand"))
        .collect();
    assert_eq!(
        located.len(),
        1,
        "one refusal at {name}.shorthand, among {errors:#?}"
    );
    assert_eq!(located[0].code, code, "{errors:#?}");
    located[0].clone()
}

fn name(value: &str) -> QualifiedName {
    value.parse().expect("a qualified name")
}

const OPERATION: &str = "tracker.ops.Operation";

#[test]
fn shorthand_struct_validates_at_ess_27() {
    let compiled = ir(&text(27, Some("op"), "String", ""));
    assert_eq!(compiled.shorthand(&name(OPERATION)), Some("op"));
    assert_eq!(compiled.shorthand(&name("tracker.ops.Page")), None);
    let document: serde_json::Value =
        serde_json::from_str(&compiled.to_canonical_json()).expect("the IR is JSON");
    assert_eq!(
        document["shorthands"],
        serde_json::json!({ OPERATION: "op" }),
        "the IR names the shorthand beside the types"
    );
    // An enum, and a newtype over an enum, are scalars on the wire too.
    for op in ["tracker.ops.Verb", "tracker.ops.OpName"] {
        assert_eq!(
            ir(&text(27, Some("op"), op, "")).shorthand(&name(OPERATION)),
            Some("op"),
            "{op}"
        );
    }
}

#[test]
fn shorthand_below_ess_27_is_refused() {
    let refused = refused_at(
        &text(26, Some("op"), "String", ""),
        OPERATION,
        ValidationCode::UnsupportedFormatVersion,
    );
    assert!(refused.message.contains("ess/27"), "{refused:#?}");
}

#[test]
fn shorthand_on_a_non_struct_is_refused() {
    for (kind, declared) in [
        ("newtype", "tracker.ops.Label"),
        ("enum", "tracker.ops.Choice"),
        ("union", "tracker.ops.Either"),
    ] {
        let source = text(27, None, "String", "").replace(
            &format!("  - name: {declared}\n"),
            &format!("  - name: {declared}\n    shorthand: op\n"),
        );
        let refused = refused_at(&source, declared, ValidationCode::UnsupportedConstruct);
        assert!(refused.message.contains(kind), "{kind}: {refused:#?}");
    }
}

#[test]
fn shorthand_outside_types_is_refused() {
    let source = MODEL
        .replace("{format}", "27")
        .replace("{shorthand}", "")
        .replace("{op}", "String")
        .replace("{fields}", "")
        .replace("{elsewhere}", "    shorthand: op\n");
    let refused = refused_at(
        &source,
        "tracker.ops.Ran",
        ValidationCode::UnsupportedConstruct,
    );
    assert!(refused.message.contains("event"), "{refused:#?}");
}

/// A command is a declaration that is not a type too: the key is refused at
/// `<command>.shorthand` as `unsupported_construct`, not as a field the command does not know
/// (review 2026-10-19, B6).
#[test]
fn shorthand_on_a_command_is_refused_at_the_command() {
    let source = text(27, None, "String", "").replace(
        "  - name: tracker.ops.Run\n    input:\n",
        "  - name: tracker.ops.Run\n    shorthand: operation\n    input:\n",
    );
    let refused = refused_at(
        &source,
        "tracker.ops.Run",
        ValidationCode::UnsupportedConstruct,
    );
    assert!(refused.message.contains("command"), "{refused:#?}");
}

#[test]
fn shorthand_naming_no_field_is_refused() {
    let refused = refused_at(
        &text(27, Some("operation"), "String", ""),
        OPERATION,
        ValidationCode::UndeclaredReference,
    );
    assert!(refused.message.contains("`operation`"), "{refused:#?}");
}

#[test]
fn shorthand_on_a_non_scalar_field_is_refused() {
    for op in [
        "tracker.ops.Page",
        "List<String>",
        "Optional<String>",
        "Map<String, String>",
        "Json",
    ] {
        let refused = refused_at(
            &text(27, Some("op"), op, ""),
            OPERATION,
            ValidationCode::TypeMismatch,
        );
        assert!(refused.message.contains("`op`"), "{op}: {refused:#?}");
    }
}

#[test]
fn shorthand_beside_a_required_field_is_refused() {
    let refused = refused_at(
        &text(
            27,
            Some("op"),
            "String",
            "      - name: project\n        type: String\n",
        ),
        OPERATION,
        ValidationCode::ConflictingDeclaration,
    );
    assert!(refused.message.contains("`project`"), "{refused:#?}");
}

#[test]
fn shorthand_naming_two_fields_is_refused() {
    for written in ["[op, paging]", "{field: op}"] {
        let errors = refusals(&text(27, Some(written), "String", ""));
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("shorthand")),
            "{written}: {errors:#?}"
        );
    }
}

#[test]
fn ir_without_shorthand_keeps_its_bytes() {
    let legacy = ir(&text(26, None, "String", ""));
    let current = ir(&text(27, None, "String", ""));
    assert_eq!(legacy.to_canonical_json(), current.to_canonical_json());
    assert_eq!(legacy.source_digest(), current.source_digest());
    let document: serde_json::Value =
        serde_json::from_str(&current.to_canonical_json()).expect("the IR is JSON");
    assert!(document.get("shorthands").is_none());
    assert_ne!(
        current.source_digest(),
        ir(&text(27, Some("op"), "String", "")).source_digest(),
        "a shorthand is part of what was compiled"
    );
}
