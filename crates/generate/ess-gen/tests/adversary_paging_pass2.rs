//! Adversary, pass 2, story `view-paging-and-caller-filters`: the `OpenAPI` operation of a paged
//! view, read as the contract a caller builds its request from.
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

fn list(ir: &EssIr) -> serde_json::Value {
    let component = ir.components().values().next().expect("a component");
    let document: serde_json::Value =
        serde_json::from_str(&ess_gen::openapi::json(ir, component)).expect("JSON");
    document["paths"]
        .as_object()
        .expect("paths")
        .values()
        .filter_map(|item| item.get("get"))
        .find(|get| get["operationId"] == "demo.jobs.JobList")
        .cloned()
        .unwrap_or_else(|| panic!("no operation reads the view:\n{document:#}"))
}

fn query_names(list: &serde_json::Value) -> Vec<String> {
    list["parameters"]
        .as_array()
        .map(|parameters| {
            parameters
                .iter()
                .map(|parameter| parameter["name"].as_str().unwrap().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// The #174 repro filters by `param.type`. Its operation now lists `page` and `size` as query
/// parameters and nothing else, so a caller reading the contract sees a complete-looking parameter
/// list without the filter parameter the view cannot be read as intended without.
#[test]
fn adversary_paging_pass2_the_paged_operation_lists_the_filter_parameter_too() {
    let list = list(&ir(JOBS));
    let names = query_names(&list);
    assert!(
        names.iter().any(|name| name == "type"),
        "the operation documents {names:?}, not the declared filter parameter `type`:\n{list:#}"
    );
}

/// With a wire name on the page parameter the query key is `pageNumber`, and the operation's
/// description still says how `page` and `size` select rows — a name no request carries.
#[test]
fn adversary_paging_pass2_the_paging_description_names_the_query_keys() {
    let text = JOBS.replace(
        "      - {name: page, type: Integer}\n",
        "      - {name: page, type: Integer, wire: pageNumber}\n",
    );
    let list = list(&ir(&text));
    let names = query_names(&list);
    assert!(names.iter().any(|name| name == "pageNumber"), "{names:?}");
    let described = list["responses"]["200"]["description"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        !described.contains("`page * size`"),
        "the description computes the offset from `page`, which is not a query key here \
         ({names:?}): {described}"
    );
}
