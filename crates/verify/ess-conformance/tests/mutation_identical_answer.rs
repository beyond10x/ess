//! A `precedence-swap` of two branches whose observable answers are identical — the same error,
//! the same or no payload, and no state change, sets or events — is `equivalent`
//! (`ESS-MUTATE-005`), with the shared answer recorded apart from `unsatisfiable_guard`
//! (<https://github.com/beyond10x/ess/issues/517>). Two refusals that differ in error or payload
//! keep their scoring
//! (<https://github.com/beyond10x/ess/issues/472>).
#![allow(clippy::missing_panics_doc)]

use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, Emission, MutantClass, MutationReport, BASELINE_DIR, MANIFEST_FILE,
    REPORT_FILE, SUITE_FILE,
};
use ess_conformance::runner::Runner;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::Value;

const OPEN: &str = "precedence-swap/desk.entry.Open/leading-hold/trailing-hold";
const CHECK: &str = "precedence-swap/desk.entry.Check/leading-hold/trailing-hold";
const GRADE: &str = "precedence-swap/desk.entry.Grade/too-low/low";

fn fixture() -> (Vec<Document>, SourceMap) {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mutation-identical-answer.yaml");
    let text = std::fs::read_to_string(&path).expect("the fixture is readable");
    let label = "mutation-identical-answer.yaml".to_owned();
    let raw = RawSpecFile::parse(&text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text);
    (vec![(Source::new(label), raw)], texts)
}

fn audit(classes: &[MutantClass]) -> MutationReport {
    let (files, texts) = fixture();
    let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
    mutate::audit(&files, &texts, classes, || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn mutant<'a>(json: &'a Value, id: &str) -> &'a Value {
    json["mutants"]
        .as_array()
        .expect("mutants")
        .iter()
        .find(|it| it["id"] == id)
        .unwrap_or_else(|| panic!("no {id} in {json:#}"))
}

/// The emission of `classes`, with a report beside every suite: the interpreter's, where every
/// scenario it ran is then recorded as passed. Deliberate collector input, not evidence that this
/// target passed.
fn emitted(classes: &[MutantClass]) -> (Emission, BTreeMap<String, String>) {
    emitted_for(classes, None)
}

/// [`emitted`], scoped to `component` where one is named.
fn emitted_for(
    classes: &[MutantClass],
    component: Option<&str>,
) -> (Emission, BTreeMap<String, String>) {
    let (files, texts) = fixture();
    let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
    let emission = mutate::emit_for(&files, &texts, classes, component).expect("the fixture emits");
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
        let mut report: Value = serde_json::from_str(&text).unwrap();
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
        let text = serde_json::to_string(&report).unwrap();
        CountReport::from_json(&text, &admitted).expect("coherent report/2 collector input");
        written.insert(format!("{dir}/{REPORT_FILE}"), text);
    }
    (emission, written)
}

const SHARED: &str =
    "refuse with `desk.entry.Invalid` and no payload; neither changes state, sets or emits anything";

#[test]
fn a_swap_of_two_refusals_answering_alike_is_equivalent_naming_the_shared_answer() {
    let report = audit(&[MutantClass::PrecedenceSwap]);
    let text = report.render_text();
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    let open = mutant(&json, OPEN);
    assert_eq!(open["verdict"], "equivalent", "{text}");
    assert_eq!(open["identical_answer"], SHARED, "{text}");
    assert!(
        open.get("unsatisfiable_guard").is_none(),
        "the guards overlap on `hold hold`; what makes the order moot is the answer, not a dead \
         overlap: {open:#}"
    );
    assert_eq!(
        json["format"], "ess-mutation-report/7",
        "`ess-mutation-report/4`, which 0.53.0 to 0.56.0 wrote, has no `identical_answer`; `/7`, \
         because `CHECK` carries `unchanged_suite`"
    );
    assert!(
        text.lines()
            .any(|line| line.starts_with(&format!("equivalent {OPEN}: "))
                && line.ends_with(&format!(" — ESS-MUTATE-005: both branches {SHARED}"))),
        "{text}"
    );
}

#[test]
fn refusals_differing_in_error_or_payload_keep_their_scoring() {
    let report = audit(&[MutantClass::PrecedenceSwap]);
    let text = report.render_text();
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    for (id, verdict) in [
        // The same error, a payload read from different inputs that no synthesized scenario tells
        // apart: its suite is the baseline's (https://github.com/beyond10x/ess/issues/586).
        (CHECK, "unwitnessed"),
        // Different errors, overlapping below 10 (https://github.com/beyond10x/ess/issues/472).
        (GRADE, "killed"),
    ] {
        let entry = mutant(&json, id);
        assert_eq!(entry["verdict"], verdict, "{id}\n{text}");
        assert!(entry.get("identical_answer").is_none(), "{id}: {entry:#}");
    }
}

