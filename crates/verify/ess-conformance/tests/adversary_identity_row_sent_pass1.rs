//! Adversary pass 1 over
//! `story:identity-guard-decided-on-the-row-sent-on-every-synthesis-path`.
//!
//! The story's two models, varied along what its words cover: the guard forms (`==`, `!=`,
//! membership, negated membership), Integer, enum and struct identities, a compound guard, a
//! creator naming the identity through another input, two identity inputs held equal, a row-set
//! accepting branch and an identity guard beside a row set, the refusal arm where no row meets
//! the guard and the silence where no row can be arranged; and the paths the unit names as not
//! done (a stored-row refusal beside the held-state branches, a further input refusal sent in
//! each held state). Every model is inline and neutral.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

/// The story's `complete.yaml`: `ClosePass` refuses the pass `"frozen"` as a complete refusal and
/// otherwise deletes it.
const COMPLETE: &str = r#"format: ess/36
system: demo
version: v1
domain: demo.passes
summary: A pass is opened and closed; the pass frozen cannot be closed.
types:
  - {name: demo.passes.PassId, kind: newtype, of: String}
entities:
  - name: demo.passes.Pass
    identity: {name: pass_id, type: demo.passes.PassId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
views:
  - name: demo.passes.Passes
    source: demo.passes.Pass
    consistency: read_your_writes
    fields:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
actors:
  - name: demo.passes.User
    may: [demo.passes.OpenPass, demo.passes.ClosePass]
errors:
  - name: demo.passes.Frozen
    fields:
      - {name: pass_id, type: demo.passes.PassId}
commands:
  - name: demo.passes.OpenPass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: demo.passes.Pass
        instance: pass_id
        sets: {note: input.note}
        emits: [demo.passes.PassOpened]
        payload:
          demo.passes.PassOpened: {pass_id: input.pass_id}
  - name: demo.passes.ClosePass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: reason, type: String}
    outcomes:
      - name: frozen
        when: 'pass_id == "frozen"'
        error: demo.passes.Frozen
        payload:
          demo.passes.Frozen: {pass_id: input.pass_id}
      - name: closed
        deletes: demo.passes.Pass
        instance: pass_id
        emits: [demo.passes.PassClosed]
        payload:
          demo.passes.PassClosed: {pass_id: input.pass_id}
events:
  - name: demo.passes.PassOpened
    fields:
      - {name: pass_id, type: demo.passes.PassId}
  - name: demo.passes.PassClosed
    fields:
      - {name: pass_id, type: demo.passes.PassId}
"#;

/// The story's `state.yaml`: `ClosePass` refuses every pass but `"main"`, then closes an open pass
/// and refuses a closed one.
const STATE: &str = r#"format: ess/36
system: demo
version: v1
domain: demo.passes
summary: A pass is opened and closed; only the pass main can be closed.
types:
  - {name: demo.passes.PassId, kind: newtype, of: String}
entities:
  - name: demo.passes.Pass
    identity: {name: pass_id, type: demo.passes.PassId}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
views:
  - name: demo.passes.Passes
    source: demo.passes.Pass
    consistency: read_your_writes
    fields:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: state, type: demo.passes.Pass.State}
      - {name: note, type: String}
actors:
  - name: demo.passes.User
    may: [demo.passes.OpenPass, demo.passes.ClosePass]
errors:
  - name: demo.passes.Frozen
    fields:
      - {name: pass_id, type: demo.passes.PassId}
  - name: demo.passes.NotOpen
    fields:
      - {name: pass_id, type: demo.passes.PassId}
commands:
  - name: demo.passes.OpenPass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: demo.passes.Pass
        instance: pass_id
        sets: {note: input.note}
        emits: [demo.passes.PassOpened]
        payload:
          demo.passes.PassOpened: {pass_id: input.pass_id}
  - name: demo.passes.ClosePass
    input:
      - {name: pass_id, type: demo.passes.PassId}
      - {name: reason, type: String}
    outcomes:
      - name: frozen
        when: 'pass_id != "main"'
        error: demo.passes.Frozen
        payload:
          demo.passes.Frozen: {pass_id: input.pass_id}
      - name: closed
        when_subject_state: Open
        moves: demo.passes.Pass.close
        instance: pass_id
        emits: [demo.passes.PassClosed]
        payload:
          demo.passes.PassClosed: {pass_id: input.pass_id}
      - name: not-open
        when_subject_state: Closed
        error: demo.passes.NotOpen
        payload:
          demo.passes.NotOpen: {pass_id: input.pass_id}
