//! Adversary pass 2 on `story:an-inexact-authored-decimal-is-refused`.
//!
//! The changelog claims an inexact `Decimal` literal is refused "at the `sets:` or `payload:` value
//! or the guard (`when`, `when_subject`, and the other predicate positions) that wrote it". The
//! first two cases write one in a `sets:` and a predicate position of an `affects:` entry, which
//! neither the predicate sites of `Written::from_document` nor the `sets:` scan of
//! `inexact_decimal::authored` (`commands[i].outcomes[j].sets|payload` only) reach. The rest probe
//! path claiming and the pre-filter.

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

/// A command that updates a `Source` and, through `affects:`, the `SeenDocument` rows: `entry` is
/// the one `affects:` entry, indented under the list.
fn feed(entry: &str) -> String {
    format!(
        "format: ess/35
system: demo
version: v1
domain: demo.feed
types:
  - {{name: demo.feed.SourceId, kind: newtype, of: String}}
  - {{name: demo.feed.DocumentId, kind: newtype, of: String}}
  - name: demo.feed.AppliedDocument
    kind: struct
    fields:
      - {{name: document_id, type: demo.feed.DocumentId}}
      - {{name: weight, type: Decimal}}
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
  - name: demo.feed.SourceAdded
    fields: [{{name: source_id, type: demo.feed.SourceId}}]
  - name: demo.feed.SourceRan
    fields: [{{name: source_id, type: demo.feed.SourceId}}]
errors: [{{name: demo.feed.NoSuchSource}}]
actors:
  - name: demo.feed.Operator
    may: [demo.feed.AddSource, demo.feed.RunSource]
commands:
  - name: demo.feed.AddSource
    input: [{{name: label, type: String}}]
    outcomes:
      - name: added
        creates: demo.feed.Source
        instance: source_id
        emits: [demo.feed.SourceAdded]
        payload: {{demo.feed.SourceAdded: {{source_id: {{generated: true}}}}}}
        sets: {{label: input.label}}
  - name: demo.feed.RunSource
    input:
      - {{name: source_id, type: demo.feed.SourceId}}
      - {{name: label, type: String}}
      - {{name: applied, type: List<demo.feed.AppliedDocument>}}
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
  - name: demo.feed.SeenDocuments
    source: demo.feed.SeenDocument
    consistency: read_your_writes
    fields:
      - {{name: document_id, type: demo.feed.DocumentId}}
      - {{name: source_id, type: demo.feed.SourceId}}
      - {{name: weight, type: Decimal}}
"
    )
}

/// An `affects:` entry selecting rows by `where:`, its `sets:` writing `weight` as given.
fn where_entry(filter: &str, weight: &str) -> String {
    format!(
        "          - entity: demo.feed.SeenDocument
            where: {filter}
            sets: {{weight: {weight}}}
"
    )
}

/// An `affects:` entry writing one row per element of `input.applied` that `filter` selects.
fn each_entry(filter: &str) -> String {
    format!(
        "          - entity: demo.feed.SeenDocument
            each:
              in: input.applied
              as: doc
              where: {filter}
              repeated: last_wins
            instance: doc.document_id
            sets: {{source_id: input.source_id, weight: doc.weight}}
"
    )
}

/// BLOCKER: a literal in an `affects:` entry's `sets:` is a `sets:` value, and it still validates.
/// `authored` only reads `commands[i].outcomes[j].sets`, never `…affects[k].sets`.
#[test]
fn an_affects_sets_literal_binary64_cannot_hold_is_refused() {
    validates("feed.yaml", &feed(&where_entry("weight < 1", "0.5")));
    // The same entry's `where:` is a predicate site the change reaches: the fixture refuses there.
    let at_where = inexact(
        "feed.yaml",
        &feed(&where_entry(&format!("weight < {INEXACT}"), "0.5")),
    );
    assert_eq!(at_where.len(), 1, "control:\n{}", at_where.join("\n"));
    let found = inexact("feed.yaml", &feed(&where_entry("weight < 1", INEXACT)));
    assert_eq!(
        found.len(),
        1,
        "`affects[0].sets: {{weight: {INEXACT}}}` must be refused once as \
         inexact_decimal_literal, got:\n{}",
        found.join("\n")
    );
    assert!(found[0].contains("affects"), "{}", found[0]);
}

