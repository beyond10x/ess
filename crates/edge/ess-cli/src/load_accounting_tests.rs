use super::LoadedSpec;
use serde_json::Value;
use std::fs;

fn load(text: &str) -> LoadedSpec {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("system.yaml");
    fs::write(&path, text).unwrap();
    super::specification(&path).unwrap()
}

fn compiled(text: &str) -> (usize, Value) {
    match load(text) {
        LoadedSpec::Compiled { ir, files_read } => (files_read, serde_json::to_value(ir).unwrap()),
        LoadedSpec::Refused {
            problems,
            diagnostics,
            ..
        } => panic!(
            "expected compiled specification: {problems:?} {:?}",
            diagnostics.as_slice()
        ),
    }
}

fn refused(text: &str) -> (usize, String) {
    match load(text) {
        LoadedSpec::Refused {
            files_read,
            problems,
            diagnostics,
        } => (
            files_read,
            format!("{problems:?} {:?}", diagnostics.as_slice()),
        ),
        LoadedSpec::Compiled { .. } => panic!("expected specification refusal"),
    }
}

const READING: &str = "format: ess/3\nsystem: chronology\nversion: v1\ndomain: chronology.reading\ntypes:\n  - name: chronology.reading.OffsetText\n    kind: newtype\n    of: String\n    reading:\n      encoding: offset_date_time_text\n      origins: [{role: producer_process, offset: encoded_offset}]\n  - name: chronology.reading.LocalText\n    kind: newtype\n    of: String\n    reading:\n      encoding: local_date_time_millis_literal_z\n      origins: [{role: producer_process, offset: requires_observation}, {role: consumer_process, offset: requires_observation}]\n  - name: chronology.reading.EpochSeconds\n    kind: newtype\n    of: Integer\n    reading:\n      encoding: unix_seconds\n      origins: [{role: producer_process, offset: encoding_defined_epoch}]\n  - name: chronology.reading.Record\n    kind: struct\n    fields: [{name: value, type: String}]\n  - name: chronology.reading.Kind\n    kind: enum\n    variants: [One]\n  - name: chronology.reading.Choice\n    kind: union\n    tag: kind\n    variants: {one: chronology.reading.Record}\n";

#[test]
fn specification_direct_file_preserves_reading_contracts_and_refuses_invalid_owners() {
    let (files_read, ir) = compiled(READING);
    assert_eq!(files_read, 1);
    assert_eq!(
        ir["types"]["chronology.reading.OffsetText"]["reading"]["encoding"],
        "offset_date_time_text"
    );
    assert_eq!(
        ir["types"]["chronology.reading.LocalText"]["reading"]["origins"][1]["role"],
        "consumer_process"
    );
    assert_eq!(
        ir["types"]["chronology.reading.EpochSeconds"]["reading"]["origins"][0]["offset"],
        "encoding_defined_epoch"
    );

    for owner in ["Record", "Kind", "Choice"] {
        let marker = format!("  - name: chronology.reading.{owner}\n");
        let invalid = READING.replace(
            &marker,
            &format!("{marker}    reading: {{encoding: unix_seconds, origins: [{{role: producer_process, offset: encoding_defined_epoch}}]}}\n"),
        );
        let (files_read, refusal) = refused(&invalid);
        assert_eq!(files_read, 1);
        assert!(refusal.contains("reading"), "{owner}: {refusal}");
    }
    let invalid = READING.replace("encoded_offset", "requires_observation");
    let (files_read, refusal) = refused(&invalid);
    assert_eq!(files_read, 1);
    assert!(refusal.contains("offset"), "{refusal}");
}

const SETTINGS: &str = "format: ess/1\nsystem: connectors\nversion: v1\ndomain: connectors.config\ntypes:\n  - name: connectors.config.StateRoot\n    kind: newtype\n    of: String\nentities:\n  - name: connectors.config.Workspace\n    identity: {name: workspace_id, type: Uuid}\n    fields: []\n    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}\ncomponents:\n  - component: connectors-cli\n    owns: {domains: [connectors.config]}\n    settings:\n      - {name: state-root, type: connectors.config.StateRoot, required: true, summary: Where connector state is kept.}\n      - {name: api-token, type: String, required: true, secret: true}\n      - {name: retry-window, type: 'Optional<connectors.config.StateRoot>', required: false, value: public}\n";