events:
  - name: demo.passes.PassOpened
    fields:
      - {name: pass_id, type: demo.passes.PassId}
  - name: demo.passes.PassClosed
    fields:
      - {name: pass_id, type: demo.passes.PassId}
"#;

const STRING_ID: &str = "  - {name: demo.passes.PassId, kind: newtype, of: String}\n";
const COMPLETE_GUARD: &str = "        when: 'pass_id == \"frozen\"'\n";
const STATE_GUARD: &str = "        when: 'pass_id != \"main\"'\n";
const OPEN_INPUT: &str = "    input:\n      - {name: pass_id, type: demo.passes.PassId}\n      - {name: note, type: String}\n";
const OPEN_PAYLOAD: &str = "demo.passes.PassOpened: {pass_id: input.pass_id}";

/// The four scenarios the story names on `state.yaml`.
const HELD: [&str; 4] = [
    "demo.passes.ClosePass/outcome/closed",
    "demo.passes.ClosePass/outcome/not-open",
    "demo.passes.Pass/state/Closed/refuses/demo.passes.ClosePass",
    "demo.passes.Pass/transition/close/by/demo.passes.ClosePass/closed",
];

const FROZEN: &str = "demo.passes.ClosePass/outcome/frozen";

fn replaced(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

struct Run {
    synthesis: Synthesis,
    results: BTreeMap<String, ScenarioResult>,
}

fn run(text: &str) -> Run {
    let model = ir(text);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let results = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect();
    Run { synthesis, results }
}

fn refused(run: &Run, wanted: &str) -> Vec<String> {
    run.synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(wanted)
        })
        .map(|refusal| format!("[{}] {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn written(run: &Run, wanted: &str) -> Option<Value> {
    run.synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == wanted)
        .map(|(_, scenario)| serde_json::to_value(scenario).unwrap())
}

fn assert_all_pass(run: &Run) {
    let failed: Vec<String> = run
        .results
        .iter()
        .filter(|(_, result)| result.status != Status::Passed)
        .map(|(id, result)| {
            format!(
                "{id}: {:?} {:#?}",
                result.status,
                result.diagnostics().collect::<Vec<_>>()
            )
        })
        .collect();
    assert!(
        failed.is_empty(),
        "every scenario passes on the interpreted target:\n{}",
        failed.join("\n")
    );
}

/// The `field` input every `OpenPass` step of `wanted` sends, as written (`{"value": ...}`).
fn opened(run: &Run, wanted: &str, field: &str) -> Vec<Value> {
    let scenario = written(run, wanted).unwrap_or_else(|| {
        panic!(
            "{wanted} is written, not refused: {:#?}",
            refused(run, wanted)
        )
    });
    let sent: Vec<Value> = scenario["steps"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter(|step| {
            step["step"] == "execute_command" && step["command"] == "demo.passes.OpenPass"
        })
        .map(|step| step["input"][field].clone())
        .collect();
    assert!(!sent.is_empty(), "{wanted} opens a pass: {scenario:#}");
    sent
}

/// `wanted` is written and every pass it opens meets `meets`.
fn opens_only(run: &Run, wanted: &str, field: &str, what: &str, meets: impl Fn(&Value) -> bool) {
    let sent = opened(run, wanted, field);
    assert!(
        sent.iter().all(|value| meets(&value["value"])),
        "{wanted} opens only passes {what}: {sent:?}"
    );
}

fn text_in(allowed: &'static [&'static str]) -> impl Fn(&Value) -> bool {
    move |value| value.as_str().is_some_and(|text| allowed.contains(&text))
}

// ---------------------------------------------------------------------------------------------
// addressed_row: a complete input-guarded refusal
// ---------------------------------------------------------------------------------------------

