//! Adversary, pass 2, `story:feature-request-278` (beyond10x/ess#278), against correction 1.
//!
//! Correction 1 added `admit_alike` (two input guards admitting the same inputs) and used it only to
//! word ESS-SYNTH-003 for a refused wrong-state witness: a twin of the branch's own input guard is
//! left out of `none of:`, and a twin declared before the branch, or a refusal, is named as what
//! took "every such input first". These cases check the wording against what the guards admit
//! and against the precedence order (`docs/design/cross-record-and-stored-field-guards.md`, "The
//! precedence order": input-guarded refusals first, then accepting branches in declaration order).

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::facts::FactValue;

const HEAD: &str = r"format: ess/19
system: demo
version: v1
domain: demo.rollout
summary: A deployment held for promotion unless it promotes itself.
types:
  - {name: demo.rollout.Result, kind: enum, variants: [Healthy, Regression]}
entities:
  - name: demo.rollout.Deployment
    identity: {name: deployment_id, type: Uuid}
    fields:
      - {name: auto_promote, type: Optional<Boolean>}
      - {name: automatic_rollback, type: Optional<Boolean>}
    lifecycle:
      initial: Canary
      states: [Canary, Held, Promoted, RolledBack]
      terminal: [Promoted, RolledBack]
      transitions:
        - {name: hold, from: [Canary], to: Held}
        - {name: promote, from: [Canary], to: Promoted}
        - {name: rollback, from: [Canary], to: RolledBack}
        - {name: approve, from: [Held], to: Promoted}
        - {name: reject, from: [Held], to: RolledBack}
errors:
  - name: demo.rollout.Conflict
    summary: wrong state
    fields:
      - {name: state, type: demo.rollout.Deployment.State}
  - name: demo.rollout.Refused
    summary: refused by its input
events:
  - name: demo.rollout.Moved
    fields: [{name: deployment_id, type: Uuid}]
actors:
  - name: demo.rollout.Engine
    may: [demo.rollout.Deploy, demo.rollout.RecordResult, demo.rollout.Decide]
commands:
  - name: demo.rollout.Deploy
    input:
      - {name: auto_promote, type: Optional<Boolean>}
      - {name: automatic_rollback, type: Optional<Boolean>}
    outcomes:
      - name: deployed
        creates: demo.rollout.Deployment
        instance: deployment_id
        sets:
          auto_promote: input.auto_promote
          automatic_rollback: input.automatic_rollback
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: {generated: true}}
  - name: demo.rollout.RecordResult
    input:
      - {name: deployment_id, type: Uuid}
      - {name: result, type: RESULT_TYPE}
      - {name: code, type: String}
    outcomes:
";

const TAIL: &str = r#"      - name: held-for-rollback
        when: result == Regression
        when_subject:
          predicate: {any: ["not defined(automatic_rollback)", automatic_rollback == false]}
        moves: demo.rollout.Deployment.hold
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: rolled-back
        moves: demo.rollout.Deployment.rollback
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: wrong-state
        wrong_state: true
        error: demo.rollout.Conflict
  - name: demo.rollout.Decide
    input:
      - {name: deployment_id, type: Uuid}
      - {name: decision, type: demo.rollout.Result}
    outcomes:
      - name: promoted
        when: decision == Healthy
        moves: demo.rollout.Deployment.approve
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: rolled-back
        moves: demo.rollout.Deployment.reject
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: wrong-state
        wrong_state: true
        error: demo.rollout.Conflict
views:
  - name: demo.rollout.Deployments
    source: demo.rollout.Deployment
    consistency: read_your_writes
    fields:
      - {name: deployment_id, type: Uuid}
      - {name: auto_promote, type: Optional<Boolean>}
      - {name: automatic_rollback, type: Optional<Boolean>}
      - {name: state, type: demo.rollout.Deployment.State}
"#;

/// The #278 mixed branch, its input guard given as `WHEN`.
const HELD: &str = r#"      - name: held-for-promotion
        when: WHEN
        when_subject:
          predicate: {any: ["not defined(auto_promote)", auto_promote == false]}
        moves: demo.rollout.Deployment.hold
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
"#;

