//! What the enum refusal's parenthetical says for every shape that reaches the enum.
//!
//! `ValueType` carries `declaring_variants: Option<String>`, filled from the resolution's
//! `terminal`, and the message reads `` `<enum>` (Text, reached through `<declared>`) `` when the
//! two differ and `` `<enum>` (Text) `` when they do not. Nothing else pins that clause:
//! `tests/enum_variant_refusals.rs` asserts only that the enum is named, and `tests/expression.rs`
//! asserts three `contains` calls, every one of which is satisfied without it. Before this file,
//! `let reached = String::new();` was a surviving mutant — the clause could be deleted outright
//! and the whole suite stayed green.
//!
//! Six shapes, one registry: the enum declared directly, one newtype over it, a newtype over that
//! newtype, the enum behind `Optional`, a list element, and a newtype over a non-enum. The direct
//! case is the one that catches an empty or duplicated clause, because it is the only shape where
//! `terminal` and `declared` are the same type.
//!
//! The second half is what a refused author is told to write instead, for the refusals a
//! generator of three-valued case records hits (beyond10x/ess#426, part b): a YAML boolean
//! written as a variant, a boolean literal over an enum that declares the variant `True`, and a
//! finite coverage proof that declines past its cap or over an open field.

use ess_domain::expression::{check_predicate, DomainEnvironment};
use ess_domain::{Field, NamedType, Naming, TypeBody, TypeRef, TypeRegistry};
use ess_primitives::error::ValidationCode;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

fn field(name: &str, kind: &str) -> Field {
    Field::new(name, TypeRef::parse(kind).unwrap())
}

fn newtype(of: &str) -> TypeBody {
    TypeBody::Newtype {
        alphabet: None,
        prefix: None,
        of: TypeRef::parse(of).unwrap(),
        invariants: vec![],
    }
}

/// An enum, a newtype over it, a newtype over that newtype, and a newtype over a non-enum.
fn registry() -> TypeRegistry {
    let mut registry = TypeRegistry::new();
    for (name, body) in [
        (
            "sample.Channel",
            TypeBody::Enum {
                variants: vec!["Email".into(), "Post".into(), "Portal".into()],
            },
        ),
        ("sample.WrappedChannel", newtype("sample.Channel")),
        ("sample.DoubleWrapped", newtype("sample.WrappedChannel")),
        ("sample.Code", newtype("String")),
    ] {
        registry
            .insert(NamedType {
                reading: None,
                name: name.parse().unwrap(),
                body,
                naming: Naming::default(),
            })
            .unwrap();
    }
    registry
}

fn fields() -> Vec<Field> {
    vec![
        field("channel", "sample.Channel"),
        field("wrapped_channel", "sample.WrappedChannel"),
        field("double_wrapped", "sample.DoubleWrapped"),
        field("optional_channel", "Optional<sample.Channel>"),
        field("channels", "List<sample.Channel>"),
        field("code", "sample.Code"),
    ]
}

/// Every `UndeclaredReference` message the checker produces for `<fact> == Fax`.
fn refusals(fact: &str) -> Vec<String> {
    let registry = registry();
    let fields = fields();
    let environment = DomainEnvironment::new(&registry, &fields);
    let predicate = Predicate::Compare {
        kind: ess_primitives::predicate::CompareKind::Value,
        left: Operand::Fact(FactPath::new(fact).unwrap()),
        op: CompareOp::Eq,
        right: Operand::Literal(FactValue::text("Fax")),
    };
    check_predicate(&environment, &predicate, "owner")
        .errors
        .iter()
        .filter(|error| error.code == ValidationCode::UndeclaredReference)
        .map(|error| error.message.clone())
        .collect()
}

/// The one refusal for `<fact> == Fax`.
fn refusal(fact: &str) -> String {
    let found = refusals(fact);
    assert_eq!(
        found.len(),
        1,
        "`{fact} == Fax` is refused exactly once: {found:?}"
    );
    found[0].clone()
}

