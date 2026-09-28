//! `defined(x)` over an `Optional` struct, list or map (beyond10x/ess#176, source format `ess/16`).
//!
//! Presence is a property of the `Optional`, not of what it holds, so `defined()` admits any
//! `Optional<T>` from `ess/16` on, in an invariant as in a guard. Below `ess/16` an `Optional`
//! aggregate is refused as a format the document does not declare. An aggregate that is not
//! `Optional`, and `truthy` over any aggregate, stay refused as a type mismatch.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const QUEUE: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/defined-over-optional-aggregates.yaml"
);

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("queue.yaml"), raw)])
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn at_format(text: &str, format: &str) -> String {
    text.replace("format: ess/16", &format!("format: {format}"))
}

/// The fixture with one more command whose single outcome is guarded by `when` over `input`.
fn guarded(input: &str, when: &str) -> String {
    guarded_in(QUEUE, input, when)
}

fn guarded_in(base: &str, input: &str, when: &str) -> String {
    base.replace(
        "events:\n",
        &format!(
            "  - name: demo.queue.Probe\n    input:\n{input}    outcomes:\n      - name: probed\n        when: '{when}'\n        emits: [demo.queue.QueueOpened]\n        payload: {{demo.queue.QueueOpened: {{queue_id: {{generated: true}}}}}}\n      - name: declined\n        emits: [demo.queue.QueueResumed]\n        payload: {{demo.queue.QueueResumed: {{queue_id: {{generated: true}}}}}}\nevents:\n"
        ),
    )
    .replace(
        "may: [demo.queue.OpenQueue,",
        "may: [demo.queue.Probe, demo.queue.OpenQueue,",
    )
}

#[test]
fn issue_176_the_presence_invariant_over_an_optional_struct_validates() {
    if let Err(errors) = assemble(QUEUE) {
        panic!("{}", listed(&errors));
    }
}

#[test]
fn below_ess_16_defined_over_an_optional_struct_is_refused_as_a_format_the_document_does_not_declare(
) {
    let errors = assemble(&at_format(QUEUE, "ess/15")).expect_err("must refuse under ess/15");
    let refusal = errors
        .as_slice()
        .iter()
        .find(|error| error.code == ValidationCode::UnsupportedFormatVersion)
        .unwrap_or_else(|| panic!("no unsupported_format_version:\n{}", listed(&errors)));
    assert!(
        refusal.message.contains("ess/16") && refusal.message.contains("defined(metrics)"),
        "{}",
        listed(&errors)
    );
    assert!(
        !errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::TypeMismatch),
        "an Optional aggregate is a format question below ess/16, not a type mismatch:\n{}",
        listed(&errors)
    );
}

#[test]
fn defined_admits_an_optional_list_map_and_struct_in_a_guard() {
    for (input, when) in [
        (
            "      - {name: tags, type: Optional<List<String>>}\n",
            "defined(tags)",
        ),
        (
            "      - {name: counts, type: 'Optional<Map<String, Integer>>'}\n",
            "defined(counts)",
        ),
        (
            "      - {name: metrics, type: Optional<demo.queue.Metrics>}\n",
            "not defined(metrics)",
        ),
    ] {
        let text = guarded(input, when);
        if let Err(errors) = assemble(&text) {
            panic!("`{when}` must validate:\n{}\n{text}", listed(&errors));
        }
        let errors = assemble(&at_format(&text, "ess/15")).expect_err("refused under ess/15");
        assert!(
            errors.as_slice().iter().any(|error| error.code
                == ValidationCode::UnsupportedFormatVersion
                && error.message.contains("ess/16")),
            "`{when}` under ess/15:\n{}",
            listed(&errors)
        );
    }
}

#[test]
fn defined_admits_a_struct_reached_through_an_optional() {
    let base = QUEUE.replace(
        "      - {name: waiting, type: Integer}\n",
        "      - {name: waiting, type: Integer}\n  - name: demo.queue.Holder\n    kind: struct\n    fields:\n      - {name: inner, type: demo.queue.Metrics}\n",
    );
    let text = guarded_in(
        &base,
        "      - {name: holder, type: Optional<demo.queue.Holder>}\n",
        "defined(holder.inner)",
    );
    if let Err(errors) = assemble(&text) {
        panic!("{}\n{text}", listed(&errors));
    }
}

#[test]
fn an_aggregate_that_is_not_optional_and_truthy_over_any_aggregate_stay_type_mismatches() {
    for (input, when) in [
        (
            "      - {name: metrics, type: demo.queue.Metrics}\n",
            "defined(metrics)",
        ),
        (
            "      - {name: metrics, type: Optional<demo.queue.Metrics>}\n",
            "metrics",
        ),
    ] {
        let errors = assemble(&guarded(input, when)).expect_err("must refuse");
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::TypeMismatch
                    && error.location.contains("demo.queue.Probe")
                    && error.message.contains("aggregate")),
            "`{when}`:\n{}",
            listed(&errors)
        );
    }
}