const PROMOTED: &str = r"      - name: promoted
        when: WHEN
        moves: demo.rollout.Deployment.promote
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
";

/// A twin that moves nothing and is taken only on a row whose `auto_promote` is unset or false.
const NOTED_GUARDED: &str = r#"      - name: noted
        when: WHEN
        when_subject:
          predicate: {any: ["not defined(auto_promote)", auto_promote == false]}
        preserves: demo.rollout.Deployment
        instance: deployment_id
"#;

/// A twin that moves nothing and reads no row.
const NOTED: &str = r"      - name: noted
        when: WHEN
        preserves: demo.rollout.Deployment
        instance: deployment_id
";

/// An input-guarded refusal.
const REFUSED: &str = r"      - name: refused
        when: WHEN
        error: demo.rollout.Refused
";

const HELD_REFUSES: &str = "demo.rollout.Deployment/state/Held/refuses/demo.rollout.RecordResult";

fn branch(template: &str, when: &str) -> String {
    template.replace("WHEN", when)
}

/// The model with `branches` first in `RecordResult`; the `promote` transition is dropped where no
/// branch takes it, which validation requires.
fn spec(result_type: &str, branches: &[String]) -> String {
    let branches = branches.concat();
    let mut head = HEAD.replace("RESULT_TYPE", result_type);
    if !branches.contains("demo.rollout.Deployment.promote") {
        head = head.replace(
            "        - {name: promote, from: [Canary], to: Promoted}\n",
            "",
        );
    }
    format!("{head}{branches}{TAIL}")
}

/// Every deployment created with `auto_promote: true`: no row admits `held-for-promotion`.
fn pinned(text: &str) -> String {
    let pinned = text.replace(
        "          auto_promote: input.auto_promote\n",
        "          auto_promote: 'true'\n",
    );
    assert_ne!(pinned, text);
    pinned
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("rollout.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            let scenario = refusal
                .scenario
                .as_ref()
                .map_or_else(String::new, ToString::to_string);
            format!("{scenario}: {}: {}", refusal.cause.code(), refusal.cause)
        })
        .collect()
}

fn refusal_of(result: &Synthesis, id: &str) -> String {
    let refused = refusals(result);
    refused
        .iter()
        .find(|refusal| refusal.starts_with(id))
        .cloned()
        .unwrap_or_else(|| panic!("{id} is not refused: {refused:#?}"))
}

/// The branch names after `taken first by` in `message`, in the order written.
fn taken_by(message: &str) -> Vec<String> {
    let Some((_, rest)) = message.split_once("taken first by ") else {
        return Vec::new();
    };
    rest.split("), ")
        .filter_map(|part| part.strip_prefix('`'))
        .filter_map(|part| part.split_once('`'))
        .map(|(name, _)| name.to_owned())
        .collect()
}

// ---- attack 1: `admit_alike` calls two guards alike that admit different inputs ---------------