#[test]
fn the_manifest_and_the_collected_report_name_the_shared_answer_apart_from_a_dead_guard() {
    let (emission, written) = emitted(&[MutantClass::PrecedenceSwap]);
    let manifest: Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    assert_eq!(manifest["format"], "ess-mutation-manifest/5");
    let open = mutant(&manifest, OPEN);
    assert_eq!(open["identical_answer"], SHARED, "{open:#}");
    assert!(open.get("unsatisfiable_guard").is_none(), "{open:#}");
    for id in [CHECK, GRADE] {
        assert!(
            mutant(&manifest, id).get("identical_answer").is_none(),
            "{id}"
        );
    }
    let mutant_file: Value = serde_json::from_str(&emission.files[&format!("{OPEN}/mutant.json")])
        .expect("the mutant's own entry");
    assert_eq!(mutant_file["identical_answer"], SHARED);

    let report =
        mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"));
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    assert_eq!(json["format"], "ess-mutation-report/7");
    let open = mutant(&json, OPEN);
    assert_eq!(open["verdict"], "equivalent", "{}", report.render_text());
    assert_eq!(open["identical_answer"], SHARED);
    let check = mutant(&json, CHECK);
    assert_eq!(check["verdict"], "unwitnessed", "{}", report.render_text());
    assert_eq!(check["unchanged_suite"], true, "{check:#}");
}

#[test]
fn a_version_3_manifest_carrying_identical_answer_is_refused() {
    let (emission, mut written) = emitted(&[MutantClass::GuardNegate]);
    assert_eq!(emission.manifest.format, "ess-mutation-manifest/3");
    let mut manifest: Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    let carrier = manifest["mutants"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|it| it.get("dir").is_some())
        .expect("a mutant with a suite");
    carrier["identical_answer"] = SHARED.into();
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("`ess-mutation-manifest/3` has no `identical_answer`");
    let said = refusal.to_string();
    assert!(
        said.contains("`identical_answer`") && said.contains("ess-mutation-manifest/5"),
        "{said}"
    );
}

#[test]
fn identical_answer_on_a_mutant_that_swaps_nothing_is_refused() {
    let (emission, mut written) = emitted(&[MutantClass::GuardNegate, MutantClass::PrecedenceSwap]);
    assert_eq!(emission.manifest.format, "ess-mutation-manifest/5");
    let mut manifest: Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    let carrier = manifest["mutants"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|it| it["class"] == "guard-negate" && it.get("dir").is_some())
        .expect("a guard-negate mutant with a suite");
    carrier["identical_answer"] = SHARED.into();
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("only a `precedence-swap` has two answers to compare");
    let said = refusal.to_string();
    assert!(
        said.contains("`identical_answer`") && said.contains("precedence-swap"),
        "{said}"
    );
}

#[test]
fn a_version_4_manifest_carrying_identical_answer_is_refused_naming_version_5() {
    // 0.53.0 to 0.56.0 wrote `ess-mutation-manifest/4` without `identical_answer`; a member new to
    // a released format changes its meaning, so it has a version of its own.
    let (emission, mut written) = emitted(&[MutantClass::PrecedenceSwap]);
    assert!(
        emission
            .manifest
            .mutants
            .iter()
            .any(|it| it.identical_answer.is_some()),
        "the swap of `{OPEN}` answers alike"
    );
    let manifest = written[MANIFEST_FILE].replace(
        &format!("\"{}\"", emission.manifest.format),
        "\"ess-mutation-manifest/4\"",
    );
    assert!(
        manifest.contains("\"ess-mutation-manifest/4\""),
        "{manifest}"
    );
    written.insert(MANIFEST_FILE.to_owned(), manifest);
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("`ess-mutation-manifest/4` has no `identical_answer`");
    let said = refusal.to_string();
    assert!(
        said.contains("`identical_answer`")
            && said.contains("`ess-mutation-manifest/4`")
            && said.contains("`ess-mutation-manifest/5`"),
        "{said}"
    );
}

/// `actual` is byte for byte the fixture `name`; where it is not, the bytes written go beside the
/// test binary's scratch directory for a reviewer to compare.
fn assert_fixture_bytes(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let expected = std::fs::read_to_string(&path).unwrap_or_default();
    if expected != actual {
        let written = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        std::fs::write(&written, actual).expect("the scratch directory is writable");
        panic!(
            "{name} is not the bytes written; they are at {}",
            written.display()
        );
    }
}

#[test]
fn an_emission_whose_mutants_carry_no_identical_answer_keeps_version_4_bytes() {
    // Scoped to its component, so `/4`; no `guard-negate` mutant has two answers to compare. The
    // fixtures are the bytes this emission and its collection wrote before version 5 existed.
    let (emission, written) = emitted_for(&[MutantClass::GuardNegate], Some("desk-server"));
    assert_eq!(emission.manifest.format, "ess-mutation-manifest/4");
    assert_fixture_bytes("mutation-manifest-4-scoped.json", &written[MANIFEST_FILE]);
    let report =
        mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"));
    assert_eq!(report.format, "ess-mutation-report/4");
    // The interpreter names this crate's version, which the fixture writes as `VERSION`.
    let text = report.to_canonical_json().replace(
        &format!("\"interpreted {}\"", env!("CARGO_PKG_VERSION")),
        "\"interpreted VERSION\"",
    );
    assert_fixture_bytes("mutation-report-4-scoped.json", &text);
}

