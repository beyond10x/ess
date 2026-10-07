//! A branch guarded by an input condition beside a stored-field condition, declared before a
//! sibling with the same input condition, is synthesized (beyond10x/ess#278).
//!
//! `RecordResult` declares `held-for-promotion` (`when: result == Healthy` beside `when_subject:`
//! over `auto_promote`) before `promoted` (`when: result == Healthy` alone). On a row whose
//! `auto_promote` is unset or false, both guards hold. Accepting branches answer in declaration
//! order (`docs/design/input-guard-overlap-precedence.md`, beyond10x/ess#217;
//! `docs/design/cross-record-and-stored-field-guards.md`, "The precedence order", step 5), so
//! `held-for-promotion` answers there. Synthesis used to require exactly one selected branch and
//! refused `held-for-promotion`, its transition and every wrong-state scenario of the command
//! (ESS-SYNTH-003), whose condition also read `result == Healthy and none of: result == Healthy, …`.
use std::collections::BTreeSet;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted,
    report::Status,
    synthesize::{synthesize, Synthesis},
    AdmittedSuite, Runner,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

/// The #278 shape, reduced: one deployment entity, a command that creates it, the command whose
/// first branch reads both its input and the row, and a command deciding a held deployment.
const ROLLOUT: &str = r#"format: ess/19
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
      - {name: result, type: demo.rollout.Result}
    outcomes:
      - name: held-for-promotion
        when: result == Healthy
        when_subject:
          predicate: {any: ["not defined(auto_promote)", auto_promote == false]}
        moves: demo.rollout.Deployment.hold
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: promoted
        when: result == Healthy
        moves: demo.rollout.Deployment.promote
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: held-for-rollback
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

/// The scenarios #278 found refused.
const REQUIRED: [&str; 5] = [
    "demo.rollout.RecordResult/outcome/held-for-promotion",
    "demo.rollout.Deployment/transition/hold/by/demo.rollout.RecordResult/held-for-promotion",
    "demo.rollout.Deployment/state/Held/refuses/demo.rollout.RecordResult",
    "demo.rollout.Deployment/state/Promoted/refuses/demo.rollout.RecordResult",
    "demo.rollout.Deployment/state/RolledBack/refuses/demo.rollout.RecordResult",
];

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("rollout.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
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

fn scenario_ids(result: &Synthesis) -> BTreeSet<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

/// The swap of #217's precedence test: `promoted` declared before `held-for-promotion`.
fn swapped(text: &str) -> String {
    let held = "      - name: held-for-promotion\n        when: result == Healthy\n        when_subject:\n          predicate: {any: [\"not defined(auto_promote)\", auto_promote == false]}\n        moves: demo.rollout.Deployment.hold\n        instance: deployment_id\n        emits: [demo.rollout.Moved]\n        payload:\n          demo.rollout.Moved: {deployment_id: input.deployment_id}\n";
    let promoted = "      - name: promoted\n        when: result == Healthy\n        moves: demo.rollout.Deployment.promote\n        instance: deployment_id\n        emits: [demo.rollout.Moved]\n        payload:\n          demo.rollout.Moved: {deployment_id: input.deployment_id}\n";
    let swapped = text.replace(&format!("{held}{promoted}"), &format!("{promoted}{held}"));
    assert_ne!(swapped, text);
    swapped
}

/// #278's acceptance: the mixed branch, its transition and the three wrong-state scenarios are
/// synthesized, and nothing about them is refused.
#[test]
fn issue_278_the_mixed_branch_and_its_wrong_state_scenarios_are_synthesized() {
    let result = synthesize(&ir(ROLLOUT));
    let ids = scenario_ids(&result);
    let refused = refusals(&result);
    for id in REQUIRED {
        assert!(ids.contains(id), "no scenario {id}; refusals: {refused:#?}");
        assert!(
            !refused.iter().any(|refusal| refusal.starts_with(id)),
            "{id} is refused: {refused:#?}"
        );
    }
    assert!(
        !refused
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-003")),
        "{refused:#?}"
    );
}

/// The model passes its own suite against the interpreter: no scenario fails or errors. A scenario
/// the interpreter cannot execute (a stored-field guard) is `Unsupported`, which is a capability
/// gap of the reference target and not a contradiction.
#[test]
fn issue_278_the_suite_does_not_fail_against_the_interpreter() {
    let result = synthesize(&ir(ROLLOUT));
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
    let report = Runner::for_suite(&result.suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir(ROLLOUT)))
        .into_report();
    let bad: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| matches!(scenario.status, Status::Failed | Status::Error))
        .map(|scenario| format!("{scenario:#?}"))
        .collect();
    assert!(bad.is_empty(), "{bad:#?}");
    for id in REQUIRED {
        let status = report
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario.to_string() == id)
            .map(|scenario| scenario.status);
        eprintln!("{id}: {status:?}");
        assert!(status.is_some(), "{id} is not in the report");
    }
}

