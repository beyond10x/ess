//! The `OpenAPI` document says an event reaches consumers only when the component declares it under
//! `publishes:`.
//!
//! A branch may emit an event its component keeps to itself: an audit or bookkeeping record that
//! no consumer receives. The document used to call every emitted event "published to consumers",
//! so a client author could subscribe to an event that never arrives, and the component's
//! `AsyncAPI` document, which reads `publishes`, disagreed with it about the same event.
//!
//! What the generated servers write under `published` is a different question, and both answer it
//! the same way: every event the branch emits, in emit order, whether the component publishes it
//! or not (`ess-synth`'s `rust/wire.rs` `outcome_encoder` and `go/http.rs` `published_list`, both
//! through `items::outcome_event_fields`, which walks `outcome.emits` unfiltered). The property's
//! description says so.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

/// One component publishing `Opened` only. `ops.core.Open`'s `opened` branch emits `Opened` and
/// then `Audited`; `ops.core.Audit`'s `audited` branch emits `Audited` alone.
const MODEL: &str = "format: ess/15
system: ops
version: v1
domain: ops.core
events:
  - name: ops.core.Opened
    fields:
      - {name: id, type: String}
  - name: ops.core.Audited
    fields:
      - {name: id, type: String}
commands:
  - name: ops.core.Open
    input:
      - {name: id, type: String}
    outcomes:
      - name: opened
        emits: [ops.core.Opened, ops.core.Audited]
        payload:
          ops.core.Opened: {id: input.id}
          ops.core.Audited: {id: input.id}
  - name: ops.core.Audit
    input:
      - {name: id, type: String}
    outcomes:
      - name: audited
        emits: [ops.core.Audited]
        payload:
          ops.core.Audited: {id: input.id}
components:
  - component: ops-service
    owns: {domains: [ops.core]}
    accepts: {commands: [ops.core.Open, ops.core.Audit]}
    publishes: {events: [ops.core.Opened]}
    reached_by: network
";

const PUBLISHES: &str = "    publishes: {events: [ops.core.Opened]}\n";

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("spec.yaml"),
        RawSpecFile::parse(text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The one component's document, as JSON.
fn document(text: &str) -> Value {
    let ir = ir(text);
    let component = ir.components().values().next().expect("one component");
    serde_json::from_str(&ess_gen::openapi::json(&ir, component)).expect("JSON")
}

/// The outcome's own text: the description of its response body.
fn outcome_text<'a>(document: &'a Value, outcome: &str) -> &'a str {
    document["components"]["schemas"][format!("{outcome}.Response")]["description"]
        .as_str()
        .unwrap_or_else(|| panic!("`{outcome}.Response` has a description"))
}

/// Every response description of the operation serving `command`.
fn status_descriptions<'a>(document: &'a Value, command: &str) -> Vec<&'a str> {
    let operation = document["paths"]
        .as_object()
        .expect("paths")
        .values()
        .filter_map(|item| item.get("post"))
        .find(|operation| operation["operationId"] == command)
        .unwrap_or_else(|| panic!("an operation for `{command}`"));
    operation["responses"]
        .as_object()
        .expect("responses")
        .values()
        .map(|response| response["description"].as_str().expect("a description"))
        .collect()
}

fn info_description(document: &Value) -> &str {
    document["info"]["description"]
        .as_str()
        .expect("a description")
}

#[test]
fn an_outcome_text_says_published_only_for_published_events() {
    let document = document(MODEL);
    let text = outcome_text(&document, "ops.core.Open.opened");
    assert!(
        text.ends_with(
            "Emits `Opened` and `Audited`, listed under `published`. `Opened` is published to \
             consumers; `Audited` is not published and stays inside the component."
        ),
        "{text}"
    );
    let statuses = status_descriptions(&document, "ops.core.Open");
    assert!(
        statuses.iter().any(|description| description.ends_with(
            "Events this branch emits are listed under `published`: `Opened` is published to \
             consumers, and `Audited` is not published and stays inside the component."
        )),
        "{statuses:#?}"
    );
}

#[test]
fn an_outcome_emitting_only_unpublished_events_says_nothing_reaches_consumers() {
    let document = document(MODEL);
    let text = outcome_text(&document, "ops.core.Audit.audited");
    assert!(
        text.ends_with(
            "Emits `Audited`, listed under `published`. `Audited` is not published and stays \
             inside the component."
        ),
        "{text}"
    );
    assert!(!text.contains("published to consumers"), "{text}");
    let statuses = status_descriptions(&document, "ops.core.Audit");
    for description in &statuses {
        assert!(
            !description.contains("published to consumers"),
            "{description}"
        );
    }
    assert!(
        statuses.iter().any(|description| description.ends_with(
            "Events this branch emits are listed under `published`: `Audited` is not published \
             and stays inside the component."
        )),
        "{statuses:#?}"
    );
}

#[test]
fn the_document_description_follows_publishes() {
    let publishing = document(MODEL);
    let description = info_description(&publishing);
    assert!(
        description.contains(
            "This component publishes `Opened` to consumers through the event transport; an event \
             a branch emits that is not among them stays inside the component. The `published` \
             property of every response body lists every event its branch emitted, in the order \
             it emits them, published or not."
        ),
        "{description}"
    );
    assert!(
        !description.contains("Events emitted by a branch are published to consumers"),
        "{description}"
    );

    let silent = document(&MODEL.replace(PUBLISHES, ""));
    let description = info_description(&silent);
    assert!(
        description.contains(
            "This component publishes no event to consumers: an event a branch emits stays inside \
             the component. The `published` property of every response body still lists every \
             event its branch emitted, in the order it emits them."
        ),
        "{description}"
    );
    assert!(
        !description.contains("published to consumers"),
        "{description}"
    );
}

#[test]
fn the_published_property_description_matches_what_the_runtime_lists() {
    let document = document(MODEL);
    let schemas = &document["components"]["schemas"];

    // The servers list every emitted event, so the schema's items are the branch's emits, the
    // unpublished one included, in emit order.
    let open = &schemas["ops.core.Open.opened.Response"]["properties"]["published"];
    let listed: Vec<&Value> = open["prefixItems"]
        .as_array()
        .expect("prefixItems")
        .iter()
        .map(|item| &item["properties"]["event"]["const"])
        .collect();
    assert_eq!(listed, ["ops.core.Opened", "ops.core.Audited"], "{open:#}");
    assert_eq!(
        open["description"],
        "Every event this branch emitted, in the order it emits them: its qualified name and its \
         payload. `ops.core.Audited` is listed too, although this component does not publish it \
         to consumers.",
        "{open:#}"
    );

    let audit = &schemas["ops.core.Audit.audited.Response"]["properties"]["published"];
    assert_eq!(
        audit["description"],
        "Every event this branch emitted, in the order it emits them: its qualified name and its \
         payload. `ops.core.Audited` is listed too, although this component does not publish it \
         to consumers.",
        "{audit:#}"
    );

    // A branch whose every event is published keeps the description it always had.
    let all = document_with_every_event_published();
    assert_eq!(
        all["components"]["schemas"]["ops.core.Open.opened.Response"]["properties"]["published"]
            ["description"],
        "Every event this branch published, in publication order: its qualified name and its \
         payload."
    );
}

fn document_with_every_event_published() -> Value {
    document(&MODEL.replace(
        PUBLISHES,
        "    publishes: {events: [ops.core.Opened, ops.core.Audited]}\n",
    ))
}
