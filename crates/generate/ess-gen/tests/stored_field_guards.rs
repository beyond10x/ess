//! The published contract names the stored fields a guarded branch reads (ess#75).
//!
//! `docs/design/cross-record-and-stored-field-guards.md`, "Projections": the predicate form renders
//! as `SubjectState` does — the predicate through `Display` — in the documentation and in `OpenAPI`,
//! and a refusal it decides answers `409`, as every refusal the subject decides does.
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const PARCELS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/stored-field-guards.yaml");

/// The design example with one component accepting both commands, so an `OpenAPI` document is
/// published for it.
const COMPONENT: &str = "components:\n  - component: parcel-service\n    owns:\n      domains: [shipping.parcel]\n    accepts:\n      commands: [shipping.parcel.Create, shipping.parcel.Dispatch]\n    publishes:\n      events: [shipping.parcel.Created, shipping.parcel.Dispatched]\n";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(&format!("{PARCELS}{COMPONENT}")).unwrap();
    let spec = Specification::assemble([(Source::new("parcels.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn openapi(ir: &EssIr) -> String {
    ess_gen::generate_all(ir)
        .unwrap()
        .into_iter()
        .filter(|(path, _)| path.contains("openapi"))
        .map(|(path, artifact)| format!("== {path}\n{}", artifact.contents))
        .collect::<Vec<_>>()
        .join("\n")
}

fn artifact(ir: &EssIr, suffix: &str) -> String {
    ess_gen::generate_all(ir)
        .unwrap()
        .into_iter()
        .filter(|(path, _)| path.ends_with(suffix))
        .map(|(_, artifact)| artifact.contents)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_documentation_names_the_stored_fields_the_refusal_reads() {
    let docs = artifact(&ir(), ".md");
    assert!(
        docs.contains(
            "Taken when the existing subject's stored fields satisfy `(service == Express and weight_kg > 20)`."
        ),
        "{docs}"
    );
    assert!(
        !docs.contains("SubjectPredicate"),
        "no Rust structure in a page"
    );
}

#[test]
fn the_openapi_contract_names_the_stored_fields_and_answers_conflict() {
    let ir = ir();
    let everything = openapi(&ir);
    assert!(
        everything.starts_with("== "),
        "an OpenAPI artifact is generated"
    );
    assert!(
        everything.contains(
            "Taken when the existing subject's stored fields satisfy `(service == Express and weight_kg > 20)`."
        ),
        "{everything}"
    );
    let dispatch = &ir.commands()[&"shipping.parcel.Dispatch".parse().unwrap()];
    assert_eq!(ess_gen::http::status(&dispatch.outcomes[0]), "409");
}
