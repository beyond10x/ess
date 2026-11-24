//! `presence: omitted_or_null` (beyond10x/ess#678, the wire half): a member that may be left out or
//! sent as `null`, both read as one absent value. Projected as the nullable, not required property;
//! never carried into a suite, so a runner admits both spellings as it does for a field without a
//! policy.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::direct_response::Observation;
use ess_conformance::{bind, Completeness, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::artifact::run;
use ess_gen::schema::JsonSchema;
use ess_primitives::node::Node;
use serde_json::json;
use std::collections::BTreeMap;

const MODEL: &str = "format: ess/31
system: demo
version: v1
domain: demo.notes
types:
  - name: demo.notes.Edit
    kind: struct
    fields:
      - {name: reason, type: Optional<String>, presence: omitted_or_null}
commands:
  - name: demo.notes.Revise
    input:
      - {name: edit, type: demo.notes.Edit}
    response:
      - {name: edit, type: demo.notes.Edit}
    outcomes:
      - name: revised
        returns: true
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn schema(model: &EssIr, path: &str) -> serde_json::Value {
    let artifacts = run(&JsonSchema, model).expect("generates");
    let artifact = artifacts.get(path).unwrap_or_else(|| {
        panic!(
            "no `{path}` among {:?}",
            artifacts.keys().collect::<Vec<_>>()
        )
    });
    serde_json::from_str(&artifact.contents).expect("JSON")
}

fn edit(reason: Option<Node>) -> BTreeMap<String, Node> {
    let mut edit = BTreeMap::new();
    if let Some(reason) = reason {
        edit.insert("reason".to_owned(), reason);
    }
    BTreeMap::from([("edit".to_owned(), Node::Map(edit))])
}

#[test]
fn a_struct_field_admits_missing_and_null() {
    let model = ir(MODEL);
    for path in [
        "schema/commands/demo.notes.Revise.schema.json",
        "schema/responses/demo.notes.Revise.schema.json",
    ] {
        let schema = schema(&model, path);
        let declared = &schema["$defs"]["demo.notes.Edit"];
        assert_eq!(
            declared["properties"]["reason"],
            json!({"anyOf": [{"type": "string"}, {"type": "null"}]}),
            "{path}: {schema:#}"
        );
        assert!(
            declared["required"]
                .as_array()
                .is_none_or(|required| !required.contains(&json!("reason"))),
            "{path}: {schema:#}"
        );
        let validator = jsonschema::validator_for(&schema).expect("a valid schema");
        for member in [json!({}), json!({"reason": null}), json!({"reason": "x"})] {
            let instance = json!({ "edit": member });
            assert!(
                validator.is_valid(&instance),
                "{path}: {instance} against {schema:#}"
            );
        }
    }

    let command = model
        .commands()
        .get(&"demo.notes.Revise".parse().unwrap())
        .expect("the command");
    let observation =
        Observation::of(&model, command, None, BTreeMap::new()).expect("an observation");
    let observed = serde_json::to_string(&observation).expect("serialises");
    assert!(!observed.contains("presence"), "{observed}");
    for (which, payload) in [
        ("missing", edit(None)),
        ("null", edit(Some(Node::Null))),
        ("a value", edit(Some(Node::Text("x".to_owned())))),
    ] {
        bind(&model, &command.input, &payload, Completeness::Total)
            .unwrap_or_else(|errors| panic!("the input with the member {which}: {errors:?}"));
        observation
            .compare(Some(&payload))
            .unwrap_or_else(|error| panic!("the response with the member {which}: {error}"));
    }
}

/// A synthesized suite never carries the value: its leaves read as a field without a policy, and
/// the suite stays off the presence formats.
#[test]
fn a_synthesized_suite_carries_no_omitted_or_null() {
    let text = "format: ess/31
system: demo
version: v1
domain: demo.notes
events:
  - name: demo.notes.Edited
    fields:
      - {name: reason, type: Optional<String>, presence: omitted_or_null}
actors:
  - {name: demo.notes.Clerk, may: [demo.notes.Edit]}
commands:
  - name: demo.notes.Edit
    input:
      - {name: reason, type: Optional<String>, presence: omitted_or_null}
    outcomes:
      - name: edited
        emits: [demo.notes.Edited]
        payload:
          demo.notes.Edited: {reason: input.reason}
";
    let result = ess_conformance::synthesize::synthesize(&ir(text));
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let leaves: Vec<_> = result
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .filter_map(|step| match step {
            ScenarioStep::ExpectEvent { shape, .. } => Some(shape),
            _ => None,
        })
        .filter_map(|shape| shape.leaves().get("reason").cloned())
        .collect();
    assert!(!leaves.is_empty(), "the suite expects the event");
    assert!(leaves.iter().all(|leaf| leaf.presence.is_none()));
    assert!(!ess_conformance::presence::used_by(&result.suite));
    let bytes = result.suite.to_canonical_json().unwrap();
    assert!(!bytes.contains("omitted_or_null"), "{bytes}");
}

/// A view carrying its source fields' policies (beyond10x/ess#677): the synthesized suite passes
/// against the interpreted target, which leaves an absent `omitted_when_absent` member out.
#[test]
fn a_view_carrying_its_source_presence_passes_its_own_suite() {
    let text = "format: ess/31
system: demo
version: v1
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: String}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: embed_token, type: Optional<String>, presence: null_when_absent}
      - {name: reason, type: Optional<String>, presence: omitted_or_null}
      - {name: hint, type: Optional<String>, presence: omitted_when_absent}
      - {name: note, type: Optional<String>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.notes.Opened
    fields:
      - {name: note_id, type: demo.notes.NoteId}
actors:
  - {name: demo.notes.Clerk, may: [demo.notes.Open]}
commands:
  - name: demo.notes.Open
    input:
      - {name: embed_token, type: Optional<String>}
      - {name: reason, type: Optional<String>}
      - {name: hint, type: Optional<String>}
      - {name: note, type: Optional<String>}
    outcomes:
      - name: opened
        creates: demo.notes.Note
        instance: note_id
        emits: [demo.notes.Opened]
        payload:
          demo.notes.Opened: {note_id: {generated: true}}
        sets:
          embed_token: input.embed_token
          reason: input.reason
          hint: input.hint
          note: input.note
views:
  - name: demo.notes.Notes
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: embed_token, type: Optional<String>}
      - {name: reason, type: Optional<String>}
      - {name: hint, type: Optional<String>}
      - {name: note, type: Optional<String>}
