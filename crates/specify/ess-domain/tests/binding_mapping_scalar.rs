//! A binding `mapping:` value may be a `Boolean`, `Integer` or `Decimal` constant
//! (beyond10x/ess#445), checked by the one rule `sets:` and `payload:` already type a literal by.
//!
//! An unquoted YAML scalar and the quoted text of one are both read; whether either fills the
//! target is decided by the target, as it is for `sets:`. An unquoted scalar over text or an enum is
//! refused with the quoted spelling as its repair.

use std::path::PathBuf;

use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationError};

fn assemble(document: &str) -> Result<Specification, Vec<ValidationError>> {
    let raw = RawSpecFile::parse(document).map_err(|error| {
        vec![ValidationError::new(
            ValidationCode::UnsupportedConstruct,
            "parse",
            error.to_string(),
        )]
    })?;
    Specification::assemble([(Source::new("calls.yaml"), raw)])
        .map_err(|errors| errors.as_slice().to_vec())
}

/// One command taking `leg_id` and `slot: <ty>`, and one binding filling `slot` with `value`.
fn binding(ty: &str, value: &str) -> String {
    format!(
        "format: ess/22
system: demo
version: v1
domain: demo.calls
types:
  - {{name: demo.calls.Weight, kind: newtype, of: Integer}}
  - {{name: demo.calls.Kind, kind: enum, variants: [inbound, outbound]}}
events:
  - name: demo.calls.LegJoined
    fields:
      - {{name: leg_id, type: String}}
  - name: demo.calls.LegRecorded
    fields:
      - {{name: leg_id, type: String}}
commands:
  - name: demo.calls.RecordLeg
    input:
      - {{name: leg_id, type: String}}
      - {{name: slot, type: {ty}}}
    outcomes:
      - name: recorded
        emits: [demo.calls.LegRecorded]
        payload:
          demo.calls.LegRecorded: {{leg_id: input.leg_id}}
bindings:
  - id: joined
    when: {{event: demo.calls.LegJoined}}
    invoke: {{command: demo.calls.RecordLeg}}
    mapping:
      leg_id: event.leg_id
      slot: {value}
    delivery: at_least_once
    on_failure: retry
"
    )
}

/// One entity holding `slot: <ty>`, and one command whose branch `sets:` it to `value`.
fn sets(ty: &str, value: &str) -> String {
    format!(
        "format: ess/22
system: demo
version: v1
domain: demo.calls
types:
  - {{name: demo.calls.LegId, kind: newtype, of: Uuid}}
  - {{name: demo.calls.Weight, kind: newtype, of: Integer}}
  - {{name: demo.calls.Kind, kind: enum, variants: [inbound, outbound]}}
entities:
  - name: demo.calls.Leg
    identity: {{name: leg_id, type: demo.calls.LegId}}
    fields:
      - {{name: slot, type: {ty}}}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {{name: close, from: [Open], to: Closed}}
events:
  - name: demo.calls.LegClosed
    fields:
      - {{name: leg_id, type: demo.calls.LegId}}
commands:
  - name: demo.calls.CloseLeg
    input:
      - {{name: leg_id, type: demo.calls.LegId}}
    outcomes:
      - name: closed
        moves: demo.calls.Leg.close
        instance: leg_id
        sets:
          slot: {value}
        emits: [demo.calls.LegClosed]
        payload:
          demo.calls.LegClosed: {{leg_id: input.leg_id}}
"
    )
}

/// The one refusal at `binding.joined.mapping.slot`, or every refusal when there is not one.
fn refusal_at_slot(errors: &[ValidationError]) -> &ValidationError {
    let at: Vec<&ValidationError> = errors
        .iter()
        .filter(|error| error.location.ends_with("mapping.slot"))
        .collect();
    assert_eq!(at.len(), 1, "one refusal at the mapping entry: {errors:#?}");
    at[0]
}

/// The one `type_mismatch` a `sets:` entry is refused with.
fn sets_refusal(errors: &[ValidationError]) -> &ValidationError {
    let found: Vec<&ValidationError> = errors
        .iter()
        .filter(|error| error.code == ValidationCode::TypeMismatch)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "one type_mismatch on the sets: entry: {errors:#?}"
    );
    found[0]
}

#[test]
fn binding_mapping_quoted_scalar_matches_sets_rule() {
    let types = [
        "Boolean",
        "Integer",
        "Decimal",
        "demo.calls.Weight",
        "Optional<Boolean>",
    ];
    let values = [
        "'true'", "'false'", "'3'", "'-3'", "'0.5'", "'03'", "'+3'", "'1e3'", "'yes'", "true",
        "false", "3", "-3", "0.5", "3.0",
    ];
    // The comparison means something only if each model is otherwise sound.
    if let Err(errors) = assemble(&sets("Boolean", "'true'")) {
        panic!("the sets: model is sound: {errors:#?}");
    }
    if let Err(errors) = assemble(&binding("String", "'text'")) {
        panic!("the binding model is sound: {errors:#?}");
    }
    for ty in types {
        for value in values {
            let by_sets = assemble(&sets(ty, value)).is_ok();
            let by_mapping = assemble(&binding(ty, value));
            assert_eq!(
                by_mapping.is_ok(),
                by_sets,
                "`slot: {value}` over `{ty}`: sets: admits it {by_sets}, mapping: {by_mapping:#?}"
            );
            if let Err(errors) = &by_mapping {
                let mapping = refusal_at_slot(errors);
                assert_eq!(mapping.code, ValidationCode::TypeMismatch, "{mapping:#?}");
            }
        }
    }
    // The three the issue asks for are among the admitted ones.
    for (ty, value) in [
        ("Boolean", "'true'"),
        ("Integer", "'3'"),
        ("Decimal", "'0.5'"),
    ] {
        assert!(
            assemble(&binding(ty, value)).is_ok(),
            "`slot: {value}` fills `{ty}`"
        );
    }
}

