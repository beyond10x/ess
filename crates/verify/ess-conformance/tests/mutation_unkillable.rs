//! A mutant no scenario could kill is not scored `survived`.
//!
//! Issue #218, two shapes.
//!
//! 1. A guard mutant that leaves its outcome's guard satisfied by no input — `any: [x == A, x == B]`
//!    flipped to `all:` — makes the outcome dead by construction. Decided over the finite witness
//!    domain synthesis searches (the candidates `boundaries` decides over), such a mutant is
//!    `equivalent`, with the guard named as `unsatisfiable_guard`. A failing scored scenario still
//!    kills it; where the domain cannot decide the guard, the scoring is what it was.
//! 2. A mutant on an outcome the baseline suite already does not witness — its outcome scenario
//!    refused at synthesis — cannot be killed by either suite: it is `unwitnessed`, naming the
//!    baseline refusal as `baseline_refusals`.
//!
//! A mutant on a witnessed outcome that no scenario kills is still `survived`. Both the built-in
//! `--target` path and `--emit`/`--collect` are covered; the stand-in runner that passes whatever
//! it is given is fabricated by writing a passing report beside each emitted suite.

use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, Emission, MutantClass, MutantEntry, MutateCode, MutationReport, RefusalKey,
    Verdict, BASELINE_DIR, MANIFEST_FILE, MANIFEST_FORMAT, MANIFEST_FORMAT_1, MUTANT_FILE,
    REPORT_FILE, SUITE_FILE,
};
use ess_conformance::runner::Runner;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

const DEAD: &str = "guard-connective/shop.order.ReportStatus/settled/0";
const DEAD_GUARD: &str = "(status == Paid and status == Shipped)";
const ON_REFUSED: &str = "error-swap/desk.ticket.FileTicket/contradictory";
const ON_WITNESSED: &str = "error-swap/desk.ticket.FileTicket/refused";
const REFUSED_SCENARIO: &str = "desk.ticket.FileTicket/outcome/contradictory";
const REFUSED_SUBJECT: &str = "outcome desk.ticket.FileTicket/contradictory";

fn fixture(name: &str) -> (Vec<Document>, SourceMap) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(&path).expect("the fixture is readable");
    parsed(&path.display().to_string(), &text)
}

fn parsed(label: &str, text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert(label.to_owned(), text.to_owned());
    (vec![(Source::new(label.to_owned()), raw)], texts)
}

/// The #203 shop: `settled` is `any: [status == Paid, status == Shipped]`.
fn shop() -> (Vec<Document>, SourceMap) {
    fixture("mutation-gained-refusal.yaml")
}

/// A desk whose `contradictory` refusal no input reaches, so the baseline refuses its scenario.
fn desk() -> (Vec<Document>, SourceMap) {
    fixture("mutation-unwitnessed-outcome.yaml")
}

/// A text guard, decided over the candidate ladder rather than a closed enum's variants.
fn coded() -> (Vec<Document>, SourceMap) {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mutation-gained-refusal.yaml");
    let text = std::fs::read_to_string(&path)
        .expect("the fixture is readable")
        .replace(
            "      - name: status\n        type: shop.order.Status\n    outcomes:",
            "      - name: status\n        type: shop.order.Status\n      - name: code\n        \
             type: String\n    outcomes:",
        )
        .replace(
            "            - status == Paid\n            - status == Shipped",
            "            - code == \"paid\"\n            - code == \"shipped\"",
        );
    assert!(
        text.contains("code == \"paid\""),
        "the fixture was rewritten"
    );
    parsed(&format!("{}#coded", path.display()), &text)
}

/// An ordering guard: `amount >= 5 and amount <= 5`, whose boundary mutants no integer satisfies
/// but whose domain the candidates do not exhaust in general.
fn ordered() -> (Vec<Document>, SourceMap) {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mutation-gained-refusal.yaml");
    let text = std::fs::read_to_string(&path)
        .expect("the fixture is readable")
        .replace(
            "      - name: status\n        type: shop.order.Status\n    outcomes:",
            "      - name: status\n        type: shop.order.Status\n      - name: amount\n        \
             type: Integer\n    outcomes:",
        )
        .replace(
            "          any:\n            - status == Paid\n            - status == Shipped",
            "          all:\n            - amount >= 5\n            - amount <= 5",
        );
    assert!(text.contains("amount >= 5"), "the fixture was rewritten");
    parsed(&format!("{}#ordered", path.display()), &text)
}

