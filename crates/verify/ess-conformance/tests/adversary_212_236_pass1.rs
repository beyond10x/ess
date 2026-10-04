//! Adversary, pass 1, of beyond10x/ess#212 (`sets-drop`, `precedence-swap`, the `-outward` and
//! `equality-<n>` arms of `guard-boundary`) and beyond10x/ess#236 (`mutate --emit/--collect
//! --component`).
//!
//! Each case asserts what the issue, the story's decision or the unit's own documents promise, and
//! runs the code the unit wrote against it.

use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, Emission, MutantClass, MutationReport, Verdict, BASELINE_DIR, REPORT_FILE,
    SUITE_FILE,
};
use ess_conformance::runner::Runner;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::Value;

fn parsed(label: &str, text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.to_owned(), text.to_owned());
    (vec![(Source::new(label.to_owned()), raw)], texts)
}

fn arms_text() -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mutation-arms.yaml"),
    )
    .expect("the unit's fixture is readable")
}

fn audit(spec: &(Vec<Document>, SourceMap), classes: &[MutantClass]) -> MutationReport {
    let (files, texts) = spec;
    let ir = mutate::compile(files.clone(), texts).expect("the fixture compiles");
    mutate::audit(files, texts, classes, || Interpreted::for_model(ir.clone()))
        .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn verdicts(report: &MutationReport) -> BTreeMap<String, Verdict> {
    report
        .mutants
        .iter()
        .map(|entry| (entry.id.clone(), entry.verdict))
        .collect()
}

/// The emission's files, with the unmutated model's interpreter's report beside every suite.
fn reported(spec: &(Vec<Document>, SourceMap), emission: &Emission) -> BTreeMap<String, String> {
    let (files, texts) = spec;
    let ir = mutate::compile(files.clone(), texts).expect("the fixture compiles");
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
        written.insert(
            format!("{dir}/{REPORT_FILE}"),
            CountReport::from_run(&run, &admitted)
                .expect("a complete run has a report/2")
                .to_canonical_json()
                .expect("report/2 serializes"),
        );
    }
    written
}

// ---- #236: a component's own survivor ------------------------------------------------------------

/// The unit's notes fixture, with one component that owns its only domain, accepts every command
/// and publishes every event: the component *is* the system.
fn notes_as_one_component() -> (Vec<Document>, SourceMap) {
    let mut text = arms_text();
    text.push_str(
        "\ncomponents:\n  - component: notes-service\n    owns:\n      domains: [notes.core]\n    \
         accepts:\n      commands: [notes.core.Write, notes.core.Edit, notes.core.Grade, \
         notes.core.Sort, notes.core.Measure, notes.core.Admit]\n    publishes:\n      events: \
         [notes.core.Written, notes.core.Edited, notes.core.Small, notes.core.Flagged, \
         notes.core.Other]\n",
    );
    parsed("notes-as-one-component.yaml", &text)
}

/// The story's decision keeps out of scope the mutants "whose changed scenarios all belong to
/// another component" (story `feature-request-236`, Fit review 2 and 4). A mutant on a command the
/// component accepts, whose suite pins nothing it changed, is the audit's finding — a survivor — and
/// is that component's to answer. Measured: `sets-drop/notes.core.Edit/edited/memo` survives the
/// whole-system audit (`mutation_arms.rs` pins it), and the same emission scoped to the component
/// that accepts `Edit`, and everything else, lists it out of scope, uncounted: the survivor, and
/// the non-zero exit it causes, are gone.
#[test]
fn adv236_a_survivor_on_a_command_the_component_accepts_is_scored_not_out_of_scope() {
    let spec = notes_as_one_component();
    let memo = "sets-drop/notes.core.Edit/edited/memo";

    let whole = audit(&spec, &[MutantClass::SetsDrop]);
    assert_eq!(
        verdicts(&whole).get(memo),
        Some(&Verdict::Survived),
        "precondition: the whole-system audit finds the survivor\n{}",
        whole.render_text()
    );

    let (files, texts) = &spec;
    let emission = mutate::emit_for(
        files,
        texts,
        &[MutantClass::SetsDrop],
        Some("notes-service"),
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let scoped = mutate::collect_for(
        {
            let written = reported(&spec, &emission);
            move |path: &str| written.get(path).cloned()
        },
        None,
    )
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let listed: Vec<&str> = scoped
        .out_of_scope
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|it| it.id.as_str())
        .collect();
    assert!(
        !listed.contains(&memo),
        "`{memo}` is on `notes.core.Edit`, which `notes-service` accepts, yet the component \
         audit lists it out of scope instead of scoring the survivor\n{}",
        scoped.render_text()
    );
    assert_eq!(
        scoped.counts.survived,
        whole.counts.survived,
        "a component that is the whole system must find the whole system's survivors\n{}",
        scoped.render_text()
    );
}