#[test]
fn specification_direct_file_preserves_component_settings_and_named_refusals() {
    let (files_read, ir) = compiled(SETTINGS);
    assert_eq!(files_read, 1);
    let settings = ir["components"]["connectors-cli"]["settings"]
        .as_array()
        .unwrap();
    assert_eq!(settings.len(), 3);
    assert_eq!(settings[0]["name"], "state-root");
    assert_eq!(settings[0]["type"]["name"], "connectors.config.StateRoot");
    assert_eq!(settings[0]["required"], true);
    assert_eq!(settings[0]["summary"], "Where connector state is kept.");
    assert_eq!(settings[1]["secret"], true);
    assert_eq!(settings[2]["value"], "public");
    assert_eq!(settings[2]["required"], false);

    let without = SETTINGS.replace("    settings:\n      - {name: state-root, type: connectors.config.StateRoot, required: true, summary: Where connector state is kept.}\n      - {name: api-token, type: String, required: true, secret: true}\n      - {name: retry-window, type: 'Optional<connectors.config.StateRoot>', required: false, value: public}\n", "");
    let (files_read, omitted) = compiled(&without);
    assert_eq!(files_read, 1);
    assert!(omitted["components"]["connectors-cli"]
        .get("settings")
        .is_none());

    for (invalid, expected) in [
        (SETTINGS.replace("api-token", "state-root"), "duplicate"),
        (
            SETTINGS.replace(
                "type: connectors.config.StateRoot, required: true",
                "type: connectors.config.Workspace, required: true",
            ),
            "type",
        ),
        (
            SETTINGS.replace(
                "required: true, secret: true",
                "required: false, secret: true, value: fixed",
            ),
            "required",
        ),
    ] {
        let (files_read, refusal) = refused(&invalid);
        assert_eq!(files_read, 1);
        assert!(refusal.contains(expected), "{refusal}");
    }
}

const SUBJECT_STATE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/subject-state.yaml");

#[test]
fn specification_direct_file_preserves_subject_state_selection() {
    let (files_read, ir) = compiled(SUBJECT_STATE);
    assert_eq!(files_read, 1);
    let outcomes = ir["commands"]["calls.core.Report"]["outcomes"]
        .as_array()
        .unwrap();
    assert_eq!(outcomes.len(), 4);
    for (index, state) in [(0, "Init"), (1, "Ringing"), (2, "Bridged")] {
        assert_eq!(outcomes[index]["condition"]["kind"], "subject_state");
        assert_eq!(outcomes[index]["condition"]["state"], state);
        assert_eq!(outcomes[index]["test_strategy"], "construct_input_in_state");
        assert_eq!(
            outcomes[index]["condition"]["predicate"],
            "incoming == Ringing"
        );
    }
    let invalid = SUBJECT_STATE.replace("when_subject_state: Init", "when_subject_state: Missing");
    let (files_read, refusal) = refused(&invalid);
    assert_eq!(files_read, 1);
    assert!(refusal.contains("state"), "{refusal}");
    let invalid = SUBJECT_STATE.replace(
        "when_subject_state: Init\n        when:",
        "when_subject_state: Init\n        external: provider\n        when:",
    );
    let (files_read, refusal) = refused(&invalid);
    assert_eq!(files_read, 1);
    assert!(
        refusal.contains("external") || refusal.contains("condition"),
        "{refusal}"
    );
}

const RESPONSE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/response-payload.yaml");