/// Membership: `frozen` guarded by `pass_id in ["frozen", "held"]` opens one of the two.
#[test]
fn complete_membership_guard_opens_a_member() {
    let text = replaced(
        COMPLETE,
        COMPLETE_GUARD,
        "        when: {pass_id: [\"frozen\", \"held\"]}\n",
    );
    let run = run(&text);
    opens_only(
        &run,
        FROZEN,
        "pass_id",
        "frozen or held",
        text_in(&["frozen", "held"]),
    );
    assert_all_pass(&run);
}

/// `!=`: `frozen` refuses every pass but `"main"`; it opens another pass and `closed` opens
/// `"main"`.
#[test]
fn complete_not_equal_guard_leaves_each_branch_its_row() {
    let text = replaced(COMPLETE, COMPLETE_GUARD, STATE_GUARD);
    let run = run(&text);
    opens_only(&run, FROZEN, "pass_id", "other than main", |value| {
        value.as_str().is_some_and(|text| text != "main")
    });
    opens_only(
        &run,
        "demo.passes.ClosePass/outcome/closed",
        "pass_id",
        "main",
        text_in(&["main"]),
    );
    assert_all_pass(&run);
}

/// An Integer identity: `frozen` guarded by `pass_id > 5` opens a pass above 5.
#[test]
fn complete_integer_range_opens_a_pass_inside_it() {
    let text = replaced(
        &replaced(
            COMPLETE,
            STRING_ID,
            "  - {name: demo.passes.PassId, kind: newtype, of: Integer}\n",
        ),
        COMPLETE_GUARD,
        "        when: pass_id > 5\n",
    );
    let run = run(&text);
    opens_only(&run, FROZEN, "pass_id", "above 5", |value| {
        value.as_f64().is_some_and(|number| number > 5.0)
    });
    assert_all_pass(&run);
}

/// An enum identity: `frozen` guarded by `pass_id == b` opens the pass `b`.
#[test]
fn complete_enum_identity_opens_the_variant() {
    let text = replaced(
        &replaced(
            COMPLETE,
            STRING_ID,
            "  - {name: demo.passes.PassId, kind: enum, variants: [a, b, c]}\n",
        ),
        COMPLETE_GUARD,
        "        when: pass_id == b\n",
    );
    let run = run(&text);
    opens_only(&run, FROZEN, "pass_id", "b", text_in(&["b"]));
    assert_all_pass(&run);
}

/// A compound guard reading the identity beside another input.
#[test]
fn complete_compound_guard_opens_the_identity_part() {
    let text = replaced(
        COMPLETE,
        COMPLETE_GUARD,
        "        when: {all: ['pass_id == \"frozen\"', 'reason == \"late\"']}\n",
    );
    let run = run(&text);
    opens_only(&run, FROZEN, "pass_id", "frozen", text_in(&["frozen"]));
    assert_all_pass(&run);
}

/// The creator names the identity through an input of another name.
#[test]
fn complete_creator_input_of_another_name_is_steered() {
    let text = replaced(
        &replaced(
            COMPLETE,
            OPEN_INPUT,
            "    input:\n      - {name: code, type: demo.passes.PassId}\n      - {name: note, type: String}\n",
        ),
        OPEN_PAYLOAD,
        "demo.passes.PassOpened: {pass_id: input.code}",
    );
    let run = run(&text);
    opens_only(&run, FROZEN, "code", "frozen", text_in(&["frozen"]));
    assert_all_pass(&run);
}

/// A struct identity of two parts, the guard on one part.
#[test]
fn complete_struct_identity_guard_on_one_part_opens_that_part() {
    let text = replaced(
        &replaced(
            COMPLETE,
            STRING_ID,
            "  - name: demo.passes.PassId\n    kind: struct\n    fields:\n      - {name: region, type: String}\n      - {name: number, type: String}\n",
        ),
        COMPLETE_GUARD,
        "        when: 'pass_id.region == \"eu\"'\n",
    );
    let run = run(&text);
    opens_only(&run, FROZEN, "pass_id", "in region eu", |value| {
        value.to_string().contains("\"eu\"")
    });
    assert_all_pass(&run);
}

