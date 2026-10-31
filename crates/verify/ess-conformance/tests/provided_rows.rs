//! A fixture may be a row the deployment already holds
//! (story:a-fixture-may-be-a-row-the-deployment-already-holds,
//! architecture-decision-record:a-deployment-may-provide-an-entitys-rows).
//!
//! An entity declaring `provided: true` (ess/33) is one whose rows the deployment holds. Where a
//! `when_related: {via: input.<field>}` guard reads a row of it and no outcome creates one, the
//! scenario resolves an entity fixture `{name, type, entity, state}` before execution and sends its
//! identity; the `exists: false` branch is sent a fresh identity. Ordinary arrangement keeps
//! precedence, every other position is refused by name, and the suite carrying such a fixture is
//! `ess-conformance/60`, which every reader refuses under a lower major.
use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{Note, Synthesis};
use ess_conformance::{AdmittedSuite, ConformanceScenario, ScenarioStep, ScenarioValue};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const LINKS: &str = include_str!("fixtures/provided-related-row.yaml");

const LINKED: &str = "linking.links.CreateLink/outcome/linked";
const UNKNOWN_APP: &str = "linking.links.CreateLink/outcome/unknown-app";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("links.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

/// Each refusal as `<scenario id> <debug>: <cause>`, so a filter can name the scenario.
fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            let id = refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default();
            format!("{id} {refusal:?}: {}", refusal.cause)
        })
        .collect()
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    let Some((_, scenario)) = result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
    else {
        panic!(
            "no scenario {id}\n ids: {:#?}\n refusals: {:#?}",
            result
                .suite
                .scenarios
                .keys()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            refusals(result)
        )
    };
    scenario
}

/// The value the scenario's one `CreateLink` sends as `app_id`.
fn sent_app(scenario: &ConformanceScenario) -> &ScenarioValue {
    let sent: Vec<_> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "linking.links.CreateLink" =>
            {
                input.get("app_id")
            }
            _ => None,
        })
        .collect();
    assert_eq!(sent.len(), 1, "{:#?}", scenario.steps);
    sent[0]
}

#[test]
fn the_related_row_is_resolved_as_an_entity_fixture_and_sent_by_identity() {
    let result = synthesis(LINKS);
    assert!(result.refusals.is_empty(), "{:#?}", refusals(&result));
    let linked = scenario(&result, LINKED);
    let ScenarioStep::ResolveFixtures { fixtures } = &linked.steps[0] else {
        panic!("the prelude resolves the row first: {:#?}", linked.steps);
    };
    assert_eq!(fixtures.fields.len(), 1, "{fixtures:#?}");
    let field = &fixtures.fields[0];
    assert_eq!(
        field.entity.as_ref().map(ToString::to_string).as_deref(),
        Some("linking.links.App")
    );
    assert_eq!(
        field.state.as_ref().map(ToString::to_string).as_deref(),
        Some("Active")
    );
    assert_eq!(field.type_ref.to_string(), "linking.links.AppId");
    let ScenarioValue::Fixture { fixture } = sent_app(linked) else {
        panic!("the row is sent as its fixture: {:#?}", linked.steps);
    };
    assert_eq!(fixture, &field.name);
    assert_eq!(
        linked
            .steps
            .iter()
            .filter(|step| matches!(step, ScenarioStep::ResolveFixtures { .. }))
            .count(),
        1,
        "one prelude: {:#?}",
        linked.steps
    );
    assert_eq!(
        result.suite.provenance.suite_version.to_string(),
        "ess-conformance/60"
    );
}

#[test]
fn the_absent_branch_is_sent_a_fresh_identity() {
    let result = synthesis(LINKS);
    let unknown = scenario(&result, UNKNOWN_APP);
    assert!(
        matches!(sent_app(unknown), ScenarioValue::Literal { .. }),
        "{:#?}",
        unknown.steps
    );
    assert!(
        !unknown
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ResolveFixtures { .. })),
        "{:#?}",
        unknown.steps
    );
}

