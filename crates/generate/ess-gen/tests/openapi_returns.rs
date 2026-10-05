//! An outcome declaring `returns: true` answers its caller with the command's response
//! (beyond10x/ess#423) and is answered `200`, not `202` (beyond10x/ess#424), from `ess/22`.
//!
//! The model is the issues' own reproduction. Below `ess/22` the same model keeps the status and
//! the body it was published with, because a client of an `ess/17` to `ess/21` document was told
//! `202` and no `response` member.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::{json, Value};

const SYSTEM: &str = "format: ess/22
system: catalogue
version: v1

domains:
  - catalogue.search
";

const COMPONENTS: &str = "components:
  - component: catalogue-reader
    summary: Answers searches over the catalogue.
    owns:
      domains:
        - catalogue.search
    accepts:
      commands:
        - catalogue.search.FindTitles
    publishes:
      events:
        - catalogue.search.TitlesFound
    reached_by: network
";

const DOMAIN: &str = "domain: catalogue.search

naming:
  wire: search
  display: Search

entities:
  - name: catalogue.search.Title
    identity:
      name: title_id
      type: Uuid
    fields:
      - name: name
        type: String
    lifecycle:
      initial: Listed
      states: [Listed]
      terminal: [Listed]

events:
  - name: catalogue.search.TitlesFound
    fields:
      - name: count
        type: Integer

commands:
  - name: catalogue.search.FindTitles
    naming:
      wire: find-titles
      display: Find titles
    input:
      - name: text
        type: String
    response:
      - name: names
        type: List<String>
    outcomes:
      - name: found
        returns: true
        emits:
          - catalogue.search.TitlesFound
        payload:
          catalogue.search.TitlesFound:
            count: {generated: true}
        summary: The names of every title whose name contains the text.

actors:
  - name: catalogue.search.Reader
    may:
      - catalogue.search.FindTitles
";

const PATH: &str = "/search/commands/find-titles";

fn ir(format: &str) -> EssIr {
    let system = SYSTEM.replace("ess/22", format);
    let files = [
        ("system.yaml", system.as_str()),
        ("components.yaml", COMPONENTS),
        ("domains/search.yaml", DOMAIN),
    ];
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in files {
        sources.insert(label, text);
        parsed.push((
            Source::new(label),
            RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}")),
        ));
    }
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("{format} validates: {errors}"));
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("{format} compiles: {errors}"))
}

fn document(model: &EssIr) -> Value {
    let component = model.components().values().next().expect("one component");
    serde_json::from_str(&ess_gen::openapi::json(model, component)).expect("JSON")
}

/// Validates `answer` against the schema the document declares for `status` on the command.
fn validate(document: &Value, status: &str, answer: &Value) -> Result<(), String> {
    let at = &document["paths"][PATH]["post"]["responses"][status]["content"]["application/json"]
        ["schema"];
    if !at.is_object() {
        return Err(format!("the document declares no {status}"));
    }
    let mut schema = document.clone();
    for key in ["$ref", "oneOf"] {
        if let Some(value) = at.get(key) {
            schema[key] = value.clone();
        }
    }
    let validator = jsonschema::draft202012::new(&schema).expect("the schema compiles");
    let errors: Vec<String> = validator
        .iter_errors(answer)
        .map(|error| format!("{error} at {}", error.instance_path()))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn answer(response: Option<Value>) -> Value {
    let mut answer = json!({
        "outcome": "found",
        "published": [{"event": "catalogue.search.TitlesFound", "payload": {"count": 2}}],
    });
    if let Some(response) = response {
        answer["response"] = response;
    }
    answer
}

#[test]
fn a_returns_outcome_carries_the_command_response_in_its_success_body() {
    let document = document(&ir("ess/22"));
    let schemas = &document["components"]["schemas"];
    for (name, schema) in schemas.as_object().expect("components.schemas") {
        jsonschema::draft202012::meta::validate(schema)
            .unwrap_or_else(|error| panic!("{name} is not a valid 2020-12 schema: {error}"));
    }
    assert!(
        document["info"]["description"]
            .as_str()
            .expect("a description")
            .contains("answered 200"),
        "the document states the convention it applies"
    );
    let result = &schemas["catalogue.search.FindTitles.Result"];
    assert_eq!(
        result["properties"]["names"]["type"], "array",
        "the declared response is published: {result}"
    );
    let found = &schemas["catalogue.search.FindTitles.found.Response"];
    assert!(
        found["required"]
            .as_array()
            .expect("required")
            .contains(&json!("response")),
        "{found}"
    );
    assert_eq!(
        found["properties"]["response"]["$ref"],
        "#/components/schemas/catalogue.search.FindTitles.Result"
    );
    validate(
        &document,
        "200",
        &answer(Some(json!({"names": ["Dune", "Dune Messiah"]}))),
    )
    .expect("an answer carrying the response validates");
    assert!(
        validate(&document, "200", &answer(None)).is_err(),
        "an answer without the response does not validate"
    );
    assert!(
        validate(&document, "200", &answer(Some(json!({"names": [1]})))).is_err(),
        "a response of the wrong type does not validate"
    );
}

#[test]
fn a_returns_outcome_is_answered_200_and_not_202() {
    let model = ir("ess/22");
    let document = document(&model);
    let responses = document["paths"][PATH]["post"]["responses"]
        .as_object()
        .expect("responses");
    assert!(responses.contains_key("200"), "{responses:?}");
    assert!(!responses.contains_key("202"), "{responses:?}");
    let command = model.commands().values().next().expect("one command");
    let outcome = &command.outcomes[0];
    assert_eq!(
        ess_gen::http::outcome_status(&model, outcome),
        "200",
        "the server and the document read one status"
    );
}

#[test]
fn below_ess_22_a_returns_outcome_keeps_202_and_no_response_member() {
    for format in ["ess/17", "ess/21"] {
        let model = ir(format);
        let document = document(&model);
        let responses = document["paths"][PATH]["post"]["responses"]
            .as_object()
            .expect("responses");
        assert!(responses.contains_key("202"), "{format}: {responses:?}");
        assert!(!responses.contains_key("200"), "{format}: {responses:?}");
        let schemas = &document["components"]["schemas"];
        assert!(
            schemas.get("catalogue.search.FindTitles.Result").is_none(),
            "{format}"
        );
        assert!(
            schemas["catalogue.search.FindTitles.found.Response"]["properties"]
                .get("response")
                .is_none(),
            "{format}"
        );
        validate(&document, "202", &answer(None)).expect("the old answer still validates");
        assert!(
            !document["info"]["description"]
                .as_str()
                .expect("a description")
                .contains("answered 200"),
            "{format}"
        );
        let command = model.commands().values().next().expect("one command");
        assert_eq!(
            ess_gen::http::outcome_status(&model, &command.outcomes[0]),
            "202",
            "{format}"
        );
    }
}
