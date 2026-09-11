//! External outcomes cannot supply held-state coverage authority.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model(external_first: bool, complete: bool) -> String {
    let external = "      - name: external-observed\n        external: a separate observer reports progress\n        updates: calls.core.Observer\n        instance: observer_id\n        emits: [calls.core.Observed]\n";
    let held = "      - name: init\n        when_subject_state: Init\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n";
    let rest = if complete {
        "      - name: bridged\n        when_subject_state: Bridged\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n"
    } else {
        ""
    };
    let outcomes = if external_first {
        format!("{external}{held}{rest}")
    } else {
        format!("{held}{rest}{external}")
    };
    format!("format: ess/3\nsystem: calls\nversion: v1\ndomain: calls.core\nentities:\n  - name: calls.core.Call\n    identity: {{name: call_id, type: Uuid}}\n    fields: []\n    lifecycle:\n      initial: Init\n      states: [Init, Bridged]\n      terminal: [Bridged]\n      transitions:\n        - {{name: bridge, from: [Init], to: Bridged}}\n  - name: calls.core.Observer\n    identity: {{name: observer_id, type: Uuid}}\n    fields: []\n    lifecycle:\n      initial: Init\n      states: [Init]\n      terminal: [Init]\nevents:\n  - name: calls.core.Observed\n    fields: []\ncommands:\n  - name: calls.core.Enrich\n    input:\n      - {{name: call_id, type: Uuid}}\n      - {{name: observer_id, type: Uuid}}\n    outcomes:\n{outcomes}")
}

fn assemble(source: &str) -> Result<Specification, String> {
    let source = source.replace("commands:\n", "commands:\n  - name: calls.core.Bridge\n    input: [{name: call_id, type: Uuid}]\n    outcomes:\n      - name: bridged\n        moves: calls.core.Call.bridge\n        instance: call_id\n        emits: [calls.core.Observed]\n      - name: already-bridged\n        wrong_state: true\n        refuses: false\n");
    let raw = RawSpecFile::parse(&source).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("external-subject.yaml"), raw)])
        .map_err(|error| error.to_string())
}

#[test]
fn external_subject_cannot_hide_uncovered_held_state() {
    // Establish a genuinely admitted full specification using both subjects first.
    assemble(&model(false, true)).expect("valid complete two-entity control");
    let missing = assemble(&model(false, false)).unwrap_err();
    assert!(
        missing.contains("held state Bridged") && missing.contains("select 0 branches"),
        "{missing}"
    );
    let reordered = assemble(&model(true, false));
    assert!(
        reordered.is_err(),
        "external-first ordering admitted an uncovered Call.Bridged state"
    );
}

#[test]
fn external_subject_cannot_reject_valid_held_state() {
    assemble(&model(false, true)).expect("valid complete two-entity control");
    assemble(&model(true, true))
        .expect("external outcome ordering cannot change declared Call states");
}