";
    let model = ir(text);
    let view = model
        .views()
        .get(&"demo.notes.Notes".parse().unwrap())
        .expect("the view");
    assert_eq!(
        view.field("hint").expect("a view field").naming.presence,
        Some(ess_domain::types::Presence::OmittedWhenAbsent)
    );
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let admitted = ess_conformance::AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = ess_conformance::runner::Runner::for_suite(&synthesis.suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model.clone()),
        )
        .into_report();
    assert_eq!(
        report.status,
        ess_conformance::ConformanceStatus::Passed,
        "{report:#?}"
    );
}

/// A view member an outcome clears (beyond10x/ess#799): `AddBin/added` sets `note: {cleared: true}`
/// and `Bins` publishes `note`. `{FORMAT}` and `{PRESENCE}` are filled per variant.
const BINS: &str = "format: {FORMAT}
system: demo
version: v1
domain: demo.bins
types:
  - {name: demo.bins.BinId, kind: newtype, of: String}
entities:
  - name: demo.bins.Bin
    identity: {name: bin_id, type: demo.bins.BinId}
    fields:
      - {name: label, type: String}
      - {name: note, type: Optional<String>{PRESENCE}}
    lifecycle:
      initial: Stocked
      states: [Stocked, Noted]
      terminal: [Noted]
      transitions:
        - {name: note, from: [Stocked], to: Noted}
errors:
  - name: demo.bins.Duplicate
  - name: demo.bins.UnknownBin
  - name: demo.bins.AlreadyNoted