#[test]
fn binding_mapping_scalar_over_text_says_quote_it() {
    let errors = assemble(&binding("String", "3")).expect_err("`3` over a String is refused");
    let refusal = refusal_at_slot(&errors);
    assert_eq!(refusal.code, ValidationCode::TypeMismatch, "{refusal:#?}");
    assert_eq!(
        refusal.hint.as_deref(),
        Some("quote it: `slot: '3'`"),
        "{refusal:#?}"
    );
    // The same over an enum, and the quoted text is admitted where it names nothing else.
    let errors = assemble(&binding("demo.calls.Kind", "true")).expect_err("over an enum");
    let refusal = refusal_at_slot(&errors);
    assert_eq!(refusal.code, ValidationCode::TypeMismatch, "{refusal:#?}");
    assert!(
        refusal
            .hint
            .as_deref()
            .is_some_and(|hint| hint.starts_with("variants:") || hint.starts_with("quote it")),
        "{refusal:#?}"
    );
    assert!(assemble(&binding("String", "'3'")).is_ok());
}

#[test]
fn binding_mapping_scalar_type_mismatch_named() {
    for (ty, value) in [("Integer", "true"), ("Boolean", "1"), ("Integer", "0.5")] {
        let mapping = assemble(&binding(ty, value))
            .expect_err("the mapping is refused as the sets: entry is");
        let set = assemble(&sets(ty, value)).expect_err("the sets: entry is refused");
        let mapping = refusal_at_slot(&mapping);
        let set = sets_refusal(&set);
        assert_eq!(mapping.code, ValidationCode::TypeMismatch, "{mapping:#?}");
        assert_eq!(
            mapping.hint, set.hint,
            "`slot: {value}` over `{ty}` carries the sets: hint"
        );
        assert!(
            mapping.message.contains(&format!("`{value}`")) && mapping.message.contains(ty),
            "the refusal names the value and the type: {mapping:#?}"
        );
    }
}

#[test]
fn binding_mapping_scalar_round_trips_typed() {
    // The authored mapping table writes each constant back as the scalar it was read as.
    let table: ess_domain::binding::MappingTable =
        serde_yaml::from_str("{flag: true, weight: 3, share: 0.5, text: '3'}").expect("parses");
    assert_eq!(
        serde_json::to_string(&table).expect("serialises"),
        r#"{"flag":true,"weight":3,"share":0.5,"text":"3"}"#
    );
    assert!(
        assemble(&binding("Boolean", "true")).is_ok(),
        "`true` fills a Boolean"
    );
}

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(repository().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

/// The text from `heading` to the next heading of the same or a higher level.
fn section<'a>(page: &'a str, heading: &str) -> Option<&'a str> {
    let start = page.find(&format!("\n{heading}\n"))? + 1;
    let level = heading.chars().take_while(|c| *c == '#').count();
    let body = &page[start + heading.len()..];
    let end = body
        .match_indices("\n#")
        .find(|(at, _)| {
            let hashes = body[at + 1..].chars().take_while(|c| *c == '#').count();
            hashes <= level && body[at + 1 + hashes..].starts_with(' ')
        })
        .map_or(body.len(), |(at, _)| at);
    Some(&page[start..start + heading.len() + end])
}

#[test]
fn binding_literal_docs_state_the_typed_rule() {
    let module = read("crates/specify/ess-domain/src/binding.rs");
    let table: Vec<&str> = module
        .lines()
        .take_while(|line| line.starts_with("//!"))
        .filter(|line| line.starts_with("//! |"))
        .collect();
    let anything = table
        .iter()
        .find(|row| row.starts_with("//! | anything else"))
        .unwrap_or_else(|| panic!("the module table has no `anything else` row: {table:#?}"));
    assert!(
        !anything.contains("Integer"),
        "the `anything else` row still names `Integer`: {anything}"
    );
    assert!(
        table.iter().any(|row| row.contains("`Boolean`")
            && row.contains("`Integer`")
            && row.contains("`Decimal`")
            && row.contains("as `sets:` does")),
        "the module table has no row admitting a Boolean, Integer or Decimal literal \"as `sets:` \
         does\": {table:#?}"
    );

    let design = read("docs/design/typed-literals-and-unknown-instances.md");
    let positions = section(&design, "### Positions")
        .unwrap_or_else(|| panic!("the design page has no `### Positions` table"));
    assert!(
        positions
            .lines()
            .any(|line| line.starts_with('|') && line.contains("`mapping:`")),
        "the `### Positions` table has no `mapping:` row:\n{positions}"
    );

    let guide = read("website/docs/guides/specify/bindings-and-components.md");
    let heading = "## Fill a command input with a constant";
    let constant = section(&guide, heading)
        .unwrap_or_else(|| panic!("the binding guide has no heading `{heading}`"));
    for phrase in ["`is_bridged: true`", "as `sets:`", "quote it"] {
        assert!(
            constant.contains(phrase),
            "the section `{heading}` does not contain {phrase:?}:\n{constant}"
        );
    }
    let failure = guide
        .find("\n## A binding says what happens when it fails\n")
        .expect("the guide keeps its failure section");
    assert!(
        guide.find(&format!("\n{heading}\n")).unwrap() > failure,
        "`{heading}` comes after `## A binding says what happens when it fails`"
    );
}