fn audit(spec: &(Vec<Document>, SourceMap), classes: &[MutantClass]) -> MutationReport {
    let (files, texts) = spec;
    let ir = mutate::compile(files.clone(), texts).expect("the fixture compiles");
    mutate::audit(files, texts, classes, || Interpreted::for_model(ir.clone()))
        .unwrap_or_else(|refusal| panic!("{refusal}"))
}

/// The emission, with a report beside every suite from the unmutated model's interpreter; with
/// `stand_in`, every report says each scenario passed.
fn emitted(
    spec: &(Vec<Document>, SourceMap),
    classes: &[MutantClass],
    stand_in: bool,
) -> (Emission, BTreeMap<String, String>) {
    let (files, texts) = spec;
    let ir = mutate::compile(files.clone(), texts).expect("the fixture compiles");
    let emission = mutate::emit(files, texts, classes).expect("the fixture emits");
    let mut written = emission.files.clone();
    let mut dirs = vec![BASELINE_DIR.to_owned()];
    dirs.extend(
        emission
            .manifest
            .mutants
            .iter()
            .filter_map(|it| it.dir.clone()),
    );
    for dir in dirs {
        let admitted = AdmittedSuite::from_json(&emission.files[&format!("{dir}/{SUITE_FILE}")])
            .expect("an emitted suite is admitted");
        let run = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(ir.clone()));
        let text = CountReport::from_run(&run, &admitted)
            .unwrap()
            .to_canonical_json()
            .unwrap();
        let mut report: serde_json::Value = serde_json::from_str(&text).unwrap();
        if stand_in {
            // Deliberate collector input, not evidence that this target passed.
            let ids: Vec<_> = admitted.suite().scenarios.keys().collect();
            report["outcomes"] = serde_json::json!({
                "passed": ids, "failed": [], "error": [], "unsupported": [], "skipped": []
            });
            report["counts"] = serde_json::json!({
                "total": ids.len(), "passed": ids.len(), "failed": 0,
                "error": 0, "unsupported": 0, "skipped": 0
            });
            report["execution_status"] = "passed".into();
            report["conformance_status"] = "inconclusive".into();
        }
        let text = serde_json::to_string(&report).unwrap();
        CountReport::from_json(&text, &admitted).expect("coherent report/2 collector input");
        written.insert(format!("{dir}/{REPORT_FILE}"), text);
    }
    (emission, written)
}

fn collect(written: &BTreeMap<String, String>) -> MutationReport {
    mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"))
}

fn entry<'a>(report: &'a MutationReport, id: &str) -> &'a MutantEntry {
    report
        .mutants
        .iter()
        .find(|entry| entry.id == id)
        .unwrap_or_else(|| panic!("no mutant {id} in {:#?}", report.mutants))
}

fn refused_at_baseline() -> RefusalKey {
    RefusalKey {
        code: "ESS-SYNTH-003".to_owned(),
        scenario: Some(REFUSED_SCENARIO.to_owned()),
        subject: Some(REFUSED_SUBJECT.to_owned()),
    }
}

// ---- 1. a guard no input satisfies ---------------------------------------------------------------

#[test]
fn a_mutant_whose_guard_no_input_satisfies_is_equivalent_and_names_the_guard() {
    let report = collect(&emitted(&shop(), &[MutantClass::GuardConnective], true).1);
    let dead = entry(&report, DEAD);
    assert_eq!(dead.verdict, Verdict::Equivalent, "{dead:?}");
    assert_eq!(dead.unsatisfiable_guard.as_deref(), Some(DEAD_GUARD));
    assert_eq!(report.counts.equivalent, 1);
    assert_eq!(report.counts.survived, 0);
    assert_eq!(report.counts.unwitnessed, 0);
    let json: serde_json::Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    assert_eq!(json["format"], "ess-mutation-report/3");
    assert_eq!(json["counts"]["equivalent"], 1);
    assert_eq!(json["mutants"][0]["verdict"], "equivalent");
    assert_eq!(json["mutants"][0]["unsatisfiable_guard"], DEAD_GUARD);
}