#[test]
fn specification_direct_file_preserves_response_payload_ownership() {
    let (files_read, ir) = compiled(RESPONSE);
    assert_eq!(files_read, 1);
    let command = &ir["commands"]["demo.api.Cancel"];
    assert_eq!(command["response"][0]["name"], "item");
    assert_eq!(command["response"][0]["type_ref"]["name"], "demo.api.Item");
    let fields = command["outcomes"][0]["payload"][0]["fields"]
        .as_array()
        .unwrap();
    assert_eq!(fields[0]["value"]["kind"], "response_field");
    assert_eq!(fields[0]["value"]["field"], "item");
    assert_eq!(fields[1]["value"]["kind"], "generated");

    let missing = RESPONSE.replace(
        "    response:\n      - {name: item, type: demo.api.Item}\n",
        "",
    );
    let (files_read, refusal) = refused(&missing);
    assert_eq!(files_read, 1);
    assert!(
        refusal.contains("response") && refusal.contains("item"),
        "{refusal}"
    );
    let subject_response = RESPONSE
        .replace(
            "events:\n",
            "entities:\n  - name: demo.api.Order\n    identity: {name: order_id, type: Uuid}\n    fields:\n      - {name: remaining, type: Integer}\n      - {name: created, type: Timestamp}\n    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}\nevents:\n",
        )
        .replace(
            "  - name: demo.api.Cancel\n    response:",
            "  - name: demo.api.Cancel\n    input: [{name: order_id, type: Uuid}]\n    response:",
        )
        .replace(
            "      - name: cancelled\n        emits:",
            "      - name: cancelled\n        updates: demo.api.Order\n        instance: order_id\n        emits:",
        );
    let conflict = subject_response.replace(
        "        payload:\n          demo.api.Returned:\n            item: {response: item}\n            receipt: {generated: true}",
        "        sets:\n          remaining: {response: item}\n          created: {generated: true}",
    );
    let (files_read, refusal) = refused(&conflict);
    assert_eq!(files_read, 1);
    assert!(
        refusal.contains("conflicting")
            || (refusal.contains("response") && refusal.contains("sets")),
        "{refusal}"
    );
}

const ERRORS: &str = "format: ess/4\nsystem: desk\nversion: v1\ndomain: desk.api\nerrors:\n  - name: desk.api.Invalid\n    summary: Invalid request.\n    naming: {wire: bad_request, display: Bad request}\n";

#[test]
fn specification_direct_file_preserves_error_naming() {
    let (files_read, ir) = compiled(ERRORS);
    assert_eq!(files_read, 1);
    let error = &ir["errors"]["desk.api.Invalid"];
    assert_eq!(error["name"], "desk.api.Invalid");
    assert_eq!(error["summary"], "Invalid request.");
    assert_eq!(error["naming"]["wire"], "bad_request");
    assert_eq!(error["naming"]["display"], "Bad request");

    let duplicate_wire =
        format!("{ERRORS}  - name: desk.api.Other\n    naming: {{wire: bad_request}}\n");
    let (files_read, ir) = compiled(&duplicate_wire);
    assert_eq!(files_read, 1);
    assert_eq!(
        ir["errors"]["desk.api.Other"]["naming"]["wire"],
        "bad_request"
    );
    assert_ne!(
        ir["errors"]["desk.api.Invalid"]["name"],
        ir["errors"]["desk.api.Other"]["name"]
    );
    let old_format = ERRORS.replace("ess/4", "ess/3");
    let (files_read, refusal) = refused(&old_format);
    assert_eq!(files_read, 1);
    assert!(refusal.contains("format ess/4"), "{refusal}");
    let legacy = ERRORS.replace(
        "    naming: {wire: bad_request, display: Bad request}\n",
        "",
    );
    let (files_read, ir) = compiled(&legacy);
    assert_eq!(files_read, 1);
    assert!(ir["errors"]["desk.api.Invalid"].get("naming").is_none());
}

const PERIODIC: &str = include_str!("../../../specify/ess-domain/tests/fixtures/periodic.yaml");