/// No decoy row of the provided entity stands beside the one named, and the scenario says so.
#[test]
fn a_scenario_reading_a_provided_row_notes_that_no_decoy_stands_beside_it() {
    let result = synthesis(LINKS);
    let noted: Vec<_> = result
        .notes
        .iter()
        .filter_map(|note| match note {
            Note::ProvidedRelatedRow { scenario, entity } => {
                Some((scenario.to_string(), entity.to_string()))
            }
            _ => None,
        })
        .collect();
    assert!(
        noted.contains(&(LINKED.to_owned(), "linking.links.App".to_owned())),
        "{noted:#?}"
    );
    assert!(
        noted.iter().all(|(scenario, _)| scenario != UNKNOWN_APP),
        "{noted:#?}"
    );
}

/// A command acting on a provided row is refused, naming the deployment's rows: no scenario can
/// make one rest under a command's subject.
#[test]
fn a_provided_subject_is_refused_by_name() {
    let text = LINKS
        .replace(
            "commands:\n",
            "commands:
  - name: linking.links.RetitleApp
    input:
      - {name: app_id, type: linking.links.AppId}
      - {name: title, type: String}
    outcomes:
      - name: retitled
        updates: linking.links.App
        instance: app_id
        sets: {title: input.title}
        emits: [linking.links.Retitled]
        payload: {linking.links.Retitled: {app_id: input.app_id}}
",
        )
        .replace(
            "events:\n",
            "events:
  - name: linking.links.Retitled
    fields:
      - {name: app_id, type: linking.links.AppId}
",
        );
    let result = synthesis(&text);
    let about: Vec<_> = refusals(&result)
        .into_iter()
        .filter(|refusal| refusal.contains("RetitleApp"))
        .collect();
    assert!(!about.is_empty(), "{:#?}", refusals(&result));
    assert!(
        about
            .iter()
            .all(|refusal| refusal.contains("the deployment provides its rows")),
        "{about:#?}"
    );
    // The related-row position is still served beside it.
    scenario(&result, LINKED);
}

/// A guard over the provided row's fields is refused by name: the deployment holds their values,
/// so no state a fixture names decides the branch.
#[test]
fn a_guard_over_a_provided_rows_fields_is_refused_by_name() {
    let text = LINKS
        .replace(
            "      - name: linked\n",
            "      - name: retitled-app
        when_related: {via: input.app_id, predicate: title != input.title}
        error: linking.links.AppRetitled
      - name: linked\n",
        )
        .replace(
            "      - {name: app_id, type: linking.links.AppId}\n    outcomes:",
            "      - {name: app_id, type: linking.links.AppId}\n      - {name: title, type: String}\n    outcomes:",
        )
        .replace(
            "errors:\n",
            "errors:\n  - {name: linking.links.AppRetitled, fields: []}\n",
        );
    let result = synthesis(&text);
    // Both branches that read the row through the guard need its `title`, which the deployment
    // holds: each is refused by that name. The absent branch reads no field and is still served.
    for id in ["linking.links.CreateLink/outcome/retitled-app", LINKED] {
        let about: Vec<_> = refusals(&result)
            .into_iter()
            .filter(|refusal| refusal.starts_with(&format!("{id} ")))
            .collect();
        assert_eq!(about.len(), 1, "{id}: {:#?}", refusals(&result));
        assert!(
            about[0].contains("the guards read fields of the row"),
            "{about:#?}"
        );
    }
    scenario(&result, UNKNOWN_APP);
}