commands:
  - name: demo.bins.AddBin
    input:
      - {name: bin_id, type: demo.bins.BinId}
      - {name: label, type: String}
    outcomes:
      - {name: duplicate, existing_instance: true, error: demo.bins.Duplicate}
      - name: added
        creates: demo.bins.Bin
        instance: bin_id
        sets:
          label: input.label
          note: {cleared: true}
        emits: [demo.bins.BinAdded]
        payload:
          demo.bins.BinAdded: {bin_id: input.bin_id}
  - name: demo.bins.NoteBin
    input:
      - {name: bin_id, type: demo.bins.BinId}
      - {name: note, type: String}
    outcomes:
      - {name: unknown-bin, unknown_instance: true, error: demo.bins.UnknownBin}
      - name: noted
        moves: demo.bins.Bin.note
        instance: bin_id
        sets: {note: input.note}
        emits: [demo.bins.BinNoted]
        payload:
          demo.bins.BinNoted: {bin_id: input.bin_id}
      - {name: already-noted, wrong_state: true, error: demo.bins.AlreadyNoted}
events:
  - name: demo.bins.BinAdded
    fields:
      - {name: bin_id, type: demo.bins.BinId}
  - name: demo.bins.BinNoted
    fields:
      - {name: bin_id, type: demo.bins.BinId}
views:
  - name: demo.bins.Bins
    source: demo.bins.Bin
    consistency: read_your_writes
    fields:
      - {name: bin_id, type: demo.bins.BinId}
      - {name: label, type: String}
      - {name: note, type: Optional<String>}
      - {name: state, type: demo.bins.Bin.State}
components:
  - component: bins-service
    reached_by: network
    owns:
      domains: [demo.bins]
    accepts:
      commands: [demo.bins.AddBin, demo.bins.NoteBin]
    publishes:
      events: [demo.bins.BinAdded, demo.bins.BinNoted]
";

const ADDED: &str = "demo.bins.AddBin/outcome/added";

/// The model of one variant: its format and the `presence:` of the entity field `note`.
fn bins(format: &str, presence: Option<&str>) -> String {
    BINS.replace("{FORMAT}", format).replace(
        "{PRESENCE}",
        &presence.map_or_else(String::new, |presence| format!(", presence: {presence}")),
    )
}

/// The value the `Bins` read of `AddBin/added` claims for `note`, if it claims one.
fn note_claim(suite: &ess_conformance::ConformanceSuite) -> Option<ess_conformance::ScenarioValue> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == ADDED)
        .map_or_else(|| panic!("no scenario {ADDED}"), |(_, scenario)| scenario);
    let rows: Vec<_> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                view,
                expectation: ess_conformance::ViewExpectation::Contains { fields },
            } if view.to_string() == "demo.bins.Bins" => Some(fields),
            _ => None,
        })
        .collect();
    assert_eq!(rows.len(), 1, "{scenario:#?}");
    rows[0].get("note").cloned()
}

fn failed_on_interpreted(model: &EssIr, suite: &ess_conformance::ConformanceSuite) -> Vec<String> {
    let admitted = ess_conformance::AdmittedSuite::from_suite(suite).unwrap();
    ess_conformance::runner::Runner::for_suite(admitted.suite())
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model.clone()),
        )
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != ess_conformance::report::Status::Passed)
        .map(|result| result.scenario.to_string())
        .collect()
}

