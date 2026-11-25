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
        type: {op_type}
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
      - name: performed
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
    model_of(shorthand, "String")
}

/// The model with `op` of `op_type`.
fn model_of(shorthand: bool, op_type: &str) -> EssIr {
    let text = SHORTHAND_MODEL
        .replace(
            "{shorthand}",
            if shorthand { "    shorthand: op\n" } else { "" },
        )
        .replace("{op_type}", op_type);
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
    for target in [Target::Go, Target::Web] {
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
        ("go", ess_synth::go::workspace(&short, &plan).err()),
        ("web", ess_synth::web::workspace(&short, &plan).err()),
    ] {
        let failure = failure.unwrap_or_else(|| panic!("{target} refuses a shorthand struct"));
        let json = failure.to_canonical_json();
        assert!(names_the_shorthand(&json), "{target}: {json}");
    }

    // The same model without the key is synthesized by each.
    let long_form = shorthand_model(false);
    for target in [Target::Rust, Target::Go, Target::Web] {
        if let Err(failure) = synthesize_for(&long_form, target) {
            panic!("{target:?} synthesizes the long-form model: {failure}");
        }
    }
}

/// The text of every `Operation` decoder one synthesis writes, by artifact path.
fn operation_decoders(synthesis: &ess_synth::Synthesis) -> Vec<(String, String)> {
    let doc = "/// Reads `tracker.ops.Operation` from JSON.";
    synthesis
        .artifacts
        .iter()
        .filter_map(|(path, artifact)| {
            let start = artifact.contents.find(doc)?;
            let rest = &artifact.contents[start..];
            let end = rest.find("\n}\n").map_or(rest.len(), |end| end + 3);
            Some((path.clone(), rest[..end].to_owned()))
        })
        .collect()
}

/// The Rust target synthesizes a shorthand struct in both layouts (TD-D7 A), and its decoder reads
/// a value that is not an object as the shorthand field alone, every other field absent.
#[test]
fn the_rust_target_decodes_a_shorthand_struct_from_both_spellings_in_both_layouts() {
    let short = shorthand_model(true);
    let plan = ess_synth::SynthesisPlan::of(&short);
    if let Err(failure) = ess_synth::rust::workspace(&short, &plan) {
        panic!("rust workspace synthesizes a shorthand struct: {failure}");
    }
    for layout in [
        ess_synth::OutputLayout::Workspace,
        ess_synth::OutputLayout::Crate,
    ] {
        let synthesis = ess_synth::synthesize_laid_out(&short, Target::Rust, layout)
            .unwrap_or_else(|failure| panic!("{layout:?} synthesizes: {failure}"));
        let decoders = operation_decoders(&synthesis);
        assert!(!decoders.is_empty(), "{layout:?}: no Operation decoder");
        for (path, decoder) in decoders {
            for read in [
                "if !matches!(value, json::Value::Object(_)) {",
                "op: json::text_at(value, at, \"a string\")?.to_owned(),",
                "paging: None,",
            ] {
                assert!(
                    decoder.contains(read),
                    "{layout:?} {path} lacks `{read}`:\n{decoder}"
                );
            }
        }
    }

    // A struct without the key keeps its object-only decoder.
    let long_form = synthesize_for(&shorthand_model(false), Target::Rust)
        .unwrap_or_else(|failure| panic!("{failure}"));
    for (path, decoder) in operation_decoders(&long_form) {
        assert!(
            !decoder.contains("json::Value::Object(_)"),
            "{path}:\n{decoder}"
        );
    }
}

/// A numeric shorthand field is read in the short form too (SB-D4 A): the synthesized reader
/// keeps a number in the spelling it arrived in, so no serde limit applies.
#[test]
fn the_rust_target_decodes_a_numeric_shorthand_field_in_the_short_form() {
    for (op_type, read) in [
        (
            "Integer",
            "op: json::integer_at(value, at, \"an integer\")?,",
        ),
        (
            "Decimal",
            "json::decimal_at(value, at, \"a decimal string\")?",
        ),
        ("Binary64", "op: binary64_at(value, at)?,"),
    ] {
        let synthesis = synthesize_for(&model_of(true, op_type), Target::Rust)
            .unwrap_or_else(|failure| panic!("{op_type}: {failure}"));
        let decoders = operation_decoders(&synthesis);
        assert!(!decoders.is_empty(), "{op_type}: no Operation decoder");
        for (path, decoder) in decoders {
            assert!(
                decoder.contains("if !matches!(value, json::Value::Object(_)) {")
                    && decoder.contains(read),
                "{op_type} {path} lacks `{read}`:\n{decoder}"
            );
        }
    }
}
