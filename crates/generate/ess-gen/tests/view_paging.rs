//! The published contract says how a paged view is paged (`paging:`, ess/16, beyond10x/ess#174,
//! `docs/design/view-paging.md`).
//!
//! The `OpenAPI` operation of a view that declares `paging:` takes its page and size as optional
//! integer query parameters and its description says what they select and whether a total is
//! answered; the documentation page says the same. Every declared parameter is a query parameter,
//! so a view without `paging:` takes its filter parameter and nothing else, and gains no sentence.
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

/// The `GET` operation that reads `demo.jobs.JobList`.
fn list(document: &serde_json::Value) -> &serde_json::Value {
    document["paths"]
        .as_object()
        .expect("paths")
        .values()
        .filter_map(|item| item.get("get"))
        .find(|get| get["operationId"] == "demo.jobs.JobList")
        .unwrap_or_else(|| panic!("no operation reads the view:\n{document:#}"))
}

fn docs(ir: &EssIr) -> String {
    ess_gen::generate_all(ir)
        .unwrap()
        .into_iter()
        .filter(|(path, _)| {
            std::path::Path::new(path)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        })
        .map(|(_, artifact)| artifact.contents)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_view_operation_takes_its_page_and_size_as_query_parameters() {
    let document = document(&ir(JOBS));
    let list = list(&document);
    let parameters = list["parameters"].as_array().expect("parameters");
    let named: Vec<&str> = parameters
        .iter()
        .map(|parameter| parameter["name"].as_str().unwrap())
        .collect();
    assert_eq!(named, ["type", "page", "size"], "{list:#}");
    let filter = &parameters[0];
    assert_eq!(filter["required"], false, "{filter:#}");
    let parameters = &parameters[1..];
    for parameter in parameters {
        assert_eq!(parameter["in"], "query", "{parameter:#}");
        assert_eq!(parameter["required"], false, "{parameter:#}");
        assert_eq!(parameter["schema"]["type"], "integer", "{parameter:#}");
    }
    let page = &parameters[0];
    assert_eq!(page["schema"]["minimum"], 0, "{page:#}");
    assert_eq!(parameters[1]["schema"]["minimum"], 1, "{list:#}");
    let described = list["responses"]["200"]["description"].as_str().unwrap();
    assert!(
        described.contains(
            "Paged: `size` rows of the declared order starting at `page * size`, pages numbered \
             from 0; a read that sends neither answers every row."
        ),
        "{described}"
    );
    assert!(
        described.contains("The answer carries the number of rows the filter admits."),
        "{described}"
    );
}

#[test]
fn pages_numbered_from_one_are_documented_so() {
    let text = JOBS.replace(
        "paging: {page: page, size: size, total: true}",
        "paging: {page: page, size: size, first_page: 1}",
    );
    let document = document(&ir(&text));
    let list = list(&document);
    assert_eq!(list["parameters"][1]["schema"]["minimum"], 1, "{list:#}");
    let described = list["responses"]["200"]["description"].as_str().unwrap();
    assert!(
        described.contains("starting at `(page - 1) * size`, pages numbered from 1"),
        "{described}"
    );
    assert!(
        !described.contains("carries the number of rows"),
        "{described}"
    );
}

#[test]
fn the_documentation_says_how_the_view_is_paged() {
    let docs = docs(&ir(JOBS));
    assert!(
        docs.contains(
            "It is paged: `page` and `size` select `size` rows of that order starting at \
             `page * size`, pages numbered from 0, and a read that sends neither answers every \
             row. The answer carries the number of rows the filter admits."
        ),
        "{docs}"
    );
}

#[test]
fn a_view_without_paging_takes_only_its_filter_parameter() {
    let text = JOBS
        .replace("    paging: {page: page, size: size, total: true}\n", "")
        .replace(
            "      - {name: page, type: Integer}\n      - {name: size, type: Integer}\n",
            "",
        );
    let document = document(&ir(&text));
    let list = list(&document);
    let named: Vec<&str> = list["parameters"]
        .as_array()
        .expect("parameters")
        .iter()
        .map(|parameter| parameter["name"].as_str().unwrap())
        .collect();
    assert_eq!(named, ["type"], "{list:#}");
    assert!(!document.to_string().contains("Paged:"));
    let response = &document["components"]["schemas"]["demo.jobs.JobList.Response"];
    assert!(
        response["properties"].get("total").is_none(),
        "{response:#}"
    );
    assert!(!docs(&ir(&text)).contains("It is paged"));
}

#[test]
fn a_paged_response_holds_one_page_and_an_optional_total() {
    let paged = document(&ir(JOBS));
    let response = &paged["components"]["schemas"]["demo.jobs.JobList.Response"];
    assert_eq!(
        response["properties"]["total"]["type"], "integer",
        "{response:#}"
    );
    assert_eq!(
        response["required"],
        serde_json::json!(["rows"]),
        "{response:#}"
    );
    let described = list(&paged)["responses"]["200"]["description"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(
        described.starts_with("One page of the rows of `demo.jobs.JobList`"),
        "{described}"
    );
    let untotalled = JOBS.replace(
        "paging: {page: page, size: size, total: true}",
        "paging: {page: page, size: size}",
    );
    let other = document(&ir(&untotalled));
    let response = &other["components"]["schemas"]["demo.jobs.JobList.Response"];
    assert!(
        response["properties"].get("total").is_none(),
        "{response:#}"
    );
}
