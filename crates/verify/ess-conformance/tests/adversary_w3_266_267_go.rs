//! Adversary pass 1 against beyond10x/ess#266 and #267, in the generated Go runtime: the two
//! synthesized scenarios that fail an honest binding-running target in the reference runner fail it
//! in Go as well (the parity helper asserts Go's verdicts equal Rust's).

mod support_binding_arrangement;
mod support_go;

use ess_conformance::synthesize::synthesize;
use support_binding_arrangement::*;

/// `with_relabel_bound_to` of `adversary_w3_266_267_pass1.rs`, repeated: test crates share no code.
fn with_relabel_bound_to(event: &str) -> String {
    let text = rewrite(
        FIXTURE,
        "events:\n",
        "events:\n  - name: jobs.job.Relabelled\n    fields: [{name: job_id, type: jobs.job.JobId}]\n",
    );
    let text = rewrite(
        &text,
        "  - name: jobs.job.Stop\n",
        "  - name: jobs.job.Relabel
    input:
      - {name: job_id, type: jobs.job.JobId}
    outcomes:
      - name: relabelled
        updates: jobs.job.Job
        instance: job_id
        sets: {label: relabelled}
        emits: [jobs.job.Relabelled]
        payload: {jobs.job.Relabelled: {job_id: input.job_id}}
      - {name: not-found, unknown_instance: true, error: jobs.job.JobNotFound}

  - name: jobs.job.Stop
",
    );
    format!(
        "{text}
  - id: relabels-on-{lower}
    when: {{event: jobs.job.{event}}}
    invoke: {{command: jobs.job.Relabel}}
    mapping: {{job_id: event.job_id}}
    delivery: at_least_once
    on_failure: drop
",
        lower = event.to_lowercase()
    )
}

#[test]
fn adv_go_drop_passes_an_honest_target_with_a_sibling_binding_on_the_row() {
    let text = with_relabel_bound_to("Kicked");
    let suite = synthesize(&model(&text)).suite;
    let native = support_go::rust_outcomes_admitted(
        &ess_conformance::AdmittedSuite::from_suite(&suite).unwrap(),
        &interpreted(&text),
    );
    eprintln!(
        "suite {} scenarios; native {:?}",
        suite.scenarios.len(),
        native.get(DROP)
    );
    let verdicts = support_go::assert_parity("adv-sibling-drop", &suite, interpreted(&text));
    assert_eq!(
        verdicts.get(DROP).map(String::as_str),
        Some("passed"),
        "{verdicts:?}"
    );
}

#[test]
fn adv_go_relabel_flow_passes_an_honest_target_beside_a_state_moving_binding() {
    let text = with_relabel_bound_to("Created");
    let suite = synthesize(&model(&text)).suite;
    let verdicts = support_go::assert_parity("adv-two-on-created", &suite, interpreted(&text));
    for id in [
        "relabels-on-created/binding/flow",
        "relabels-on-created/binding/delivery",
    ] {
        assert_eq!(
            verdicts.get(id).map(String::as_str),
            Some("passed"),
            "{id}: {verdicts:?}"
        );
    }
}
