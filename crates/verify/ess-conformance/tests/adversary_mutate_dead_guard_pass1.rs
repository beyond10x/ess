//! Adversary pass 1 against the #218 unit (dead guards and unwitnessed outcomes).
//!
//! Each case states what the story or the coordinator's decisions require and drives the
//! implementation against it.
#![allow(
    clippy::too_many_lines,
    clippy::missing_panics_doc,
    clippy::needless_raw_string_hashes
)]

use std::collections::BTreeMap;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, Emission, MutantClass, MutantEntry, MutationReport, Verdict, BASELINE_DIR,
    MANIFEST_FILE, REPORT_FILE, SUITE_FILE,
};
use ess_conformance::runner::Runner;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

fn parsed(label: &str, text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert(label.to_owned(), text.to_owned());
    (vec![(Source::new(label.to_owned()), raw)], texts)
}

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

// ---- 1. soundness: a guard called dead that an admitted input satisfies ---------------------------

/// `flagged` is `any: [kind == Kf, tier == Tf, code != "C1"]` over two six-variant enums and a
/// `Code` whose invariant is `starts_with: "C"`. The root flip to `all:` is satisfied by
/// `{kind: Kf, tier: Tf, code: "Ccode"}` — every value of it is admitted by its type.
const GATED: &str = r#"format: ess/16
system: gate
version: v1
domain: gate.pass
summary: A pass whose flag depends on two closed choices and a constrained code.
naming:
  wire: pass
  display: Pass

types:
  - name: gate.pass.Kind
    kind: enum
    variants: [Ka, Kb, Kc, Kd, Ke, Kf]
  - name: gate.pass.Tier
    kind: enum
    variants: [Ta, Tb, Tc, Td, Te, Tf]
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
    naming:
      wire: issue_pass
      display: Issue a pass
    input:
      - name: kind
        type: gate.pass.Kind
      - name: tier
        type: gate.pass.Tier
      - name: code
        type: gate.pass.Code
    outcomes:
      - name: flagged
        when:
          any:
            - kind == Kf
            - tier == Tf
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
    naming:
      wire: pass_issued
      display: Pass issued
    fields:
      - name: pass_id
        type: Uuid
"#;

const GATED_FLIP: &str = "guard-connective/gate.pass.IssuePass/flagged/0";

fn gated() -> (Vec<Document>, SourceMap) {
    parsed("gated.yaml", GATED)
}

#[test]
fn a_guard_an_admitted_input_satisfies_is_not_called_dead() {
    // The witness: every value is admitted by its type (`Ccode` starts with `C`), and the flipped
    // guard holds of it, decided by the crate's own evaluator on the flipped model.
    let flipped_text = GATED.replace(
        "        when:\n          any:\n",
        "        when:\n          all:\n",
    );
    assert_ne!(flipped_text, GATED, "the flipped model was written");
    let (flipped_files, flipped_texts) = parsed("gated-flipped.yaml", &flipped_text);
    let flipped = mutate::compile(flipped_files, &flipped_texts).expect("the flip compiles");
    let command = flipped
        .commands()
        .values()
        .find(|it| it.name.to_string() == "gate.pass.IssuePass")
        .expect("the command");
    let outcome = command
        .outcomes
        .iter()
        .find(|it| it.name.to_string() == "flagged")
        .expect("the outcome");
    let guard = ess_conformance::when(outcome).expect("a guard");
    let witness: BTreeMap<String, ess_primitives::node::Node> = BTreeMap::from([
        (
            "kind".to_owned(),
            ess_primitives::node::Node::Text("Kf".to_owned()),
        ),
        (
            "tier".to_owned(),
            ess_primitives::node::Node::Text("Tf".to_owned()),
        ),
        (
            "code".to_owned(),
            ess_primitives::node::Node::Text("Ccode".to_owned()),
        ),
    ]);
    let facts = ess_conformance::input::flatten(&flipped, command, &witness)
        .unwrap_or_else(|it| panic!("{it:?}"));
    assert!(
        facts.decide(guard).is_satisfied(),
        "the flipped guard `{guard}` holds of the witness"
    );

    let (emission, written) = emitted(&gated(), &[MutantClass::GuardConnective], true);
    let flip = emission
        .manifest
        .mutants
        .iter()
        .find(|mutant| mutant.id == GATED_FLIP)
        .unwrap_or_else(|| panic!("no {GATED_FLIP} in {:#?}", emission.manifest.mutants));
    assert!(
        flip.change.contains(" and "),
        "the mutant is the any→all flip: {}",
        flip.change
    );
    assert_eq!(
        flip.unsatisfiable_guard, None,
        "{{kind: Kf, tier: Tf, code: \"Ccode\"}} satisfies the flipped guard, so it is not dead; \
         the manifest says it is"
    );
    let report = collect(&written);
    let scored = entry(&report, GATED_FLIP);
    assert_ne!(
        scored.verdict,
        Verdict::Equivalent,
        "a live rule nothing witnessed is scored equivalent: {scored:?}"
    );
}