/// Declared the other way round, `promoted` answers wherever `result == Healthy`, so
/// `held-for-promotion` is unreachable and is refused rather than witnessed on a row `promoted`
/// claims.
/// Declared after its twin, the mixed branch would never be taken under declaration order and
/// always under the precedence order; validation refuses that order (beyond10x/ess#486).
#[test]
fn issue_278_a_mixed_branch_declared_after_its_twin_is_refused() {
    let source = swapped(ROLLOUT);
    let raw = RawSpecFile::parse(&source).unwrap_or_else(|error| panic!("{error}\n{source}"));
    let errors = Specification::assemble([(Source::new("rollout.yaml"), raw)]).map_or_else(
        |errors| errors.to_string(),
        |_| panic!("the model validates:\n{source}"),
    );
    assert!(
        errors.contains(
            "`held-for-promotion` is selected by the held state, which answers before the \
             accepting branch `promoted` declared above it"
        ),
        "{errors}"
    );
}

/// Every deployment is created with `auto_promote: true`, so no row admits `held-for-promotion`'s
/// stored guard: the branch, and the wrong-state scenarios it is the attempt of, stay refused.
fn pinned(text: &str) -> String {
    let pinned = text.replace(
        "          auto_promote: input.auto_promote\n",
        "          auto_promote: 'true'\n",
    );
    assert_ne!(pinned, text);
    pinned
}

/// The `c and none of: …` conditions in `message` whose list repeats `c` or one of its conjuncts.
fn contradictions(message: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = message;
    while let Some(at) = rest.find(" and none of: ") {
        let own_start = rest[..at].rfind('`').map_or(0, |tick| tick + 1);
        let own = &rest[own_start..at];
        let list_start = at + " and none of: ".len();
        let list_end = rest[list_start..]
            .find('`')
            .map_or(rest.len(), |end| list_start + end);
        let list = &rest[list_start..list_end];
        let list = list.split(", on ").next().unwrap_or(list);
        let items: Vec<&str> = list.split(", ").map(str::trim).collect();
        for conjunct in own.split(" and ").map(str::trim) {
            if items.contains(&conjunct) {
                found.push(format!("`{own} and none of: {list}` repeats `{conjunct}`"));
            }
        }
        rest = &rest[list_start..];
    }
    found
}

/// The specifications [`no_condition_contradicts_itself`] reads: this file's, and every
/// single-document fixture under `tests/fixtures`.
fn fixture_set() -> Vec<(String, EssIr)> {
    let mut set = vec![
        ("rollout".to_owned(), ir(ROLLOUT)),
        ("rollout pinned".to_owned(), ir(&pinned(ROLLOUT))),
    ];
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|it| it == "yaml"))
        .collect();
    paths.sort();
    for path in paths {
        let text = std::fs::read_to_string(&path).unwrap();
        let Ok(raw) = RawSpecFile::parse(&text) else {
            continue;
        };
        let label = path.file_name().unwrap().to_string_lossy().to_string();
        let Ok(spec) = Specification::assemble([(Source::new(label.clone()), raw)]) else {
            continue;
        };
        let Ok(ir) = compile(&spec, &SourceMap::new()) else {
            continue;
        };
        set.push((label, ir));
    }
    set
}

/// #278's second acceptance: no condition a synthesis states has the form `c and none of: c, …`,
/// over the fixture set — the refused `pinned` model included, whose wrong-state witness is still
/// sought for `result == Healthy` beside a sibling guarded by `result == Healthy`.
#[test]
fn no_condition_contradicts_itself() {
    assert_eq!(
        contradictions("satisfies `c == 1 and none of: c == 1, d == 2`"),
        vec!["`c == 1 and none of: c == 1, d == 2` repeats `c == 1`".to_owned()],
        "the check recognises the form it rules out"
    );
    let set = fixture_set();
    assert!(set.len() > 3, "the fixtures under tests/fixtures compile");
    let mut found = Vec::new();
    for (label, ir) in &set {
        for message in refusals(&synthesize(ir)) {
            found.extend(
                contradictions(&message)
                    .into_iter()
                    .map(|it| format!("{label}: {it}")),
            );
        }
    }
    assert!(found.is_empty(), "{found:#?}");
}

/// Where the branch under test is refused beside a sibling with its own input guard, the refusal
/// names the row it needed rather than a contradiction.
#[test]
fn issue_278_a_refused_wrong_state_witness_names_the_row_it_needed() {
    let result = synthesize(&ir(&pinned(ROLLOUT)));
    let refused = refusals(&result);
    let held = refused
        .iter()
        .find(|refusal| refusal.starts_with(REQUIRED[2]))
        .unwrap_or_else(|| panic!("{refused:#?}"));
    assert!(
        held.contains(
            "result == Healthy and none of: result == Regression, on a \
             `demo.rollout.Deployment` row in this state holding"
        ),
        "{held}"
    );
}
