//! `unknown_instance:` beside a `when_related:` guard (source format `ess/27`, downstream report,
//! 2026-10-18), executed against the generated Rust and Go targets.
//!
//! `ChangeTier` addresses a seat by its input and reads the tier its input names: an unknown tier
//! answers first, then an unknown seat, then a closed tier, then the change. The model, served by
//! one network component, is emitted, built as its generated entry point with no hand-written code,
//! and driven over a real socket by the suite the same model synthesizes, whose `unknown-seat`
//! scenario names a closed tier.
//!
//! A faulty control skips the seat's existence check made before the tier's refusal, so it reads
//! the closed tier first: the suite must fail it.

#[allow(dead_code)]
mod related_guard_served;
use related_guard_served as served;

use ess_synth::{CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/related-guard-unknown-instance.yaml"
);

fn model() -> ess_compiler::ir::EssIr {
    served::model(&served::served(MODEL, "seats"))
}

/// The existence checks of `ChangeTier`'s generated behaviour made before the tier's refusal — at
/// step 2 of the one precedence rule, and again for the branch the request selects — and a
/// replacement of each that always finds a row.
fn existence_checks(target: Target) -> [(&'static str, &'static str); 2] {
    match target {
        Target::Rust => [
            (
                "if SeatStorage::get(&self.ports, &input.seat_id).is_none() {",
                "if false {",
            ),
            (
                "let Some(held) = SeatStorage::get(&self.ports, &input.seat_id) else {",
                "let Some(held) = Some(()) else {",
            ),
        ],
        Target::Go => [
            (
                "held, found := b.ports.SeatStorage.Get(input.SeatId)",
                "held, found := 0, true",
            ),
            (
                "held, found := b.ports.SeatStorage.Get(input.SeatId)",
                "held, found := 0, true",
            ),
        ],
        _ => unreachable!("the case emits Rust and Go"),
    }
}

fn unknown_instance_served(target: Target) {
    let ir = model();
    let synthesis = ess_synth::synthesize_for(&ir, target).expect("the model synthesizes");
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.seats.ChangeTier"),
        Some(&SynthesisDisposition::Generated),
        "{target:?}:\n{}",
        synthesis.plan.to_markdown()
    );
    let (root, _) = served::emit(&ir, target, "unknown-instance");
    let binary = served::build(&root, target, "demo", "seats", false);
    let statuses = served::run(&ir, &binary);
    served::assert_passes(
        &format!("{target:?} unknown instance beside a related guard"),
        &statuses,
        &[
            "ChangeTier/outcome/unknown-seat",
            "ChangeTier/outcome/unknown-tier",
            "ChangeTier/outcome/tier-closed",
            "ChangeTier/outcome/changed",
        ],
    );
    let _ = std::fs::remove_dir_all(&root);

    // Faulty control: the closed tier read before the seat's existence.
    let (root, _) = served::emit(&ir, target, "unknown-instance-faulty");
    for (from, to) in existence_checks(target) {
        served::mutate(&served::behaviour_path(&root, target, "demo"), from, to);
    }
    let binary = served::build(&root, target, "demo", "seats", false);
    let statuses = served::run(&ir, &binary);
    assert_ne!(
        statuses.get("demo.seats.ChangeTier/outcome/unknown-seat"),
        Some(&ess_conformance::report::Status::Passed),
        "{target:?}: the faulty target fails the unknown-seat scenario: {statuses:#?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn unknown_instance_beside_a_related_guard_in_the_generated_rust_target() {
    unknown_instance_served(Target::Rust);
}

#[test]
fn unknown_instance_beside_a_related_guard_in_the_generated_go_target() {
    assert!(
        served::go_available(),
        "Go is required: the generated Go target is unchecked without it"
    );
    unknown_instance_served(Target::Go);
}

// ---- create-or-update beside a related-row refusal (beyond10x/ess#641) ----------------------

const BIND_MODEL: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/create-or-update-beside-related-refusal.yaml"
);

