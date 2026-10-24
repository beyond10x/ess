//! Adversary pass 1 on U14: a shadowed `when_related:` refusal refused by authoring
//! (`ESS-AUTHOR-041`), and a related guard over an absent optional input witnessed by synthesis.
//!
//! `tests/fixtures/adversary-u14/lockers.yaml` holds refusals over one related box row whose
//! guards imply each other semantically but not by shape. The absent-input cases reuse
//! `tests/fixtures/related-guard-absent-input.yaml` and mutate its guard.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Instant;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite as Suite;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;

const BOXES: &str = include_str!("fixtures/adversary-u14/lockers.yaml");
const SHADOWED: &str = include_str!("fixtures/related-refusal-shadowed.yaml");
const ABSENT: &str = include_str!("fixtures/related-guard-absent-input.yaml");
const GUARD: &str = r#"any: ["not defined(input.tenant)", tenant != input.tenant]"#;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(SpecSource::new("adversary-u14.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error}"))
}

fn authoring(model: &EssIr, text: &str) -> Authoring {
    compile_authored(model, &[Source::new("scenario.yaml", text)])
}

fn refusal_texts(compiled: &Authoring) -> Vec<(String, String)> {
    compiled
        .refusals
        .iter()
        .map(|refusal| (refusal.code().to_string(), refusal.to_string()))
        .collect()
}

/// One `PutItem` act on a box installed at `site` with `slots` free slots, closed first when
/// `close`, expecting `expect` (an `outcome:` line).
fn put_item(site: &str, slots: i64, close: bool, expect: &str) -> String {
    let mut acts = format!(
        "  - at: 2026-01-05T09:00:00Z\n    command: depots.box.InstallBox\n    \
         input: {{site: {site}, slots: {slots}}}\n    outcome: installed\n    \
         capture: {{instance: box, event: depots.box.BoxInstalled, field: box_id}}\n"
    );
    if close {
        acts.push_str(
            "  - at: 2026-01-05T09:00:01Z\n    command: depots.box.CloseBox\n    \
             input: {box_id: {$instance: box}}\n    outcome: closed\n",
        );
    }
    write!(
        acts,
        "  - at: 2026-01-05T09:00:02Z\n    command: depots.box.PutItem\n    \
         input: {{item_code: IT-1, box_id: {{$instance: box}}}}\n    outcome: {expect}\n"
    )
    .expect("writing to a String cannot fail");
    format!(
        "type: ess-scenario/1\ndomain: depots.box\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\n\
         arrange:\n  - instance: box\n    entity: depots.box.Box\ntimeline:\n{acts}"
    )
}

/// The authored scenario run alone against the interpreter of `model`; its status.
fn run_authored(model: &EssIr, text: &str) -> Status {
    let authored = authoring(model, text);
    assert!(authored.is_complete(), "{:?}", refusal_texts(&authored));
    let mut suite = ess_conformance::synthesize::synthesize(model).suite;
    suite.scenarios = authored.scenarios;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report();
    let run = report
        .scenarios
        .iter()
        .find(|run| run.scenario.to_string().ends_with("authored/a-scenario"))
        .unwrap_or_else(|| panic!("the authored scenario ran: {report:?}"));
    run.status
}

/// The act is refused with `ESS-AUTHOR-041` naming `earlier`; the interpreter answers `earlier`
/// on the same arrangement, so nothing is left for the interpreted run to fail.
fn shadowed(model: &EssIr, site: &str, slots: i64, close: bool, expect: &str, earlier: &str) {
    let answered = put_item(site, slots, close, earlier);
    assert_eq!(
        run_authored(model, &answered),
        Status::Passed,
        "the interpreter answers `{earlier}` on this arrangement"
    );
    let compiled = authoring(model, &put_item(site, slots, close, expect));
    let refused = refusal_texts(&compiled);
    assert!(
        refused
            .iter()
            .any(|(code, text)| code == "ESS-AUTHOR-041" && text.contains(&format!("`{earlier}`"))),
        "an act expecting `{expect}` where `{earlier}` answers first is refused, naming it; \
         refusals: {refused:#?}"
    );
}

#[test]
fn a_stricter_numeric_related_guard_is_shadowed_by_the_weaker_earlier_one() {
    // `free_slots < 1` implies `free_slots < 5`: `few-slots` answers wherever `no-slots` would.
    let model = ir(BOXES);
    shadowed(&model, "North", 0, false, "no-slots", "few-slots");
}

#[test]
fn a_negated_boolean_related_guard_is_shadowed_by_the_equal_earlier_one() {
    // On a non-optional Boolean, `accepting != true` implies `accepting == false`.
    let model = ir(BOXES);
    shadowed(&model, "North", 10, true, "not-accepting", "closed");
}

#[test]
fn a_weaker_related_guard_that_does_not_imply_the_earlier_one_stays_admitted_and_passes() {
    // `{any: [accepting == false, site == Depot]}` does not imply `accepting == false`: an open
    // box at the depot reaches `closed-or-depot`.
    let model = ir(BOXES);
    let text = put_item("Depot", 10, false, "closed-or-depot");
    let compiled = authoring(&model, &text);
    assert!(compiled.is_complete(), "{:#?}", refusal_texts(&compiled));
    assert_eq!(run_authored(&model, &text), Status::Passed);
}

