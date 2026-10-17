//! A struct with a shorthand (`ess/27`, beyond10x/ess#594) in the synthesized targets: the Rust
//! and Go servers and the web app do not decode the short form yet, so each refuses a model that
//! reaches one, by name, rather than refusing an honest short-form value at run time. A model with
//! no shorthand is synthesized as before.

use super::*;

/// `{shorthand}` is the line written on `Operation`.
const SHORTHAND_MODEL: &str = "\
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
commands:
  - name: tracker.ops.Run
    input:
      - name: operation
        type: tracker.ops.Operation
    response:
      - name: operation
        type: tracker.ops.Operation
    outcomes:
      - name: ran
        returns: true
components:
  - component: ops
    owns: {domains: [tracker.ops]}
    accepts: {commands: [tracker.ops.Run]}
    reached_by: network
";

fn shorthand_model(shorthand: bool) -> EssIr {
    let text = SHORTHAND_MODEL.replace(
        "{shorthand}",
        if shorthand { "    shorthand: op\n" } else { "" },
    );
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("tracker.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn names_the_shorthand(text: &str) -> bool {
    text.contains("types.tracker.ops.Operation.shorthand") && text.contains("`shorthand: op`")
}

#[test]
fn synthesis_refuses_a_shorthand_struct_by_name() {
    let short = shorthand_model(true);
    for target in [Target::Rust, Target::Go, Target::Web] {
        let failure = synthesize_for(&short, target)
            .err()
            .unwrap_or_else(|| panic!("{target:?} refuses a shorthand struct"));
        let text = failure.to_string();
        assert!(names_the_shorthand(&text), "{target:?}: {text}");
        assert!(
            failure
                .causes()
                .iter()
                .all(|cause| cause.code() == TargetFailureCode::MissingRepresentation),
            "{target:?}: {text}"
        );
    }
    let plan = ess_synth::SynthesisPlan::of(&short);
    for (target, failure) in [
        ("rust", ess_synth::rust::workspace(&short, &plan).err()),
        ("go", ess_synth::go::workspace(&short, &plan).err()),
        ("web", ess_synth::web::workspace(&short, &plan).err()),
    ] {
        let failure = failure.unwrap_or_else(|| panic!("{target} refuses a shorthand struct"));
        let json = failure.to_canonical_json();
        assert!(names_the_shorthand(&json), "{target}: {json}");
    }

    // The same model without the key is synthesized by each.
    let plain = shorthand_model(false);
    for target in [Target::Rust, Target::Go, Target::Web] {
        if let Err(failure) = synthesize_for(&plain, target) {
            panic!("{target:?} synthesizes the long-form model: {failure}");
        }
    }
}