/// The edit that makes a generated `Bind` answer a missing agent before the seat's existence.
fn missing_agent_first(target: Target) -> (&'static str, &'static str) {
    match target {
        Target::Rust => (
            "        // The addressed row, read before the branches that select by it.\n        let Some(held) = BindingStorage::get(&self.ports, &input.seat_id) else {",
            "        if AgentStorage::get(&self.ports, &input.agent_id).is_none() {\n            return Ok(crate::desk::BindOutcome::NoAgent { error: crate::desk::NoAgent });\n        }\n        let Some(held) = BindingStorage::get(&self.ports, &input.seat_id) else {",
        ),
        Target::Go => (
            "\t// The addressed row, read before the branches that select by it.\n\theld, found := b.ports.BindingStorage.Get(input.SeatId)\n",
            "\tif _, found := b.ports.AgentStorage.Get(input.AgentId); !found {\n\t\treturn desk.BindOutcomeNoAgent{Error: desk.NoAgent{}}, nil\n\t}\n\theld, found := b.ports.BindingStorage.Get(input.SeatId)\n",
        ),
        _ => unreachable!("the case emits Rust and Go"),
    }
}

/// `Bind` creates a seat no row carries, keeps one bound to the agent sent, refuses an agent no
/// row carries and otherwise rebinds. The generated behaviour, built with no hand-written code,
/// passes every scenario the model synthesizes, among them `bound` sent for an unknown seat and an
/// agent no row carries. A faulty control answering the missing agent before the seat's existence
/// fails `bound`.
fn create_or_update_served(target: Target) {
    let ir = served::model(&served::served(BIND_MODEL, "desk"));
    let synthesis = ess_synth::synthesize_for(&ir, target).expect("the model synthesizes");
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.desk.Bind"),
        Some(&SynthesisDisposition::Generated),
        "{target:?}:\n{}",
        synthesis.plan.to_markdown()
    );
    let (root, _) = served::emit(&ir, target, "create-or-update");
    let binary = served::build(&root, target, "demo", "desk", false);
    let statuses = served::run(&ir, &binary);
    served::assert_passes(
        &format!("{target:?} create-or-update beside a related refusal"),
        &statuses,
        &[
            "Bind/outcome/bound",
            "Bind/outcome/no-agent",
            "Bind/outcome/rebound",
            "Bind/outcome/already-bound",
        ],
    );
    let _ = std::fs::remove_dir_all(&root);

    let (root, _) = served::emit(&ir, target, "create-or-update-faulty");
    let (from, to) = missing_agent_first(target);
    served::mutate(&served::behaviour_path(&root, target, "demo"), from, to);
    let binary = served::build(&root, target, "demo", "desk", false);
    let statuses = served::run(&ir, &binary);
    assert_ne!(
        statuses.get("demo.desk.Bind/outcome/bound"),
        Some(&ess_conformance::report::Status::Passed),
        "{target:?}: the target reading the agent first fails `bound`: {statuses:#?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_generated_create_or_update_beside_a_related_refusal_passes_every_branch() {
    create_or_update_served(Target::Rust);
}

#[test]
fn a_generated_go_create_or_update_beside_a_related_refusal_passes_every_branch() {
    assert!(
        served::go_available(),
        "Go is required: the generated Go target is unchecked without it"
    );
    create_or_update_served(Target::Go);
}

// ---- the one precedence rule (G4) ----------------------------------------------------------

const RULE: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/precedence-rule-format-a.yaml");

/// The edit that makes a generated `ChangeTier` answer a missing tier before the seat's existence
/// and held state, as the order below the one rule reads it.
fn missing_tier_first(target: Target) -> (&'static str, &'static str) {
    match target {
        Target::Rust => (
            "        let _ = &related;\n        // The addressed row's existence and held state",
            "        let _ = &related;\n        if reference.is_some() && related.is_none() {\n            return Ok(crate::rule::ChangeTierOutcome::UnknownTier { error: crate::rule::UnknownTier });\n        }\n        // The addressed row's existence and held state",
        ),
        Target::Go => (
            "\t_ = related\n\t// The addressed row's existence and held state",
            "\t_ = related\n\tif reference != nil && related == nil {\n\t\treturn rule.ChangeTierOutcomeUnknownTier{Error: rule.UnknownTier{}}, nil\n\t}\n\t// The addressed row's existence and held state",
        ),
        _ => unreachable!("the case emits Rust and Go"),
    }
}

