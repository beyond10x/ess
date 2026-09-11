//! An open input partition cannot prove that a fallback move accepts every held state.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = r"format: ess/3
system: calls
version: v1
domain: calls.core
entities:
  - name: calls.core.Call
    identity: {name: call_id, type: Uuid}
    fields: []
    lifecycle:
      initial: Init
      states: [Init, Bridged]
      terminal: [Bridged]
      transitions:
        - {name: bridge, from: [Init], to: Bridged}
events:
  - name: calls.core.Observed
    fields: []
commands:
  - name: calls.core.Bridge
    input: [{name: call_id, type: Uuid}]
    outcomes:
      - name: bridged
        moves: calls.core.Call.bridge
        instance: call_id
        emits: [calls.core.Observed]
      - name: already-bridged
        wrong_state: true
        refuses: false
  - name: calls.core.Enrich
    input:
      - {name: call_id, type: Uuid}
      - {name: amount, type: Integer}
    outcomes:
      - name: positive-init
        when_subject_state: Init
        when: amount > 0
        updates: calls.core.Call
        instance: call_id
        emits: [calls.core.Observed]
      - name: fallback
        updates: calls.core.Call
        instance: call_id
        emits: [calls.core.Observed]
";

fn assemble(source: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(source).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("open-default.yaml"), raw)])
        .map_err(|error| error.to_string())
}

#[test]
fn open_input_default_move_must_not_accept_an_invalid_held_state() {
    assemble(MODEL).expect("open input with state-preserving fallback is valid");
    let invalid = MODEL.replace(
        "      - name: fallback\n        updates: calls.core.Call",
        "      - name: fallback\n        moves: calls.core.Call.bridge",
    );
    // At held state Bridged, the Init guard is false for every admitted amount,
    // so fallback is selected although bridge can start only at Init.
    assert!(
        assemble(&invalid).is_err(),
        "open input plus a genuine default admitted bridge from held state Bridged"
    );
}

#[test]
fn open_input_ordinary_move_requires_safe_sources_too() {
    let invalid = MODEL.replace(
        "      - name: fallback\n",
        "      - name: negative\n        when: amount < 0\n        moves: calls.core.Call.bridge\n        instance: call_id\n        emits: [calls.core.Observed]\n      - name: fallback\n",
    );
    let error = assemble(&invalid).unwrap_err();
    assert!(
        error.contains("`Bridged` cannot take move `bridge`"),
        "{error}"
    );
}

#[test]
fn open_input_keeps_explicit_state_moves_and_external_authority() {
    let state_qualified = MODEL.replace(
        "        when: amount > 0\n        updates: calls.core.Call",
        "        when: amount > 0\n        moves: calls.core.Call.bridge",
    );
    assemble(&state_qualified).expect("explicit Init guard proves bridge source");
    let external = state_qualified.replace(
        "      - name: fallback\n",
        "      - name: externally-bridged\n        external: upstream connection confirmed\n        moves: calls.core.Call.bridge\n        instance: call_id\n        emits: [calls.core.Observed]\n      - name: fallback\n",
    );
    assemble(&external).expect("external effects keep their separate selection authority");
}
