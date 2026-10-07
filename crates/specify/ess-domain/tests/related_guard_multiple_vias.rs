//! A command guarded by more than one related row (beyond10x/ess#283, `ess/22`): one `exists: false`
//! branch per `via`, each row's predicates checked against its own entity, and one precedence
//! across rows — missing rows first, in the declaration order of their `exists: false` branches,
//! then the present-related predicate refusals in declaration order, before every accepting branch.
//! Two refusals over one row still overlap ambiguously; below `ess/22` a second row is refused
//! naming the format that admits it.
use ess_domain::{
    command::{OutcomeCondition, RelatedTest, RelatedVia},
    spec::RawSpecFile,
    system::Source,
    Specification,
};
use ess_primitives::error::{ValidationCode, ValidationErrors};

/// The issue's shape: a run starts only on a switch that is not paused, under a capability that is
/// not revoked — two rows, each with its own `exists: false` answer.
const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-multiple.yaml");

const NO_SWITCH: &str = "      - name: no-such-switch\n        when_related: {via: input.switch, exists: false}\n        error: demo.run.NoSuchSwitch\n";
const SWITCH_PAUSED: &str = "      - name: switch-paused\n        when_related: {via: input.switch, predicate: state == Paused}\n        error: demo.run.SwitchIsPaused\n";
const NO_CAPABILITY: &str = "      - name: no-such-capability\n        when_related: {via: input.capability, exists: false}\n        error: demo.run.NoSuchCapability\n";
const CAPABILITY_REVOKED: &str = "      - name: capability-revoked\n        when_related: {via: input.capability, predicate: state == Revoked}\n        error: demo.run.CapabilityIsRevoked\n";
const STARTED: &str = "      - name: started\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("multiple-related.yaml"), raw)])
}

fn accepted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn refused(text: &str) -> ValidationErrors {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
}

fn has(errors: &ValidationErrors, code: ValidationCode, fragment: &str) -> bool {
    errors
        .as_slice()
        .iter()
        .any(|error| error.code == code && error.to_string().contains(fragment))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn start_outcomes(body: &str) -> String {
    replaced(
        MODEL,
        &format!("{NO_SWITCH}{SWITCH_PAUSED}{NO_CAPABILITY}{CAPABILITY_REVOKED}{STARTED}"),
        body,
    )
}

#[test]
fn issue_283_two_vias_with_one_exists_false_each_validate_at_ess_22() {
    let spec = accepted(MODEL);
    let command = &spec.commands()[&"demo.run.StartRun".parse().unwrap()];
    let read: Vec<(&str, &RelatedVia, bool)> = command
        .outcomes
        .iter()
        .filter_map(|outcome| match &outcome.condition {
            OutcomeCondition::Related { via, test, .. } => {
                Some((outcome.name.as_str(), via, *test == RelatedTest::Absent))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        read.iter()
            .map(|(name, via, absent)| format!("{name}:{via}:{absent}"))
            .collect::<Vec<_>>(),
        [
            "no-such-switch:input.switch:true",
            "switch-paused:input.switch:false",
            "no-such-capability:input.capability:true",
            "capability-revoked:input.capability:false",
        ],
        "each branch keeps the row its own `via` names"
    );
}

#[test]
fn issue_283_declaration_order_of_the_rows_is_free() {
    accepted(&start_outcomes(&format!(
        "{NO_CAPABILITY}{CAPABILITY_REVOKED}{NO_SWITCH}{SWITCH_PAUSED}{STARTED}"
    )));
    accepted(&start_outcomes(&format!(
        "{SWITCH_PAUSED}{CAPABILITY_REVOKED}{NO_CAPABILITY}{NO_SWITCH}{STARTED}"
    )));
}

#[test]
fn issue_283_a_second_row_below_ess_22_is_refused_naming_ess_22() {
    for format in ["ess/21", "ess/20"] {
        let text = replaced(MODEL, "format: ess/22\n", &format!("format: {format}\n"));
        let errors = refused(&text);
        assert!(
            has(&errors, ValidationCode::UnsupportedFormatVersion, "ess/22"),
            "{format}: a second related row is refused naming ess/22: {errors}"
        );
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.to_string().contains("no-such-capability")
                    || error.to_string().contains("input.capability")),
            "{format}: the refusal names the second row: {errors}"
        );
    }
}

#[test]
fn issue_283_two_exists_false_branches_on_one_row_stay_refused() {
    let second = "      - name: also-no-switch\n        when_related: {via: input.switch, exists: false}\n        error: demo.run.NoSuchSwitch\n";
    let errors = refused(&start_outcomes(&format!(
        "{NO_SWITCH}{second}{SWITCH_PAUSED}{NO_CAPABILITY}{CAPABILITY_REVOKED}{STARTED}"
    )));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "more than one answer for a missing row"
        ),
        "{errors}"
    );
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "also-no-switch"
        ),
        "the refusal names the second branch over the same row: {errors}"
    );
}

