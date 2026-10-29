//! Adversarial cases for a command whose input and response fields share a name: the response is
//! its own struct and its own field scope, `fields:{command}.response`.
//!
//! Each case here reads the refusal a collision now carries. The cases that build the generated
//! code or serve a real answer through it are in `feasibility.rs`.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::synthesize;

fn fixture(documents: &[(&str, &str)]) -> Result<EssIr, String> {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (label, text) in documents {
        let raw = RawSpecFile::parse(text).map_err(|error| format!("parse: {error}"))?;
        sources.insert((*label).to_owned(), (*text).to_owned());
        labels.push((*label).to_owned());
        parsed.push((Source::new((*label).to_owned()), raw));
    }
    let specification =
        Specification::assemble(parsed).map_err(|errors| format!("assemble: {errors}"))?;
    compile_locating(&specification, &sources, &labels)
        .map_err(|errors| format!("compile: {errors}"))
}

fn details(failure: &ess_synth::TargetFailure) -> String {
    failure
        .causes()
        .iter()
        .map(|cause| cause.detail().to_owned())
        .collect::<Vec<_>>()
        .join("\n")
}

/// A served command whose input and response share every field name but one, and that one pair
/// differs only by case and underscores: both become `foo_bar` in Rust.
const SHARED: &str = include_str!("fixtures/response-echoes-input.yaml");

/// The response scope `fields:demo.core.Send.response` is a string; a declaration whose qualified
/// name spells the same string allocates its fields in the same scope.
#[test]
fn a_declaration_named_like_the_response_scope_is_no_collision() {
    let system = "format: ess/22\nsystem: demo\nversion: v1\ndomains:\n  - demo.core\n";
    let core = "domain: demo.core\ntypes:\n  - name: demo.core.Send.response\n    naming: {code: Envelope}\n    kind: struct\n    fields:\n      - {name: id, type: String}\nevents:\n  - name: demo.core.Sent\n    fields:\n      - {name: id, type: String}\ncommands:\n  - name: demo.core.Send\n    input:\n      - {name: text, type: String}\n    response:\n      - {name: id, type: String}\n    outcomes:\n      - name: sent\n        returns: true\n        emits: [demo.core.Sent]\n        payload:\n          demo.core.Sent:\n            id: {generated: true}\n";
    let wiring = "components:\n  - component: messages\n    owns: {domains: [demo.core]}\n    accepts: {commands: [demo.core.Send]}\n    publishes: {events: [demo.core.Sent]}\n    reached_by: network\n";
    let ir = match fixture(&[
        ("system.yaml", system),
        ("core.yaml", core),
        ("wiring.yaml", wiring),
    ]) {
        Ok(ir) => ir,
        // The specification refuses the model: nobody reaches the shared scope.
        Err(error) => {
            eprintln!("refused before synthesis: {error}");
            return;
        }
    };
    if let Err(failure) = synthesize(&ir) {
        let detail = details(&failure);
        assert!(
            !detail.contains("in `fields:demo.core.Send.response`"),
            "two structs share one field scope by spelling: {detail}"
        );
    }
}

/// A type that takes the generated `{Command}Response` name collides with the response struct.
/// That collision is not field against field, so it keeps the `code:` remedy.
#[test]
fn a_mixed_collision_with_the_response_struct_keeps_the_code_remedy() {
    let model = SHARED.replace(
        "types:\n",
        "types:\n  - name: demo.core.SendResponse\n    kind: struct\n    fields:\n      - {name: other, type: String}\n",
    );
    let ir = fixture(&[("demo.yaml", &model)]).expect("the model is ESS");
    let detail = details(
        &synthesize(&ir)
            .err()
            .expect("`SendResponse` is allocated twice"),
    );
    assert!(
        detail.contains("`SendResponse` is allocated 2 times in `domain:"),
        "{detail}"
    );
    assert!(detail.contains("naming: { code: … }"), "{detail}");
    assert!(!detail.contains("naming: { wire: … }"), "{detail}");
}

/// Two response fields of one wire name write one JSON member twice; Rust refuses the same for
/// input fields (`wire_fields`).
#[test]
fn two_response_fields_of_one_wire_name_do_not_reach_the_codec() {
    let model = SHARED.replace(
        "    response:\n      - {name: id, type: String}\n      - {name: foo_bar, type: String}\n      - {name: type, type: String}\n      - {name: body, type: demo.core.Body}\n      - {name: bodies, type: List<demo.core.Body>}\n      - {name: note, type: Optional<demo.core.Body>}\n",
        "    response:\n      - {name: reply, type: String}\n      - {name: other, type: String, naming: {wire: reply}}\n",
    );
    assert_ne!(model, SHARED, "the response block is replaced");
    let ir = match fixture(&[("demo.yaml", &model)]) {
        Ok(ir) => ir,
        Err(error) => {
            assert!(error.contains("reply"), "{error}");
            return;
        }
    };
    match synthesize(&ir) {
        Err(failure) => assert!(details(&failure).contains("reply"), "{}", details(&failure)),
        Ok(synthesis) => {
            let wire = &synthesis.artifacts["crates/demo-server/src/wire.rs"].contents;
            let encoder = &wire[wire
                .find("pub fn encode_response_")
                .expect("the response encoder")..];
            let encoder = &encoder[..encoder.find("\n}\n").expect("it closes")];
            assert_eq!(
                encoder.matches("json::member(out, \"reply\");").count(),
                1,
                "the response encoder writes `reply` twice: {encoder}"
            );
        }
    }
}
