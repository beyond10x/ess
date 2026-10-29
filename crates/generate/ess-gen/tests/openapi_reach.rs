//! A component not declared `reached_by: network` keeps its `OpenAPI` document and its command
//! paths, and the document says that no request reaches them
//! (<https://github.com/beyond10x/ess/issues/651>).
//!
//! Only a network component's document is load-bearing; every other one documents the commands its
//! callers, deployed with it or at its command line, invoke without HTTP. Its first `info`
//! paragraph says so, and only a network document carries `info.x-ess-reached-by`.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::openapi::OpenApi;
use serde_json::Value;

const MODEL: &str = include_str!("../../../specify/ess-http/tests/fixtures/catalog.yaml");

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("well formed");
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("catalog.yaml", text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The items service's document, generated from `text`.
fn document(text: &str) -> Value {
    let ir = model(text);
    let artifacts = run(&OpenApi, &ir).expect("generates");
    let yaml: ess_yaml::Value =
        ess_yaml::from_str(&artifacts["openapi/items-service.yaml"].contents).expect("YAML");
    serde_json::to_value(yaml).expect("JSON")
}

fn first_paragraph(document: &Value) -> &str {
    let description = document["info"]["description"]
        .as_str()
        .expect("description");
    description.split("\n\n").next().expect("a first paragraph")
}

fn command_paths(document: &Value) -> Vec<&str> {
    document["paths"]
        .as_object()
        .expect("paths")
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn a_component_not_reached_over_the_network_says_no_request_reaches_its_paths() {
    let network = document(MODEL);
    assert_eq!(
        first_paragraph(&network),
        "The HTTP surface of `items-service`, one of the components of `catalog` v1.",
        "a network document keeps its first paragraph"
    );
    let in_process = document(&MODEL.replace("    reached_by: network\n", ""));
    assert_eq!(
        first_paragraph(&in_process),
        "The HTTP surface of `items-service`, one of the components of `catalog` v1. It is not \
         declared `reached_by: network`: its callers are deployed with it, so these paths describe \
         its commands and no request reaches them."
    );
    assert_eq!(
        command_paths(&in_process),
        ["/items/commands/ListItem", "/items/commands/TouchItem"],
        "its command paths stay"
    );
    assert!(
        in_process["info"].get("x-ess-reached-by").is_none(),
        "{}",
        in_process["info"]
    );
    assert_eq!(network["info"]["x-ess-reached-by"], "network");
}

#[test]
fn a_command_line_component_names_its_command_line() {
    let command_line = document(&MODEL.replace(
        "    reached_by: network\n",
        "    reached_by: command_line\n    cli: {binary: items, commands: [catalog.items.ListItem, catalog.items.TouchItem], views: [catalog.items.ItemById]}\n",
    ));
    assert_eq!(
        first_paragraph(&command_line),
        "The HTTP surface of `items-service`, one of the components of `catalog` v1. It is not \
         declared `reached_by: network`: its callers reach it through its command line, so these \
         paths describe its commands and no request reaches them."
    );
    assert_eq!(
        command_paths(&command_line),
        ["/items/commands/ListItem", "/items/commands/TouchItem"]
    );
    assert!(command_line["info"].get("x-ess-reached-by").is_none());
}
