//! An inexact `Decimal` literal in an `affects:` entry (`story:an-inexact-authored-decimal-is-refused`).
//!
//! An entry writes a literal in three places besides its `where:`: its `sets:` table, a filtered
//! read inside that table, and the `where:` of its `each:`. Each is refused once, at the place it
//! was written, in the compact and the mapping form alike.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const INEXACT: &str = "0.1000000000000000000001";
const AT: &str = "[inexact_decimal_literal] command.demo.feed.RunSource.outcomes.ran";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("feed.yaml"), raw)])
}

fn validates(text: &str) {
    if let Err(errors) = assemble(text) {
        panic!("control: must validate:\n{errors}\n{text}");
    }
}

fn inexact(text: &str) -> Vec<String> {
    match assemble(text) {
        Ok(_) => Vec::new(),
        Err(errors) => errors
            .as_slice()
            .iter()
            .filter(|error| error.code == ValidationCode::InexactDecimalLiteral)
            .map(ToString::to_string)
            .collect(),
    }
}

/// A command updating a `Source` whose outcome writes `entry` under `affects:`.
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
"
    )
}

fn where_entry(filter: &str, sets: &str) -> String {
    format!(
        "          - entity: demo.feed.SeenDocument
            where: {filter}
            sets: {sets}
"
    )
}

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

#[test]
fn an_affects_sets_literal_is_refused_at_the_field_it_sets() {
    validates(&feed(&where_entry("weight < 1", "{weight: 0.5}")));
    let found = inexact(&feed(&where_entry(
        "weight < 1",
        &format!("{{weight: {INEXACT}}}"),
    )));
    assert_eq!(found.len(), 1, "{}", found.join("\n"));
    assert!(
        found[0].starts_with(&format!("{AT}.affects[0].sets.weight:")),
        "{}",
        found[0]
    );
}

#[test]
fn a_second_affects_entry_is_refused_at_its_own_position() {
    let entries = |second: &str| {
        format!(
            "{}{}",
            where_entry("weight < 1", "{weight: 0.5}"),
            where_entry("weight < 2", &format!("{{weight: {second}}}")),
        )
    };
    validates(&feed(&entries("0.25")));
    let found = inexact(&feed(&entries(INEXACT)));
    assert_eq!(found.len(), 1, "{}", found.join("\n"));
    assert!(
        found[0].starts_with(&format!("{AT}.affects[1].sets.weight:")),
        "{}",
        found[0]
    );
}

#[test]
fn an_each_entry_sets_literal_is_refused_beside_its_element_reads() {
    let entry = format!(
        "          - entity: demo.feed.SeenDocument
            each: {{in: input.applied, as: doc, repeated: last_wins}}
            instance: doc.document_id
            sets: {{source_id: input.source_id, weight: {INEXACT}}}
"
    );
    let found = inexact(&feed(&entry));
    assert_eq!(found.len(), 1, "{}", found.join("\n"));
    assert!(
        found[0].starts_with(&format!("{AT}.affects[0].sets.weight:")),
        "{}",
        found[0]
    );
}

#[test]
fn an_each_where_literal_is_refused_at_the_each_where() {
    validates(&feed(&each_entry("doc.weight < 0.5")));
    let found = inexact(&feed(&each_entry(&format!("doc.weight < {INEXACT}"))));
    assert_eq!(found.len(), 1, "{}", found.join("\n"));
    assert!(
        found[0].starts_with(&format!("{AT}.affects[0].each.where:")),
        "{}",
        found[0]
    );
}

#[test]
fn a_mapping_form_each_where_is_refused_as_the_compact_form_is() {
    let mapping = |literal: &str| format!("{{doc.weight: {{lt: {literal}}}}}");
    validates(&feed(&each_entry(&mapping("0.5"))));
    let found = inexact(&feed(&each_entry(&mapping(INEXACT))));
    assert_eq!(found.len(), 1, "{}", found.join("\n"));
    assert!(
        found[0].starts_with(&format!("{AT}.affects[0].each.where:")),
        "{}",
        found[0]
    );
}

#[test]
fn a_filtered_read_inside_an_affects_sets_is_refused_at_its_field() {
    let read = |literal: &str| {
        format!(
            "{{weight: {{related: {{entity: demo.feed.SeenDocument, where: {{weight: {{lt: \
             {literal}}}}}, field: weight}}}}}}"
        )
    };
    validates(&feed(&where_entry("weight < 1", &read("0.5"))));
    let found = inexact(&feed(&where_entry("weight < 1", &read(INEXACT))));
    assert_eq!(found.len(), 1, "{}", found.join("\n"));
    assert!(
        found[0].starts_with(&format!("{AT}.affects[0].sets.weight:")),
        "{}",
        found[0]
    );
}
