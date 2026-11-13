//! Adversary, pass 1, of `story:refusal-overlap-witnessed-for-distinct-guard`: pins of branches the
//! unit's own suite does not hold, each green on the unit's commit.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate;
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

const MISSING: &str = "demo.cfg.Create/outcome/missing";

/// `missing` before `limit`, a refusal over a length no candidate reaches.
const LIMIT_MODEL: &str = r#"format: ess/32
system: demo
version: v1
domain: demo.cfg
summary: A config created with a site and a list of items, refused for a missing site and for too many items.
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
  - {name: demo.cfg.Limit, fields: []}
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
      - name: limit
        when: {all: ["defined(items)", "items.count > 5000"]}
        error: demo.cfg.Limit
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

const LIMIT_GUARD: &str = r#"{all: ["defined(items)", "items.count > 5000"]}"#;

fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds {from}");
    text.replace(from, to)
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

fn blank_or_absent_site(input: &BTreeMap<String, Node>) -> bool {
    match input.get("site") {
        None | Some(Node::Null) => true,
        Some(Node::Text(text)) => text.is_empty(),
        _ => false,
    }
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

fn audit(text: &str) -> mutate::MutationReport {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    let files = vec![(Source::new("fixture.yaml"), raw)];
    let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
    mutate::audit(
        &files,
        &texts,
        &[mutate::MutantClass::PrecedenceSwap],
        || Interpreted::for_model(ir.clone()),
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

/// Three input refusals, one `distinct`: a third refusal over the site, `banned`, overlaps
/// `duplicate` and not `missing`. Each precedence-swap of an
/// overlapping pair is killed; no mutant survives or is unwitnessed.
#[test]
fn three_refusals_one_distinct_every_overlapping_swap_is_killed() {
    let model = replaced(
        MODEL,
        "      - name: created\n",
        "      - name: banned\n        when: 'site == \"banned\"'\n        error: demo.cfg.Missing\n      - name: created\n",
    );
    let report = audit(&model);
    let text = report.render_text();
    for id in [
        "precedence-swap/demo.cfg.Create/missing/duplicate",
        "precedence-swap/demo.cfg.Create/duplicate/banned",
    ] {
        let entry = report
            .mutants
            .iter()
            .find(|entry| entry.id == id)
            .unwrap_or_else(|| panic!("no site {id}:\n{text}"));
        assert_eq!(entry.verdict, mutate::Verdict::Killed, "{id}:\n{text}");
    }
    assert_eq!(report.counts.survived, 0, "{text}");
    assert_eq!(report.counts.unwitnessed, 0, "{text}");
}

/// A `distinct` over a list of scalars (no `by:`): two copies of element 0 are one text twice.
#[test]
fn scalar_list_distinct_overlap_is_sent_in_the_first_declared_scenario() {
    let model = replaced(
        &replaced(
            MODEL,
            "{name: items, type: Optional<List<demo.cfg.Item>>}",
            "{name: tags, type: Optional<List<String>>}",
        ),
        DUPLICATE_GUARD,
        r#"{all: ["defined(tags)", {not: {distinct: {in: tags, as: t}}}]}"#,
    );
    let result = synthesis(&model);
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    let sent = sends(scenario(&result.suite, MISSING));
    assert!(
        sent.iter().any(|(input, outcome)| {
            let tags = seq(input, "tags");
            blank_or_absent_site(input)
                && tags.len() >= 2
                && tags[1..].contains(&tags[0])
                && outcome.as_deref() == Some("missing")
        }),
        "{sent:#?}"
    );
    assert_eq!(overlap_notes(&result, MISSING), Vec::new());
}

/// A key that is `Optional`: the copies carry it where element 0 does.
#[test]
fn optional_key_distinct_overlap_is_sent_or_noted() {
    let model = replaced(
        MODEL,
        "      - {name: name, type: String}",
        "      - {name: name, type: Optional<String>}",
    );
    let result = synthesis(&model);
    let sent = sends(scenario(&result.suite, MISSING));
    let overlap = sent.iter().any(|(input, outcome)| {
        let items = seq(input, "items");
        blank_or_absent_site(input)
            && items.len() >= 2
            && key(&items[0], "name").is_some()
            && items[1..]
                .iter()
                .any(|item| key(item, "name") == key(&items[0], "name"))
            && outcome.as_deref() == Some("missing")
    });
    let noted = overlap_notes(&result, MISSING);
    assert!(
        overlap || noted == vec![("missing".to_owned(), "duplicate".to_owned())],
        "sent {sent:#?}\nnotes {:#?}\nrefusals {:#?}",
        result.notes,
        result.refusals
    );
}

/// Acceptance 4 on an overlap that is inhabited and that two copies cannot reach: `spread` holds
/// of three or more items of distinct names, which a blank site beside three named items satisfies.
/// Either the `missing` scenario sends that, or the pair is named in a note.
#[test]
fn positive_distinct_overlap_is_sent_or_noted() {
    let model = replaced(
        MODEL,
        DUPLICATE_GUARD,
        r#"{all: ["defined(items)", {distinct: {in: items, as: i, by: i.name}}, "items.count > 2"]}"#,
    );
    let result = synthesis(&model);
    let sent = sends(scenario(&result.suite, MISSING));
    let overlap = sent.iter().any(|(input, outcome)| {
        let items = seq(input, "items");
        let names: Vec<Option<Node>> = items.iter().map(|item| key(item, "name")).collect();
        blank_or_absent_site(input)
            && items.len() > 2
            && names
                .iter()
                .enumerate()
                .all(|(at, name)| !names[at + 1..].contains(name))
            && outcome.as_deref() == Some("missing")
    });
    let noted = overlap_notes(&result, MISSING);
    assert!(
        overlap || noted == vec![("missing".to_owned(), "duplicate".to_owned())],
        "sent {sent:#?}\nnotes {:#?}\nrefusals {:#?}",
        result.notes,
        result.refusals
    );
}

/// A `distinct` under a quantifier, over a list inside each element: `groups.0.members`.
#[test]
fn nested_distinct_overlap_is_sent_or_noted() {
    let model = replaced(
        &replaced(
            &replaced(
                MODEL,
                "      - {name: name, type: String}\n",
                "      - {name: name, type: String}\n  - name: demo.cfg.Group\n    kind: struct\n    fields:\n      - {name: members, type: List<demo.cfg.Item>}\n",
            ),
            "{name: items, type: Optional<List<demo.cfg.Item>>}",
            "{name: groups, type: Optional<List<demo.cfg.Group>>}",
        ),
        DUPLICATE_GUARD,
        r#"{all: ["defined(groups)", {not: {forall: {in: groups, as: g, that: {distinct: {in: g.members, as: m, by: m.name}}}}}]}"#,
    );
    let result = synthesis(&model);
    let sent = sends(scenario(&result.suite, MISSING));
    let overlap = sent.iter().any(|(input, outcome)| {
        blank_or_absent_site(input)
            && seq(input, "groups").iter().any(|group| match group {
                Node::Map(fields) => match fields.get("members") {
                    Some(Node::Seq(members)) => members.iter().enumerate().any(|(at, member)| {
                        members[at + 1..]
                            .iter()
                            .any(|other| key(other, "name") == key(member, "name"))
                    }),
                    _ => false,
                },
                _ => false,
            })
            && outcome.as_deref() == Some("missing")
    });
    let noted = overlap_notes(&result, MISSING);
    assert!(
        overlap || noted == vec![("missing".to_owned(), "duplicate".to_owned())],
        "sent {sent:#?}\nnotes {:#?}\nrefusals {:#?}",
        result.notes,
        result.refusals
    );
}

/// The same pair on a command that updates a stored row: the refusal pair is sent for an identity
/// no row carries (`unknown_identity_refusal`). A note there would be a false one where the scenario
/// sends the overlap; no note and no send would be the gap acceptance 4 names.
#[test]
fn updating_command_distinct_overlap_is_sent_xor_noted() {
    let model = replaced(
        &replaced(
            &replaced(
                MODEL,
                "may: [demo.cfg.Create]",
                "may: [demo.cfg.Create, demo.cfg.Amend]",
            ),
            "accepts: {commands: [demo.cfg.Create]}",
            "accepts: {commands: [demo.cfg.Create, demo.cfg.Amend]}",
        ),
        "events:\n",
        r#"  - name: demo.cfg.Amend
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
      - name: amended
        updates: demo.cfg.Config
        instance: account_id
        sets: {site: input.site}
        emits: [demo.cfg.Created]
        payload: {demo.cfg.Created: {account_id: input.account_id}}
events:
"#,
    );
    let result = synthesis(&model);
    let id = "demo.cfg.Amend/outcome/missing";
    let sent = sends(scenario(&result.suite, id));
    let overlap = sent.iter().any(|(input, outcome)| {
        let items = seq(input, "items");
        blank_or_absent_site(input)
            && items.len() >= 2
            && items[1..]
                .iter()
                .any(|item| key(item, "name") == key(&items[0], "name"))
            && outcome.as_deref() == Some("missing")
    });
    let noted = !overlap_notes(&result, id).is_empty();
    assert!(
        overlap != noted,
        "sent {overlap}, noted {noted}: {sent:#?}\nnotes {:#?}",
        result.notes
    );
}

/// The `distinct` filter of `unwitnessed_overlaps`: the same unreached pair is named where its
/// later guard also reads a `distinct`, and not where it reads only a `count`. The unit's own
/// no-note case is a pair that is witnessed, which keeps no note with the filter removed.
#[test]
fn an_unreached_pair_is_noted_only_where_a_guard_reads_a_distinct() {
    let plain = synthesis(LIMIT_MODEL);
    assert_eq!(
        overlap_notes(&plain, MISSING),
        Vec::new(),
        "{:#?}",
        plain.notes
    );
    assert!(LIMIT_MODEL.contains(LIMIT_GUARD));
    let keyed = synthesis(&LIMIT_MODEL.replace(
        LIMIT_GUARD,
        r#"{all: ["defined(items)", "items.count > 5000", {not: {distinct: {in: items, as: i, by: i.name}}}]}"#,
    ));
    assert_eq!(
        overlap_notes(&keyed, MISSING),
        vec![("missing".to_owned(), "limit".to_owned())],
        "{:#?}",
        keyed.notes
    );
}
