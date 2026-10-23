//! A `when_related:` guard over an `Optional` input that may be left out, reported by an adopter
//! (row 29, 0.69.0): `MakeChild` refuses with `other-tenant` where the input names no tenant, or
//! one other than its parent's — `{any: [not defined(input.tenant), tenant != input.tenant]}`.
//!
//! Synthesis tried only inputs that send `tenant`, so the branch had no witness, and it searched
//! boundaries no row and input can meet under three-valued comparison: a guard reading a value
//! the boundary leaves absent is `Unknown` there, never true or false. `DropChild` deletes a child
//! only `MakeChild/made` creates, so its witness is arranged through the guarded command.
//!
//! `tests/fixtures/related-guard-absent-input.yaml` is the brand-free reproduction.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/related-guard-absent-input.yaml");
const GUARD: &str = r#"any: ["not defined(input.tenant)", tenant != input.tenant]"#;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("related-guard-absent-input.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The model with `other-tenant`'s guard written as `guard`.
fn guarded(guard: &str) -> String {
    let out = MODEL.replace(GUARD, guard);
    assert_ne!(out, MODEL, "the fixture writes `{GUARD}`");
    out
}

fn refused(result: &Synthesis) -> Vec<String> {
    result
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
        .collect()
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

#[test]
fn a_related_guard_over_an_absent_optional_input_is_witnessed_and_passes() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refused(&result);
    assert!(refused.is_empty(), "{refused:#?}");
    let ran = statuses(&result, &model);
    assert!(
        ran.contains_key("tenrel.t.MakeChild/outcome/other-tenant"),
        "{:#?}",
        ran.keys().collect::<Vec<_>>()
    );
    let failed: Vec<_> = ran
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn a_target_ignoring_the_absent_input_disjunct_fails_the_suite() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    // A target taking `other-tenant` only where the input names another tenant: one that leaves
    // the tenant out creates the child. (The interpreter of `tenant != input.tenant` alone would
    // stop as undecidable on that input, not answer `made` as such a target does.)
    let mutant = ir(&guarded(
        r#"all: ["defined(input.tenant)", tenant != input.tenant]"#,
    ));
    let failed: Vec<String> = statuses(&result, &mutant)
        .into_iter()
        .filter(|(_, status)| *status == Status::Failed)
        .map(|(id, _)| id)
        .collect();
    assert!(
        failed
            .iter()
            .any(|id| id.starts_with("tenrel.t.MakeChild/outcome/other-tenant")),
        "{failed:#?}"
    );
}

#[test]
fn a_boundary_no_row_meets_under_three_valued_comparison_is_not_searched() {
    // Refuting `defined(tenant)` with the comparison held, or `defined(input.tenant)` with it
    // held, leaves the comparison `Unknown`: neither boundary is a row, so neither is refused.
    let text =
        guarded(r#"all: ["defined(tenant)", "defined(input.tenant)", tenant != input.tenant]"#);
    let model = ir(&text);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refused(&result);
    assert!(refused.is_empty(), "{refused:#?}");
    let failed: Vec<_> = statuses(&result, &model)
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn a_row_arranged_through_the_guarded_creator_keeps_its_witness() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let dropped: Vec<String> = refused(&result)
        .into_iter()
        .filter(|refusal| refusal.contains("DropChild"))
        .collect();
    assert!(dropped.is_empty(), "{dropped:#?}");
    let ran = statuses(&result, &model);
    assert_eq!(
        ran.get("tenrel.t.DropChild/outcome/dropped"),
        Some(&Status::Passed),
        "{ran:#?}"
    );
}
