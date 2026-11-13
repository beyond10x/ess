//! Adversary, pass 1, of `story:refusal-overlap-witnessed-for-distinct-guard`: a `List` input a
//! `distinct` reads is also tried as two copies of element 0 (`count_ladders`), and an input refusal
//! pair with a `distinct` guard nothing sends is a `Note::UnwitnessedOverlap`.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{Note, Synthesis};
use ess_conformance::{ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

/// The story's model: `missing` before `duplicate` (not distinct by name).
const MODEL: &str = r#"format: ess/32
system: demo
version: v1
domain: demo.cfg
summary: A config created with a site and a list of items, refused for a missing site and for repeated item names.
types:
  - {name: demo.cfg.AccountId, kind: newtype, of: Uuid}
  - name: demo.cfg.Item
    kind: struct
    fields:
      - {name: name, type: String}
entities:
  - name: demo.cfg.Config
    identity: {name: account_id, type: demo.cfg.AccountId}
    fields:
      - {name: site, type: Optional<String>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
actors:
  - name: demo.cfg.Admin
    may: [demo.cfg.Create]
errors:
  - {name: demo.cfg.Missing, fields: []}
  - {name: demo.cfg.Duplicate, fields: []}
commands:
  - name: demo.cfg.Create
    input:
      - {name: account_id, type: demo.cfg.AccountId}
      - {name: site, type: Optional<String>}
      - {name: items, type: Optional<List<demo.cfg.Item>>}
    outcomes:
      - name: missing
        when: {any: ["not defined(site)", 'site == ""']}
        error: demo.cfg.Missing
      - name: duplicate
        when: {all: ["defined(items)", {not: {distinct: {in: items, as: i, by: i.name}}}]}
        error: demo.cfg.Duplicate
      - name: created
        creates: demo.cfg.Config
        instance: account_id
        sets: {site: input.site}
        emits: [demo.cfg.Created]
        payload: {demo.cfg.Created: {account_id: input.account_id}}
events:
  - name: demo.cfg.Created
    fields: [{name: account_id, type: demo.cfg.AccountId}]
components:
  - component: cfg
    owns: {domains: [demo.cfg]}
    accepts: {commands: [demo.cfg.Create]}
    reached_by: network
"#;

/// The repository's three-refusal `distinct` model (`tests/expression_distinct.rs`), unchanged.
const POOL: &str = r"format: ess/22
system: pool
version: v1
domain: pool.files
types:
  - name: pool.files.File
    kind: struct
    fields:
      - {name: path, type: String}
      - {name: size, type: Integer}
entities:
  - name: pool.files.Bundle
    identity: {name: bundle_id, type: Uuid}
    fields:
      - {name: files, type: List<pool.files.File>}
      - {name: tags, type: List<String>}
      - {name: stamps, type: List<Timestamp>}
    invariants:
      - {distinct: {in: files, as: file, by: file.path}}
      - {distinct: {in: tags, as: tag}}
      - {distinct: {in: stamps, as: stamp}}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: pool.files.DuplicatePath, summary: Two files share a path.}
  - {name: pool.files.DuplicateTag, summary: A tag repeats.}
  - {name: pool.files.DuplicateStamp, summary: Two stamps name one instant.}
events:
  - name: pool.files.Opened
    fields:
      - {name: bundle_id, type: Uuid}
commands:
  - name: pool.files.Open
    input:
      - {name: files, type: List<pool.files.File>}
      - {name: tags, type: List<String>}
      - {name: stamps, type: List<Timestamp>}
    outcomes:
      - name: duplicate-path
        when: {not: {distinct: {in: files, as: file, by: file.path}}}
        error: pool.files.DuplicatePath
      - name: duplicate-tag
        when: {not: {distinct: {in: tags, as: tag}}}
        error: pool.files.DuplicateTag
      - name: duplicate-stamp
        when: {not: {distinct: {in: stamps, as: stamp}}}
        error: pool.files.DuplicateStamp
      - name: opened
        creates: pool.files.Bundle
        instance: bundle_id
        emits: [pool.files.Opened]
        payload:
          pool.files.Opened: {bundle_id: {generated: true}}
        sets: {files: input.files, tags: input.tags, stamps: input.stamps}
views:
  - name: pool.files.Bundles
    source: pool.files.Bundle
    consistency: read_your_writes
    fields:
      - {name: bundle_id, type: Uuid}
      - {name: files, type: List<pool.files.File>}
      - {name: tags, type: List<String>}
      - {name: stamps, type: List<Timestamp>}
";

fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds {from}");
    text.replace(from, to)
}

