//! Adversary pass 1 for beyond10x/ess#283: the joint partition's 64-assignment cap across several
//! related rows, at and just past its boundary, with and without an Optional row.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-multiple.yaml");

const STARTED: &str = "      - name: started\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("multiple-related.yaml"), raw)])
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// No default: `started` needs an active switch and reads an input enum of `variants` values, so
/// the joint partition is `variants × 2 switch states × capability cases`.
fn capped(variants: usize, optional_capability: bool) -> String {
    let names: Vec<String> = (0..variants).map(|at| format!("V{at}")).collect();
    let text = replaced(
        MODEL,
        "  - {name: demo.run.RunId, kind: newtype, of: Uuid}\n",
        &format!(
            "  - {{name: demo.run.RunId, kind: newtype, of: Uuid}}\n  - name: demo.run.Mode\n    kind: enum\n    variants: [{}]\n",
            names.join(", ")
        ),
    );
    let text = replaced(
        &text,
        "      - {name: capability, type: demo.run.CapabilityId}\n",
        &format!(
            "      - {{name: capability, type: {}}}\n      - {{name: mode, type: demo.run.Mode}}\n",
            if optional_capability {
                "Optional<demo.run.CapabilityId>"
            } else {
                "demo.run.CapabilityId"
            }
        ),
    );
    replaced(
        &text,
        STARTED,
        "      - name: started\n        when_related: {via: input.switch, predicate: state == Active}\n        when: {any: [mode == V0, mode != V0]}\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n",
    )
}

fn exceeds_cap(errors: &ValidationErrors) -> bool {
    errors.as_slice().iter().any(|error| {
        error.code == ValidationCode::NonExhaustiveBranches
            && error.to_string().contains("exceeds 64")
    })
}

#[test]
fn adversary_283_sixty_four_joint_assignments_are_proved_and_sixty_eight_need_a_default() {
    // 16 × 2 × 2 = 64: proved without a default.
    assemble(&capped(16, false)).unwrap_or_else(|errors| panic!("64 cases: {errors}"));
    // 17 × 2 × 2 = 68, though each row alone is 34: the command needs a default.
    let errors = assemble(&capped(17, false)).expect_err("68 joint cases exceed the cap");
    assert!(exceeds_cap(&errors), "{errors}");
}

#[test]
fn adversary_283_an_optional_rows_absence_counts_toward_the_cap() {
    // 10 × 2 × (2 + absent) = 60: proved.
    assemble(&capped(10, true)).unwrap_or_else(|errors| panic!("60 cases: {errors}"));
    // 11 × 2 × 3 = 66: past the cap.
    let errors = assemble(&capped(11, true)).expect_err("66 joint cases exceed the cap");
    assert!(exceeds_cap(&errors), "{errors}");
}

/// Two accepting branches over different rows beside a default, `started` reading an input enum of
/// `variants` values: the design keeps them `conflicting_declaration` where both hold.
fn two_accepting(variants: usize) -> String {
    let text = capped(variants, false);
    let text = replaced(
        &text,
        "      - name: switch-paused\n        when_related: {via: input.switch, predicate: state == Paused}\n        error: demo.run.SwitchIsPaused\n",
        "",
    );
    let text = replaced(
        &text,
        "      - name: capability-revoked\n        when_related: {via: input.capability, predicate: state == Revoked}\n        error: demo.run.CapabilityIsRevoked\n",
        "      - name: started-too\n        when_related: {via: input.capability, predicate: state == Granted}\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n",
    );
    replaced(
        &text,
        "        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n  - name: demo.run.StopRun\n",
        "        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n      - {name: refused, error: demo.run.SwitchIsPaused}\n  - name: demo.run.StopRun\n",
    )
}

fn conflicting(errors: &ValidationErrors) -> bool {
    errors.as_slice().iter().any(|error| {
        error.code == ValidationCode::ConflictingDeclaration
            && error.to_string().contains("started-too")
    })
}

#[test]
fn adversary_283_two_accepting_branches_over_two_rows_stay_ambiguous_past_the_cap() {
    // 16 × 2 × 2 = 64: an active switch under a granted capability selects both.
    let errors = assemble(&two_accepting(16)).expect_err("both accepting branches hold");
    assert!(conflicting(&errors), "{errors}");
    // 17 × 2 × 2 = 68: the same overlap, past the joint cap of rows each well under it alone.
    let errors = assemble(&two_accepting(17))
        .expect_err("past the cap the overlap is still two branches for one request");
    assert!(conflicting(&errors), "{errors}");
}