#[test]
fn the_emitted_manifest_names_the_dead_guard() {
    let (emission, _) = emitted(&shop(), &[MutantClass::GuardConnective], true);
    assert_eq!(emission.manifest.format, MANIFEST_FORMAT);
    let mutant = emission
        .manifest
        .mutants
        .iter()
        .find(|mutant| mutant.id == DEAD)
        .expect("emitted");
    assert_eq!(mutant.unsatisfiable_guard.as_deref(), Some(DEAD_GUARD));
    let identity: serde_json::Value =
        serde_json::from_str(&emission.files[&format!("{DEAD}/{MUTANT_FILE}")]).unwrap();
    assert_eq!(identity["unsatisfiable_guard"], DEAD_GUARD);
}

#[test]
fn the_built_in_audit_decides_the_guard_and_a_failure_still_kills() {
    // The interpreter of the unmutated model answers the fallback scenario the mutant's suite
    // probes the dead region with, so it is killed; the dead guard is named all the same.
    let report = audit(&shop(), &[MutantClass::GuardConnective]);
    let dead = entry(&report, DEAD);
    assert_eq!(dead.verdict, Verdict::Killed, "{dead:?}");
    assert_eq!(dead.unsatisfiable_guard.as_deref(), Some(DEAD_GUARD));
    let collected = collect(&emitted(&shop(), &[MutantClass::GuardConnective], false).1);
    assert_eq!(entry(&collected, DEAD), dead);
}

#[test]
fn a_text_guard_is_decided_over_the_candidate_ladder() {
    let report = collect(&emitted(&coded(), &[MutantClass::GuardConnective], true).1);
    let dead = entry(&report, DEAD);
    assert_eq!(dead.verdict, Verdict::Equivalent, "{dead:?}");
    assert_eq!(
        dead.unsatisfiable_guard.as_deref(),
        Some("(code == paid and code == shipped)")
    );
}

#[test]
fn an_ordering_the_domain_does_not_decide_keeps_its_scoring() {
    // `amount > 5 and amount <= 5` holds of no integer, but an ordering's satisfying values may lie
    // between the candidates tried, so the domain does not decide it: the verdict is what it was.
    let spec = ordered();
    let collected = collect(&emitted(&spec, &[MutantClass::GuardBoundary], true).1);
    assert_ne!(collected.mutants.len(), 0);
    for entry in &collected.mutants {
        assert_eq!(entry.unsatisfiable_guard, None, "{entry:?}");
        assert_ne!(entry.verdict, Verdict::Equivalent, "{entry:?}");
        assert_ne!(entry.verdict, Verdict::Survived, "{entry:?}");
    }
    assert_eq!(collected.counts.equivalent, 0);
}

#[test]
fn a_guard_the_mutant_leaves_satisfiable_is_not_equivalent() {
    // `contradictory` is dead in the baseline; flipping or negating it makes it live, which is a
    // change a scenario can see, not a dead rule.
    let report = audit(
        &desk(),
        &[MutantClass::GuardConnective, MutantClass::GuardNegate],
    );
    for entry in &report.mutants {
        assert_eq!(entry.unsatisfiable_guard, None, "{entry:?}");
        assert_ne!(entry.verdict, Verdict::Equivalent, "{entry:?}");
    }
    assert_eq!(report.counts.equivalent, 0);
}

#[test]
fn the_text_names_the_equivalent_mutant_and_its_guard() {
    let text = collect(&emitted(&shop(), &[MutantClass::GuardConnective], true).1).render_text();
    let line = text
        .lines()
        .find(|line| line.starts_with(&format!("equivalent {DEAD}:")))
        .unwrap_or_else(|| panic!("no equivalent line in:\n{text}"));
    assert!(line.contains("ESS-MUTATE-005"), "{line}");
    assert!(
        line.contains(&format!("no input satisfies `{DEAD_GUARD}`")),
        "{line}"
    );
    assert!(
        text.lines().next().unwrap().contains("1 equivalent"),
        "{text}"
    );
}

#[test]
fn equivalent_has_its_own_code() {
    assert_eq!(MutateCode::Equivalent.code().to_string(), "ESS-MUTATE-005");
    assert_eq!(Verdict::Equivalent.as_str(), "equivalent");
}

#[test]
fn a_version_1_manifest_carrying_a_dead_guard_is_refused() {
    let (_, mut written) = emitted(&shop(), &[MutantClass::GuardConnective], true);
    let mut manifest: serde_json::Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    manifest["format"] = MANIFEST_FORMAT_1.into();
    manifest["baseline"]
        .as_object_mut()
        .unwrap()
        .remove("refused");
    for mutant in manifest["mutants"].as_array_mut().unwrap() {
        mutant.as_object_mut().unwrap().remove("refused");
    }
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("`unsatisfiable_guard` is not a /1 field");
    assert!(
        refusal.to_string().contains("unsatisfiable_guard"),
        "{refusal}"
    );
}

