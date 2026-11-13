//! Adversary, pass 2, of `story:refusal-overlap-witnessed-for-distinct-guard`: the duplicate a
//! `distinct` list is tried at in `count_ladders` is three elements, the last repeating the first
//! one's key, and `duplicate_capped_below_two` shows a pair empty. Pins, each green on a1d0fa8af5.
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

const DUPLICATE_GUARD: &str =
    r#"{all: ["defined(items)", {not: {distinct: {in: items, as: i, by: i.name}}}]}"#;

const ITEM: &str = "      - {name: name, type: String}\n";

const MISSING: &str = "demo.cfg.Create/outcome/missing";

const DUPLICATE: &str = "demo.cfg.Create/outcome/duplicate";

fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds {from}");
    text.replace(from, to)
}

/// The story's model with `duplicate` guarded by `guard`, and the item struct's fields `fields`.
fn model(fields: &str, guard: &str) -> String {
    replaced(&replaced(MODEL, ITEM, fields), DUPLICATE_GUARD, guard)
}

fn synthesis(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("adversary.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    ess_conformance::synthesize::synthesize(&ir)
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> Option<&'a ConformanceScenario> {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
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

fn blank_site(input: &BTreeMap<String, Node>) -> bool {
    match input.get("site") {
        None | Some(Node::Null) => true,
        Some(Node::Text(text)) => text.is_empty(),
        _ => false,
    }
}

fn items(input: &BTreeMap<String, Node>) -> &[Node] {
    match input.get("items") {
        Some(Node::Seq(items)) => items,
        _ => &[],
    }
}

fn member(element: &Node, name: &str) -> Option<Node> {
    match element {
        Node::Map(fields) => fields.get(name).cloned(),
        _ => None,
    }
}

fn repeats(list: &[Node], name: &str) -> bool {
    list.iter().enumerate().any(|(at, element)| {
        list[at + 1..]
            .iter()
            .any(|other| member(other, name) == member(element, name))
    })
}

fn overlap_notes(result: &Synthesis) -> Vec<(String, String)> {
    result
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::UnwitnessedOverlap {
                scenario,
                first,
                other,
                ..
            } if scenario.to_string() == MISSING => Some((first.to_string(), other.to_string())),
            _ => None,
        })
        .collect()
}

/// `missing`'s scenario sends a blank site beside a list `holds` accepts, requiring `missing`.
fn sends_overlap(result: &Synthesis, holds: impl Fn(&[Node]) -> bool) -> bool {
    scenario(&result.suite, MISSING).is_some_and(|missing| {
        sends(missing).iter().any(|(input, outcome)| {
            blank_site(input) && holds(items(input)) && outcome.as_deref() == Some("missing")
        })
    })
}

fn sent_or_noted(result: &Synthesis, holds: impl Fn(&[Node]) -> bool) {
    let sent = sends_overlap(result, holds);
    let noted = overlap_notes(result) == vec![("missing".to_owned(), "duplicate".to_owned())];
    assert!(
        sent || noted,
        "neither sent nor noted:\n{:#?}\nnotes {:#?}\nrefusals {:#?}",
        scenario(&result.suite, MISSING).map(sends),
        result.notes,
        result.refusals
    );
}

/// `duplicate` at exactly two items: the ladder's duplicate at the required length.
#[test]
fn a_duplicate_of_exactly_two_is_sent_in_the_overlap() {
    let result = synthesis(&model(
        ITEM,
        r#"{all: ["defined(items)", "items.count == 2", {not: {distinct: {in: items, as: i, by: i.name}}}]}"#,
    ));
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    assert!(sends_overlap(&result, |list| list.len() == 2 && repeats(list, "name")));
}

/// `duplicate` at two items at most.
#[test]
fn a_duplicate_of_at_most_two_is_sent_in_the_overlap() {
    let result = synthesis(&model(
        ITEM,
        r#"{all: ["defined(items)", "items.count <= 2", {not: {distinct: {in: items, as: i, by: i.name}}}]}"#,
    ));
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    assert!(sends_overlap(&result, |list| list.len() <= 2 && repeats(list, "name")));
}

/// A `Boolean` key and a one-variant enum key: no panic, and the overlap is sent or noted.
#[test]
fn a_key_of_fewer_than_three_values_is_sent_or_noted() {
    for (fields, types) in [
        ("      - {name: flag, type: Boolean}\n", ""),
        (
            "      - {name: flag, type: demo.cfg.Only}\n",
            "  - {name: demo.cfg.Only, kind: enum, variants: [One]}\n",
        ),
    ] {
        let text = replaced(
            &model(
                fields,
                r#"{all: ["defined(items)", {not: {distinct: {in: items, as: i, by: i.flag}}}]}"#,
            ),
            "types:\n",
            &format!("types:\n{types}"),
        );
        let result = synthesis(&text);
        assert_eq!(result.refusals.len(), 0, "{fields}: {:#?}", result.refusals);
        sent_or_noted(&result, |list| repeats(list, "flag"));
    }
}

/// `duplicate_capped_below_two` reads a bound on another list as no bound: the pair is not empty
/// (a blank site, two `tags` and 5001 items, two of one name) and stays noted.
#[test]
fn a_bound_on_another_list_keeps_the_note() {
    let text = replaced(
        &model(
            ITEM,
            r#"{all: ["defined(items)", "tags.count < 2", "items.count > 5000", {not: {distinct: {in: items, as: i, by: i.name}}}]}"#,
        ),
        "      - {name: items, type: Optional<List<demo.cfg.Item>>}\n",
        "      - {name: items, type: Optional<List<demo.cfg.Item>>}\n      - {name: tags, type: List<String>}\n",
    );
    let result = synthesis(&text);
    assert_eq!(
        overlap_notes(&result),
        vec![("missing".to_owned(), "duplicate".to_owned())],
        "{:#?}",
        result.notes
    );
}

/// A bound below two under `any` is no bound on every input: the pair stays noted.
#[test]
fn a_bound_under_any_keeps_the_note() {
    let result = synthesis(&model(
        ITEM,
        r#"{all: ["defined(items)", {any: ["items.count < 2", "items.count > 5000"]}, "items.count > 4000", {not: {distinct: {in: items, as: i, by: i.name}}}]}"#,
    ));
    assert_eq!(
        overlap_notes(&result),
        vec![("missing".to_owned(), "duplicate".to_owned())],
        "{:#?}",
        result.notes
    );
}

/// A positive `distinct` beside `count < 2` holds of one item: the overlap is sent or noted, never
/// shown empty.
#[test]
fn a_positive_distinct_beside_a_bound_below_two_is_sent_or_noted() {
    let result = synthesis(&model(
        ITEM,
        r#"{all: ["defined(items)", "items.count < 2", {distinct: {in: items, as: i, by: i.name}}]}"#,
    ));
    sent_or_noted(&result, |list| list.len() < 2);
}

/// The duplicate refused in the `duplicate` scenario's own send is untouched by the ladder: it is
/// still the three-element one with the repeat at the ends.
#[test]
fn the_duplicate_scenario_still_sends_the_decisive_duplicate() {
    let result = synthesis(MODEL);
    let duplicate = scenario(&result.suite, DUPLICATE).expect("the duplicate scenario");
    assert!(
        sends(duplicate).iter().any(|(input, outcome)| {
            let list = items(input);
            list.len() == 3
                && member(&list[0], "name") == member(&list[2], "name")
                && member(&list[0], "name") != member(&list[1], "name")
                && outcome.as_deref() == Some("duplicate")
        }),
        "{:#?}",
        sends(duplicate)
    );
}
