//! Adversary pass 3 on `story:an-inexact-authored-decimal-is-refused`, against fix2 (`7298c996f1`).
//!
//! fix2's coverage table says every `sets:`, `payload:` and predicate position of the command
//! schema is now covered, and that bindings hold no such literal. Each case writes an inexact
//! literal in a position that table covers or rules out, in a form no earlier test writes.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const INEXACT: &str = "0.1000000000000000000001";

fn assemble(file: &str, text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new(file), raw)])
}

fn validates(file: &str, text: &str) {
    if let Err(errors) = assemble(file, text) {
        panic!("control: must validate:\n{errors}\n{text}");
    }
}

/// The `inexact_decimal_literal` refusals, rendered; empty where the document validated.
fn inexact(file: &str, text: &str) -> Vec<String> {
    match assemble(file, text) {
        Ok(_) => Vec::new(),
        Err(errors) => errors
            .as_slice()
            .iter()
            .filter(|error| error.code == ValidationCode::InexactDecimalLiteral)
            .map(ToString::to_string)
            .collect(),
    }
}

/// The unit's own model: `demo.pay.Open` creates an `Account` with `sets: {amount: <sets>}`.
fn pay(sets: &str) -> String {
    format!(
        "format: ess/35
system: demo
version: v1
domain: demo.pay
types:
  - {{name: demo.pay.AccountId, kind: newtype, of: Uuid}}
entities:
  - name: demo.pay.Account
    identity: {{name: account_id, type: demo.pay.AccountId}}
    fields:
      - {{name: amount, type: Decimal}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
events:
  - name: demo.pay.Opened
    fields: [{{name: account_id, type: demo.pay.AccountId}}]
commands:
  - name: demo.pay.Open
    input:
      - {{name: amount, type: Decimal}}
    outcomes:
      - name: opened
        creates: demo.pay.Account
        instance: account_id
        sets: {{amount: {sets}}}
        emits: [demo.pay.Opened]
        payload: {{demo.pay.Opened: {{account_id: {{generated: true}}}}}}
"
    )
}

/// fix2's own model: `RunSource` updates a `Source`, and `entry` is its one `affects:` entry.
fn feed(entry: &str) -> String {
    format!(
        "format: ess/35
system: demo
version: v1
domain: demo.feed
types:
  - {{name: demo.feed.SourceId, kind: newtype, of: String}}
  - {{name: demo.feed.DocumentId, kind: newtype, of: String}}
entities:
  - name: demo.feed.Source
    identity: {{name: source_id, type: demo.feed.SourceId}}
    fields: [{{name: label, type: String}}]
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
  - name: demo.feed.SeenDocument
    identity: {{name: document_id, type: demo.feed.DocumentId}}
    fields:
      - {{name: source_id, type: demo.feed.SourceId}}
      - {{name: weight, type: Decimal}}
    lifecycle: {{initial: Seen, states: [Seen], terminal: [Seen]}}
events:
  - name: demo.feed.SourceRan
    fields: [{{name: source_id, type: demo.feed.SourceId}}]
errors: [{{name: demo.feed.NoSuchSource}}]
actors:
  - name: demo.feed.Operator
    may: [demo.feed.RunSource]
commands:
  - name: demo.feed.RunSource
    input:
      - {{name: source_id, type: demo.feed.SourceId}}
      - {{name: label, type: String}}
    outcomes:
      - name: ran
        updates: demo.feed.Source
        instance: source_id
        emits: [demo.feed.SourceRan]
        payload: {{demo.feed.SourceRan: {{source_id: input.source_id}}}}
        sets: {{label: input.label}}
        affects:
{entry}      - {{name: no-such-source, unknown_instance: true, error: demo.feed.NoSuchSource}}
views:
  - name: demo.feed.Sources
    source: demo.feed.Source
    consistency: read_your_writes
    fields:
      - {{name: source_id, type: demo.feed.SourceId}}
      - {{name: label, type: String}}
"
    )
}

/// The `binding_mapping_scalar` model: binding `joined` fills `slot: Decimal` with `value`.
fn binding(value: &str) -> String {
    format!(
        "format: ess/22
system: demo
version: v1
domain: demo.calls
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
      - {{name: slot, type: Decimal}}
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

/// Probe: fix2 tests a filtered read inside an `affects:` `sets:` only with a mapping-form
/// `where:`. Written compact, its literal is read by the lexical parser instead of the YAML scan,
/// and must still be refused once, at the field it sets.
#[test]
fn a_compact_filtered_read_inside_an_affects_sets_is_refused_once_at_its_field() {
    let entry = |literal: &str| {
        format!(
            "          - entity: demo.feed.SeenDocument
            where: weight < 1
            sets: {{weight: {{related: {{entity: demo.feed.SeenDocument, where: \"weight < \
             {literal}\", field: weight}}}}}}
"
        )
    };
    validates("feed.yaml", &feed(&entry("0.5")));
    let found = inexact("feed.yaml", &feed(&entry(INEXACT)));
    assert_eq!(found.len(), 1, "{}", found.join("\n"));
    assert!(
        found[0].starts_with(
            "[inexact_decimal_literal] command.demo.feed.RunSource.outcomes.ran.affects[0].sets.weight:"
        ),
        "{}",
        found[0]
    );
}

/// BLOCKER candidate: `{expr: "<expression>"}` (ess/27) is a `sets:` value, and its decimal
/// literal is read by `Number::parse_decimal` as every guard literal is, but outside any
/// collecting window, so the coverage table's "every `sets:` position" does not hold for it.
#[test]
#[ignore = "story:an-inexact-decimal-in-an-expression-or-binding-mapping-is-refused"]
fn an_expression_value_in_sets_with_an_inexact_literal_is_refused() {
    let expr = |literal: &str| format!("{{expr: \"input.amount + {literal}\"}}");
    validates("pay.yaml", &pay(&expr("0.5")));
    let found = inexact("pay.yaml", &pay(&expr(INEXACT)));
    let whole = match assemble("pay.yaml", &pay(&expr(INEXACT))) {
        Ok(_) => "validated with no diagnostic".to_owned(),
        Err(errors) => errors.to_string(),
    };
    assert_eq!(found.len(), 1, "{whole}");
    assert!(
        found[0]
            .starts_with("[inexact_decimal_literal] command.demo.pay.Open.outcomes.opened.sets"),
        "{}",
        found[0]
    );
}

/// BLOCKER candidate: the coverage table says a binding holds no decimal literal. A binding
/// `mapping:` value is a `Decimal` constant since beyond10x/ess#445, typed "by the one rule `sets:`
/// and `payload:` already type a literal by", and that rule now refuses an inexact one.
#[test]
#[ignore = "story:an-inexact-decimal-in-an-expression-or-binding-mapping-is-refused"]
fn a_binding_mapping_decimal_binary64_cannot_hold_is_refused() {
    validates("calls.yaml", &binding("0.5"));
    let found = inexact("calls.yaml", &binding(INEXACT));
    let whole = match assemble("calls.yaml", &binding(INEXACT)) {
        Ok(_) => "validated with no diagnostic".to_owned(),
        Err(errors) => errors.to_string(),
    };
    assert_eq!(found.len(), 1, "{whole}");
    assert!(found[0].contains(INEXACT), "{}", found[0]);
}