// ---- 2. an outcome the baseline does not witness ------------------------------------------------

#[test]
fn the_built_in_audit_scores_a_mutant_on_an_unwitnessed_outcome_unwitnessed() {
    let report = audit(&desk(), &[MutantClass::ErrorSwap]);
    assert_eq!(report.baseline.refusals, 1);
    let swapped = entry(&report, ON_REFUSED);
    assert_eq!(swapped.verdict, Verdict::Unwitnessed, "{swapped:?}");
    assert_eq!(
        swapped.baseline_refusals.as_deref(),
        Some(&[refused_at_baseline()][..])
    );
    assert_eq!(swapped.added_refusals, None);
    assert_eq!(report.counts.survived, 0);
    assert_eq!(report.counts.unwitnessed, 1);
    // The witnessed refusal's swap is killed by its own scenario, as before.
    let witnessed = entry(&report, ON_WITNESSED);
    assert_eq!(witnessed.verdict, Verdict::Killed, "{witnessed:?}");
    assert_eq!(witnessed.baseline_refusals, None);
}

#[test]
fn a_collected_mutant_on_an_unwitnessed_outcome_is_unwitnessed_and_a_witnessed_one_survives() {
    let report = collect(&emitted(&desk(), &[MutantClass::ErrorSwap], true).1);
    let swapped = entry(&report, ON_REFUSED);
    assert_eq!(swapped.verdict, Verdict::Unwitnessed, "{swapped:?}");
    assert_eq!(
        swapped.baseline_refusals.as_deref(),
        Some(&[refused_at_baseline()][..])
    );
    // The stand-in passes the witnessed refusal's scenario too: nothing killed a rule a scenario
    // could have seen, which is what a survivor is.
    let witnessed = entry(&report, ON_WITNESSED);
    assert_eq!(witnessed.verdict, Verdict::Survived, "{witnessed:?}");
    assert_eq!(witnessed.baseline_refusals, None);
    assert_eq!(witnessed.unsatisfiable_guard, None);
    assert_eq!(report.counts.survived, 1);
    assert_eq!(report.counts.unwitnessed, 1);
    let json: serde_json::Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    let on_refused = json["mutants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|it| it["id"] == ON_REFUSED)
        .unwrap();
    assert_eq!(
        on_refused["baseline_refusals"],
        serde_json::json!([{"code": "ESS-SYNTH-003", "scenario": REFUSED_SCENARIO, "subject": REFUSED_SUBJECT}])
    );
}

#[test]
fn the_text_names_the_baseline_refusal_beside_the_verdict() {
    let text = collect(&emitted(&desk(), &[MutantClass::ErrorSwap], true).1).render_text();
    let line = text
        .lines()
        .find(|line| line.starts_with(&format!("unwitnessed {ON_REFUSED}:")))
        .unwrap_or_else(|| panic!("no unwitnessed line in:\n{text}"));
    assert!(line.contains("ESS-MUTATE-004"), "{line}");
    assert!(
        line.contains(&format!(
            "the baseline refuses ESS-SYNTH-003 `{REFUSED_SCENARIO}`"
        )),
        "{line}"
    );
    assert!(
        text.lines()
            .any(|line| line.starts_with(&format!("survived {ON_WITNESSED}:"))),
        "{text}"
    );
}

// ---- 3. the formats, `inconclusive`, and transitions (adversary pass 1) --------------------------

#[test]
fn a_version_2_manifest_without_the_new_fields_still_collects_into_version_3() {
    let (_, mut written) = emitted(&shop(), &[MutantClass::GuardConnective], true);
    let mut manifest: serde_json::Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    manifest["format"] = "ess-mutation-manifest/2".into();
    for mutant in manifest["mutants"].as_array_mut().unwrap() {
        mutant
            .as_object_mut()
            .unwrap()
            .remove("unsatisfiable_guard");
    }
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
    let report = collect(&written);
    // Without the guard a 0.41.0 emission names, the #203 mutant is what 0.41.0 scored it.
    let dead = entry(&report, DEAD);
    assert_eq!(dead.verdict, Verdict::Unwitnessed, "{dead:?}");
    assert_eq!(dead.unsatisfiable_guard, None);
    let json: serde_json::Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    assert_eq!(json["format"], "ess-mutation-report/3");
}

#[test]
fn a_dead_guard_never_overrides_inconclusive() {
    assert_eq!(
        Verdict::Inconclusive.with_dead_guard(),
        Verdict::Inconclusive
    );
    assert_eq!(Verdict::Killed.with_dead_guard(), Verdict::Killed);
    assert_eq!(Verdict::Survived.with_dead_guard(), Verdict::Equivalent);
    assert_eq!(Verdict::Unwitnessed.with_dead_guard(), Verdict::Equivalent);
    // Collected: the dead mutant's report is missing, so nothing scored it and it stays
    // `inconclusive`, with the dead guard still named.
    let (_, mut written) = emitted(&shop(), &[MutantClass::GuardConnective], true);
    written.remove(&format!("{DEAD}/{REPORT_FILE}"));
    let report = collect(&written);
    let dead = entry(&report, DEAD);
    assert_eq!(dead.verdict, Verdict::Inconclusive, "{dead:?}");
    assert_eq!(dead.unsatisfiable_guard.as_deref(), Some(DEAD_GUARD));
    assert_eq!(report.counts.equivalent, 0);
}

/// A kind of six variants and a `Code` whose invariant is `starts_with: "C"`: `any: [kind == Kf,
/// code != "C1"]` flipped to `all:` is satisfied by `{Kf, "Cx"}`, which the base witness of `Code`
/// may not be.
const CONSTRAINED: &str = r#"format: ess/16
system: gate
version: v1
domain: gate.pass
summary: A pass whose flag depends on a closed choice and a constrained code.
naming:
  wire: pass
  display: Pass

types:
  - name: gate.pass.Kind
    kind: enum
    variants: [Ka, Kb, Kc, Kd, Ke, Kf]
  - name: gate.pass.Code
    kind: newtype
    of: String
    invariants:
      - value: {starts_with: "C"}

entities:
  - name: gate.pass.Pass
    identity:
      name: pass_id
      type: Uuid
    fields:
      - name: flagged
        type: Boolean
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]

