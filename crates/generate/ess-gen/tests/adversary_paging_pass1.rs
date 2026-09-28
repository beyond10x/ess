//! Adversary, pass 1, story `view-paging-and-caller-filters`: the published contract of a paged
//! view with `total: true` says the answer carries the total, and its response schema has to have
//! somewhere to put it.
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const JOBS: &str = include_str!("../../../verify/ess-conformance/tests/fixtures/view-paging.yaml");

const COMPONENT: &str = "components:\n  - component: jobs-service\n    reached_by: network\n    owns:\n      domains: [demo.jobs]\n    accepts:\n      commands: [demo.jobs.CreateJob]\n    publishes:\n      events: [demo.jobs.JobCreated]\n";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(&format!("{text}{COMPONENT}")).unwrap();
    let spec = Specification::assemble([(Source::new("jobs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn document(ir: &EssIr) -> serde_json::Value {
    let component = ir.components().values().next().expect("a component");
    serde_json::from_str(&ess_gen::openapi::json(ir, component)).expect("the document is JSON")
}

/// The response description of the paged view promises a total; the response schema is closed
/// (`additionalProperties: false`) over `rows` alone, so a server that answers the total it was
/// told to answer returns a body its own `OpenAPI` document refuses.
#[test]
fn a_paged_view_with_a_total_has_a_response_schema_that_can_carry_it() {
    let document = document(&ir(JOBS));
    let response = &document["components"]["schemas"]["demo.jobs.JobList.Response"];
    assert!(response.is_object(), "no response schema:\n{document:#}");
    let properties = response["properties"]
        .as_object()
        .expect("the response schema has properties");
    let open = response["additionalProperties"] != serde_json::Value::Bool(false);
    assert!(
        open || properties.keys().any(|key| key != "rows"),
        "the view declares `total: true` and its operation says \"The answer carries the number of \
         rows the filter admits.\", but the response body admits only `rows`:\n{response:#}"
    );
}

/// The `rows` property still says it is every row the projection holds, which a page is not.
#[test]
fn a_paged_view_response_does_not_say_it_holds_every_row() {
    let document = document(&ir(JOBS));
    let rows = &document["components"]["schemas"]["demo.jobs.JobList.Response"]["properties"]
        ["rows"]["description"];
    assert_ne!(
        rows.as_str(),
        Some("Every row the projection holds, in the order it holds them."),
        "a paged read answers one page, and the schema says every row"
    );
}