/// BLOCKER: `affects[k].each.where` is a predicate position (ess/27) that no site of
/// `Written::from_document` reads, so neither its compact nor its mapping form is refused.
#[test]
fn an_each_where_literal_binary64_cannot_hold_is_refused() {
    validates("feed.yaml", &feed(&each_entry("doc.weight < 0.5")));
    let found = inexact(
        "feed.yaml",
        &feed(&each_entry(&format!("doc.weight < {INEXACT}"))),
    );
    assert_eq!(
        found.len(),
        1,
        "`each.where: doc.weight < {INEXACT}` must be refused once as \
         inexact_decimal_literal, got:\n{}",
        found.join("\n")
    );
    assert!(found[0].contains(INEXACT), "{}", found[0]);
}

/// The pass-1 model: `when` and `sets` written as given.
fn pay(when: &str, sets: &str) -> String {
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
errors:
  - {{name: demo.pay.TooSmall, summary: The amount is too small., fields: []}}
events:
  - name: demo.pay.Opened
    fields: [{{name: account_id, type: demo.pay.AccountId}}]
commands:
  - name: demo.pay.Open
    input:
      - {{name: amount, type: Decimal}}
    outcomes:
      - name: too-small
        when: {when}
        error: demo.pay.TooSmall
      - name: opened
        creates: demo.pay.Account
        instance: account_id
        sets: {{amount: {sets}}}
        emits: [demo.pay.Opened]
        payload: {{demo.pay.Opened: {{account_id: {{generated: true}}}}}}
"
    )
}

const WHEN: &str = "[inexact_decimal_literal] command.demo.pay.Open.outcomes.too-small.when";
const SETS: &str = "[inexact_decimal_literal] command.demo.pay.Open.outcomes.opened.sets";

/// Probe: one float anchored in a guard and aliased into `sets:` is written at two sites, and is
/// refused once at each.
#[test]
fn one_aliased_literal_is_refused_once_at_each_site_that_writes_it() {
    validates("pay.yaml", &pay("{amount: {lt: &x 0.5}}", "*x"));
    let found = inexact(
        "pay.yaml",
        &pay(&format!("{{amount: {{lt: &x {INEXACT}}}}}"), "*x"),
    );
    assert_eq!(found.len(), 2, "{}", found.join("\n"));
    assert!(found.iter().any(|it| it.starts_with(WHEN)), "{found:?}");
    assert!(found.iter().any(|it| it.starts_with(SETS)), "{found:?}");
}

/// Probe: a guard written with quoted keys in flow style is the same guard.
#[test]
fn a_quoted_key_guard_is_refused_at_its_when() {
    validates("pay.yaml", &pay("{\"amount\": {'lt': 0.5}}", "0.5"));
    let found = inexact(
        "pay.yaml",
        &pay(&format!("{{\"amount\": {{'lt': {INEXACT}}}}}"), "0.5"),
    );
    assert_eq!(found.len(), 1, "{}", found.join("\n"));
    assert!(found[0].starts_with(WHEN), "{}", found[0]);
}

/// Probe: a related selection's `where:` sits inside `sets:`; its literal is refused once.
#[test]
fn a_related_selection_where_inside_sets_is_refused_once() {
    let selection = |literal: &str| {
        format!(
            "{{related: {{entity: demo.pay.Account, where: {{amount: {{lt: {literal}}}}}, \
             field: amount}}}}"
        )
    };
    validates("pay.yaml", &pay("amount < 1", &selection("0.5")));
    let found = inexact("pay.yaml", &pay("amount < 1", &selection(INEXACT)));
    assert_eq!(found.len(), 1, "{}", found.join("\n"));
}

/// Probe: each YAML 1.2 core spelling of an inexact float reaches the scan past the pre-filter.
#[test]
fn every_core_float_spelling_of_an_inexact_literal_is_refused() {
    let mut tried = 0;
    for spelled in [
        ".1000000000000000000001",
        "+0.1000000000000000000001",
        "+1e-400",
        "1E-400",
        "1.e-400",
        "0.1000000000000000000001e0",
        "1000000000000000000000000.5",
    ] {
        let document: ess_yaml::Value =
            ess_yaml::from_str(&format!("a: {spelled}\n")).expect("reads");
        if !document.get("a").is_some_and(ess_yaml::Value::is_f64) {
            continue;
        }
        tried += 1;
        let found = inexact("pay.yaml", &pay("amount < 1", spelled));
        assert_eq!(found.len(), 1, "{spelled}\n{}", found.join("\n"));
        assert!(found[0].starts_with(SETS), "{spelled}\n{}", found[0]);
    }
    assert!(tried >= 3, "only {tried} spellings read as floats");
}