commands:
  - name: gate.pass.IssuePass
    input:
      - name: kind
        type: gate.pass.Kind
      - name: code
        type: gate.pass.Code
    outcomes:
      - name: flagged
        when:
          any:
            - kind == Kf
            - code != "C1"
        creates: gate.pass.Pass
        instance: pass_id
        sets:
          flagged: true
        emits: [gate.pass.PassIssued]
        payload:
          gate.pass.PassIssued:
            pass_id: {generated: true}
      - name: plain
        creates: gate.pass.Pass
        instance: pass_id
        sets:
          flagged: false
        emits: [gate.pass.PassIssued]
        payload:
          gate.pass.PassIssued:
            pass_id: {generated: true}

events:
  - name: gate.pass.PassIssued
    fields:
      - name: pass_id
        type: Uuid
"#;

#[test]
fn a_guard_a_value_outside_the_literals_satisfies_is_not_dead() {
    let (emission, _) = emitted(
        &parsed("constrained.yaml", CONSTRAINED),
        &[MutantClass::GuardConnective],
        true,
    );
    let flip = emission
        .manifest
        .mutants
        .iter()
        .find(|mutant| mutant.id == "guard-connective/gate.pass.IssuePass/flagged/0")
        .unwrap_or_else(|| panic!("no flip in {:#?}", emission.manifest.mutants));
    assert_eq!(flip.unsatisfiable_guard, None, "{flip:?}");
}

/// `escalate` is moved only by `escalated`, dead at baseline; `close` only by `closed`, which a
/// baseline scenario takes.
const LADDER: &str = r"format: ess/16
system: desk
version: v1
domain: desk.case
summary: A case opened, then escalated by a rule no input reaches.
naming:
  wire: case
  display: Case

types:
  - name: desk.case.Priority
    kind: enum
    variants: [Low, High, Urgent]

entities:
  - name: desk.case.Case
    identity:
      name: case_id
      type: String
    fields:
      - name: note
        type: String
    lifecycle:
      initial: Open
      states: [Open, Parked, Escalated, Closed]
      terminal: [Closed]
      transitions:
        - {name: park, from: [Open], to: Parked}
        - {name: escalate, from: [Open, Parked], to: Escalated}
        - {name: close, from: [Open, Parked, Escalated], to: Closed}
        - {name: flag, from: [Parked], to: Escalated}

