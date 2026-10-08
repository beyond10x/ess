//! Nested `any:`/`all:` connectives are witnessed per child at every depth, and a precedence swap
//! of two branches no input selects together is `equivalent` when their guards test the presence
//! of `Optional` inputs (<https://github.com/beyond10x/ess/issues/501>).
//!
//! `story:witness-nested-connectives-and-decide-optional-presence-equivalence`. Every case runs
//! the mutation audit against the interpreter, as `ess verify conform mutate --target interpreted`
//! does.
#![allow(clippy::needless_raw_string_hashes)]

use std::collections::BTreeMap;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass, MutationReport, Verdict};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

fn parsed(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

fn audit(text: &str) -> MutationReport {
    let (files, texts) = parsed(text);
    let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
    mutate::audit(&files, &texts, MutantClass::ALL, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn verdicts(report: &MutationReport) -> BTreeMap<String, Verdict> {
    report
        .mutants
        .iter()
        .map(|entry| (entry.id.clone(), entry.verdict))
        .collect()
}

fn survivors(report: &MutationReport) -> Vec<String> {
    report
        .mutants
        .iter()
        .filter(|entry| entry.verdict == Verdict::Survived)
        .map(|entry| format!("{}: {}", entry.id, entry.change))
        .collect()
}

/// Every `guard-connective` mutant of `report`, which must be at least `at_least`, is killed.
fn connectives_killed(report: &MutationReport, at_least: usize) {
    let connectives: Vec<_> = report
        .mutants
        .iter()
        .filter(|entry| entry.class == MutantClass::GuardConnective)
        .collect();
    assert!(
        connectives.len() >= at_least,
        "expected at least {at_least} connective mutants, got {:#?}",
        verdicts(report)
    );
    for entry in connectives {
        assert_eq!(
            entry.verdict,
            Verdict::Killed,
            "{} ({}) is not killed; every verdict: {:#?}",
            entry.id,
            entry.change,
            verdicts(report)
        );
    }
}

/// The issue's reproduction: a credential in exactly one of three optional places.
const OPTIONAL_PAIRS: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.req

components:
  - component: req-server
    owns:
      domains: [demo.req]
    accepts:
      commands: [demo.req.Send]
    reached_by: network

errors:
  - name: demo.req.TooMany
  - name: demo.req.NoneGiven

commands:
  - name: demo.req.Send
    input:
      - {name: header, type: Optional<String>}
      - {name: body, type: Optional<String>}
      - {name: query, type: Optional<String>}
    outcomes:
      - name: too-many
        when:
          any:
            - all: [{header: {defined: true}}, {body: {defined: true}}]
            - all: [{header: {defined: true}}, {query: {defined: true}}]
            - all: [{body: {defined: true}}, {query: {defined: true}}]
        error: demo.req.TooMany
      - name: none-given
        when:
          all: [{header: {defined: false}}, {body: {defined: false}}, {query: {defined: false}}]
        error: demo.req.NoneGiven
      - name: sent
        accepts: nothing
"#;

#[test]
fn the_issue_reproduction_has_no_survivor() {
    let report = audit(OPTIONAL_PAIRS);
    assert_eq!(
        report.counts.survived,
        0,
        "survivors: {:#?}",
        survivors(&report)
    );
    let found = verdicts(&report);
    for nested in 1..=3 {
        let id = format!("guard-connective/demo.req.Send/too-many/{nested}");
        assert_eq!(found.get(&id), Some(&Verdict::Killed), "{id}: {found:#?}");
    }
    let swap = report
        .mutants
        .iter()
        .find(|entry| entry.id == "precedence-swap/demo.req.Send/too-many/none-given")
        .unwrap_or_else(|| panic!("no precedence swap in {found:#?}"));
    assert_eq!(swap.verdict, Verdict::Equivalent, "{found:#?}");
    let overlap = swap
        .unsatisfiable_guard
        .as_deref()
        .expect("an equivalent swap names the overlap no input satisfies");
    assert!(
        overlap.contains("defined(header)"),
        "the overlap names both guards: {overlap}"
    );
    assert_eq!(report.counts.killed, 9, "{found:#?}");
    assert_eq!(report.counts.equivalent, 1, "{found:#?}");
}

/// The same shape over `Boolean` inputs.
const BOOLEAN_PAIRS: &str = r#"format: ess/23
system: catalog
version: v1
domain: catalog.orders

components:
  - component: orders-server
    owns:
      domains: [catalog.orders]
    accepts:
      commands: [catalog.orders.Place]
    reached_by: network

errors:
  - name: catalog.orders.TooManyReferences
  - name: catalog.orders.NoReference

commands:
  - name: catalog.orders.Place
    input:
      - {name: item_code, type: Boolean}
      - {name: item_sku, type: Boolean}
      - {name: item_url, type: Boolean}
    outcomes:
      - name: too-many
        when:
          any:
            - all: ["item_code == true", "item_sku == true"]
            - all: ["item_code == true", "item_url == true"]
            - all: ["item_sku == true", "item_url == true"]
        error: catalog.orders.TooManyReferences
      - name: none-given
        when:
          all: ["item_code == false", "item_sku == false", "item_url == false"]
        error: catalog.orders.NoReference
      - name: placed
        accepts: nothing
"#;

#[test]
fn the_boolean_variant_has_no_survivor() {
    let report = audit(BOOLEAN_PAIRS);
    assert_eq!(
        report.counts.survived,
        0,
        "survivors: {:#?}",
        survivors(&report)
    );
    connectives_killed(&report, 4);
}

/// `any` in `all` in `any`: the innermost disjunction and the conjunction around it are each
/// witnessed per child.
const THREE_DEEP: &str = r#"format: ess/23
system: deep
version: v1
domain: deep.ship

components:
  - component: ship-server
    owns:
      domains: [deep.ship]
    accepts:
      commands: [deep.ship.Send]
    reached_by: network

errors:
  - name: deep.ship.Held

commands:
  - name: deep.ship.Send
    input:
      - {name: express, type: Boolean}
      - {name: fragile, type: Boolean}
      - {name: bulky, type: Boolean}
      - {name: flagged, type: Boolean}
    outcomes:
      - name: held
        when:
          any:
            - all:
                - any: ["express == true", "fragile == true"]
                - "bulky == true"
            - "flagged == true"
        error: deep.ship.Held
      - name: sent
        accepts: nothing
"#;

#[test]
fn an_any_in_an_all_in_an_any_is_witnessed_per_child() {
    let report = audit(THREE_DEEP);
    connectives_killed(&report, 3);
}

/// An `all` under `not:`: its polarity is flipped, so the guarded branch witnesses it.
const NEGATED_ALL: &str = r#"format: ess/23
system: neg
version: v1
domain: neg.door

components:
  - component: door-server
    owns:
      domains: [neg.door]
    accepts:
      commands: [neg.door.Open]
    reached_by: network

errors:
  - name: neg.door.Refused

commands:
  - name: neg.door.Open
    input:
      - {name: badge, type: Boolean}
      - {name: pin, type: Boolean}
      - {name: alarm, type: Boolean}
    outcomes:
      - name: refused
        when:
          all:
            - not: {all: ["badge == true", "pin == true"]}
            - "alarm == false"
        error: neg.door.Refused
      - name: opened
        accepts: nothing
"#;

#[test]
fn an_all_under_not_is_witnessed_per_child() {
    let report = audit(NEGATED_ALL);
    connectives_killed(&report, 2);
}

/// Two refusals over the presence of one `Optional` input, which no input selects together.
const PRESENCE_PAIR: &str = r#"format: ess/23
system: pres
version: v1
domain: pres.note

components:
  - component: note-server
    owns:
      domains: [pres.note]
    accepts:
      commands: [pres.note.Post]
    reached_by: network

errors:
  - name: pres.note.HasTag
  - name: pres.note.NoTag

commands:
  - name: pres.note.Post
    input:
      - {name: tag, type: Optional<String>}
      - {name: draft, type: Boolean}
    outcomes:
      - name: has-tag
        when: {all: [{tag: {defined: true}}, "draft == true"]}
        error: pres.note.HasTag
      - name: no-tag
        when: {all: [{tag: {defined: false}}, "draft == true"]}
        error: pres.note.NoTag
      - name: posted
        accepts: nothing
"#;

#[test]
fn a_swap_of_disjoint_presence_guards_is_equivalent() {
    let report = audit(PRESENCE_PAIR);
    let swap = report
        .mutants
        .iter()
        .find(|entry| entry.id == "precedence-swap/pres.note.Post/has-tag/no-tag")
        .unwrap_or_else(|| panic!("no precedence swap in {:#?}", verdicts(&report)));
    assert_eq!(
        swap.verdict,
        Verdict::Equivalent,
        "{:#?}",
        verdicts(&report)
    );
    let overlap = swap.unsatisfiable_guard.as_deref().expect("the overlap");
    assert!(
        overlap.contains("defined(tag)") && overlap.contains("not (defined(tag))"),
        "{overlap}"
    );
}

/// Two refusals over the presence of two different `Optional` inputs, which both hold together.
const OVERLAPPING_PRESENCE: &str = r#"format: ess/23
system: both
version: v1
domain: both.note

components:
  - component: note-server
    owns:
      domains: [both.note]
    accepts:
      commands: [both.note.Post]
    reached_by: network

errors:
  - name: both.note.HasTag
  - name: both.note.HasTopic

commands:
  - name: both.note.Post
    input:
      - {name: tag, type: Optional<String>}
      - {name: topic, type: Optional<String>}
    outcomes:
      - name: has-tag
        when: {tag: {defined: true}}
        error: both.note.HasTag
      - name: has-topic
        when: {topic: {defined: true}}
        error: both.note.HasTopic
      - name: posted
        accepts: nothing
"#;

#[test]
fn a_swap_of_overlapping_presence_guards_is_scored_by_running() {
    let report = audit(OVERLAPPING_PRESENCE);
    let swap = report
        .mutants
        .iter()
        .find(|entry| entry.id == "precedence-swap/both.note.Post/has-tag/has-topic")
        .unwrap_or_else(|| panic!("no precedence swap in {:#?}", verdicts(&report)));
    assert_eq!(swap.verdict, Verdict::Killed, "{:#?}", verdicts(&report));
    assert_eq!(swap.unsatisfiable_guard, None);
    assert!(
        report
            .mutants
            .iter()
            .all(|entry| entry.verdict != Verdict::Equivalent),
        "a satisfiable presence guard is never called dead: {:#?}",
        verdicts(&report)
    );
}

/// `defined(tag) and (not defined(tag) or draft == true)`: the connective mutant of the inner
/// disjunction writes `defined(tag) and not defined(tag) and draft == true`, which no input
/// satisfies.
const DEAD_PRESENCE: &str = r#"format: ess/23
system: dead
version: v1
domain: dead.note

components:
  - component: note-server
    owns:
      domains: [dead.note]
    accepts:
      commands: [dead.note.Post]
    reached_by: network

errors:
  - name: dead.note.Held

commands:
  - name: dead.note.Post
    input:
      - {name: tag, type: Optional<String>}
      - {name: draft, type: Boolean}
    outcomes:
      - name: held
        when:
          all:
            - {tag: {defined: true}}
            - any: [{tag: {defined: false}}, "draft == true"]
        error: dead.note.Held
      - name: posted
        accepts: nothing
"#;

#[test]
fn a_mutant_guard_over_contradictory_presence_is_dead() {
    let report = audit(DEAD_PRESENCE);
    let dead: Vec<_> = report
        .mutants
        .iter()
        .filter(|entry| entry.class == MutantClass::GuardConnective)
        .filter(|entry| {
            entry
                .change
                .contains("(not (defined(tag)) and draft == true)")
        })
        .collect();
    assert_eq!(dead.len(), 1, "{:#?}", report.mutants);
    let entry = dead[0];
    assert!(
        matches!(entry.verdict, Verdict::Killed | Verdict::Equivalent),
        "{entry:#?}"
    );
    if entry.verdict == Verdict::Equivalent {
        assert!(entry.unsatisfiable_guard.is_some(), "{entry:#?}");
    }
    assert_eq!(report.counts.survived, 0, "{:#?}", survivors(&report));
}

/// The reproduction committed with the issue, read from `.engineering/repro/501/system.yaml`.
fn reproduction() -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../.engineering/repro/501/system.yaml");
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} cannot be read: {error}", path.display()))
}

/// The issue's own reproduction scores exactly one `equivalent` mutant, its precedence swap, and
/// that verdict is decided rather than assumed: with `none-given` narrowed until the two guards
/// share an input, the same swap names no unsatisfiable overlap and is not `equivalent`.
#[test]
fn the_committed_reproduction_scores_exactly_one_equivalent_precedence_swap() {
    const SWAP: &str = "precedence-swap/catalog.orders.Place/too-many/none-given";
    let text = reproduction();
    let report = audit(&text);
    let equivalent: Vec<&str> = report
        .mutants
        .iter()
        .filter(|entry| entry.verdict == Verdict::Equivalent)
        .map(|entry| entry.id.as_str())
        .collect();
    assert_eq!(equivalent, [SWAP], "{:#?}", verdicts(&report));
    assert_eq!(report.counts.equivalent, 1, "{:#?}", verdicts(&report));
    assert_eq!(
        report.counts.survived,
        0,
        "survivors: {:#?}",
        survivors(&report)
    );

    let none_given = "all: [{item_code: {defined: false}}, {item_sku: {defined: false}}, \
                      {item_url: {defined: false}}]";
    assert!(
        text.contains(none_given),
        "the reproduction changed: {text}"
    );
    let overlapping = text.replace(none_given, "{item_url: {defined: false}}");
    let report = audit(&overlapping);
    let swap = report
        .mutants
        .iter()
        .find(|entry| entry.id == SWAP)
        .unwrap_or_else(|| panic!("no precedence swap in {:#?}", verdicts(&report)));
    assert_eq!(
        swap.unsatisfiable_guard, None,
        "an input with item_code and item_sku but no item_url selects both branches"
    );
    assert_ne!(
        swap.verdict,
        Verdict::Equivalent,
        "{:#?}",
        verdicts(&report)
    );
}

/// `all: [s > "m", s > "k"]` nested in an `any`: no text is above `"m"` and not above `"k"`, so
/// the default's row where `s > "k"` alone fails exists nowhere, and an ordered text is a leaf the
/// search cannot show exhausted.
const UNSOLVED_CHILD: &str = r#"format: ess/23
system: text
version: v1
domain: text.word

components:
  - component: word-server
    owns:
      domains: [text.word]
    accepts:
      commands: [text.word.Sort]
    reached_by: network

errors:
  - name: text.word.Late

commands:
  - name: text.word.Sort
    input:
      - {name: s, type: String}
      - {name: c, type: Boolean}
    outcomes:
      - name: late
        when:
          any:
            - all: ["s > \"m\"", "s > \"k\""]
            - "c == true"
        error: text.word.Late
      - name: sorted
        accepts: nothing
"#;

/// A per-child row no search finds is named by an `ESS-SYNTH-022` refusal beside the scenario
/// that stands without it, never skipped silently.
#[test]
fn a_per_child_row_no_search_finds_is_refused_beside_its_scenario() {
    let (files, texts) = parsed(UNSOLVED_CHILD);
    let ir = mutate::compile(files, &texts).expect("the fixture compiles");
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let scenario = "text.word.Sort/outcome/sorted";
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == scenario),
        "the default's scenario stands"
    );
    let beside: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal.stands
                && refusal.code().to_string() == "ESS-SYNTH-022"
                && refusal
                    .scenario
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref()
                    == Some(scenario)
        })
        .map(ToString::to_string)
        .collect();
    assert!(
        beside.iter().any(
            |refusal| refusal.contains("leaves a connective's child unwitnessed")
                && refusal.contains("not (s > k)")
        ),
        "every refusal: {:#?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}

/// `all: [s > k, s > j]` beside `c == true` in an `any`, over three text inputs: the guarded row
/// where `c == true` alone holds is `s = "a"`, `k = "b"`, `c = true`, so the child is satisfiable,
/// but neither of the two candidates the bounded search tries is that row, and two facts compared
/// is a goal neither the exhaustive search nor the text-literal implication decides.
const SATISFIABLE_UNSOLVED_CHILD: &str = r#"format: ess/23
system: text
version: v1
domain: text.word

components:
  - component: word-server
    owns:
      domains: [text.word]
    accepts:
      commands: [text.word.Sort]
    reached_by: network

errors:
  - name: text.word.Late

commands:
  - name: text.word.Sort
    input:
      - {name: s, type: String}
      - {name: k, type: String}
      - {name: j, type: String}
      - {name: c, type: Boolean}
    outcomes:
      - name: late
        when:
          any:
            - all: ["s > k", "s > j"]
            - "c == true"
        error: text.word.Late
      - name: sorted
        accepts: nothing
"#;

/// A per-child row that exists but that the bounded search does not find is still refused with
/// `ESS-SYNTH-022`: deciding a goal empty by its text literals leaves a satisfiable one alone.
#[test]
fn a_satisfiable_per_child_row_the_search_misses_is_still_refused() {
    let (files, texts) = parsed(SATISFIABLE_UNSOLVED_CHILD);
    let ir = mutate::compile(files, &texts).expect("the fixture compiles");
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-022")
        .map(ToString::to_string)
        .collect();
    assert!(
        refused
            .iter()
            .any(|refusal| refusal
                .contains("`(not ((s > {fact: k} and s > {fact: j})) and c == true)`")),
        "every refusal: {:#?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}
