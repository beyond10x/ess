//! Adversary pass 1 over the integrated 0.73.0 wave (U1 set-effect binding causes, U2 bindings no
//! step sets off, U3 predicate refusal policies through an arranged row, U4 affects rows past a
//! creator's predicate guard).
//!
//! Each model is synthesized and every scenario run against the interpreter of the same model.
//! The fixtures under `fixtures/adversary-0730/` hold no `format:` line and no `system.yaml`, so the
//! repository-wide suite-bytes tables do not walk them; the system document is written here.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

fn system(domain: &str) -> String {
    format!("format: ess/25\nsystem: demo\nversion: v1\ndomains:\n  - {domain}\n")
}

fn model(files: &[(&str, &str)]) -> EssIr {
    let mut sources = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.push((Source::new(*label), raw));
    }
    let spec = Specification::assemble(sources)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn steps_of(synthesis: &Synthesis, id: &str) -> Vec<Value> {
    let document: Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` is synthesized: {:#?}", refusals(synthesis)))
        .clone()
}

/// Every scenario the interpreter of the same model does not pass, with its diagnostics.
fn failing(ir: &EssIr, synthesis: &Synthesis) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
        .into_report();
    assert_eq!(report.scenarios.len(), synthesis.suite.scenarios.len());
    report
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| {
            let diagnostics: Vec<String> = result.diagnostics().map(|d| format!("{d:?}")).collect();
            (
                result.scenario.to_string(),
                format!("{:?}: {diagnostics:#?}", result.status),
            )
        })
        .collect()
}

/// F1 (U1). `SyncAll` updates every ticket `where: grant_id == input.grant_id` and publishes one
/// `TicketSynced` per row; `OpenTicket` refuses a missing grant, so the ticket the flow needs is
/// opened naming a grant arranged first, and the trigger `SetFlag` is re-sent naming that grant in
/// place of the literal its own scenario chose. `SetFlag` refuses `blocked` `when: grant_id !=
/// "g-1"`: a bare input path, which `keeps_branch_sent` reads only under `input.` and so does not
/// count as a guard on `grant_id`. The renamed trigger carries a generated grant id, takes
/// `blocked`, publishes nothing, and the trigger's own outcome scenario and the binding's flow,
/// delivery, mapping and failure scenarios all fail against the interpreter.
#[test]
fn f1_a_trigger_input_guard_keeps_the_renamed_trigger_on_its_branch() {
    let ir = model(&[
        ("system.yaml", &system("demo.c")),
        (
            "components.yaml",
            include_str!("fixtures/adversary-0730/trigger-input-guard/components.yaml"),
        ),
        (
            "c.yaml",
            include_str!("fixtures/adversary-0730/trigger-input-guard/c.yaml"),
        ),
    ]);
    let synthesis = synthesize(&ir);
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
}

/// F2 (U3, the arrangement it reuses). `Give` refuses `not-flagged` over the grant's `flag`, and the
/// binding retries it three times. Synthesis arranges a grant with `flag == false` through
/// `Lower` and sends the trigger naming it; but `Lower` publishes `Lowered`, which the binding
/// `raise-on-lowered` answers by setting the flag back to true. At the trigger the grant is
/// flagged, `Give` answers `given` once, and the scenario's three counted attempts, like the
/// outcome scenario `Give/outcome/not-flagged`, fail against the interpreter. `refused_row`'s
/// `sets_off` filter asks only whether the arrangement sets off the binding under test.
#[test]
fn f2_a_binding_that_rewrites_the_arranged_row_is_followed() {
    let ir = model(&[
        ("system.yaml", &system("demo.d")),
        (
            "components.yaml",
            include_str!("fixtures/adversary-0730/restoring-binding/components.yaml"),
        ),
        (
            "domains/d.yaml",
            include_str!("fixtures/adversary-0730/restoring-binding/domains/d.yaml"),
        ),
    ]);
    let synthesis = synthesize(&ir);
    // The binding's refusal-policy scenario is now refused by name, which is this wave's part.
    // The outcome scenario `Give/outcome/not-flagged` already failed on the released 0.72.0: the
    // related-guard arrangement treats the row its steps leave as final and does not follow a
    // binding those steps set off. Pinned as the one known failure until
    // story:an-outcome-arrangement-follows-the-bindings-its-steps-set-off lands; that story
    // empties this list.
    let failures = failing(&ir, &synthesis);
    assert_eq!(
        failures.keys().map(String::as_str).collect::<Vec<_>>(),
        ["demo.d.Give/outcome/not-flagged"],
        "{failures:#?}"
    );
}

/// F3 (U3). The fixture of `a_predicate_refusal_policy_is_forced_through_an_arranged_row`, with
/// `not-flagged` left to the unbounded `retry: {except: [missing]}`. The refused grant can be
/// arranged exactly as there (`DefineGrant` takes `flag`, `Touch` copies the identity), but an
/// unbounded retry goes through `retried`, which never tries an arrangement; `forcible` now names
/// every predicate refusal `RelatedRefusalUnarranged`, so the refusal says no such row can be
/// arranged, and its help names two conditions that both already hold.
#[test]
fn f3_an_unbounded_retry_refusal_does_not_claim_the_row_unarrangeable() {
    let ir = model(&[
        ("system.yaml", &system("demo.d")),
        (
            "components.yaml",
            include_str!("fixtures/adversary-0730/unbounded-retry/components.yaml"),
        ),
        (
            "domains/d.yaml",
            include_str!("fixtures/adversary-0730/unbounded-retry/domains/d.yaml"),
        ),
    ]);
    let synthesis = synthesize(&ir);
    let refused: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains("give-on-touched/binding/refusal/not-flagged"))
        .collect();
    assert_eq!(refused.len(), 1, "{refused:#?}");
    assert!(
        !refused[0].contains("can be arranged before the trigger"),
        "a grant the predicate refuses is arranged for the drop policy on this same model: {}",
        refused[0]
    );
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
}