/// `Record` refuses a prompt starting with `hold ` and one ending with ` hold` with the same error
/// and no payload; the binding recording each placed order answers its refusals with `POLICY`.
const LEDGER: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.ledger
errors:
  - name: demo.ledger.Invalid
    fields: []
events:
  - name: demo.ledger.OrderPlaced
    fields:
      - {name: prompt, type: String}
  - name: demo.ledger.RecordEscalated
    fields:
      - {name: prompt, type: String}
commands:
  - name: demo.ledger.Place
    input:
      - {name: prompt, type: String}
    outcomes:
      - name: placed
        emits: [demo.ledger.OrderPlaced]
        payload:
          demo.ledger.OrderPlaced: {prompt: input.prompt}
  - name: demo.ledger.Record
    input:
      - {name: prompt, type: String}
    outcomes:
      - name: leading-hold
        when: {prompt: {starts_with: "hold "}}
        error: demo.ledger.Invalid
      - name: trailing-hold
        when: {prompt: {ends_with: " hold"}}
        error: demo.ledger.Invalid
      - name: recorded
        accepts: nothing
bindings:
  - id: notify-ledger
    when:
      event: demo.ledger.OrderPlaced
    invoke:
      command: demo.ledger.Record
    mapping:
      prompt: event.prompt
    delivery: at_least_once
POLICY"#;

const RECORD: &str = "precedence-swap/demo.ledger.Record/leading-hold/trailing-hold";

/// The `identical_answer` of the `Record` swap where the binding answers refusals with `policy`.
fn ledger_answer(policy: &str) -> Option<String> {
    let text = LEDGER.replace("POLICY", policy);
    let label = "ledger.yaml".to_owned();
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text);
    let files = vec![(Source::new(label), raw)];
    let ir = mutate::compile(files.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let report = mutate::audit(&files, &texts, &[MutantClass::PrecedenceSwap], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    report
        .mutants
        .iter()
        .find(|entry| entry.id == RECORD)
        .unwrap_or_else(|| panic!("{RECORD}: {}", report.render_text()))
        .identical_answer
        .clone()
}

/// A binding's failure policy is selected by the refusal's name, so the two refusals answer alike
/// only where every binding invoking the command answers both alike: the same selected policy and
/// the same membership of a bounded retry's `final` outcomes
/// (<https://github.com/beyond10x/ess/issues/515>).
#[test]
fn a_swap_of_refusals_a_binding_answers_differently_carries_no_identical_answer() {
    let carried: Vec<&str> = [
        (
            "drop one, escalate the other",
            "    on_failure:\n      drop: [leading-hold]\n      escalate:\n        emits: demo.ledger.RecordEscalated\n        except: [leading-hold]\n",
        ),
        (
            "one universal policy, a bounded retry final on one",
            "    on_failure:\n      retry: {attempts: 3, final: [leading-hold]}\n",
        ),
        (
            "one selected retry, final on one",
            "    on_failure:\n      retry: {outcomes: [leading-hold, trailing-hold], attempts: 3, final: [leading-hold]}\n      drop: {except: [leading-hold, trailing-hold]}\n",
        ),
    ]
    .into_iter()
    .filter(|(_, policy)| ledger_answer(policy).is_some())
    .map(|(case, _)| case)
    .collect();
    assert!(
        carried.is_empty(),
        "carried `identical_answer`: {carried:?}"
    );
}

/// The control: where the binding answers both refusals alike, the swap keeps its identical
/// answer, under one universal policy or a refusal-selected one.
#[test]
fn a_swap_of_refusals_every_binding_answers_alike_keeps_its_identical_answer() {
    let shared =
        "refuse with `demo.ledger.Invalid` and no payload; neither changes state, sets or \
                  emits anything";
    for (case, policy) in [
        ("one universal policy", "    on_failure: drop\n"),
        (
            "a universal bounded retry final on both",
            "    on_failure:\n      retry: {attempts: 3, final: [demo.ledger.Invalid]}\n",
        ),
        (
            "a selected policy answering both alike",
            "    on_failure:\n      drop: [leading-hold, trailing-hold]\n      escalate:\n        emits: demo.ledger.RecordEscalated\n        except: [leading-hold, trailing-hold]\n",
        ),
    ] {
        assert_eq!(ledger_answer(policy).as_deref(), Some(shared), "{case}");
    }
}

/// <https://github.com/beyond10x/ess/issues/539>: `done-paused-default` takes every input both
/// `done-paused` (`pause == true`) and `done-default` (`code` absent or `""`) select, so which of the
/// two is declared first decides nothing. `EARLIER` is the rest of `done-paused-default`'s guard.
const JOB: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.job
entities:
  - name: demo.job.Job
    identity: {name: job_id, type: Uuid}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - {name: demo.job.JobNotFound, summary: No such job., fields: []}
events:
  - name: demo.job.Opened
    fields: [{name: job_id, type: Uuid}]
  - {name: demo.job.PausedDefault, fields: []}
  - {name: demo.job.Paused, fields: []}
  - {name: demo.job.Defaulted, fields: []}
  - {name: demo.job.Done, fields: []}
commands:
  - name: demo.job.Open
    input:
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: demo.job.Job
        instance: job_id
        emits: [demo.job.Opened]
        payload: {demo.job.Opened: {job_id: {generated: true}}}
        sets: {note: input.note}
  - name: demo.job.Finish
    input:
      - {name: job_id, type: Uuid}
      - {name: note, type: String}
      - {name: pause, type: Boolean}
      - {name: code, type: Optional<String>}
    outcomes:
      - name: done-paused-default
EARLIER        updates: demo.job.Job
        instance: job_id
        emits: [demo.job.PausedDefault]
        sets: {note: input.note}
      - name: done-paused
        when: pause == true
        updates: demo.job.Job
        instance: job_id
        emits: [demo.job.Paused]
        sets: {note: input.note}
      - name: done-default
        when: {any: [{code: {defined: false}}, 'code == ""']}
        updates: demo.job.Job
        instance: job_id
        emits: [demo.job.Defaulted]
        sets: {note: input.note}
      - name: done
        moves: demo.job.Job.close
        instance: job_id
        emits: [demo.job.Done]
        sets: {note: input.note}
      - {name: not-found, unknown_instance: true, error: demo.job.JobNotFound}
"#;

/// `done-paused-default` as the issue declares it: `pause == true`, and `code` absent or `""`.
const COVERING: &str =
    "        when: {all: ['pause == true', {any: [{code: {defined: false}}, 'code == \"\"']}]}\n";

const SWAP: &str = "precedence-swap/demo.job.Finish/done-paused/done-default";

/// The `JOB` model with `earlier` as the rest of `done-paused-default`'s guard.
fn job(earlier: &str) -> (Vec<Document>, SourceMap) {
    let text = JOB.replace("EARLIER", earlier);
    let label = "job.yaml".to_owned();
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text);
    (vec![(Source::new(label), raw)], texts)
}

