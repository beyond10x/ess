//! Two input refusals whose guards overlap where one guard is a `distinct` predicate over a List
//! input: the first declared answers there, so synthesis sends the overlap and requires it, as it
//! does for a `count` comparison (beyond10x/ess#455).
//!
//! `missing: site == ""` is declared before `duplicate: not distinct items by name`. The overlap is
//! a blank site with two items of one name.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate;
use ess_conformance::synthesize::Note;
use ess_conformance::{
    synthesize::Synthesis, ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

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

const DISTINCT_GUARD: &str = r"{not: {distinct: {in: items, as: i, by: i.name}}}";

const MISSING: &str = "demo.cfg.Create/outcome/missing";

const SWAP: &str = "precedence-swap/demo.cfg.Create/missing/duplicate";

/// The model with its `duplicate` guard a `count` comparison, the control.
fn count_model() -> String {
    assert!(MODEL.contains(DISTINCT_GUARD), "the model's guard");
    MODEL.replace(DISTINCT_GUARD, "\"items.count > 1\"")
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("refusal-overlap-distinct.yaml"), raw)])
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

/// One send of `Create`: its `site` (`-` where not a text), the names of its `items` (`-` where not
/// a list), and the outcome and error the next steps require.
type Create = (String, Vec<String>, Option<String>, Option<String>);

fn creates(scenario: &ConformanceScenario) -> Vec<Create> {
    let mut out = Vec::new();
    for (at, step) in scenario.steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand { input, .. } = step else {
            continue;
        };
        let site = match input.get("site").and_then(ScenarioValue::as_literal) {
            Some(Node::Text(text)) => format!("{text:?}"),
            _ => "-".to_owned(),
        };
        let items = match input.get("items").and_then(ScenarioValue::as_literal) {
            Some(Node::Seq(items)) => items
                .iter()
                .map(|item| match item {
                    Node::Map(fields) => format!("{:?}", fields.get("name")),
                    other => format!("{other:?}"),
                })
                .collect(),
            _ => vec!["-".to_owned()],
        };
        let outcome = scenario.steps.get(at + 1).and_then(|next| match next {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.outcome.to_string()),
            _ => None,
        });
        let error = scenario.steps.get(at + 2).and_then(|next| match next {
            ScenarioStep::ExpectError { error, .. } => Some(error.to_string()),
            _ => None,
        });
        out.push((site, items, outcome, error));
    }
    out
}

/// Whether `sent` holds the overlap: a blank site and two items of one name, requiring `missing`.
fn sends_overlap(sent: &[Create]) -> bool {
    sent.iter().any(|(site, items, outcome, error)| {
        site == "\"\""
            && items.len() == 2
            && items[0] == items[1]
            && items[0] != "-"
            && outcome.as_deref() == Some("missing")
            && error.as_deref() == Some("demo.cfg.Missing")
    })
}

fn unwitnessed(result: &Synthesis) -> Vec<&Note> {
    result
        .notes
        .iter()
        .filter(|note| matches!(note, Note::UnwitnessedOverlap { .. }))
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

fn verdict_of(report: &mutate::MutationReport, id: &str) -> mutate::Verdict {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("no site {id}:\n{}", report.render_text()))
        .verdict
}

#[test]
fn distinct_overlap_is_sent_in_the_first_declared_scenario() {
    let result = synthesis(MODEL);
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    let sent = creates(scenario(&result.suite, MISSING));
    assert!(
        sends_overlap(&sent),
        "`missing`'s scenario sends a blank site with two items of one name and requires \
         `missing` and `Missing`: {sent:#?}"
    );
    assert_eq!(unwitnessed(&result), Vec::<&Note>::new());
}

#[test]
fn count_overlap_keeps_its_send() {
    let result = synthesis(&count_model());
    assert_eq!(result.refusals.len(), 0, "{:#?}", result.refusals);
    let sent = creates(scenario(&result.suite, MISSING));
    assert!(sends_overlap(&sent), "{sent:#?}");
}

#[test]
fn distinct_and_count_swaps_are_killed() {
    for model in [MODEL.to_owned(), count_model()] {
        let report = audit(&model);
        assert_eq!(
            verdict_of(&report, SWAP),
            mutate::Verdict::Killed,
            "{}",
            report.render_text()
        );
        assert_eq!(report.counts.survived, 0, "{}", report.render_text());
        assert_eq!(report.counts.unwitnessed, 0, "{}", report.render_text());
    }
}

/// `duplicate` read the other way: it holds of three or more items with distinct flags, which a
/// `Boolean` key cannot supply, so no input lies in its overlap with `missing`.
fn boolean_key_model() -> String {
    MODEL
        .replace("name: demo.cfg.Item\n    kind: struct\n    fields:\n      - {name: name, type: String}", "name: demo.cfg.Item\n    kind: struct\n    fields:\n      - {name: flag, type: Boolean}")
        .replace(
            DISTINCT_GUARD,
            "{all: [{distinct: {in: items, as: i, by: i.flag}}, \"items.count > 2\"]}",
        )
}

#[test]
fn distinct_overlap_no_candidate_reaches_is_noted() {
    let result = synthesis(&boolean_key_model());
    let noted: Vec<(String, String)> = result
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
        .collect();
    assert_eq!(
        noted,
        vec![("missing".to_owned(), "duplicate".to_owned())],
        "{:#?}",
        result.notes
    );
}

#[test]
fn models_without_a_distinct_guard_gain_no_note() {
    let result = synthesis(&count_model());
    assert_eq!(unwitnessed(&result), Vec::<&Note>::new());
}
