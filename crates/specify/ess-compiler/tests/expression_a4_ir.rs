//! Family F part A4 in the IR (beyond10x/ess#233, `docs/design/expression-family-source22.md`): a
//! value read through `input.<path>` keeps the `field` key, now holding the declared segments, and
//! the type it is read at; an input fallback after `else:` is the one mapping
//! `{"input": {"field", "type_ref"}}`, while a literal fallback keeps its bytes.
use ess_compiler::ir::{EssIr, ResolvedFallback, ResolvedPayloadValue, ResolvedTypeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("fixtures/input-value-paths.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("leases.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn opened(ir: &EssIr) -> &ess_compiler::ir::ResolvedOutcome {
    ir.commands()
        .values()
        .flat_map(|command| &command.outcomes)
        .find(|outcome| outcome.name.as_str() == "opened")
        .expect("opened")
}

fn set<'a>(
    outcome: &'a ess_compiler::ir::ResolvedOutcome,
    target: &str,
) -> &'a ResolvedPayloadValue {
    &outcome
        .sets
        .iter()
        .find(|set| set.target == target)
        .unwrap_or_else(|| panic!("no `{target}`"))
        .value
}

#[test]
fn a4_a_path_is_read_at_its_last_segment_type() {
    let ir = ir(MODEL);
    let opened = opened(&ir);
    let ResolvedPayloadValue::InputField { field, type_ref } = set(opened, "generation_id") else {
        panic!("an input read");
    };
    assert_eq!(field, "opening.generation_id");
    assert!(
        matches!(type_ref, ResolvedTypeRef::Declared { name } if name.name().to_string() == "leases.pool.GenerationId"),
        "{type_ref:?}"
    );
    let ResolvedPayloadValue::InputField { field, type_ref } = set(opened, "sealed_label") else {
        panic!("an input read");
    };
    assert_eq!(field, "sealed.label", "through a newtype over a struct");
    assert_eq!(type_ref.to_string(), "String");
}

#[test]
fn a4_an_optional_parent_reads_an_optional_value() {
    let ir = ir(MODEL);
    let ResolvedPayloadValue::InputField { field, type_ref } =
        set(opened(&ir), "previous_generation")
    else {
        panic!("an input read");
    };
    assert_eq!(field, "previous.generation_id");
    assert_eq!(type_ref.to_string(), "Optional<leases.pool.GenerationId>");
}

#[test]
fn a4_an_input_fallback_is_typed_in_the_ir() {
    let ir = ir(MODEL);
    let ResolvedPayloadValue::InputOrGenerated {
        field,
        type_ref,
        otherwise: Some(ResolvedFallback::Input { input }),
    } = set(opened(&ir), "label")
    else {
        panic!("an input with an input fallback");
    };
    assert_eq!(field, "previous.label");
    assert_eq!(type_ref.to_string(), "Optional<String>");
    assert_eq!(input.field, "settings.defaults.label");
    assert_eq!(input.type_ref.to_string(), "String");
    let json = ir.to_canonical_json();
    assert!(
        json.contains(r#""field": "settings.defaults.label""#) && json.contains(r#""input": {"#),
        "the input fallback is the one mapping"
    );
    assert!(json.contains(r#""field": "opening.generation_id""#));
}

#[test]
fn a4_the_error_payload_reads_the_path() {
    let ir = ir(MODEL);
    let rejected = ir
        .commands()
        .values()
        .flat_map(|command| &command.outcomes)
        .find(|outcome| outcome.name.as_str() == "rejected")
        .expect("rejected");
    let read = rejected
        .error_payload
        .iter()
        .find(|field| field.target == "generation_id")
        .expect("the error field");
    assert!(
        matches!(&read.value, ResolvedPayloadValue::InputField { field, .. } if field == "opening.generation_id")
    );
}

/// A literal after `else:` keeps the bytes an `ess/16` document compiled to.
#[test]
fn a4_a_literal_fallback_keeps_its_bytes() {
    let literal = MODEL
        .replace(
            "          label: {input: previous.label, else: input.settings.defaults.label}\n          sealed_label",
            "          label: {input: previous.label, else: 'none yet'}\n          sealed_label",
        );
    assert_ne!(literal, MODEL);
    let ir = ir(&literal);
    let ResolvedPayloadValue::InputOrGenerated { otherwise, .. } = set(opened(&ir), "label") else {
        panic!("an input with a literal fallback");
    };
    assert_eq!(
        otherwise,
        &Some(ResolvedFallback::Literal("none yet".to_owned()))
    );
    assert!(
        ir.to_canonical_json()
            .contains(r#""otherwise": "none yet""#),
        "a literal is still the string it was"
    );
}

#[test]
fn a4_describe_names_the_path_and_the_fallback() {
    let ir = ir(MODEL);
    assert_eq!(
        set(opened(&ir), "label").describe(),
        "input.previous.label, else input.settings.defaults.label"
    );
    assert_eq!(
        set(opened(&ir), "generation_id").describe(),
        "input.opening.generation_id"
    );
}