/// The parenthetical names the enum once, and the wrapper only when there is one.
///
/// Four shapes, and the question each answers:
///
/// * `channel` — declared *as* the enum. `terminal` and `declared` are the same type, so the
///   parenthetical must not carry an empty or duplicated `reached through` clause.
/// * `wrapped_channel` — one newtype. The enum is named and the wrapper is kept beside it.
/// * `double_wrapped` — a newtype over a newtype. The declared type is the outer wrapper and the
///   terminal is the enum; the middle wrapper is named by neither.
/// * `optional_channel` — the enum behind `Optional`, which `resolve` unwraps transparently.
#[test]
fn the_parenthetical_names_the_enum_once_and_the_wrapper_only_when_there_is_one() {
    let direct = refusal("channel");
    assert!(
        direct.contains("`sample.Channel` (Text)"),
        "declared as the enum: no wrapper to report, so no clause at all: {direct}"
    );
    assert!(
        !direct.contains("reached through"),
        "declared as the enum: `terminal` and `declared` are one type: {direct}"
    );

    let wrapped = refusal("wrapped_channel");
    assert!(
        wrapped.contains("`sample.Channel` (Text, reached through `sample.WrappedChannel`)"),
        "one newtype: the enum, and how the field reached it: {wrapped}"
    );

    let double = refusal("double_wrapped");
    assert!(
        double.contains("`sample.Channel` (Text, reached through `sample.DoubleWrapped`)"),
        "a newtype over a newtype: the enum, and the type the field declares: {double}"
    );

    let optional = refusal("optional_channel");
    assert!(
        optional.contains("`sample.Channel`"),
        "behind `Optional`, which the story's scope names as a shape to preserve: {optional}"
    );
}

/// A newtype over a non-enum carries no variants, so nothing is refused and nothing is named.
#[test]
fn a_newtype_over_a_non_enum_is_not_an_enum_refusal() {
    assert!(
        refusals("code").is_empty(),
        "`sample.Code` is a newtype over `String` and declares no variants: {:?}",
        refusals("code")
    );
}

/// A declared variant is admitted through every wrapper shape, or the refusal above is worthless.
#[test]
fn a_declared_variant_is_admitted_through_every_wrapper() {
    let registry = registry();
    let fields = fields();
    let environment = DomainEnvironment::new(&registry, &fields);
    for fact in [
        "channel",
        "wrapped_channel",
        "double_wrapped",
        "optional_channel",
    ] {
        let predicate = Predicate::Compare {
            kind: ess_primitives::predicate::CompareKind::Value,
            left: Operand::Fact(FactPath::new(fact).unwrap()),
            op: CompareOp::Eq,
            right: Operand::Literal(FactValue::text("Email")),
        };
        let checked = check_predicate(&environment, &predicate, "owner");
        assert!(
            checked.errors.is_empty(),
            "`{fact} == Email` names a declared variant: {}",
            checked.validation_errors()
        );
    }
}

// ---- What the refusal says to write instead (beyond10x/ess#426, part b) ----------------------

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationErrors;