#[test]
fn a_live_guard_synthesis_could_not_witness_stays_unwitnessed_and_fails_the_run() {
    // The same mutant, scored: nothing of its suite witnesses the live `all:` rule, so it is
    // `unwitnessed` (exit 3), not `equivalent` (which lets the run exit 0).
    let (_, written) = emitted(&gated(), &[MutantClass::GuardConnective], true);
    let report = collect(&written);
    let scored = entry(&report, GATED_FLIP);
    assert_eq!(scored.verdict, Verdict::Unwitnessed, "{scored:?}");
}

// ---- 2. a transition only a dead outcome performs -----------------------------------------------

/// `escalate` is performed only by `escalated`, whose guard no input satisfies, so the baseline
/// refuses every scenario of it. A mutant on that transition cannot be killed by either suite.
const LADDER: &str = r#"format: ess/16
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
"#;

#[test]
fn a_mutant_on_a_transition_only_a_dead_outcome_performs_is_not_survived() {
    let spec = parsed("ladder.yaml", LADDER);
    let (files, texts) = &spec;
    let ir = mutate::compile(files.clone(), texts).expect("the fixture compiles");
    let report = mutate::audit(
        files,
        texts,
        &[MutantClass::TransitionTo, MutantClass::FromDrop],
        || Interpreted::for_model(ir.clone()),
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    assert!(
        report.baseline.refusals > 0,
        "the baseline refuses the dead outcome: {report:#?}"
    );
    let on_escalate: Vec<&MutantEntry> = report
        .mutants
        .iter()
        .filter(|entry| entry.id.contains("desk.case.Case.escalate"))
        .collect();
    assert!(
        on_escalate
            .iter()
            .any(|entry| entry.verdict != Verdict::Stillborn),
        "a mutant on `escalate` ran: {:#?}",
        report.mutants
    );
    for entry in on_escalate {
        eprintln!(
            "{} {:?} added={:?} baseline={:?} stillborn={:?}",
            entry.id, entry.verdict, entry.added_refusals, entry.baseline_refusals, entry.stillborn
        );
        assert_ne!(
            entry.verdict,
            Verdict::Survived,
            "only the dead `escalated` outcome performs `escalate`, and the baseline refuses its \
             scenario; the correct target cannot kill this mutant, so it is not a survivor: \
             {entry:?}"
        );
    }
}

// ---- 3. the format versions the coordinator decided ---------------------------------------------

#[test]
fn the_new_fields_are_written_under_version_3() {
    let shop = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/mutation-gained-refusal.yaml"),
    )
    .expect("the fixture is readable");
    let (emission, written) = emitted(
        &parsed("shop.yaml", &shop),
        &[MutantClass::GuardConnective],
        true,
    );
    assert_eq!(
        emission.manifest.format, "ess-mutation-manifest/3",
        "`ess-mutation-manifest/2` shipped in 0.41.0 without `unsatisfiable_guard`"
    );
    let report = collect(&written);
    let json: serde_json::Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    assert_eq!(
        json["format"], "ess-mutation-report/3",
        "`ess-mutation-report/2` shipped in 0.41.0 without `equivalent`"
    );
}

#[test]
fn a_version_2_manifest_carrying_a_dead_guard_is_refused() {
    let shop = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/mutation-gained-refusal.yaml"),
    )
    .expect("the fixture is readable");
    let (_, mut written) = emitted(
        &parsed("shop.yaml", &shop),
        &[MutantClass::GuardConnective],
        true,
    );
    let mut manifest: serde_json::Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    manifest["format"] = "ess-mutation-manifest/2".into();
    assert!(
        manifest["mutants"]
            .as_array()
            .unwrap()
            .iter()
            .any(|it| it.get("unsatisfiable_guard").is_some()),
        "the emission names a dead guard"
    );
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
    let collected = mutate::collect(|path| written.get(path).cloned());
    assert!(
        collected.is_err(),
        "a 0.41.0 `/2` manifest has no `unsatisfiable_guard`; one carrying it is refused, as a `/1` \
         manifest carrying it is"
    );
}
