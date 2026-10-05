//! The held lifecycle state as a value source, `{subject: state}` (ess/23, beyond10x/ess#458),
//! resolved: a `SubjectState` source of its own, typed as the entity's `State`, in an error payload,
//! an event payload and `sets:`, beside the literal the requested state stays.

use ess_compiler::ir::{EssIr, ResolvedPayloadField, ResolvedPayloadValue, ResolvedTypeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/subject-state-source.yaml");

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap();
    let spec = Specification::assemble([(Source::new("docs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

fn field<'a>(fields: &'a [ResolvedPayloadField], name: &str) -> &'a ResolvedPayloadValue {
    &fields
        .iter()
        .find(|field| field.target == name)
        .unwrap_or_else(|| panic!("`{name}` in {fields:#?}"))
        .value
}

fn held_state(value: &ResolvedPayloadValue) {
    match value {
        ResolvedPayloadValue::SubjectState {
            type_ref: ResolvedTypeRef::Declared { name },
        } => assert_eq!(name.name().to_string(), "demo.docs.Doc.State"),
        other => panic!("not the held state: {other:#?}"),
    }
}

#[test]
fn subject_state_source_validates_on_wrong_state_and_the_ir_carries_it() {
    let ir = ir();
    let command = |name: &str| &ir.commands()[&name.parse().unwrap()];
    let publish = command("demo.docs.PublishDoc");
    let conflict = publish
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "conflict")
        .unwrap();
    held_state(field(&conflict.error_payload, "current"));
    assert_eq!(
        field(&conflict.error_payload, "requested"),
        &ResolvedPayloadValue::Literal {
            value: "Published".to_owned()
        }
    );
    let published = &publish.outcomes[0];
    held_state(field(&published.payload[0].fields, "from"));
    let archived = &command("demo.docs.ArchiveDoc").outcomes[0];
    held_state(field(&archived.sets, "previous"));
    // The canonical document names it as a source of its own.
    assert!(ir.to_canonical_json().contains("subject_state"));
}