#[test]
#[ignore = "story:the-first-of-overlapping-related-refusals-is-synthesized"]
fn the_shadowed_reproduction_synthesizes_its_reachable_refusals() {
    // `staff-closed` (a closed locker, recipient staff) and `locker-closed` (a closed locker,
    // another recipient) are both reachable: an authored act reaching `locker-closed` passes on
    // the interpreter (`authored_outcome_guards.rs`). Synthesis must witness them.
    let model = ir(SHADOWED);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{:?} {}: {}",
                refusal.scenario.as_ref().map(ToString::to_string),
                refusal.cause.code(),
                refusal.cause
            )
        })
        .collect();
    let reachable: Vec<&String> = refused
        .iter()
        .filter(|text| text.contains("/staff-closed") || text.contains("/locker-closed"))
        .collect();
    assert!(reachable.is_empty(), "{reachable:#?}");
}

/// Each scenario of `result` run against the interpreter of `target`.
fn statuses(result: &Synthesis, target: &EssIr) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(target.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn guarded(guard: &str) -> String {
    let out = ABSENT.replace(GUARD, guard);
    assert_ne!(out, ABSENT, "the fixture writes `{GUARD}`");
    out
}

fn failed(map: &BTreeMap<String, Status>) -> Vec<String> {
    map.iter()
        .filter(|(_, status)| **status == Status::Failed)
        .map(|(id, _)| id.clone())
        .collect()
}

#[test]
fn a_target_answering_made_for_a_tenanted_parent_and_no_input_tenant_fails_the_suite() {
    // This target differs from the specification only where the parent holds a tenant and the
    // input leaves `tenant` out: the specification refuses (`not defined(input.tenant)`), the
    // target creates the child, as one that defaults an absent tenant to the parent's would.
    let model = ir(ABSENT);
    let result = ess_conformance::synthesize::synthesize(&model);
    let mutant = ir(&guarded(
        r#"any: [{all: ["not defined(input.tenant)", "not defined(tenant)"]}, {all: ["defined(input.tenant)", tenant != input.tenant]}]"#,
    ));
    let ran = statuses(&result, &mutant);
    let failed = failed(&ran);
    assert!(
        failed
            .iter()
            .any(|id| id.starts_with("tenrel.t.MakeChild/")),
        "no MakeChild scenario fails against the defaulting target: {ran:#?}"
    );
}

fn canonical(suite: &Suite) -> String {
    suite
        .to_canonical_json()
        .unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn synthesis_of_the_absent_input_and_shadowed_models_is_deterministic() {
    for text in [ABSENT, SHADOWED, BOXES] {
        let model = ir(text);
        let first = ess_conformance::synthesize::synthesize(&model);
        let second = ess_conformance::synthesize::synthesize(&model);
        assert_eq!(canonical(&first.suite), canonical(&second.suite));
        assert_eq!(
            format!("{:?}", first.refusals),
            format!("{:?}", second.refusals)
        );
    }
}

const MANY: &str = r#"any: ["not defined(input.tenant)", "not defined(input.region)", "not defined(input.zone)", "not defined(input.lane)", tenant != input.tenant]"#;

/// The absent-input model with three more optional inputs on `MakeChild` and `guard` on
/// `other-tenant`.
fn many(guard: &str) -> String {
    let inputs = "      - {name: tenant, type: Optional<String>, presence: omitted_when_absent}\n      - {name: parent_id";
    let wider = "      - {name: tenant, type: Optional<String>, presence: omitted_when_absent}\n      \
                 - {name: region, type: Optional<String>, presence: omitted_when_absent}\n      \
                 - {name: zone, type: Optional<String>, presence: omitted_when_absent}\n      \
                 - {name: lane, type: Optional<String>, presence: omitted_when_absent}\n      - {name: parent_id";
    let text = guarded(guard).replace(inputs, wider);
    assert!(
        text.contains("name: lane"),
        "MakeChild's inputs were widened"
    );
    text
}

#[test]
fn a_related_guard_over_several_absent_inputs_is_witnessed_and_each_disjunct_is_needed() {
    let model = ir(&many(MANY));
    let started = Instant::now();
    let result = ess_conformance::synthesize::synthesize(&model);
    eprintln!(
        "synthesis over four absent inputs took {:?}",
        started.elapsed()
    );
    let refused: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect();
    assert!(refused.is_empty(), "{refused:#?}");
    let ran = statuses(&result, &model);
    let bad: Vec<_> = ran.iter().filter(|(_, s)| **s != Status::Passed).collect();
    assert!(bad.is_empty(), "{bad:#?}");

    let mut survivors = Vec::new();
    for input in ["region", "zone", "lane"] {
        let dropped = MANY.replace(&format!("\"not defined(input.{input})\", "), "");
        let mutant = ir(&many(&dropped));
        let ran = statuses(&result, &mutant);
        // `Unsupported` on a required scenario fails conformance as `Failed` does (§28).
        if ran.values().all(|status| *status == Status::Passed) {
            survivors.push((input, ran));
        }
    }
    assert!(
        survivors.is_empty(),
        "a target ignoring `not defined(input.<name>)` passes the suite: {survivors:#?}"
    );
}
