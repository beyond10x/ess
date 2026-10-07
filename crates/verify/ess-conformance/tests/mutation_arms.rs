//! The mutation classes and guard arms beyond10x/ess#212 adds: `sets-drop` on a branch that acts on
//! an existing row, `precedence-swap` of two adjacent guarded branches, and the `==`/`!=` and
//! outward-literal arms of `guard-boundary`.
//!
//! Each is held to a killed control and a control that is not killed, against the interpreter of
//! the unmutated model, and the emitted manifest is held to the format that can say it.

use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, Document, MutantClass, MutationReport, Verdict, BASELINE_DIR, MANIFEST_FILE,
    MANIFEST_FORMAT, MANIFEST_FORMAT_4, SUITE_FILE,
};
use ess_conformance::reference::Billing;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::Value;

fn parsed(label: &str, text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert(label.to_owned(), text.to_owned());
    (vec![(Source::new(label.to_owned()), raw)], texts)
}

fn fixture(name: &str) -> (Vec<Document>, SourceMap) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let text = std::fs::read_to_string(&path).expect("the fixture is readable");
    parsed(&path.display().to_string(), &text)
}

/// Notes written and edited, graded by overlapping guards, sorted by disjoint ones, measured by two
/// overlapping refusals and admitted at a boundary.
fn arms() -> (Vec<Document>, SourceMap) {
    fixture("mutation-arms.yaml")
}

fn example(name: &str) -> (Vec<Document>, SourceMap) {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("`{name}` exists: {error}"));
    let mut found = Vec::new();
    let mut pending = vec![base];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path.display().to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).expect("well formed");
        texts.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    (parsed, texts)
}

fn described(
    spec: &(Vec<Document>, SourceMap),
    classes: &[MutantClass],
) -> BTreeMap<String, String> {
    mutate::mutants(&spec.0, classes)
        .into_iter()
        .map(|mutant| (mutant.id, mutant.change))
        .collect()
}

fn audit(spec: &(Vec<Document>, SourceMap), classes: &[MutantClass]) -> MutationReport {
    let (files, texts) = spec;
    let ir = mutate::compile(files.clone(), texts).expect("the fixture compiles");
    mutate::audit(files, texts, classes, || Interpreted::for_model(ir.clone()))
        .unwrap_or_else(|refusal| panic!("{refusal}"))
}

fn verdicts(report: &MutationReport) -> BTreeMap<&str, Verdict> {
    report
        .mutants
        .iter()
        .map(|entry| (entry.id.as_str(), entry.verdict))
        .collect()
}

// ---- the classes ----------------------------------------------------------------------------------

#[test]
fn the_two_new_classes_are_named_as_the_cli_takes_them() {
    assert_eq!(MutantClass::SetsDrop.as_str(), "sets-drop");
    assert_eq!(MutantClass::PrecedenceSwap.as_str(), "precedence-swap");
    assert!(MutantClass::ALL.contains(&MutantClass::SetsDrop));
    assert!(MutantClass::ALL.contains(&MutantClass::PrecedenceSwap));
    // `order-flip` already means a view's ranking key, so the requested `outcome-order-flip` is
    // not a class name.
    assert!(MutantClass::ALL
        .iter()
        .all(|class| class.as_str() != "outcome-order-flip"));
}

// ---- the sites ------------------------------------------------------------------------------------