/// A case record whose `Truth` declares its variants quoted, set to `"True"` on creation.
const CASE: &str = "format: ess/22
system: probe
version: v1
domain: probe.case
types:
  - {name: probe.case.CaseId, kind: newtype, of: Uuid}
  - name: probe.case.Truth
    kind: enum
    variants: [\"True\", \"False\", Unknown]
entities:
  - name: probe.case.Case
    identity: {name: case_id, type: probe.case.CaseId}
    fields:
      - {name: tests_pass, type: probe.case.Truth}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: probe.case.Opened
    fields:
      - {name: case_id, type: probe.case.CaseId}
commands:
  - name: probe.case.Open
    input: []
    outcomes:
      - name: opened
        creates: probe.case.Case
        instance: case_id
        emits: [probe.case.Opened]
        payload:
          probe.case.Opened: {case_id: {generated: true}}
        sets: {tests_pass: \"True\"}
";

/// A case record of four three-valued claims, decided by a guarded success and its guarded
/// negation, with no default branch: 3 × 3 × 3 × 3 = 81 joint assignments.
const FOUR_CLAIMS: &str = "format: ess/22
system: probe
version: v1
domain: probe.claims
types:
  - {name: probe.claims.CaseId, kind: newtype, of: Uuid}
  - name: probe.claims.Truth
    kind: enum
    variants: [Holds, Fails, Unknown]
entities:
  - name: probe.claims.Case
    identity: {name: case_id, type: probe.claims.CaseId}
    fields:
      - {name: a, type: probe.claims.Truth}
      - {name: b, type: probe.claims.Truth}
      - {name: c, type: probe.claims.Truth}
      - {name: d, type: probe.claims.Truth}
    lifecycle:
      initial: Open
      states: [Open, Accepted]
      terminal: [Accepted]
      transitions:
        - {name: accept, from: [Open], to: Accepted}
errors:
  - {name: probe.claims.NotReady, summary: A claim does not hold., fields: []}
events:
  - name: probe.claims.Opened
    fields:
      - {name: case_id, type: probe.claims.CaseId}
  - name: probe.claims.Accepted
    fields: []
commands:
  - name: probe.claims.Open
    input: []
    outcomes:
      - name: opened
        creates: probe.claims.Case
        instance: case_id
        emits: [probe.claims.Opened]
        payload:
          probe.claims.Opened: {case_id: {generated: true}}
        sets: {a: Unknown, b: Unknown, c: Unknown, d: Unknown}
  - name: probe.claims.Accept
    input:
      - {name: case_id, type: probe.claims.CaseId}
    outcomes:
      - name: not-ready
        when_subject: {predicate: {not: {all: [a == Holds, b == Holds, c == Holds, d == Holds]}}}
        error: probe.claims.NotReady
      - name: accepted
        when_subject: {predicate: {all: [a == Holds, b == Holds, c == Holds, d == Holds]}}
        moves: probe.claims.Case.accept
        instance: case_id
        emits: [probe.claims.Accepted]
";

/// The parcels of `docs/design/cross-record-and-stored-field-guards.md`.
const PARCELS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/stored-field-guards.yaml");

fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the fixture writes {from:?}");
    text.replacen(from, to, 1)
}

/// What assembling `text` refuses. The reader must accept it: every refusal here is one a
/// declaration's own pass makes, at a path.
fn refused_document(text: &str) -> ValidationErrors {
    let raw = RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the reader accepts the document: {error}\n{text}"));
    Specification::assemble([(Source::new("case.yaml"), raw)])
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
}

fn the_one(errors: &ValidationErrors, code: ValidationCode, location: &str) -> String {
    let found: Vec<_> = errors
        .as_slice()
        .iter()
        .filter(|error| error.code == code && error.location == location)
        .collect();
    assert_eq!(found.len(), 1, "one {code:?} at `{location}`:\n{errors}");
    found[0].to_string()
}

#[test]
fn the_case_record_validates_as_written() {
    let raw = RawSpecFile::parse(CASE).expect("parses");
    Specification::assemble([(Source::new("case.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn boolean_enum_variant_is_refused_with_quote_repair() {
    let errors = refused_document(&replaced(
        CASE,
        "variants: [\"True\", \"False\", Unknown]",
        "variants: [True, False, Unknown]",
    ));
    let refusal = the_one(
        &errors,
        ValidationCode::TypeMismatch,
        "types.probe.case.Truth.variants",
    );
    assert!(refusal.contains("(hint: quote it"), "{refusal}");
    assert!(refusal.contains("boolean"), "{refusal}");
}

#[test]
fn boolean_literal_matching_a_variant_names_the_quoted_variant() {
    let errors = refused_document(&replaced(
        CASE,
        "sets: {tests_pass: \"True\"}",
        "sets: {tests_pass: True}",
    ));
    let refusal = errors
        .as_slice()
        .iter()
        .find(|error| error.code == ValidationCode::TypeMismatch)
        .unwrap_or_else(|| panic!("a type mismatch:\n{errors}"));
    assert!(
        refusal
            .location
            .ends_with("outcomes.opened.sets.tests_pass"),
        "{refusal}"
    );
    assert_eq!(
        refusal.hint.as_deref(),
        Some("quote it: `tests_pass: 'True'`"),
        "{refusal}"
    );
}

#[test]
fn boolean_literal_matching_no_variant_keeps_variant_list() {
    let text = replaced(
        &replaced(
            CASE,
            "variants: [\"True\", \"False\", Unknown]",
            "variants: [Holds, Fails, Unknown]",
        ),
        "sets: {tests_pass: \"True\"}",
        "sets: {tests_pass: true}",
    );
    let errors = refused_document(&text);
    let refusal = errors
        .as_slice()
        .iter()
        .find(|error| error.code == ValidationCode::TypeMismatch)
        .unwrap_or_else(|| panic!("a type mismatch:\n{errors}"));
    assert_eq!(
        refusal.hint.as_deref(),
        Some("variants: Holds, Fails, Unknown"),
        "{refusal}"
    );
}

#[test]
fn finite_cap_decline_names_the_count() {
    let errors = refused_document(FOUR_CLAIMS);
    let refusal = the_one(
        &errors,
        ValidationCode::NonExhaustiveBranches,
        "command.probe.claims.Accept.outcomes",
    );
    assert!(
        refusal.contains("81 joint assignments exceed 64"),
        "{refusal}"
    );
}

#[test]
fn finite_open_domain_decline_names_the_field() {
    // Both branches guarded, so no default answers what the proof cannot.
    let text = replaced(
        PARCELS,
        "      - name: dispatched\n",
        "      - name: dispatched\n        when_subject: {predicate: weight_kg <= 20}\n",
    );
    let errors = refused_document(&text);
    let refusal = the_one(
        &errors,
        ValidationCode::NonExhaustiveBranches,
        "command.shipping.parcel.Dispatch.outcomes",
    );
    assert!(refusal.contains("`weight_kg`"), "{refusal}");
    assert!(!refusal.contains("exceed"), "not the cap: {refusal}");
}

#[test]
fn typed_literals_table_matches_behaviour() {
    let page = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../docs/design/typed-literals-and-unknown-instances.md"),
    )
    .expect("the typed-literals design page");
    let row = |written: &str| {
        page.lines()
            .find(|line| line.starts_with(&format!("| `{written}` |")))
            .unwrap_or_else(|| panic!("the table has a `{written}` row"))
            .rsplit('|')
            .nth(1)
            .expect("the last column")
            .trim()
            .to_owned()
    };
    // The enum column states the conditional rule: quote it where the quoted form is admitted, or
    // where an enum declares the boolean's variant in another case; otherwise the quoted form's
    // own refusal.
    let boolean = row("false");
    for phrase in [
        "**quote it**",
        "where `'false'` is admitted",
        "another case",
        "otherwise as `'false'` is",
    ] {
        assert!(
            boolean.contains(phrase),
            "`false` row lacks {phrase:?}: {boolean}"
        );
    }
    for (written, quoted) in [("0", "'0'"), ("1.5", "'1.5'")] {
        let cell = row(written);
        for phrase in [
            "**quote it**".to_owned(),
            format!("where `{quoted}` is admitted"),
            format!("otherwise as `{quoted}` is"),
        ] {
            assert!(
                cell.contains(&phrase),
                "`{written}` row lacks {phrase:?}: {cell}"
            );
        }
    }
}
