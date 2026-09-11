//! Held state is a separate authority from the caller's input namespace.
use ess_domain::command::{Outcome, RawOutcome};

fn outcome(text: &str) -> Result<Outcome, String> {
    let raw: RawOutcome = serde_yaml::from_str(text).map_err(|error| error.to_string())?;
    raw.try_into()
        .map_err(|error: ess_primitives::error::ValidationErrors| error.to_string())
}

#[test]
fn subject_state_round_trips_with_an_independent_input_guard() {
    let source = "name: preserved\nwhen: incoming == Ringing\nwhen_subject_state: Bridged\nupdates: calls.core.Call\ninstance: call_id\n";
    let parsed = outcome(source).unwrap();
    let encoded = serde_yaml::to_string(&parsed).unwrap();
    assert!(encoded.contains("when_subject_state: Bridged"));
    assert!(encoded.contains("incoming == Ringing"));
    assert_eq!(outcome(&encoded).unwrap(), parsed);
}

#[test]
fn subject_state_without_input_guard_is_conditional() {
    let parsed = outcome("name: preserved\nwhen_subject_state: Bridged\nupdates: calls.core.Call\ninstance: call_id\n").unwrap();
    assert!(!parsed.is_unconditional());
    assert_ne!(parsed.test_strategy().as_str(), "construct_input");
}

#[test]
fn subject_state_cannot_borrow_external_or_create_authority() {
    for extra in [
        "external: provider unavailable\n",
        "wrong_state: true\n",
        "creates: calls.core.Call\ninstance: call_id\n",
        "",
    ] {
        let source = format!("name: preserved\nwhen_subject_state: Bridged\n{extra}");
        let error = outcome(&source).unwrap_err();
        assert!(
            !error.contains("unknown field"),
            "expected semantic refusal: {error}"
        );
    }
}

#[test]
fn finite_state_only_partition_and_joint_bound() {
    use ess_domain::{
        command::finite::{analyze_with_states, StateGuard},
        entity::StateName,
        expression::DomainEnvironment,
        types::TypeRegistry,
    };
    let registry = TypeRegistry::new();
    let environment = DomainEnvironment::new(&registry, &[]);
    let init = StateName::new("Init").unwrap();
    let bridged = StateName::new("Bridged").unwrap();
    let states = [init.clone(), bridged.clone()].into_iter().collect();
    let guards = [
        StateGuard {
            state: Some(&init),
            predicate: None,
        },
        StateGuard {
            state: Some(&bridged),
            predicate: None,
        },
    ];
    let cases = analyze_with_states(&environment, &guards, &states).unwrap();
    assert_eq!(cases.len(), 2);
    assert!(cases
        .iter()
        .all(|case| case.input.values.is_empty() && case.input.selected.len() == 1));
    assert_eq!(
        cases
            .iter()
            .find(|case| case.state == init)
            .unwrap()
            .input
            .selected,
        [0]
    );
    let oversized = (0..65)
        .map(|index| StateName::new(format!("State{index}")).unwrap())
        .collect();
    assert!(analyze_with_states(&environment, &guards, &oversized).is_none());
}

fn model(guard: &str) -> String {
    format!("format: ess/3\nsystem: calls\nversion: v1\ndomain: calls.core\nentities:\n  - name: calls.core.Call\n    identity: {{name: call_id, type: Uuid}}\n    fields: []\n    lifecycle:\n      initial: Bridged\n      states: [Bridged]\n      terminal: [Bridged]\nevents:\n  - name: calls.core.Observed\n    fields: []\ncommands:\n  - name: calls.core.Enrich\n    input:\n      - {{name: call_id, type: Uuid}}\n    outcomes:\n      - name: preserved\n        when_subject_state: {guard}\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n")
}

fn specification(text: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("subject.yaml"), raw)])
        .map_err(|error| error.to_string())
}

#[test]
fn specification_admits_declared_state_and_refuses_unknown_or_old_format() {
    specification(&model("Bridged")).unwrap();
    let unknown = specification(&model("Missing")).unwrap_err();
    assert!(unknown.contains("not a declared state"), "{unknown}");
    for format in ["ess/1", "ess/2"] {
        let error = specification(&model("Bridged").replace("ess/3", format)).unwrap_err();
        assert!(
            error.contains("require specification format ess/3"),
            "{error}"
        );
    }
}

#[test]
fn uncovered_or_overlapping_state_partition_is_refused() {
    let uncovered = specification(&model("Missing")).unwrap_err();
    assert!(uncovered.contains("select 0 branches"), "{uncovered}");
    let mut overlap = model("Bridged");
    overlap.push_str("      - name: duplicate\n        when_subject_state: Bridged\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n");
    let error = specification(&overlap).unwrap_err();
    assert!(error.contains("select 2 branches"), "{error}");
}

#[test]
fn open_input_with_default_still_refuses_a_guard_outside_the_move_source() {
    let mut text = model("Bridged")
        .replace("initial: Bridged", "initial: Init")
        .replace("states: [Bridged]", "states: [Init, Bridged]")
        .replace("terminal: [Bridged]", "terminal: [Bridged]\n      transitions:\n        - {name: bridge, from: [Init], to: Bridged}")
        .replace("    input:\n", "    input:\n      - {name: amount, type: Integer}\n")
        .replace("when_subject_state: Bridged", "when_subject_state: Bridged\n        when: amount > 0")
        .replace("updates: calls.core.Call", "moves: calls.core.Call.bridge");
    text.push_str("      - name: default\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n");
    let error = specification(&text).unwrap_err();
    assert!(
        error.contains("held-state guard `Bridged` cannot take move `bridge`"),
        "{error}"
    );
}