#[test]
fn guard_boundary_adds_an_equality_leaf_and_an_outward_literal_beside_the_strictness_swap() {
    let found = described(&arms(), &[MutantClass::GuardBoundary]);
    assert_eq!(
        found.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "guard-boundary/notes.core.Admit/admitted/0",
            "guard-boundary/notes.core.Admit/admitted/0-outward",
            "guard-boundary/notes.core.Admit/admitted/equality-0",
            "guard-boundary/notes.core.Grade/flagged/0",
            "guard-boundary/notes.core.Grade/small/0",
            "guard-boundary/notes.core.Measure/low/0",
            "guard-boundary/notes.core.Measure/too-low/0",
        ],
        "a strict ordering gets no outward arm, because the strictness swap already moves it \
         outward; `kind == A` alone is the whole guard of `Sort/first`, which `guard-negate` \
         already flips: {found:#?}"
    );
    assert_eq!(
        found["guard-boundary/notes.core.Admit/admitted/0-outward"],
        "`amount >= 10` becomes `amount >= 9`"
    );
    assert_eq!(
        found["guard-boundary/notes.core.Admit/admitted/equality-0"],
        "`kind == A` becomes `kind != A`"
    );
    assert_eq!(
        found["guard-boundary/notes.core.Admit/admitted/0"],
        "`amount >= 10` becomes `amount > 10`"
    );
}

#[test]
fn an_outward_literal_moves_a_non_strict_bound_away_from_its_accepting_side() {
    let text = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/mutation-arms.yaml"),
    )
    .expect("readable")
    .replace("- amount >= 10", "- amount <= 10");
    let found = described(&parsed("arms#le", &text), &[MutantClass::GuardBoundary]);
    assert_eq!(
        found["guard-boundary/notes.core.Admit/admitted/0-outward"],
        "`amount <= 10` becomes `amount <= 11`"
    );
}

#[test]
fn sets_drop_is_offered_only_on_a_branch_that_acts_on_an_existing_row() {
    let found = described(&arms(), &[MutantClass::SetsDrop]);
    assert_eq!(
        found,
        BTreeMap::from([
            (
                "sets-drop/notes.core.Edit/edited/memo".to_owned(),
                "`memo: input.memo` is no longer set".to_owned()
            ),
            (
                "sets-drop/notes.core.Edit/edited/title".to_owned(),
                "`title: input.title` is no longer set".to_owned()
            ),
        ]),
        "`Write` creates its row, where a dropped write only weakens the specification"
    );
}

#[test]
fn precedence_swap_pairs_adjacent_branches_guarded_by_the_input_alone() {
    let found = described(&arms(), &[MutantClass::PrecedenceSwap]);
    assert_eq!(
        found,
        BTreeMap::from([
            (
                "precedence-swap/notes.core.Grade/small/flagged".to_owned(),
                "`small` (`amount < 100`) and `flagged` (`amount > 50`) change places, so \
                 `flagged` answers where both hold"
                    .to_owned()
            ),
            (
                "precedence-swap/notes.core.Measure/too-low/low".to_owned(),
                "`too-low` (`amount < 10`) and `low` (`amount < 20`) change places, so `low` \
                 answers where both hold"
                    .to_owned()
            ),
            (
                "precedence-swap/notes.core.Sort/first/second".to_owned(),
                "`first` (`kind == A`) and `second` (`kind == B`) change places, so `second` \
                 answers where both hold"
                    .to_owned()
            ),
        ]),
        "a default is never paired, and an accepting branch is never paired with a refusal, \
         which comes first whatever the order"
    );
}

#[test]
fn branches_a_held_state_or_a_stored_row_also_guards_are_left_out_of_precedence_swap() {
    // `ringing`, `refreshed` and `preserved` are adjacent and each has a `when:`, but each also has
    // a `when_subject_state:`, which the held row answers before declaration order does.
    assert!(described(
        &fixture("subject-state.yaml"),
        &[MutantClass::PrecedenceSwap]
    )
    .is_empty());
}

#[test]
fn every_new_mutant_changes_exactly_what_its_description_says() {
    let (files, texts) = arms();
    for mutant in mutate::mutants(
        &files,
        &[
            MutantClass::SetsDrop,
            MutantClass::PrecedenceSwap,
            MutantClass::GuardBoundary,
        ],
    ) {
        let mutated = mutate::apply(&files, &mutant.mutation)
            .unwrap_or_else(|why| panic!("{}: {why}", mutant.id));
        assert_ne!(
            format!("{mutated:?}"),
            format!("{files:?}"),
            "{} changed nothing",
            mutant.id
        );
        mutate::compile(mutated, &texts)
            .unwrap_or_else(|stillborn| panic!("{} is stillborn: {stillborn:?}", mutant.id));
    }
}