errors:
  - name: desk.case.WrongState
    fields: []

commands:
  - name: desk.case.OpenCase
    input:
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: desk.case.Case
        instance: case_id
        sets: {note: input.note}
        emits: [desk.case.CaseOpened]
        payload:
          desk.case.CaseOpened: {case_id: {generated: true}}
  - name: desk.case.Park
    input:
      - {name: id, type: String}
    outcomes:
      - {name: wrong-state, wrong_state: true, error: desk.case.WrongState}
      - name: parked
        moves: desk.case.Case.park
        instance: id
        emits: [desk.case.CaseReviewed]
        payload:
          desk.case.CaseReviewed: {case_id: input.id}
  - name: desk.case.Flag
    input:
      - {name: id, type: String}
    outcomes:
      - {name: wrong-state, wrong_state: true, error: desk.case.WrongState}
      - name: flagged
        moves: desk.case.Case.flag
        instance: id
        emits: [desk.case.CaseReviewed]
        payload:
          desk.case.CaseReviewed: {case_id: input.id}
  - name: desk.case.Review
    input:
      - {name: id, type: String}
      - {name: priority, type: desk.case.Priority}
    outcomes:
      - {name: wrong-state, wrong_state: true, error: desk.case.WrongState}
      - name: escalated
        when:
          all:
            - priority == High
            - priority == Urgent
        moves: desk.case.Case.escalate
        instance: id
        emits: [desk.case.CaseReviewed]
        payload:
          desk.case.CaseReviewed: {case_id: input.id}
      - name: closed
        moves: desk.case.Case.close
        instance: id
        emits: [desk.case.CaseReviewed]
        payload:
          desk.case.CaseReviewed: {case_id: input.id}

events:
  - name: desk.case.CaseOpened
    fields:
      - {name: case_id, type: String}
  - name: desk.case.CaseReviewed
    fields:
      - {name: case_id, type: String}
";

const TRANSITIONS: [MutantClass; 2] = [MutantClass::FromDrop, MutantClass::TransitionTo];

fn assert_transitions(report: &MutationReport) {
    assert!(
        report.baseline.refusals > 0,
        "the baseline refuses the dead outcome"
    );
    let on_escalate: Vec<&MutantEntry> = report
        .mutants
        .iter()
        .filter(|entry| entry.id.contains("desk.case.Case.escalate"))
        .collect();
    assert!(
        TRANSITIONS.iter().all(|class| on_escalate
            .iter()
            .any(|entry| entry.class == *class && entry.verdict != Verdict::Stillborn)),
        "both transition classes ran on `escalate`: {:#?}",
        report.mutants
    );
    for entry in on_escalate {
        if entry.verdict == Verdict::Stillborn || entry.verdict == Verdict::Killed {
            continue;
        }
        assert_eq!(entry.verdict, Verdict::Unwitnessed, "{entry:?}");
        let named = entry
            .baseline_refusals
            .as_deref()
            .unwrap_or_else(|| panic!("no baseline refusal named: {entry:?}"));
        assert!(
            named
                .iter()
                .all(|key| key.scenario.as_deref() == Some("desk.case.Review/outcome/escalated")),
            "{entry:?}"
        );
        assert_ne!(named.len(), 0);
    }
    // A transition a witnessed outcome performs keeps its scoring.
    for entry in report
        .mutants
        .iter()
        .filter(|entry| entry.id.contains("desk.case.Case.close"))
    {
        assert_eq!(entry.baseline_refusals, None, "{entry:?}");
    }
}

#[test]
fn the_built_in_audit_scores_a_transition_only_a_dead_outcome_performs_unwitnessed() {
    let spec = parsed("ladder.yaml", LADDER);
    assert_transitions(&audit(&spec, &TRANSITIONS));
}

#[test]
fn a_collected_transition_only_a_dead_outcome_performs_is_unwitnessed() {
    let spec = parsed("ladder.yaml", LADDER);
    let report = collect(&emitted(&spec, &TRANSITIONS, true).1);
    assert_transitions(&report);
    assert!(
        report
            .mutants
            .iter()
            .filter(|entry| entry.id.contains("desk.case.Case.close"))
            .any(|entry| entry.verdict == Verdict::Survived),
        "the stand-in passes a witnessed transition's mutant, which survives: {:#?}",
        report.mutants
    );
}