#[test]
fn issue_283_each_row_read_by_a_predicate_needs_its_own_exists_false() {
    let errors = refused(&start_outcomes(&format!(
        "{NO_SWITCH}{SWITCH_PAUSED}{CAPABILITY_REVOKED}{STARTED}"
    )));
    assert!(
        has(
            &errors,
            ValidationCode::NonExhaustiveBranches,
            "input.capability"
        ),
        "the row with no `exists: false` answer is named: {errors}"
    );
}

#[test]
fn issue_283_a_row_without_predicates_needs_only_its_exists_false() {
    accepted(&start_outcomes(&format!(
        "{NO_SWITCH}{SWITCH_PAUSED}{NO_CAPABILITY}{STARTED}"
    )));
}

#[test]
fn issue_283_each_predicate_is_checked_against_its_own_rows_entity() {
    // `Paused` is a state of `Switch`, not of `Capability`.
    let wrong = "      - name: capability-revoked\n        when_related: {via: input.capability, predicate: state == Paused}\n        error: demo.run.CapabilityIsRevoked\n";
    let errors = refused(&start_outcomes(&format!(
        "{NO_SWITCH}{SWITCH_PAUSED}{NO_CAPABILITY}{wrong}{STARTED}"
    )));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("capability-revoked")
                && error.to_string().contains("demo.run.Capability.State")),
        "the predicate is refused at its own branch, against its own row's entity: {errors}"
    );
}

#[test]
fn issue_283_two_overlapping_refusals_over_one_row_stay_ambiguous() {
    let also = "      - name: switch-paused-again\n        when_related: {via: input.switch, predicate: state != Active}\n        error: demo.run.SwitchIsPaused\n";
    let errors = refused(&start_outcomes(&format!(
        "{NO_SWITCH}{SWITCH_PAUSED}{also}{NO_CAPABILITY}{CAPABILITY_REVOKED}{STARTED}"
    )));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "switch-paused-again"
        ),
        "{errors}"
    );
}

#[test]
fn issue_283_overlapping_accepting_branches_over_two_rows_stay_ambiguous() {
    let errors_text = start_outcomes(&format!(
        "{NO_SWITCH}{NO_CAPABILITY}      - name: started\n        when_related: {{via: input.switch, predicate: state == Active}}\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {{demo.run.RunStarted: {{run_id: {{generated: true}}}}}}\n      - name: started-again\n        when_related: {{via: input.capability, predicate: state == Granted}}\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {{demo.run.RunStarted: {{run_id: {{generated: true}}}}}}\n      - {{name: refused, error: demo.run.SwitchIsPaused}}\n"
    ));
    let errors = refused(&errors_text);
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "started-again"
        ),
        "only refusals are ordered across rows: {errors}"
    );
}

#[test]
fn issue_283_an_optional_second_row_still_needs_its_absent_case_answered() {
    let text = replaced(
        MODEL,
        "      - {name: capability, type: demo.run.CapabilityId}\n",
        "      - {name: capability, type: Optional<demo.run.CapabilityId>}\n",
    );
    accepted(&text);
    // Without a default, the run starts only on a granted capability: a present row is answered
    // in each of its states, and an absent capability on an active switch by nothing.
    let granted = "      - name: started\n        when_related: {via: input.capability, predicate: state == Granted}\n        creates: demo.run.Run\n        instance: run_id\n        emits: [demo.run.RunStarted]\n        payload: {demo.run.RunStarted: {run_id: {generated: true}}}\n";
    let without_default = replaced(&text, STARTED, granted);
    let errors = refused(&without_default);
    assert!(
        has(&errors, ValidationCode::NonExhaustiveBranches, "absent"),
        "the absent capability is left unanswered: {errors}"
    );
    // Required, the same branches answer every row that exists.
    accepted(&replaced(MODEL, STARTED, granted));
}

#[test]
fn issue_283_a_stored_via_beside_an_input_via_stays_refused() {
    // A stored-field `via` is read at its own later step; beside an input one no order is stated.
    let text = replaced(
        MODEL,
        "  - name: demo.run.StopRun\n    input:\n      - {name: run_id, type: demo.run.RunId}\n    outcomes:\n",
        "  - name: demo.run.StopRun\n    input:\n      - {name: run_id, type: demo.run.RunId}\n      - {name: switch, type: demo.run.SwitchId}\n    outcomes:\n      - name: no-such-switch\n        when_related: {via: input.switch, exists: false}\n        error: demo.run.NoSuchSwitch\n      - name: no-such-capability\n        when_related: {via: capability, exists: false}\n        error: demo.run.NoSuchCapability\n",
    );
    let text = replaced(
        &text,
        "    identity: {name: run_id, type: demo.run.RunId}\n    fields: []\n",
        "    identity: {name: run_id, type: demo.run.RunId}\n    fields:\n      - {name: capability, type: Optional<demo.run.CapabilityId>}\n",
    );
    let errors = refused(&text);
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "one related row"
        ) || has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "more than one answer for a missing row"
        ),
        "refused as a command reading one related row: {errors}"
    );
    assert!(
        !has(&errors, ValidationCode::NonExhaustiveBranches, "row `"),
        "not answered row by row: {errors}"
    );
}