// ---- killed and not killed --------------------------------------------------------------------------

#[test]
fn each_new_arm_is_killed_where_a_witness_tells_it_apart_and_not_where_none_can() {
    let report = audit(
        &arms(),
        &[
            MutantClass::SetsDrop,
            MutantClass::PrecedenceSwap,
            MutantClass::GuardBoundary,
        ],
    );
    let found = verdicts(&report);
    let text = report.render_text();
    for (id, verdict) in [
        // Killed: each mutant's suite asks a question its two models answer differently.
        (
            "guard-boundary/notes.core.Admit/admitted/0-outward",
            Verdict::Killed,
        ),
        (
            "guard-boundary/notes.core.Admit/admitted/equality-0",
            Verdict::Killed,
        ),
        (
            "precedence-swap/notes.core.Grade/small/flagged",
            Verdict::Killed,
        ),
        // Two refusals: `low` declared first is witnessed below 10, where the target refuses
        // `too-low`.
        (
            "precedence-swap/notes.core.Measure/too-low/low",
            Verdict::Killed,
        ),
        // Equivalent: `kind == A` and `kind == B` share no input, so their order decides nothing.
        (
            "precedence-swap/notes.core.Sort/first/second",
            Verdict::Equivalent,
        ),
        // Killed: the view reads `title` back, and the edited title was sent apart from the stored
        // one, so the row the target shows is not the row the mutant keeps.
        ("sets-drop/notes.core.Edit/edited/title", Verdict::Killed),
        // Survived: nothing reads `memo`, so a dropped write of it is not observable, and the
        // mutant's suite is the baseline's.
        ("sets-drop/notes.core.Edit/edited/memo", Verdict::Survived),
    ] {
        assert_eq!(found.get(id), Some(&verdict), "{id}\n{text}");
    }
    let sort = report
        .mutants
        .iter()
        .find(|entry| entry.id == "precedence-swap/notes.core.Sort/first/second")
        .expect("the swap is scored");
    assert_eq!(
        sort.unsatisfiable_guard.as_deref(),
        Some("(kind == A and kind == B)"),
        "{text}"
    );
}

/// `IssueInvoice` writes `issued_at: Optional<Timestamp>`, which nothing wrote before. Dropped, the
/// row holds it absent, and no synthesized row expectation states an absence: measured, the mutant
/// survived against the billing reference with a suite that asks nothing about the field. So an
/// `Optional` field is left out of `sets-drop` by name, and billing offers no site.
#[test]
fn a_dropped_write_of_an_optional_field_is_not_offered() {
    let (files, texts) = example("billing");
    assert!(described(&(files.clone(), texts.clone()), &[MutantClass::SetsDrop]).is_empty());
    let refusal =
        mutate::audit(&files, &texts, &[MutantClass::SetsDrop], Billing::new).expect_err("no site");
    assert!(
        matches!(refusal, mutate::AuditRefusal::NoSite { .. }),
        "{refusal}"
    );
}

// ---- the separated witness ----------------------------------------------------------------------------

/// The `Edit` scenario of the suite emitted at `dir`: what `Write` stored as `title`, what `Edit`
/// sent as `title`, and what the view is required to show.
fn edit_titles(files: &BTreeMap<String, String>, dir: &str) -> (Value, Value, Value) {
    let suite: Value =
        serde_json::from_str(&files[&format!("{dir}/{SUITE_FILE}")]).expect("a suite");
    let steps = suite["scenarios"]["notes.core.Edit/outcome/edited"]["steps"]
        .as_array()
        .expect("the edit scenario")
        .clone();
    let sent = |command: &str| {
        let step = steps
            .iter()
            .find(|step| step["step"] == "execute_command" && step["command"] == command)
            .unwrap_or_else(|| panic!("no {command} in {steps:#?}"));
        step["input"]["title"]["value"].clone()
    };
    let shown = steps
        .iter()
        .find(|step| step["step"] == "expect_view")
        .map(|step| step["expectation"]["fields"]["title"]["value"].clone())
        .expect("the row is read back");
    (sent("notes.core.Write"), sent("notes.core.Edit"), shown)
}

