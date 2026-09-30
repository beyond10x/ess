//! The sources an outcome declares for its error's fields (ess/19, `story:error-payload-sources`)
//! land in the IR as `error_payload`, in the error's declaration order and resolved as an event
//! payload's are. An outcome declaring none keeps its bytes.

use ess_compiler::ir::{EssIr, ResolvedOutcome, ResolvedPayloadValue};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/error-payload-sources.yaml");

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn outcome<'i>(ir: &'i EssIr, command: &str, name: &str) -> &'i ResolvedOutcome {
    ir.commands()
        .get(&command.parse().unwrap())
        .unwrap_or_else(|| panic!("{command} is declared"))
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == name)
        .unwrap_or_else(|| panic!("{command}/{name} is declared"))
}

#[test]
fn an_input_guarded_refusal_carries_its_error_sources_in_declaration_order() {
    let ir = ir();
    let refusal = outcome(&ir, "demo.order.PlaceOrder", "too-many");
    let targets: Vec<&str> = refusal
        .error_payload
        .iter()
        .map(|field| field.target.as_str())
        .collect();
    assert_eq!(targets, ["requested", "limit", "reason"]);
    assert!(matches!(
        &refusal.error_payload[0].value,
        ResolvedPayloadValue::InputField { field, .. } if field == "quantity"
    ));
    assert!(matches!(
        &refusal.error_payload[1].value,
        ResolvedPayloadValue::Literal { value } if value == "10"
    ));
    assert!(matches!(
        &refusal.error_payload[2].value,
        ResolvedPayloadValue::Literal { value } if value == "Quantity"
    ));
    assert!(refusal.payload.is_empty(), "no event payload on a refusal");
}

#[test]
fn a_wrong_state_refusal_reads_the_row_its_siblings_name() {
    let ir = ir();
    let refusal = outcome(&ir, "demo.order.CloseOrder", "already-closed");
    let quantity = refusal
        .error_payload
        .iter()
        .find(|field| field.target == "quantity")
        .expect("the stored field is a source");
    assert!(
        matches!(&quantity.value, ResolvedPayloadValue::SubjectField { field, .. } if field == "quantity"),
        "{quantity:?}"
    );
}

#[test]
fn the_ir_names_error_payload_only_where_one_is_declared() {
    let ir = ir();
    let json = serde_json::to_value(&ir).expect("the IR serialises");
    let text = json.to_string();
    assert_eq!(
        text.matches("\"error_payload\"").count(),
        3,
        "one key per refusal declaring sources, and none elsewhere"
    );
}
