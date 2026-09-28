//! Adversary, pass 1, story:current-time-guard-operand (beyond10x/ess#171).
//!
//! The acceptance: "A Timestamp guard can compare an input with the current time" under `ess/16`.
//! The unit admits the operand where `OutcomeCondition::When` is checked, and refuses it in every
//! other condition — including the input `when:` that sits beside `when_subject_state:`, which is
//! the ordinary input guard of a command outcome, read while the request is handled. The refusal
//! then tells the author the operand is "admitted only in a command outcome's `when:`" about a
//! guard written in exactly that key.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model(guard: &str) -> String {
    format!(
        "format: ess/16\nsystem: calls\nversion: v1\ndomain: calls.core\nentities:\n  - name: calls.core.Call\n    identity: {{name: call_id, type: Uuid}}\n    fields: []\n    lifecycle:\n      initial: Bridged\n      states: [Bridged]\n      terminal: [Bridged]\nevents:\n  - name: calls.core.Observed\n    fields: []\nerrors:\n  - name: calls.core.Stale\n    summary: The reading is more than a minute old.\n    fields: []\ncommands:\n  - name: calls.core.Enrich\n    input:\n      - {{name: call_id, type: Uuid}}\n      - {{name: read_at, type: Timestamp}}\n    outcomes:\n      - name: stale\n        when_subject_state: Bridged\n        when: {guard}\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n      - name: preserved\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n"
    )
}

fn specification(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("subject.yaml"), raw)]).map_err(|error| {
        error
            .as_slice()
            .iter()
            .map(|e| format!("{:?} {} {}", e.code, e.location, e.message))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// Control: the same model with a fixed instant validates, so the shape is admitted.
#[test]
fn adversary_now_control_a_fixed_instant_beside_a_held_state_validates() {
    specification(&model("read_at < '2020-01-01T00:00:00Z'"))
        .unwrap_or_else(|errors| panic!("the control validates:\n{errors}"));
}

/// The input guard beside a held state is a command outcome's `when:`, read while the request is
/// handled; the operand is admitted there as it is in a plain `when:`.
#[test]
fn adversary_now_an_input_when_beside_a_held_state_admits_now() {
    specification(&model("read_at < now - 60s")).unwrap_or_else(|errors| {
        panic!("`when: read_at < now - 60s` beside `when_subject_state` is refused:\n{errors}")
    });
}