/// The row the interpreted target answers for `Bins` after `AddBin` created one bin.
fn added_row(model: &EssIr) -> ess_conformance::target::ViewRow {
    use ess_conformance::target::{
        ConformanceTarget, Deadline, SemanticCommandRequest, SemanticViewRequest,
    };
    use ess_primitives::{consistency::QueryConsistency, ids::CorrelationId, time::Timestamp};
    let target = ess_conformance::interpret::Interpreted::for_model(model.clone());
    target
        .execute_command(SemanticCommandRequest {
            command: "demo.bins.AddBin".parse().unwrap(),
            actor: None,
            caller: None,
            input: BTreeMap::from([
                ("bin_id".to_owned(), Node::Text("bin-1".to_owned())),
                ("label".to_owned(), Node::Text("shelf".to_owned())),
            ]),
            correlation: CorrelationId::new("probe").unwrap(),
        })
        .unwrap();
    let mut rows = target
        .query_view(SemanticViewRequest {
            view: "demo.bins.Bins".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: CorrelationId::new("probe").unwrap(),
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows;
    assert_eq!(rows.len(), 1, "{rows:#?}");
    rows.remove(0)
}

/// Each variant: whether the suite claims `null` for the cleared member, and whether the
/// interpreted target writes it as `null` (`true`) or leaves it out (`false`). Below `ess/31` a view
/// field carries no presence, so the rule for none applies.
const VARIANTS: [(&str, Option<&str>, bool, bool); 5] = [
    ("ess/37", None, false, false),
    ("ess/37", Some("omitted_when_absent"), false, false),
    ("ess/37", Some("null_when_absent"), true, true),
    ("ess/37", Some("omitted_or_null"), false, true),
    ("ess/30", None, false, false),
];

#[test]
fn a_cleared_view_member_follows_its_presence_in_suite_and_interpreter() {
    for (format, presence, claimed, written) in VARIANTS {
        let which = format!("{format}, presence {presence:?}");
        let model = ir(&bins(format, presence));
        let synthesis = ess_conformance::synthesize::synthesize(&model);
        assert!(
            synthesis.refusals.is_empty(),
            "{which}: {:#?}",
            synthesis.refusals
        );
        let claim = note_claim(&synthesis.suite);
        if claimed {
            assert_eq!(
                claim,
                Some(ess_conformance::ScenarioValue::Literal { value: Node::Null }),
                "{which}"
            );
        } else {
            assert_eq!(claim, None, "{which}");
        }
        assert_eq!(
            failed_on_interpreted(&model, &synthesis.suite),
            Vec::<String>::new(),
            "{which}"
        );
        let row = added_row(&model);
        if written {
            assert_eq!(row.get("note"), Some(&Node::Null), "{which}: {row:#?}");
        } else {
            assert!(!row.contains_key("note"), "{which}: {row:#?}");
        }
    }
}

#[test]
fn interpreted_view_rows_validate_against_their_row_schema() {
    for (format, presence, _, _) in VARIANTS {
        let which = format!("{format}, presence {presence:?}");
        let model = ir(&bins(format, presence));
        let component = model
            .components()
            .values()
            .next()
            .unwrap_or_else(|| panic!("{which}: no component"));
        let document: serde_json::Value =
            serde_json::from_str(&ess_gen::openapi::json(&model, component)).expect("JSON");
        let schema = json!({
            "$ref": "#/components/schemas/demo.bins.Bins.Row",
            "components": document["components"],
        });
        let validator = jsonschema::validator_for(&schema).expect("a valid schema");
        let row = serde_json::to_value(added_row(&model)).expect("serialises");
        assert!(
            validator.is_valid(&row),
            "{which}: {row} against {:#}",
            document["components"]["schemas"]["demo.bins.Bins.Row"]
        );
    }
}

/// A suite an earlier release wrote for the member with no presence claims `null` for it; the
/// interpreted target, which now leaves the member out, fails that claim.
#[test]
fn an_old_null_claim_for_a_member_with_no_presence_now_fails() {
    let model = ir(&bins("ess/37", None));
    let mut suite = ess_conformance::synthesize::synthesize(&model).suite;
    let scenario = suite
        .scenarios
        .iter_mut()
        .find(|(id, _)| id.to_string() == ADDED)
        .map_or_else(|| panic!("no scenario {ADDED}"), |(_, scenario)| scenario);
    let mut claimed = 0;
    for step in &mut scenario.steps {
        if let ScenarioStep::ExpectView {
            view,
            expectation: ess_conformance::ViewExpectation::Contains { fields },
        } = step
        {
            if view.to_string() == "demo.bins.Bins" {
                fields.insert(
                    "note".to_owned(),
                    ess_conformance::ScenarioValue::Literal { value: Node::Null },
                );
                claimed += 1;
            }
        }
    }
    assert_eq!(claimed, 1);
    assert_eq!(
        failed_on_interpreted(&model, &suite),
        vec![ADDED.to_owned()]
    );
}