/// `precedence-rule-format-a.yaml` under the one rule (`docs/design/generic-language-features.md`
/// § G4): the generated behaviour answers the blank reason, then the unknown seat, then the seat no
/// longer held, then the unknown and the closed tier, and passes the suite the model synthesizes. A
/// faulty control answering the missing tier before the seat's existence and held state fails it.
fn rule_served(target: Target) {
    let ir = served::model(&served::served(RULE, "seats"));
    let synthesis = ess_synth::synthesize_for(&ir, target).expect("the model synthesizes");
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.rule.ChangeTier"),
        Some(&SynthesisDisposition::Generated),
        "{target:?}:\n{}",
        synthesis.plan.to_markdown()
    );
    let (root, _) = served::emit(&ir, target, "precedence-rule");
    let binary = served::build(&root, target, "demo", "seats", false);
    let statuses = served::run(&ir, &binary);
    served::assert_passes(
        &format!("{target:?} the one precedence rule"),
        &statuses,
        &[
            "ChangeTier/outcome/blank-reason",
            "ChangeTier/outcome/unknown-seat",
            "ChangeTier/outcome/unknown-tier",
            "ChangeTier/outcome/tier-closed",
            "ChangeTier/outcome/changed",
            "Seat/state/Released/refuses/demo.rule.ChangeTier",
        ],
    );
    let _ = std::fs::remove_dir_all(&root);

    let (root, _) = served::emit(&ir, target, "precedence-rule-faulty");
    let (from, to) = missing_tier_first(target);
    served::mutate(&served::behaviour_path(&root, target, "demo"), from, to);
    let binary = served::build(&root, target, "demo", "seats", false);
    let statuses = served::run(&ir, &binary);
    served::assert_fails(
        &format!("{target:?} the missing tier read first"),
        &statuses,
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn format_a_fixture_suite_passes_on_the_generated_rust_target() {
    rule_served(Target::Rust);
}

#[test]
fn format_a_fixture_suite_passes_on_the_generated_go_target() {
    assert!(
        served::go_available(),
        "Go is required: the generated Go target is unchecked without it"
    );
    rule_served(Target::Go);
}

/// The guard-pair corpus without the pair between `from` and `to`, and its grant.
fn without_pair(text: &str, (from, to): (&str, &str), command: &str) -> String {
    let start = text.find(from).expect("the pair");
    let end = text.find(to).expect("the pair after it");
    format!("{}{}", &text[..start], &text[end..]).replacen(&format!("      - {command}\n"), "", 1)
}

/// The guard-pair corpus without its `input_absent:` pair, for which neither target has a seam (a
/// request with no input document); and, where `served`, without its `before_row:` external pair,
/// whose provider an HTTP run cannot answer, which `external_before_row_served.rs` runs in process.
fn pairs(served: bool) -> String {
    let text =
        include_str!("../../../verify/ess-conformance/tests/fixtures/guard-pair-corpus.yaml");
    let text = without_pair(
        text,
        (
            "  # step 0 (`input_absent:`)",
            "  # `wrong_state:` (end of step 3)",
        ),
        "demo.pairs.AbsentBesideRelated",
    );
    if !served {
        return text;
    }
    without_pair(
        &text,
        (
            "  # step 1 (`external:` + `before_row: true`)",
            "  # `when_subject_state:` beside `when_subject:`",
        ),
        "demo.pairs.BeforeRowBesideRelated",
    )
}

/// The edit that makes the generated `SubjectBesideRelated` answer a missing tier before the
/// seat's existence and its stored `kind`.
fn pairs_missing_tier_first(target: Target) -> (&'static str, &'static str) {
    match target {
        Target::Rust => (
            "input: crate::pairs::SubjectBesideRelated) -> Result<crate::pairs::SubjectBesideRelatedOutcome, UnmetObligation> {\n        let _ = &input;\n",
            "input: crate::pairs::SubjectBesideRelated) -> Result<crate::pairs::SubjectBesideRelatedOutcome, UnmetObligation> {\n        let _ = &input;\n        if TierStorage::get(&self.ports, &input.tier_id).is_none() {\n            return Ok(crate::pairs::SubjectBesideRelatedOutcome::UnknownTier { error: crate::pairs::UnknownTier });\n        }\n",
        ),
        Target::Go => (
            "SubjectBesideRelated(input pairs.SubjectBesideRelated) (pairs.SubjectBesideRelatedOutcome, *obligation.UnmetObligation) {\n",
            "SubjectBesideRelated(input pairs.SubjectBesideRelated) (pairs.SubjectBesideRelatedOutcome, *obligation.UnmetObligation) {\n\tif _, found := b.ports.TierStorage.Get(input.TierId); !found {\n\t\treturn pairs.SubjectBesideRelatedOutcomeUnknownTier{Error: pairs.UnknownTier{}}, nil\n\t}\n",
        ),
        _ => unreachable!("the case emits Rust and Go"),
    }
}

/// Every command of the guard-pair corpus but the `input_absent:` one is generated in Rust and Go,
/// in the order of its plan, and every one an HTTP run can answer passes the suite the corpus
/// synthesizes. A faulty control
/// answering a missing tier before the seat's existence and stored fields fails it.
fn pairs_served(target: Target) {
    let generated = served::model(&served::served(&pairs(false), "pairs"));
    let synthesis = ess_synth::synthesize_for(&generated, target).expect("the model synthesizes");
    assert_eq!(
        synthesis.plan.disposition_of(
            CapabilityKind::CommandBehavior,
            "demo.pairs.BeforeRowBesideRelated"
        ),
        Some(&SynthesisDisposition::Generated),
        "{target:?}:\n{}",
        synthesis.plan.to_markdown()
    );
    let ir = served::model(&served::served(&pairs(true), "pairs"));
    let synthesis = ess_synth::synthesize_for(&ir, target).expect("the model synthesizes");
    for command in [
        "demo.pairs.StateBesideRelated",
        "demo.pairs.SubjectBesideRelated",
        "demo.pairs.WrongStateBesideRelatedMove",
        "demo.pairs.SubjectBesideState",
        "demo.pairs.StateBesideWrongState",
        "demo.pairs.TwoRefusalsOneRow",
    ] {
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, command),
            Some(&SynthesisDisposition::Generated),
            "{target:?} {command}:\n{}",
            synthesis.plan.to_markdown()
        );
    }
    let (root, _) = served::emit(&ir, target, "guard-pairs");
    let binary = served::build(&root, target, "demo", "pairs", false);
    let statuses = served::run(&ir, &binary);
    served::assert_passes(
        &format!("{target:?} the guard-pair corpus"),
        &statuses,
        &[
            "StateBesideRelated/outcome/released",
            "SubjectBesideRelated/outcome/vip",
            "WrongStateBesideRelatedMove/outcome/released",
            "SubjectBesideState/outcome/vip",
            "StateBesideWrongState/outcome/parked",
            "TwoRefusalsOneRow/outcome/full",
        ],
    );
    let _ = std::fs::remove_dir_all(&root);

    let (root, _) = served::emit(&ir, target, "guard-pairs-faulty");
    let (from, to) = pairs_missing_tier_first(target);
    served::mutate(&served::behaviour_path(&root, target, "demo"), from, to);
    let binary = served::build(&root, target, "demo", "pairs", false);
    let statuses = served::run(&ir, &binary);
    served::assert_fails(
        &format!("{target:?} the missing tier read first"),
        &statuses,
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn guard_pair_corpus_passes_on_the_generated_rust_target() {
    pairs_served(Target::Rust);
}

#[test]
fn guard_pair_corpus_passes_on_the_generated_go_target() {
    assert!(
        served::go_available(),
        "Go is required: the generated Go target is unchecked without it"
    );
    pairs_served(Target::Go);
}