/// Two identity inputs a guard holds equal: the refusal is witnessed, not refused.
#[test]
fn complete_guard_holding_two_identity_inputs_equal_is_witnessed() {
    let text = replaced(
        &replaced(
            COMPLETE,
            "      - {name: pass_id, type: demo.passes.PassId}\n      - {name: reason, type: String}\n",
            "      - {name: pass_id, type: demo.passes.PassId}\n      - {name: other_id, type: demo.passes.PassId}\n      - {name: reason, type: String}\n",
        ),
        COMPLETE_GUARD,
        "        when: pass_id == other_id\n",
    );
    let run = run(&text);
    assert!(
        written(&run, FROZEN).is_some(),
        "{FROZEN} is witnessed, two inputs naming one pass: {:#?}",
        refused(&run, FROZEN)
    );
    assert_all_pass(&run);
}

/// No pass `"frozen"` can be opened: `frozen` is refused `ESS-SYNTH-003` naming its guard.
#[test]
fn complete_guard_no_row_meets_is_refused_naming_it() {
    let text = replaced(
        COMPLETE,
        "      - name: opened\n",
        "      - name: refused\n        when: 'pass_id == \"frozen\"'\n        error: demo.passes.Frozen\n        payload:\n          demo.passes.Frozen: {pass_id: input.pass_id}\n      - name: opened\n",
    );
    let run = run(&text);
    assert!(
        written(&run, FROZEN).is_none(),
        "{FROZEN} is reached by no row and is not written: {:#}",
        written(&run, FROZEN).unwrap_or_default()
    );
    let refusals = refused(&run, FROZEN);
    assert!(
        !refusals.is_empty()
            && refusals.iter().all(|refusal| {
                refusal.starts_with("[ESS-SYNTH-003]") && refusal.contains("pass_id")
            }),
        "{FROZEN} is refused ESS-SYNTH-003 naming its guard: {refusals:#?}"
    );
    assert_all_pass(&run);
}

/// No pass can be arranged at all (no command creates one): `frozen` is not refused
/// `ESS-SYNTH-003` for want of a row meeting its guard.
#[test]
fn complete_guard_without_a_creator_is_not_refused_on_the_guard() {
    let open = "  - name: demo.passes.OpenPass\n    input:\n      - {name: pass_id, type: demo.passes.PassId}\n      - {name: note, type: String}\n    outcomes:\n      - name: opened\n        creates: demo.passes.Pass\n        instance: pass_id\n        sets: {note: input.note}\n        emits: [demo.passes.PassOpened]\n        payload:\n          demo.passes.PassOpened: {pass_id: input.pass_id}\n";
    let text = replaced(
        &replaced(COMPLETE, open, ""),
        "    may: [demo.passes.OpenPass, demo.passes.ClosePass]\n",
        "    may: [demo.passes.ClosePass]\n",
    );
    let run = run(&text);
    let refusals = refused(&run, FROZEN);
    assert!(
        refusals
            .iter()
            .all(|refusal| !refusal.starts_with("[ESS-SYNTH-003]")),
        "{FROZEN} is not refused ESS-SYNTH-003 where no row can be arranged: {refusals:#?}"
    );
    assert_all_pass(&run);
}

/// The target mints the pass identity, so no row meeting `pass_id == <literal>` can be arranged:
/// the refusal is not written for the interpreter to fail.
#[test]
#[ignore = "backlog: identity-row-remaining-paths, story:an-identity-guard-is-decided-on-every-row-synthesis-arranges"]
fn complete_guard_on_a_minted_identity_is_not_written_failing() {
    let text = replaced(
        &replaced(
            &replaced(
                &replaced(
                    COMPLETE,
                    STRING_ID,
                    "  - {name: demo.passes.PassId, kind: newtype, of: Uuid}\n",
                ),
                OPEN_INPUT,
                "    input:\n      - {name: note, type: String}\n",
            ),
            OPEN_PAYLOAD,
            "demo.passes.PassOpened: {pass_id: {generated: true}}",
        ),
        COMPLETE_GUARD,
        "        when: 'pass_id == \"00000000-0000-4000-8000-0000000000ff\"'\n",
    );
    let run = run(&text);
    assert_all_pass(&run);
}