// ---- #212: sets-drop of an input under another name ----------------------------------------------

/// A counter opened with an amount and adjusted to a new one: `sets: {amount: input.new_amount}`,
/// the input named apart from the field it writes, read back by a view.
const TALLY: &str = r"format: ess/16
system: tally
version: v1
domain: tally.core
summary: A counter opened with an amount and adjusted to another.
naming:
  wire: tally
  display: Tally

entities:
  - name: tally.core.Counter
    identity: {name: counter_id, type: Uuid}
    fields:
      - {name: amount, type: Integer}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]

events:
  - name: tally.core.Opened
    fields:
      - {name: counter_id, type: Uuid}
  - name: tally.core.Adjusted
    fields:
      - {name: counter_id, type: Uuid}

commands:
  - name: tally.core.Open
    input:
      - {name: amount, type: Integer}
    outcomes:
      - name: opened
        creates: tally.core.Counter
        instance: counter_id
        sets: {amount: input.amount}
        emits: [tally.core.Opened]
        payload:
          tally.core.Opened:
            counter_id: {generated: true}
  - name: tally.core.Adjust
    input:
      - {name: counter_id, type: Uuid}
      - {name: new_amount, type: Integer}
    outcomes:
      - name: adjusted
        updates: tally.core.Counter
        instance: counter_id
        sets: {amount: input.new_amount}
        emits: [tally.core.Adjusted]
        payload:
          tally.core.Adjusted:
            counter_id: input.counter_id

views:
  - name: tally.core.Counters
    source: tally.core.Counter
    consistency: read_your_writes
    fields:
      - {name: counter_id, type: Uuid}
      - {name: amount, type: Integer}
";

/// The `Adjust` scenario of the suite at `dir`: what `Open` stored, what `Adjust` sent, and what the
/// view must show afterwards.
fn adjust_amounts(files: &BTreeMap<String, String>, dir: &str) -> (Value, Value, Value) {
    let suite: Value =
        serde_json::from_str(&files[&format!("{dir}/{SUITE_FILE}")]).expect("a suite");
    let steps = suite["scenarios"]["tally.core.Adjust/outcome/adjusted"]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("no adjust scenario in {dir}"))
        .clone();
    let sent = |command: &str, field: &str| {
        let step = steps
            .iter()
            .find(|step| step["step"] == "execute_command" && step["command"] == command)
            .unwrap_or_else(|| panic!("no {command} in {steps:#?}"));
        step["input"][field]["value"].clone()
    };
    let shown = steps
        .iter()
        .find(|step| step["step"] == "expect_view")
        .map(|step| step["expectation"]["fields"]["amount"]["value"].clone())
        .expect("the row is read back");
    (
        sent("tally.core.Open", "amount"),
        sent("tally.core.Adjust", "new_amount"),
        shown,
    )
}

/// `unread_apart` moves only an input named like the field it would collide with (`synthesize.rs`,
/// "Only the same-named field"). A `sets-drop` of `amount: input.new_amount` leaves `new_amount`
/// unread under a name no field has, so the mutant's suite sends it at the plain witness, equal to
/// what `Open` stored, and a target that still writes it shows the same row. Yet the baseline suite
/// does pin the write: `freshened` sent `new_amount` apart from the stored value there, and its view
/// expects the new one. So the audit reports a survivor — a rule synthesis "does not pin" — that
/// synthesis does pin.
#[test]
fn adv212_sets_drop_of_an_input_named_apart_from_its_field_is_killed_where_the_baseline_pins_it() {
    let spec = parsed("tally.yaml", TALLY);
    let (files, texts) = &spec;
    let site = "sets-drop/tally.core.Adjust/adjusted/amount";

    let emission = mutate::emit(files, texts, &[MutantClass::SetsDrop]).expect("tally emits");
    // The baseline pins the write: sent apart from the stored value, and the new value expected.
    let (stored, sent, shown) = adjust_amounts(&emission.files, BASELINE_DIR);
    assert_ne!(
        sent, stored,
        "precondition: the baseline sends the new amount apart"
    );
    assert_eq!(
        shown, sent,
        "precondition: the baseline expects the row to take it"
    );

    let report = audit(&spec, &[MutantClass::SetsDrop]);
    let (stored, sent, _) = adjust_amounts(&emission.files, site);
    assert_eq!(
        verdicts(&report).get(site),
        Some(&Verdict::Killed),
        "the mutant's suite sends `new_amount` {sent} over a stored `amount` {stored}, so a target \
         that writes it is indistinguishable from one that drops it\n{}",
        report.render_text()
    );
}

