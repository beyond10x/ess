//! A struct with a shorthand (`ess/27`, beyond10x/ess#594) in conformance synthesis: a target may
//! answer such a value in its short form, which no observer decodes yet, so every scenario that
//! would observe one — in a response or an event payload — is withdrawn and refused naming
//! `shorthand` and the type. A model with no shorthand synthesizes the same suite as before.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::synthesize;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// `{shorthand}` is the line written on `Operation`.
const MODEL: &str = "\
format: ess/27
system: tracker
version: v1
domain: tracker.ops
types:
  - name: tracker.ops.Operation
    kind: struct
{shorthand}    fields:
      - name: op
        type: String
      - name: paging
        type: Optional<String>
actors:
  - name: tracker.ops.Operator
    may:
      - tracker.ops.Run
      - tracker.ops.Read
commands:
  - name: tracker.ops.Run
    input:
      - name: operation
        type: tracker.ops.Operation
    outcomes:
      - name: ran
        emits: [tracker.ops.Ran]
        payload:
          tracker.ops.Ran:
            operation: input.operation
  - name: tracker.ops.Read
    input:
      - name: operation
        type: tracker.ops.Operation
    response:
      - name: operation
        type: tracker.ops.Operation
    outcomes:
      - name: read
        returns: true
events:
  - name: tracker.ops.Ran
    fields:
      - name: operation
        type: tracker.ops.Operation
";

fn model(shorthand: bool) -> EssIr {
    let text = MODEL.replace(
        "{shorthand}",
        if shorthand { "    shorthand: op\n" } else { "" },
    );
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("tracker.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

/// The scenario ids of `ir`'s synthesized suite that name `command`.
fn scenarios_of(ir: &EssIr, command: &str) -> Vec<String> {
    synthesize(ir)
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.contains(command))
        .collect()
}

#[test]
fn conformance_synthesis_refuses_an_observed_shorthand_struct_by_name() {
    let plain = model(false);
    for command in ["tracker.ops.Run", "tracker.ops.Read"] {
        assert!(
            !scenarios_of(&plain, command).is_empty(),
            "the long-form model observes `{command}`: {:#?}",
            synthesize(&plain).refusals
        );
    }

    let short = model(true);
    let synthesis = synthesize(&short);
    for command in ["tracker.ops.Run", "tracker.ops.Read"] {
        let observed: Vec<String> = synthesis
            .suite
            .scenarios
            .values()
            .flat_map(|scenario| scenario.steps.iter())
            .map(|step| serde_json::to_string(step).expect("a step serializes"))
            .filter(|step| {
                ((step.contains("\"step\":\"expect_event\"")
                    || step.contains("\"step\":\"eventually_event\""))
                    && step.contains("tracker.ops.Ran"))
                    || step.contains("\"step\":\"expect_direct_response\"")
            })
            .collect();
        assert!(
            observed.is_empty(),
            "no step observes the shorthand struct: {observed:#?}"
        );
        let refused: Vec<String> = synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .filter(|text| text.contains(command))
            .collect();
        assert!(
            refused
                .iter()
                .any(|text| text.contains("shorthand") && text.contains("tracker.ops.Operation")),
            "`{command}` is refused naming `shorthand` and the type: {refused:#?}"
        );
    }
}