#[test]
fn a_sets_drop_mutant_sends_the_edited_value_apart_from_the_stored_one() {
    let (files, texts) = arms();
    let emission =
        mutate::emit(&files, &texts, &[MutantClass::SetsDrop]).expect("the fixture emits");

    // The mutant writes nothing to `title`, so its row keeps what `Write` stored. The input is sent
    // apart from that value, so an implementation that still writes it shows something else.
    let (stored, sent, shown) =
        edit_titles(&emission.files, "sets-drop/notes.core.Edit/edited/title");
    assert_ne!(
        sent, stored,
        "the dropped input must differ from the stored value"
    );
    assert_eq!(shown, stored, "the mutant's row keeps the stored value");

    // The baseline is unchanged: it writes `title`, and the row shows what was sent.
    let (stored, sent, shown) = edit_titles(&emission.files, BASELINE_DIR);
    assert_ne!(sent, stored);
    assert_eq!(shown, sent);
}

// ---- the manifest ------------------------------------------------------------------------------------

#[test]
fn an_emission_with_a_new_class_declares_version_4_and_one_without_stays_version_3() {
    let (files, texts) = arms();
    let new = mutate::emit(&files, &texts, &[MutantClass::SetsDrop]).expect("emits");
    assert_eq!(new.manifest.format, MANIFEST_FORMAT_4);
    let swapped = mutate::emit(&files, &texts, &[MutantClass::PrecedenceSwap]).expect("emits");
    assert_eq!(swapped.manifest.format, MANIFEST_FORMAT_4);
    // The new guard arms are sites of an existing class, which a version 3 reader already scores.
    let old = mutate::emit(&files, &texts, &[MutantClass::GuardBoundary]).expect("emits");
    assert_eq!(old.manifest.format, MANIFEST_FORMAT);
    assert_eq!(MANIFEST_FORMAT, "ess-mutation-manifest/3");
    assert_eq!(MANIFEST_FORMAT_4, "ess-mutation-manifest/4");
}

#[test]
fn a_version_3_manifest_naming_a_new_class_is_refused_naming_version_4() {
    let (files, texts) = arms();
    let emission = mutate::emit(&files, &texts, &[MutantClass::SetsDrop]).expect("emits");
    let manifest = emission.files[MANIFEST_FILE].replace(MANIFEST_FORMAT_4, MANIFEST_FORMAT);
    let refusal = mutate::Manifest::from_json(&manifest).expect_err("sets-drop is version 4");
    assert!(refusal.contains("sets-drop"), "{refusal}");
    assert!(refusal.contains(MANIFEST_FORMAT_4), "{refusal}");
}

#[test]
fn a_manifest_version_this_reader_does_not_know_is_refused_naming_the_ones_it_reads() {
    let (files, texts) = arms();
    let emission = mutate::emit(&files, &texts, &[MutantClass::SetsDrop]).expect("emits");
    let manifest =
        emission.files[MANIFEST_FILE].replace(MANIFEST_FORMAT_4, "ess-mutation-manifest/5");
    let refusal = mutate::Manifest::from_json(&manifest).expect_err("/5 is not read");
    for known in [
        MANIFEST_FORMAT_4,
        MANIFEST_FORMAT,
        mutate::MANIFEST_FORMAT_2,
        mutate::MANIFEST_FORMAT_1,
    ] {
        assert!(refusal.contains(known), "{refusal}");
    }
}