// ---- #212: the outward literal at the bounds of i64 ----------------------------------------------

fn admit_bound(bound: &str) -> (Vec<Document>, SourceMap) {
    let text = arms_text().replace("- amount >= 10", &format!("- amount {bound}"));
    assert!(text.contains(bound), "the fixture was rewritten");
    parsed(&format!("arms#{bound}"), &text)
}

fn boundary_ids(spec: &(Vec<Document>, SourceMap)) -> BTreeMap<String, String> {
    mutate::mutants(&spec.0, &[MutantClass::GuardBoundary])
        .into_iter()
        .filter(|it| it.id.contains("notes.core.Admit"))
        .map(|it| (it.id, it.change))
        .collect()
}

#[test]
fn adv212_an_outward_literal_one_step_inside_i64_moves_to_the_bound_and_is_killed() {
    // (bound, the change described, the verdict today)
    for (bound, moved, verdict) in [
        (
            ">= -9223372036854775807",
            "`amount >= -9223372036854775807` becomes `amount >= -9223372036854775808`",
            Verdict::Killed,
        ),
        // F4 of this pass, pre-existing and infeasible today: synthesis draws its witnesses as
        // binary64, which cannot reach i64::MAX, so no witness sits on the moved bound and the
        // mutant survives (the strictness swap at the same literal survives at base too). This
        // half asserts that state; it moves to `Killed` when witnesses are exact integers.
        (
            "<= 9223372036854775806",
            "`amount <= 9223372036854775806` becomes `amount <= 9223372036854775807`",
            Verdict::Survived,
        ),
    ] {
        let spec = admit_bound(bound);
        let found = boundary_ids(&spec);
        let id = "guard-boundary/notes.core.Admit/admitted/0-outward";
        assert_eq!(found.get(id).map(String::as_str), Some(moved), "{found:#?}");
        let report = audit(&spec, &[MutantClass::GuardBoundary]);
        assert_eq!(
            verdicts(&report).get(id),
            Some(&verdict),
            "{bound}: beyond10x/ess#212 outward arm; a `Survived` here is the f64 witness limit \
             (binary64 witnesses cannot reach i64::MAX), not a missing kill\n{}",
            report.render_text()
        );
    }
}

#[test]
fn adv212_an_outward_literal_at_the_bound_of_i64_has_no_site() {
    for bound in [">= -9223372036854775808", "<= 9223372036854775807"] {
        let found = boundary_ids(&admit_bound(bound));
        assert!(
            !found.keys().any(|id| id.ends_with("-outward")),
            "{bound}: {found:#?}"
        );
    }
}

// ---- #212: the equality arm on enum and optional operands ----------------------------------------

#[test]
fn adv212_an_equality_flip_on_an_enum_inequality_is_killed() {
    let text = arms_text().replace("- kind == A", "- kind != A");
    let spec = parsed("arms#ne", &text);
    let id = "guard-boundary/notes.core.Admit/admitted/equality-0";
    assert_eq!(
        boundary_ids(&spec).get(id).map(String::as_str),
        Some("`kind != A` becomes `kind == A`")
    );
    let report = audit(&spec, &[MutantClass::GuardBoundary]);
    assert_eq!(
        verdicts(&report).get(id),
        Some(&Verdict::Killed),
        "{}",
        report.render_text()
    );
}

#[test]
fn adv212_an_equality_flip_on_an_optional_operand_is_killed() {
    let text = arms_text()
        .replace(
            "      - {name: kind, type: notes.core.Kind}\n      - {name: amount, type: Integer}\n    \
             outcomes:\n      - name: admitted",
            "      - {name: tag, type: Optional<String>}\n      - {name: amount, type: Integer}\n    \
             outcomes:\n      - name: admitted",
        )
        .replace("            - kind == A\n", "            - defined(tag)\n            - tag == \"x\"\n");
    assert!(text.contains("tag == \"x\""), "the fixture was rewritten");
    let spec = parsed("arms#optional", &text);
    let found = boundary_ids(&spec);
    let id = "guard-boundary/notes.core.Admit/admitted/equality-0";
    assert!(found.contains_key(id), "{found:#?}");
    let report = audit(&spec, &[MutantClass::GuardBoundary]);
    assert_eq!(
        verdicts(&report).get(id),
        Some(&Verdict::Killed),
        "{}",
        report.render_text()
    );
}