#[test]
fn specification_direct_file_preserves_periodic_host_mapping_and_delivery() {
    let (files_read, ir, hosted_by, reacts_to) = match load(PERIODIC) {
        LoadedSpec::Compiled { ir, files_read } => {
            let graph = ess_compiler::graph::SemanticDependencyGraph::of(&ir);
            let hosted_by = graph
                .edges()
                .filter(|edge| edge.relation == ess_compiler::graph::DependencyRelation::HostedBy)
                .map(ToString::to_string)
                .collect::<Vec<_>>();
            let reacts_to = graph
                .edges()
                .any(|edge| edge.relation == ess_compiler::graph::DependencyRelation::ReactsTo);
            (
                files_read,
                serde_json::to_value(ir).unwrap(),
                hosted_by,
                reacts_to,
            )
        }
        LoadedSpec::Refused { .. } => panic!("periodic control must compile"),
    };
    assert_eq!(files_read, 1);
    assert_eq!(
        hosted_by,
        ["binding poll-status is hosted by component poll-service"]
    );
    assert!(!reacts_to);
    let binding = &ir["bindings"]["poll-status"];
    assert_eq!(binding["periodic"]["every"], "PT2S");
    assert_eq!(
        binding["periodic"]["host"]["authority"],
        "authenticated-session-status"
    );
    assert_eq!(binding["periodic"]["host"]["owner"], "poll-service");
    assert_eq!(binding["mapping"][0]["value"]["kind"], "host_context");
    assert_eq!(binding["mapping"][0]["value"]["field"], "agent_id");
    assert_eq!(binding["mapping"][1]["value"]["kind"], "host_read");
    assert_eq!(binding["mapping"][1]["value"]["field"], "status");
    assert_eq!(binding["delivery"], "at_most_once");

    let invalid = PERIODIC.replace(
        "agent_id: host_context.agent_id",
        "agent_id: host_context.missing",
    );
    let (files_read, refusal) = refused(&invalid);
    assert_eq!(files_read, 1);
    assert!(refusal.contains("missing"), "{refusal}");
    let invalid = PERIODIC.replace(
        "when:\n      periodic:",
        "when:\n      event: example.poll.Updated\n      periodic:",
    );
    let (files_read, refusal) = refused(&invalid);
    assert_eq!(files_read, 1);
    assert!(
        refusal.contains("event") || refusal.contains("periodic"),
        "{refusal}"
    );
    let invalid = PERIODIC.replace("delivery: at_most_once", "delivery: exactly_once");
    let (files_read, refusal) = refused(&invalid);
    assert_eq!(files_read, 1);
    assert!(
        refusal.contains("delivery") || refusal.contains("exactly_once"),
        "{refusal}"
    );
}

const SELECTION: &str =
    include_str!("../../../specify/ess-domain/tests/fixtures/binding-selection.yaml");
const ACCESSOR: &str = include_str!("../tests/fixtures/bounded-accessor.yaml");

#[test]
fn specification_direct_file_preserves_selection_and_event_accessors() {
    let (files_read, ir) = compiled(SELECTION);
    assert_eq!(files_read, 1);
    let binding = &ir["bindings"]["choose"];
    assert_eq!(binding["event"], "selection.core.Arrived");
    assert_eq!(binding["selection"]["plan"]["inputs"][0]["name"], "legs");
    assert_eq!(
        binding["selection"]["plan"]["selectors"]
            .as_array()
            .unwrap()
            .len(),
        5
    );
    assert_eq!(
        binding["selection"]["plan"]["selectors"]
            .as_array()
            .unwrap()
            .iter()
            .map(|selector| selector["name"].as_str().unwrap())
            .collect::<Vec<_>>(),
        [
            "first_agent",
            "first_external",
            "first_identified",
            "agent",
            "external"
        ]
    );
    assert_eq!(binding["mapping"][0]["value"]["kind"], "selection");
    assert_eq!(binding["mapping"][0]["value"]["selector"], 3);
    assert_eq!(
        binding["mapping"][0]["value"]["projection"]["segments"],
        serde_json::json!(["item", "id"])
    );
    assert_eq!(
        binding["mapping"][0]["value"]["projection"]["root"]["name"],
        "item"
    );

    let (files_read, ir) = compiled(ACCESSOR);
    assert_eq!(files_read, 1);
    let value = &ir["bindings"]["project"]["mapping"][0]["value"];
    assert_eq!(value["kind"], "event_accessor");
    assert_eq!(
        value["plan"]["segments"],
        serde_json::json!(["data", "status"])
    );
    assert_eq!(value["plan"]["root"]["name"], "data");
    assert_eq!(value["types"]["accessor.core.Body"], "accessor.core.Body");

    let invalid = SELECTION.replace("selection: agent", "selection: missing");
    let (files_read, refusal) = refused(&invalid);
    assert_eq!(files_read, 1);
    assert!(refusal.contains("missing"), "{refusal}");
    let invalid = SELECTION.replace("excluding: [first_agent]", "excluding: [first_external]");
    let (files_read, refusal) = refused(&invalid);
    assert_eq!(files_read, 1);
    assert!(refusal.contains("must be declared earlier"), "{refusal}");
    let invalid = ACCESSOR.replace("event.data.status", "event.data.missing");
    let (files_read, refusal) = refused(&invalid);
    assert_eq!(files_read, 1);
    assert!(refusal.contains("missing"), "{refusal}");
}