// ---------------------------------------------------------------------------------------------
// addressed_row: row sets
// ---------------------------------------------------------------------------------------------

const FROZEN_BRANCH: &str = "      - name: frozen\n        when: 'pass_id == \"frozen\"'\n        error: demo.passes.Frozen\n        payload:\n          demo.passes.Frozen: {pass_id: input.pass_id}\n";
const HELD_BRANCH: &str = "      - name: held\n        when_related:\n          entity: demo.passes.Hold\n          where: pass_id == input.pass_id\n          exists: true\n        error: demo.passes.Held\n";
const VIP_BRANCH: &str =
    "      - name: vip\n        when: 'pass_id == \"vip\"'\n        accepts: nothing\n";

/// [`COMPLETE`] with a labelled hold on a pass, a `PlaceHold` creating it, and `branches` (full
/// outcomes) in place of `frozen`.
fn with_row_set(branches: &str) -> String {
    let text = replaced(COMPLETE, FROZEN_BRANCH, branches);
    let text = replaced(
        &text,
        STRING_ID,
        "  - {name: demo.passes.PassId, kind: newtype, of: String}\n  - {name: demo.passes.HoldId, kind: newtype, of: Uuid}\n",
    );
    let text = replaced(
        &text,
        "views:\n",
        "  - name: demo.passes.Hold\n    identity: {name: hold_id, type: demo.passes.HoldId}\n    fields:\n      - {name: pass_id, type: demo.passes.PassId}\n      - {name: label, type: String}\n    lifecycle: {initial: Placed, states: [Placed], terminal: [Placed]}\nviews:\n",
    );
    let text = replaced(
        &text,
        "    may: [demo.passes.OpenPass, demo.passes.ClosePass]\n",
        "    may: [demo.passes.OpenPass, demo.passes.ClosePass, demo.passes.PlaceHold]\n",
    );
    let text = replaced(
        &text,
        "errors:\n",
        "errors:\n  - {name: demo.passes.Held, fields: []}\n",
    );
    let text = replaced(
        &text,
        "  - name: demo.passes.ClosePass\n",
        "  - name: demo.passes.PlaceHold\n    input:\n      - {name: pass_id, type: demo.passes.PassId}\n      - {name: label, type: String}\n    outcomes:\n      - name: placed\n        creates: demo.passes.Hold\n        instance: hold_id\n        sets: {pass_id: input.pass_id, label: input.label}\n        emits: [demo.passes.HoldPlaced]\n        payload:\n          demo.passes.HoldPlaced: {hold_id: {generated: true}}\n  - name: demo.passes.ClosePass\n",
    );
    format!("{text}  - name: demo.passes.HoldPlaced\n    fields:\n      - {{name: hold_id, type: demo.passes.HoldId}}\n")
}

/// The model of [`row_set_accepting_branch_opens_the_row_its_identity_guard_selects`]: every
/// scenario, `vip`'s included, passes on the interpreted target.
#[test]
#[ignore = "backlog: identity-row-remaining-paths, story:an-identity-guard-is-decided-on-every-row-synthesis-arranges"]
fn row_set_accepting_branch_below_a_row_set_refusal_passes() {
    let run = run(&with_row_set(&format!("{HELD_BRANCH}{VIP_BRANCH}")));
    assert_all_pass(&run);
}

/// A row-set accepting branch guarded by the identity, below the row-set refusal the compiler
/// orders before it: `vip` opens the pass `"vip"`.
#[test]
#[ignore = "backlog: identity-row-remaining-paths, story:an-identity-guard-is-decided-on-every-row-synthesis-arranges"]
fn row_set_accepting_branch_opens_the_row_its_identity_guard_selects() {
    let text = with_row_set(&format!("{HELD_BRANCH}{VIP_BRANCH}"));
    let run = run(&text);
    opens_only(
        &run,
        "demo.passes.ClosePass/outcome/vip",
        "pass_id",
        "vip",
        text_in(&["vip"]),
    );
    assert_all_pass(&run);
}