fn overlap_notes(result: &Synthesis, id: &str) -> Vec<(String, String)> {
    result
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::UnwitnessedOverlap {
                scenario,
                first,
                other,
                ..
            } if scenario.to_string() == id => Some((first.to_string(), other.to_string())),
            _ => None,
        })
        .collect()
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("adversary.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

/// Each send of the scenario: its literal fields and the outcome the next step requires.
fn sends(scenario: &ConformanceScenario) -> Vec<(BTreeMap<String, Node>, Option<String>)> {
    let mut out = Vec::new();
    for (at, step) in scenario.steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand { input, .. } = step else {
            continue;
        };
        let literal: BTreeMap<String, Node> = input
            .iter()
            .filter_map(|(name, value)| {
                ScenarioValue::as_literal(value).map(|node| (name.clone(), node.clone()))
            })
            .collect();
        let outcome = scenario.steps.get(at + 1).and_then(|next| match next {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.outcome.to_string()),
            _ => None,
        });
        out.push((literal, outcome));
    }
    out
}

fn seq<'a>(input: &'a BTreeMap<String, Node>, field: &str) -> &'a [Node] {
    match input.get(field) {
        Some(Node::Seq(items)) => items,
        _ => &[],
    }
}

fn key(element: &Node, member: &str) -> Option<Node> {
    match element {
        Node::Map(fields) => fields.get(member).cloned(),
        _ => None,
    }
}

/// The `distinct` witness design (`decisive_first`, `docs/design/expression-family-source22.md`):
/// a refusing `distinct` send repeats a key between elements that are neither neighbours nor whole
/// copies, so a target comparing neighbours only, or whole elements, is failed by it. The
/// `duplicate-path` scenario of the repository's three-refusal model now also sends the pair
/// overlap with `duplicate-tag`, and that send's files are two whole copies side by side; it is
/// the scenario's last send, which `expression_distinct.rs` reads as its witness.
#[test]
fn every_refusing_path_send_repeats_a_path_between_non_neighbours_that_differ() {
    let result = synthesis(POOL);
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    let duplicate = scenario(&result.suite, "pool.files.Open/outcome/duplicate-path");
    let mut weak = Vec::new();
    for (input, outcome) in sends(duplicate) {
        if outcome.as_deref() != Some("duplicate-path") {
            continue;
        }
        let files = seq(&input, "files");
        let paths: Vec<Option<Node>> = files.iter().map(|file| key(file, "path")).collect();
        let adjacent = paths.windows(2).any(|pair| pair[0] == pair[1]);
        let whole = files
            .iter()
            .enumerate()
            .any(|(at, file)| files[at + 1..].contains(file));
        if adjacent || whole {
            weak.push(files.to_vec());
        }
    }
    assert_eq!(
        weak,
        Vec::<Vec<Node>>::new(),
        "a refusing send of `duplicate-path` whose duplicate is adjacent or whole"
    );
}

/// An empty `distinct` overlap is not a note: `single` holds of fewer than two items and
/// `duplicate` of two or more, so no input lies in both, and the candidates try both sides of the
/// count.
#[test]
fn an_empty_distinct_overlap_is_not_noted() {
    let model = replaced(
        MODEL,
        "      - name: duplicate\n",
        "      - name: single\n        when: {all: [\"defined(items)\", \"items.count < 2\"]}\n        error: demo.cfg.Missing\n      - name: duplicate\n",
    );
    let result = synthesis(&model);
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    assert_eq!(
        overlap_notes(&result, "demo.cfg.Create/outcome/single"),
        Vec::new(),
        "{:#?}",
        result.notes
    );
}