/// The audit of `JOB`'s precedence swaps with `earlier`, as JSON, and its text.
fn job_audit(earlier: &str) -> (Value, String) {
    let (files, texts) = job(earlier);
    let ir = mutate::compile(files.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let report = mutate::audit(&files, &texts, &[MutantClass::PrecedenceSwap], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let text = report.render_text();
    (
        serde_json::from_str(&report.to_canonical_json()).unwrap(),
        text,
    )
}

#[test]
fn a_swap_whose_overlap_an_earlier_branch_takes_is_equivalent() {
    let (json, text) = job_audit(COVERING);
    let swap = mutant(&json, SWAP);
    assert_eq!(swap["verdict"], "equivalent", "{text}");
    assert_eq!(
        swap["overlap_taken_by"],
        serde_json::json!(["done-paused-default"]),
        "{swap:#}"
    );
    assert!(
        swap.get("unsatisfiable_guard").is_none() && swap.get("identical_answer").is_none(),
        "the guards overlap on `pause == true` with `code` absent, and the branches answer \
         differently: {swap:#}"
    );
    assert_eq!(
        json["format"], "ess-mutation-report/6",
        "`ess-mutation-report/5`, which 0.57.0 wrote, has no `overlap_taken_by`"
    );
    assert!(
        text.lines()
            .any(|line| line.starts_with(&format!("equivalent {SWAP}: "))
                && line.ends_with(
                    " — ESS-MUTATE-005: every input both branches select is taken first by \
                 `done-paused-default`"
                )),
        "{text}"
    );
}

#[test]
fn a_swap_whose_overlap_is_partly_uncovered_still_survives_or_is_killed() {
    // Without `code == ""`, `pause == true` with `code == ""` reaches both swapped branches.
    let (json, text) =
        job_audit("        when: {all: ['pause == true', {code: {defined: false}}]}\n");
    let swap = mutant(&json, SWAP);
    assert!(swap.get("overlap_taken_by").is_none(), "{swap:#}");
    assert_ne!(swap["verdict"], "equivalent", "{text}");
}

#[test]
fn an_earlier_branch_guarded_by_stored_state_does_not_cover() {
    // The same input guard beside a `when_subject:` predicate takes the overlap only on a row it
    // holds of; on any other row both swapped branches still compete.
    let (json, text) = job_audit(&format!(
        "{COVERING}        when_subject:\n          predicate: note == \"held\"\n"
    ));
    let swap = mutant(&json, SWAP);
    assert!(swap.get("overlap_taken_by").is_none(), "{swap:#}");
    assert_ne!(swap["verdict"], "equivalent", "{text}");
}

/// The emission of `JOB`'s precedence swaps with `COVERING`, with a report beside every suite as
/// [`emitted_for`] writes them.
fn job_emitted() -> (Emission, BTreeMap<String, String>) {
    let (files, texts) = job(COVERING);
    let ir = mutate::compile(files.clone(), &texts).expect("the model compiles");
    let emission = mutate::emit_for(&files, &texts, &[MutantClass::PrecedenceSwap], None)
        .expect("the model emits");
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
        written.insert(format!("{dir}/{REPORT_FILE}"), text);
    }
    (emission, written)
}

#[test]
fn the_manifest_and_the_collected_report_name_the_branches_taking_the_overlap() {
    let (emission, written) = job_emitted();
    let manifest: Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    assert_eq!(manifest["format"], "ess-mutation-manifest/6");
    let swap = mutant(&manifest, SWAP);
    assert_eq!(
        swap["overlap_taken_by"],
        serde_json::json!(["done-paused-default"]),
        "{swap:#}"
    );
    let mutant_file: Value = serde_json::from_str(&emission.files[&format!("{SWAP}/mutant.json")])
        .expect("the mutant's own entry");
    assert_eq!(
        mutant_file["overlap_taken_by"],
        serde_json::json!(["done-paused-default"])
    );
    let report =
        mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"));
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    assert_eq!(json["format"], "ess-mutation-report/6");
    let swap = mutant(&json, SWAP);
    assert_eq!(swap["verdict"], "equivalent", "{}", report.render_text());
    assert_eq!(
        swap["overlap_taken_by"],
        serde_json::json!(["done-paused-default"])
    );
}

#[test]
fn a_version_5_manifest_carrying_overlap_taken_by_is_refused_naming_version_6() {
    // 0.57.0 to this release wrote `ess-mutation-manifest/5` without `overlap_taken_by`; a member
    // new to a released format changes its meaning, so it has a version of its own.
    let (emission, mut written) = job_emitted();
    assert_eq!(emission.manifest.format, "ess-mutation-manifest/6");
    let manifest = written[MANIFEST_FILE]
        .replace("\"ess-mutation-manifest/6\"", "\"ess-mutation-manifest/5\"");
    written.insert(MANIFEST_FILE.to_owned(), manifest);
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("`ess-mutation-manifest/5` has no `overlap_taken_by`");
    let said = refusal.to_string();
    assert!(
        said.contains("`overlap_taken_by`")
            && said.contains("`ess-mutation-manifest/5`")
            && said.contains("`ess-mutation-manifest/6`"),
        "{said}"
    );
}

/// A swap whose overlap an earlier branch takes and which a runner's report kills still names
/// that branch, and its text line says so after a `;`, never as the `ESS-MUTATE-005` reason the
/// `equivalent` line gives (`render_text`'s branch for a verdict other than `equivalent`). Moved
/// from adversary pass 1, whose fixture swapped branches addressing different rows: those no
/// longer carry `overlap_taken_by`, so the kill comes from a report recording a failure.
#[test]
fn a_killed_swap_names_the_branch_taking_its_overlap_without_claiming_equivalence() {
    let (_, mut written) = job_emitted();
    let path = format!("{SWAP}/{REPORT_FILE}");
    let mut value: Value = serde_json::from_str(&written[&path]).unwrap();
    let passed = value["outcomes"]["passed"].as_array_mut().unwrap();
    assert!(!passed.is_empty(), "the swap's suite ran a scenario");
    let failing = passed.remove(0);
    let left = passed.len();
    value["outcomes"]["failed"] = Value::Array(vec![failing]);
    value["counts"]["passed"] = Value::from(left);
    value["counts"]["failed"] = Value::from(1);
    value["execution_status"] = Value::from("failed");
    value["conformance_status"] = Value::from("failed");
    written.insert(path, serde_json::to_string_pretty(&value).unwrap() + "\n");
    let report =
        mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"));
    let rendered = report.render_text();
    let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    let swap = mutant(&json, SWAP);
    assert_eq!(swap["verdict"], "killed", "{swap:#}\n{rendered}");
    assert_eq!(
        swap["overlap_taken_by"],
        serde_json::json!(["done-paused-default"]),
        "{swap:#}"
    );
    assert_eq!(json["format"], "ess-mutation-report/6", "{rendered}");
    let line = rendered
        .lines()
        .find(|line| line.starts_with(&format!("killed {SWAP}")))
        .unwrap_or_else(|| panic!("no killed line for the swap:\n{rendered}"));
    assert!(
        line.contains("; every input both branches select is taken first by `done-paused-default`")
            && !line.contains("ESS-MUTATE-005"),
        "{line}"
    );
}

#[test]
fn overlap_taken_by_on_a_mutant_that_swaps_nothing_is_refused() {
    let (emission, mut written) = emitted(&[MutantClass::GuardNegate]);
    assert_eq!(emission.manifest.format, "ess-mutation-manifest/3");
    let mut manifest: Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
    manifest["format"] = "ess-mutation-manifest/6".into();
    let carrier = manifest["mutants"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|it| it.get("dir").is_some())
        .expect("a mutant with a suite");
    carrier["overlap_taken_by"] = serde_json::json!(["leading-hold"]);
    written.insert(
        MANIFEST_FILE.to_owned(),
        serde_json::to_string_pretty(&manifest).unwrap(),
    );
    let refusal = mutate::collect(|path| written.get(path).cloned())
        .expect_err("only a `precedence-swap` has branches declared before both");
    let said = refusal.to_string();
    assert!(
        said.contains("`overlap_taken_by`") && said.contains("precedence-swap"),
        "{said}"
    );
}

/// A row-set command (`refused` reads a row set through the input) where `rushed`, an
/// input-guarded refusal the plan reads before the swapped pair, takes every input both select.
/// `PAIR` is the swapped two: `paused` (`pause == true`) on a book, then `TWIN` (`rush == true`),
/// which addresses a book or a shelf. The interpreter reads the row the first declared branch
/// acting on a stored row addresses before any branch is selected (`addressed_row`), so which of
/// the two is first decides whose absence refuses a request neither of them takes.
// `twin`'s text conjunct avoids the #702 refusal: the finite prover declines it, the audit decides it.
const SHELF: &str = r"format: ess/23
system: demo
version: v1
domain: demo.shelf
entities:
  - name: demo.shelf.Library
    identity: {name: library, type: String}
    fields:
      - {name: curator, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.shelf.Book
    identity: {name: book, type: String}
    fields: []
    lifecycle: {initial: Shelved, states: [Shelved], terminal: [Shelved]}
  - name: demo.shelf.Shelf
    identity: {name: shelf, type: String}
    fields: []
    lifecycle: {initial: Up, states: [Up], terminal: [Up]}
errors:
  - {name: demo.shelf.Refused, summary: A library of the curator is open., fields: []}
  - {name: demo.shelf.Unknown, summary: No row carries the identity., fields: []}
  - {name: demo.shelf.Rushed, summary: A rush is not taken., fields: []}
  - {name: demo.shelf.Idle, summary: Nothing was asked., fields: []}
events:
  - name: demo.shelf.LibraryOpened
    fields: [{name: library, type: String}]
  - name: demo.shelf.ShelfBuilt
    fields: [{name: shelf, type: String}]
  - name: demo.shelf.BookAdded
    fields: [{name: book, type: String}]
  - name: demo.shelf.BookPaused
    fields: [{name: book, type: String}]
  - name: demo.shelf.Twin
    fields: [{name: key, type: String}]
commands:
  - name: demo.shelf.OpenLibrary
    input:
      - {name: library, type: String}
      - {name: curator, type: String}
    outcomes:
      - name: opened
        creates: demo.shelf.Library
        instance: library
        sets: {curator: input.curator}
        emits: [demo.shelf.LibraryOpened]
        payload:
          demo.shelf.LibraryOpened: {library: input.library}
  - name: demo.shelf.BuildShelf
    input:
      - {name: shelf, type: String}
    outcomes:
      - name: built
        creates: demo.shelf.Shelf
        instance: shelf
        emits: [demo.shelf.ShelfBuilt]
        payload:
          demo.shelf.ShelfBuilt: {shelf: input.shelf}
  - name: demo.shelf.AddBook
    input:
      - {name: book, type: String}
    outcomes:
      - name: added
        creates: demo.shelf.Book
        instance: book
        emits: [demo.shelf.BookAdded]
        payload:
          demo.shelf.BookAdded: {book: input.book}
  - name: demo.shelf.Act
    input:
      - {name: book, type: String}
      - {name: shelf, type: String}
      - {name: curator, type: String}
      - {name: pause, type: Boolean}
      - {name: rush, type: Boolean}
    outcomes:
      - {name: unknown, unknown_instance: true, error: demo.shelf.Unknown}
      - name: refused
        when_related:
          entity: demo.shelf.Library
          where: curator == input.curator
          exists: true
        error: demo.shelf.Refused
      - {name: rushed, when: rush == true, error: demo.shelf.Rushed}
      - name: paused
        when: pause == true
        updates: demo.shelf.Book
        instance: book
        emits: [demo.shelf.BookPaused]
        payload:
          demo.shelf.BookPaused: {book: input.book}
      - name: twin
        when: {all: [rush == true, curator != nobody]}
        updates: demo.shelf.TWIN
        instance: TWIN_KEY
        emits: [demo.shelf.Twin]
        payload:
          demo.shelf.Twin: {key: input.TWIN_KEY}
      - {name: idle, error: demo.shelf.Idle}
";

const SHELF_SWAP: &str = "precedence-swap/demo.shelf.Act/paused/twin";

/// The audit of `SHELF`'s precedence swaps with `twin` acting on `entity` through `key`.
fn shelf_audit(entity: &str, key: &str) -> (Value, String) {
    let text = SHELF.replace("TWIN_KEY", key).replace("TWIN", entity);
    let label = "shelf.yaml".to_owned();
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text);
    let files = vec![(Source::new(label), raw)];
    let ir = mutate::compile(files.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let report = mutate::audit(&files, &texts, &[MutantClass::PrecedenceSwap], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let rendered = report.render_text();
    (
        serde_json::from_str(&report.to_canonical_json()).unwrap(),
        rendered,
    )
}

#[test]
fn a_swap_of_branches_addressing_different_rows_names_no_overlap_taken() {
    // Swapped, the shelf is the addressed row: a request selecting neither branch, on a store
    // holding the shelf and not the book, no longer refuses as an unknown book. The earlier branch
    // taking the overlap says nothing about that answer.
    let (json, text) = shelf_audit("Shelf", "shelf");
    let swap = mutant(&json, SHELF_SWAP);
    assert!(swap.get("overlap_taken_by").is_none(), "{swap:#}\n{text}");
    assert_ne!(swap["verdict"], "equivalent", "{swap:#}\n{text}");
    assert_ne!(json["format"], "ess-mutation-report/6", "{text}");
}

#[test]
fn a_swap_of_branches_addressing_the_same_row_names_the_overlap_taken() {
    let (json, text) = shelf_audit("Book", "book");
    let swap = mutant(&json, SHELF_SWAP);
    assert_eq!(
        swap["overlap_taken_by"],
        serde_json::json!(["rushed"]),
        "{swap:#}\n{text}"
    );
    assert_eq!(swap["verdict"], "equivalent", "{swap:#}\n{text}");
}

/// A mutant whose synthesized suite is the baseline's, scenario for scenario and step for step, is
/// `unwitnessed` (`ESS-MUTATE-004`) with `unchanged_suite`, not `survived`: run on the target, that
/// suite asks exactly what the baseline asked, so no scenario of it could kill the mutant
/// (<https://github.com/beyond10x/ess/issues/586>). A class-specific equivalence still wins.
mod identical_suite {
    use super::*;

    /// `expiry < now` to `expiry <= now`: the two differ only where the stored expiry is the
    /// target's `now`. Synthesis places that instant in a held scenario, so this mutant's suite is
    /// not the baseline's and it is no longer an unchanged-suite mutant.
    const BOUNDARY: &str = "guard-boundary/demo.codes.Redeem/expired/when_subject/0";
    /// The baseline's held scenario on that boundary: there `expiry < now` redeems.
    const HELD: &str = "demo.codes.Redeem/outcome/redeemed/held/expiry/0";
    const UNCHANGED: &str = " — ESS-MUTATE-004: the suite does not change";

    /// `explore.yaml`'s `score >= 50` to `score > 50`: its suite writes a witness at 50, so it
    /// differs from the baseline's, and still passes.
    const DIFFERING: &str = "guard-boundary/explore.desk.Grade/high/0";

    fn clocked() -> (Vec<Document>, SourceMap) {
        loaded("mutation-clocked-expiry.yaml")
    }

    fn loaded(name: &str) -> (Vec<Document>, SourceMap) {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name);
        let text = std::fs::read_to_string(&path).expect("the fixture is readable");
        let label = name.to_owned();
        let raw = RawSpecFile::parse(&text).expect("the fixture is well formed");
        let mut texts = SourceMap::new();
        texts.insert(label.clone(), text);
        (vec![(Source::new(label), raw)], texts)
    }

    /// The emission of `classes` over `files`, with the report the interpreter of the unchanged
    /// model writes beside every suite, as the project's runner would.
    fn emitted_and_run(
        (files, texts): (Vec<Document>, SourceMap),
        classes: &[MutantClass],
    ) -> BTreeMap<String, String> {
        let ir = mutate::compile(files.clone(), &texts).expect("the model compiles");
        let emission = mutate::emit_for(&files, &texts, classes, None).expect("the model emits");
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
            let admitted =
                AdmittedSuite::from_json(&emission.files[&format!("{dir}/{SUITE_FILE}")])
                    .expect("an emitted suite is admitted");
            let run = Runner::for_suite(admitted.suite())
                .run_admitted(&admitted, &Interpreted::for_model(ir.clone()));
            let text = CountReport::from_run(&run, &admitted)
                .unwrap()
                .to_canonical_json()
                .unwrap();
            written.insert(format!("{dir}/{REPORT_FILE}"), text);
        }
        written
    }

    /// The scenarios of the suite emitted for `dir`, as JSON.
    fn scenarios(written: &BTreeMap<String, String>, dir: &str) -> Value {
        let suite: Value = serde_json::from_str(&written[&format!("{dir}/{SUITE_FILE}")]).unwrap();
        suite["scenarios"].clone()
    }

    fn assert_unwitnessed(json: &Value, text: &str) {
        let entry = mutant(json, CHECK);
        assert_eq!(entry["verdict"], "unwitnessed", "{entry:#}\n{text}");
        assert_eq!(entry["unchanged_suite"], true, "{entry:#}");
        assert_eq!(json["counts"]["survived"], 0, "{text}");
        assert_eq!(json["counts"]["unwitnessed"], 1, "{text}");
        assert_eq!(
            json["format"], "ess-mutation-report/7",
            "`ess-mutation-report/6`, which 0.60.0 wrote, has no `unchanged_suite`"
        );
        assert!(
            text.lines()
                .any(|line| line.starts_with(&format!("unwitnessed {CHECK}: "))
                    && line.contains(UNCHANGED)),
            "{text}"
        );
    }

    /// `CHECK`: two refusals with the same error, whose payloads read inputs no synthesized
    /// scenario tells apart, so the swap's suite is the baseline's, scenario for scenario.
    #[test]
    fn identical_suite_mutant_is_unwitnessed() {
        let report = audit(&[MutantClass::PrecedenceSwap]);
        let text = report.render_text();
        let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
        assert_unwitnessed(&json, &text);

        // `--emit`, then `--collect`: the manifest carries nothing new, the collector decides it.
        let written = emitted_and_run(fixture(), &[MutantClass::PrecedenceSwap]);
        assert_eq!(
            scenarios(&written, BASELINE_DIR),
            scenarios(&written, CHECK),
            "the swap's suite is the baseline's"
        );
        let manifest: Value = serde_json::from_str(&written[MANIFEST_FILE]).unwrap();
        assert_eq!(
            manifest["format"], "ess-mutation-manifest/5",
            "`OPEN` carries `identical_answer`, nothing carries `unchanged_suite`: {manifest:#}"
        );
        assert!(mutant(&manifest, CHECK).get("unchanged_suite").is_none());
        let collected =
            mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"));
        let text = collected.render_text();
        let json: Value = serde_json::from_str(&collected.to_canonical_json()).unwrap();
        assert_unwitnessed(&json, &text);
    }

    /// `expiry < now` to `expiry <= now` was the unchanged-suite reproduction until synthesis held
    /// the clock on each `now` boundary (beyond10x/ess#586): the baseline's suite now carries a
    /// held scenario on the expiry boundary and the mutant's does not, so the mutant is not scored
    /// as an unchanged suite.
    #[test]
    fn a_held_now_boundary_changes_the_boundary_mutants_suite() {
        let written = emitted_and_run(clocked(), &[MutantClass::GuardBoundary]);
        let baseline = scenarios(&written, BASELINE_DIR);
        let boundary = scenarios(&written, BOUNDARY);
        assert!(baseline.get(HELD).is_some(), "{baseline:#}");
        assert!(boundary.get(HELD).is_none(), "{boundary:#}");
        assert_ne!(baseline, boundary, "the held scenario tells the two apart");

        let (files, texts) = clocked();
        let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
        let report = mutate::audit(&files, &texts, &[MutantClass::GuardBoundary], || {
            Interpreted::for_model(ir.clone())
        })
        .unwrap_or_else(|refusal| panic!("{refusal}"));
        let collected =
            mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"));
        for report in [report, collected] {
            let text = report.render_text();
            let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
            let entry = mutant(&json, BOUNDARY);
            assert!(entry.get("unchanged_suite").is_none(), "{entry:#}\n{text}");
            assert_ne!(entry["verdict"], "unwitnessed", "{entry:#}\n{text}");
        }
    }

    #[test]
    fn differing_suite_mutant_still_survives() {
        let written = emitted_and_run(loaded("explore.yaml"), &[MutantClass::GuardBoundary]);
        assert_ne!(
            scenarios(&written, BASELINE_DIR),
            scenarios(&written, DIFFERING),
            "the boundary mutant's suite differs from the baseline's"
        );
        let (files, texts) = loaded("explore.yaml");
        let ir = mutate::compile(files.clone(), &texts).expect("the fixture compiles");
        let report = mutate::audit(&files, &texts, &[MutantClass::GuardBoundary], || {
            Interpreted::for_model(ir.clone())
        })
        .unwrap_or_else(|refusal| panic!("{refusal}"));
        let collected =
            mutate::collect(|path| written.get(path).cloned()).unwrap_or_else(|it| panic!("{it}"));
        for report in [report, collected] {
            let text = report.render_text();
            let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
            let survivor = mutant(&json, DIFFERING);
            assert_eq!(survivor["verdict"], "survived", "{survivor:#}\n{text}");
            assert!(survivor.get("unchanged_suite").is_none(), "{survivor:#}");
        }
    }

    #[test]
    fn class_specific_equivalence_wins() {
        let report = audit(&[MutantClass::PrecedenceSwap]);
        let json: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
        let open = mutant(&json, OPEN);
        assert_eq!(open["verdict"], "equivalent", "{open:#}");
        assert!(open.get("unchanged_suite").is_none(), "{open:#}");
        assert_eq!(
            json["format"], "ess-mutation-report/7",
            "`CHECK` carries `unchanged_suite`"
        );

        let (json, text) = job_audit(COVERING);
        let swap = mutant(&json, SWAP);
        assert_eq!(swap["verdict"], "equivalent", "{swap:#}\n{text}");
        assert!(swap.get("unchanged_suite").is_none(), "{swap:#}");
        assert_eq!(json["format"], "ess-mutation-report/6", "{text}");
    }
}