/// A row-set command whose only row read is the head `closed` publishes, so no row set answers
/// before `vip` (`row_set::routes` is false) and `vip` is arranged by `addressed_row`: it opens
/// the pass `"vip"`.
#[test]
fn addressed_row_set_accepting_branch_opens_the_row_its_identity_guard_selects() {
    let text = replaced(
        &replaced(
            &with_row_set(VIP_BRANCH),
            "          demo.passes.PassClosed: {pass_id: input.pass_id}\n",
            "          demo.passes.PassClosed:\n            pass_id: input.pass_id\n            held_by:\n              related:\n                entity: demo.passes.Hold\n                where: {all: [pass_id == input.pass_id]}\n                order_by: [label asc]\n                ties: unordered\n                first: label\n",
        ),
        "  - name: demo.passes.PassClosed\n    fields:\n      - {name: pass_id, type: demo.passes.PassId}\n",
        "  - name: demo.passes.PassClosed\n    fields:\n      - {name: pass_id, type: demo.passes.PassId}\n      - {name: held_by, type: Optional<String>}\n",
    );
    let run = run(&text);
    opens_only(
        &run,
        "demo.passes.ClosePass/outcome/vip",
        "pass_id",
        "vip",
        text_in(&["vip"]),
    );
    assert_all_pass(&run);
}

/// A complete identity-guarded refusal beside a row set opens the pass `"frozen"`.
#[test]
fn identity_guard_beside_a_row_set_opens_the_row_it_selects() {
    let text = with_row_set(&format!("{FROZEN_BRANCH}{HELD_BRANCH}"));
    let run = run(&text);
    opens_only(&run, FROZEN, "pass_id", "frozen", text_in(&["frozen"]));
    assert_all_pass(&run);
}

// ---------------------------------------------------------------------------------------------
// prepare_state_input and run_state_refusal
// ---------------------------------------------------------------------------------------------

/// Each of the story's four scenarios opens only passes meeting `meets`, and every scenario
/// passes.
fn held_open(text: &str, field: &str, what: &str, meets: impl Fn(&Value) -> bool) {
    let run = run(text);
    let wrong: Vec<String> = HELD
        .iter()
        .filter_map(|scenario| {
            let Some(_) = written(&run, scenario) else {
                return Some(format!("{scenario}: refused {:?}", refused(&run, scenario)));
            };
            let sent = opened(&run, scenario, field);
            (!sent.iter().all(|value| meets(&value["value"])))
                .then(|| format!("{scenario}: opens {sent:?}"))
        })
        .collect();
    assert!(
        wrong.is_empty(),
        "each held-state scenario opens only passes {what}:\n{}",
        wrong.join("\n")
    );
    assert_all_pass(&run);
}

/// Negated membership: `frozen` refuses every pass but `"main"` and `"spare"`.
#[test]
fn state_negated_membership_opens_a_member() {
    let text = replaced(
        STATE,
        STATE_GUARD,
        "        when: {not: {pass_id: [\"main\", \"spare\"]}}\n",
    );
    held_open(
        &text,
        "pass_id",
        "main or spare",
        text_in(&["main", "spare"]),
    );
}

/// An Integer identity: `frozen` refuses every pass below 100.
#[test]
fn state_integer_range_opens_a_pass_outside_the_refusal() {
    let text = replaced(
        &replaced(
            STATE,
            STRING_ID,
            "  - {name: demo.passes.PassId, kind: newtype, of: Integer}\n",
        ),
        STATE_GUARD,
        "        when: pass_id < 100\n",
    );
    held_open(&text, "pass_id", "at or above 100", |value| {
        value.as_f64().is_some_and(|number| number >= 100.0)
    });
}

/// A struct identity: `frozen` refuses every pass outside region `"eu"`.
#[test]
fn state_struct_identity_guard_on_one_part_opens_that_part() {
    let text = replaced(
        &replaced(
            STATE,
            STRING_ID,
            "  - name: demo.passes.PassId\n    kind: struct\n    fields:\n      - {name: region, type: String}\n      - {name: number, type: String}\n",
        ),
        STATE_GUARD,
        "        when: 'pass_id.region != \"eu\"'\n",
    );
    held_open(&text, "pass_id", "in region eu", |value| {
        value.to_string().contains("\"eu\"")
    });
}