/// Pinned (U3). The same model with `not-flagged` left to a bounded fallback retry of three
/// attempts: the grant is arranged with `flag == false`, the trigger names it, nothing is
/// injected, three attempts are counted, and every scenario passes.
#[test]
fn pinned_a_bounded_retry_of_a_predicate_refusal_counts_every_attempt() {
    let ir = model(&[
        ("system.yaml", &system("demo.d")),
        (
            "components.yaml",
            include_str!("fixtures/adversary-0730/bounded-retry/components.yaml"),
        ),
        (
            "domains/d.yaml",
            include_str!("fixtures/adversary-0730/bounded-retry/domains/d.yaml"),
        ),
    ]);
    let synthesis = synthesize(&ir);
    assert_eq!(refusals(&synthesis), Vec::<String>::new());
    let steps = steps_of(&synthesis, "give-on-touched/binding/refusal/not-flagged");
    assert!(
        steps
            .iter()
            .all(|step| step["step"] != "configure_external_outcome"),
        "{steps:#?}"
    );
    let counts: Vec<u64> = steps
        .iter()
        .filter(|step| step["step"] == "expect_invocation")
        .filter_map(|step| step["count"].as_u64())
        .collect();
    assert_eq!(counts, [3]);
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
}

/// Pinned (U2). The handover is set off through a chain: `Issued` sets off `note-on-issued`, whose
/// `Noted` sets off `hand-over-on-noted`, which rewrites a ticket's holder. Every arrangement of
/// `Reconcile` issues a ticket, so both branches stay refused, and every scenario passes.
#[test]
fn pinned_a_binding_set_off_through_a_chain_leaves_the_selector_unfollowed() {
    let ir = model(&[
        ("system.yaml", &system("demo.y")),
        (
            "domains/y.yaml",
            include_str!("fixtures/adversary-0730/chained-binding/domains/y.yaml"),
        ),
    ]);
    let synthesis = synthesize(&ir);
    let refused = refusals(&synthesis);
    for id in [
        "demo.y.Reconcile/outcome/kept",
        "demo.y.Reconcile/outcome/dropped",
    ] {
        assert!(
            refused
                .iter()
                .any(|refusal| refusal.contains(&format!("`{id}`"))
                    && refusal.contains("changes rows of the selector's entity")),
            "{id}: {refused:#?}"
        );
    }
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
}

/// Pinned (U4). Two creators of a ticket, each behind a branch over the member's `kind`
/// (`OpenTicket` refusing `Linked`, `OpenOther` refusing `Plain`): the two negations together are
/// unsatisfiable, and the reset is still arranged and passes.
#[test]
fn pinned_two_creators_with_opposite_predicates_leave_the_reset_arranged() {
    let text = format!(
        "format: ess/32\n{}",
        include_str!("fixtures/adversary-0730/two-creators/model.yaml")
    );
    let ir = model(&[("model.yaml", &text)]);
    let synthesis = synthesize(&ir);
    assert_eq!(refusals(&synthesis), Vec::<String>::new());
    steps_of(&synthesis, "demo.v.ResetMember/outcome/reset");
    assert_eq!(failing(&ir, &synthesis), BTreeMap::new());
}