/// `code != ""` and the bare `code` (truthy: non-empty and not `"false"`) over a `String` input are
/// decided alike by every candidate `admit_alike` draws (the base text and `""`), so it calls them
/// alike. They are not: `code: "false"` satisfies `held-for-promotion`'s `code != ""` and refutes
/// `refused`'s `code`. The refusal must not say every such input is taken first by `refused`.
#[test]
fn adversary_278_p2_a_truthy_text_guard_is_not_alike_a_nonempty_one() {
    // The input the claim says does not exist, decided by the predicate semantics themselves.
    let probe = FactValue::text("false");
    assert!(!probe.is_truthy(), "`code` is refuted by \"false\"");
    assert_ne!(
        probe,
        FactValue::text(""),
        "`code != \"\"` holds of \"false\""
    );

    let text = pinned(&spec(
        "demo.rollout.Result",
        &[branch(HELD, r#"code != """#), branch(REFUSED, "code")],
    ));
    let result = synthesize(&ir(&text));
    let held = refusal_of(&result, HELD_REFUSES);
    assert!(
        !taken_by(&held).iter().any(|name| name == "refused"),
        "`code: \"false\"` satisfies `code != \"\"` and is not taken by `refused` (`code`), yet: \
         {held}"
    );
}

// ---- attack 2: guards admitting alike that `admit_alike` calls different ----------------------

/// `result == Healthy` and `result in [Healthy]` over an `Optional` enum admit exactly the same
/// inputs (absent decides both `Unknown`), but `witness::exhausts` refuses an optional leaf, so
/// `admit_alike` calls them different and the refusal states `c and none of: c', …` with `c'` the
/// same guard spelled otherwise — the form #278's second acceptance rules out, which pass 1 showed
/// fixed for a required enum.
#[test]
fn adversary_278_p2_an_equivalent_twin_over_an_optional_input_is_not_listed() {
    // `held-for-rollback` before `promoted`: over an `Optional` input the finite prover does not
    // show `result == Regression` and `result in [Healthy]` disjoint, and validation refuses a
    // held-state branch after an accepting one it may overlap (beyond10x/ess#486).
    let text = rollback_first(&pinned(&spec(
        "Optional<demo.rollout.Result>",
        &[
            branch(HELD, "result == Healthy"),
            branch(PROMOTED, "{result: [Healthy]}"),
        ],
    )));
    let result = synthesize(&ir(&text));
    let contradictions: Vec<String> = refusals(&result)
        .into_iter()
        .filter(|message| message.contains("result == Healthy and none of: result in [Healthy]"))
        .collect();
    assert_eq!(contradictions, Vec::<String>::new());
}

// ---- attack 3: the sibling named as the cause ------------------------------------------------

/// `noted` (`result == Healthy` beside the same stored guard as `held-for-promotion`, moving
/// nothing) is declared before the mixed branch. Every deployment is pinned `auto_promote: true`,
/// so `noted` is selected on no row: the search passes it over on every row (its stored guard is
/// false there) and is blocked by `promoted` and the missing row instead. The refusal must not
/// name `noted` as what took every such input.
#[test]
fn adversary_278_p2_an_earlier_twin_selected_on_no_row_is_not_named_the_cause() {
    let text = pinned(&spec(
        "demo.rollout.Result",
        &[
            branch(NOTED_GUARDED, "result == Healthy"),
            branch(HELD, "result == Healthy"),
            branch(PROMOTED, "result == Healthy"),
        ],
    ));
    let result = synthesize(&ir(&text));
    let held = refusal_of(&result, HELD_REFUSES);
    assert!(
        !taken_by(&held).iter().any(|name| name == "noted"),
        "`noted` is selected on no pinned row, yet the refusal says it took every input: {held}"
    );
}

/// `noted` (accepting) declared before the mixed branch, both `result == Healthy`, is refused by
/// validation (beyond10x/ess#486): the held state selects `held-for-promotion` first in either
/// order.
#[test]
fn adversary_278_p2_an_earlier_accepting_twin_is_refused_by_validation() {
    let text = spec(
        "demo.rollout.Result",
        &[
            branch(NOTED, "result == Healthy"),
            branch(HELD, "result == Healthy"),
            branch(REFUSED, "result == Healthy"),
        ],
    );
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let errors = Specification::assemble([(Source::new("rollout.yaml"), raw)]).map_or_else(
        |errors| errors.to_string(),
        |_| panic!("the model validates:\n{text}"),
    );
    assert!(
        errors.contains(
            "`held-for-promotion` is selected by the held state, which answers before the \
             accepting branch `noted` declared above it"
        ),
        "{errors}"
    );
}

/// `text` with `held-for-rollback` declared before `held-for-promotion`.
fn rollback_first(text: &str) -> String {
    let held = text
        .find("      - name: held-for-promotion\n")
        .expect("`held-for-promotion` is declared");
    let rollback = text
        .find("      - name: held-for-rollback\n")
        .expect("`held-for-rollback` is declared");
    let rolled_back = text
        .find("      - name: rolled-back\n")
        .expect("`rolled-back` is declared");
    assert!(held < rollback && rollback < rolled_back);
    format!(
        "{}{}{}{}",
        &text[..held],
        &text[rollback..rolled_back],
        &text[held..rollback],
        &text[rolled_back..]
    )
}