/// Where an outcome creates a row of a provided entity, synthesis arranges it as before.
#[test]
fn ordinary_arrangement_keeps_precedence() {
    let text = LINKS
        .replace(
            "commands:\n",
            "commands:
  - name: linking.links.RegisterApp
    input:
      - {name: title, type: String}
    outcomes:
      - name: registered
        creates: linking.links.App
        instance: app_id
        sets: {title: input.title}
        emits: [linking.links.Registered]
        payload: {linking.links.Registered: {app_id: {generated: true}}}
",
        )
        .replace(
            "events:\n",
            "events:
  - name: linking.links.Registered
    fields:
      - {name: app_id, type: linking.links.AppId}
",
        );
    let result = synthesis(&text);
    let linked = scenario(&result, LINKED);
    assert!(
        !linked
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ResolveFixtures { .. })),
        "{:#?}",
        linked.steps
    );
    assert!(
        matches!(sent_app(linked), ScenarioValue::Instance { .. }),
        "{:#?}",
        linked.steps
    );
    assert!(result
        .notes
        .iter()
        .all(|note| !matches!(note, Note::ProvidedRelatedRow { .. })));
    assert!(
        result.suite.provenance.suite_version.major() < 60,
        "{}",
        result.suite.provenance.suite_version
    );
}

/// Without the marker nothing changes: the branch is refused as no outcome creating the row, and
/// the suite stays below `ess-conformance/60`.
#[test]
fn without_the_marker_the_branch_stays_refused_and_the_suite_keeps_its_major() {
    let result = synthesis(&LINKS.replace("    provided: true\n", ""));
    let about: Vec<_> = refusals(&result)
        .into_iter()
        .filter(|refusal| refusal.starts_with(&format!("{LINKED} ")))
        .collect();
    assert_eq!(about.len(), 1, "{:#?}", refusals(&result));
    assert!(about[0].contains("no outcome creates one"), "{about:#?}");
    assert!(
        result.suite.provenance.suite_version.major() < 60,
        "{}",
        result.suite.provenance.suite_version
    );
}

fn relabelled(result: &Synthesis, major: u32) -> String {
    let mut document: serde_json::Value =
        serde_json::from_str(&result.suite.to_canonical_json().unwrap()).unwrap();
    document["provenance"]["suite_version"] = format!("ess-conformance/{major}").into();
    serde_json::to_string(&document).unwrap()
}

#[test]
fn an_entity_fixture_requires_suite_60_and_every_reader_refuses_it_below() {
    let result = synthesis(LINKS);
    AdmittedSuite::from_json(&relabelled(&result, 60)).expect("suite/60 carries an entity fixture");
    // /52 is the oldest major that admits the rest of this suite (its suite-level event catalog).
    for major in [52, 58] {
        let Err(error) = AdmittedSuite::from_json(&relabelled(&result, major)) else {
            panic!("suite/{major} cannot carry an entity fixture");
        };
        let error = error.to_string();
        assert!(error.contains("suite/60 or /61"), "/{major}: {error}");
    }
}

/// The suite with `edit` applied to every fixture field it declares; `false` where it declares none.
fn with_fixture_fields(
    result: &Synthesis,
    edit: impl Fn(&mut serde_json::Map<String, serde_json::Value>),
) -> (String, bool) {
    let mut document: serde_json::Value =
        serde_json::from_str(&result.suite.to_canonical_json().unwrap()).unwrap();
    let mut edited = false;
    for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
        for step in scenario["steps"].as_array_mut().unwrap() {
            if step["step"] == "resolve_fixtures" {
                for field in step["fixtures"]["fields"].as_array_mut().unwrap() {
                    edit(field.as_object_mut().unwrap());
                    edited = true;
                }
            }
        }
    }
    (serde_json::to_string(&document).unwrap(), edited)
}

#[test]
fn an_entity_fixture_names_its_entity_and_its_state_together() {
    let result = synthesis(LINKS);
    let (without_state, edited) = with_fixture_fields(&result, |field| {
        assert_eq!(field.remove("state"), Some("Active".into()));
    });
    assert!(edited);
    let Err(error) = AdmittedSuite::from_json(&without_state) else {
        panic!("an entity fixture without its state is refused");
    };
    assert!(
        error.to_string().contains("entity and its state together"),
        "{error}"
    );
    let (unknown_key, _) = with_fixture_fields(&result, |field| {
        let state = field.remove("state").unwrap();
        field.insert("stated".into(), state);
    });
    assert!(AdmittedSuite::from_json(&unknown_key).is_err());
}