/// A compound guard: `frozen` refuses every pass but `"main"`, or any late close.
#[test]
fn state_compound_guard_opens_main() {
    let text = replaced(
        STATE,
        STATE_GUARD,
        "        when: {any: ['pass_id != \"main\"', 'reason == \"late\"']}\n",
    );
    held_open(&text, "pass_id", "main", text_in(&["main"]));
}

/// The creator names the identity through an input of another name.
#[test]
fn state_creator_input_of_another_name_is_steered() {
    let text = replaced(
        &replaced(
            STATE,
            OPEN_INPUT,
            "    input:\n      - {name: code, type: demo.passes.PassId}\n      - {name: note, type: String}\n",
        ),
        OPEN_PAYLOAD,
        "demo.passes.PassOpened: {pass_id: input.code}",
    );
    held_open(&text, "code", "main", text_in(&["main"]));
}

/// No view projects the held state, so the held state is read back by sends (`existence.rs`).
#[test]
fn state_without_a_state_view_still_passes() {
    let text = replaced(
        STATE,
        "      - {name: state, type: demo.passes.Pass.State}\n",
        "",
    );
    let run = run(&text);
    assert_all_pass(&run);
}

/// A further input refusal (`late`, on the reason) beside the identity guard is sent again in
/// each held state (`existence.rs` `refused_inputs_in`, a `reach_in_state` caller the unit did not
/// check): the row sent there must keep `frozen` from answering first.
#[test]
#[ignore = "backlog: identity-row-remaining-paths, story:an-identity-guard-is-decided-on-every-row-synthesis-arranges"]
fn state_further_input_refusal_in_each_held_state_passes() {
    let text = replaced(
        &replaced(
            STATE,
            "      - name: closed\n",
            "      - name: late\n        when: 'reason == \"late\"'\n        error: demo.passes.Late\n        payload:\n          demo.passes.Late: {pass_id: input.pass_id}\n      - name: closed\n",
        ),
        "errors:\n",
        "errors:\n  - name: demo.passes.Late\n    fields:\n      - {name: pass_id, type: demo.passes.PassId}\n",
    );
    let run = run(&text);
    assert_all_pass(&run);
}

/// A stored-row refusal (`stale`) beside the identity guard routes the state refusal through
/// `subject_fact::refusal_witness`, the branch of `run_state_refusal` the unit did not steer.
#[test]
#[ignore = "backlog: identity-row-remaining-paths, story:an-identity-guard-is-decided-on-every-row-synthesis-arranges"]
fn state_refusal_beside_a_stored_row_guard_opens_main() {
    let text = replaced(
        &replaced(
            STATE,
            "      - name: closed\n",
            "      - name: stale\n        when_subject: {predicate: flagged == true}\n        error: demo.passes.Stale\n        payload:\n          demo.passes.Stale: {pass_id: input.pass_id}\n      - name: closed\n",
        ),
        "errors:\n",
        "errors:\n  - name: demo.passes.Stale\n    fields:\n      - {name: pass_id, type: demo.passes.PassId}\n",
    );
    // `not-open` is the genuine default the stored-row coverage asks for: it still answers a
    // closed pass, as an effect-free default refusal of a held-state command.
    let text = replaced(&text, "        when_subject_state: Closed\n", "");
    // A Boolean stored field, so the stored-row guard has a closed set of values.
    let text = replaced(
        &text,
        "    fields:\n      - {name: note, type: String}\n    lifecycle:\n",
        "    fields:\n      - {name: note, type: String}\n      - {name: flagged, type: Boolean}\n    lifecycle:\n",
    );
    let text = replaced(
        &text,
        OPEN_INPUT,
        "    input:\n      - {name: pass_id, type: demo.passes.PassId}\n      - {name: note, type: String}\n      - {name: flagged, type: Boolean}\n",
    );
    let text = replaced(
        &text,
        "        sets: {note: input.note}\n",
        "        sets: {note: input.note, flagged: input.flagged}\n",
    );
    let run = run(&text);
    opens_only(
        &run,
        "demo.passes.Pass/state/Closed/refuses/demo.passes.ClosePass",
        "pass_id",
        "main",
        text_in(&["main"]),
    );
    assert_all_pass(&run);
}
