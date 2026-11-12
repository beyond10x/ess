//! An accepted scenario arranges the rows a `references` relation carried by an input field names
//! (`docs/design/referenced-input-arrangement.md`): a creating command writes `sources` from its
//! input, the model relates `sources` to `Source` rows, and the scenario the suite calls accepted
//! sends only identities an earlier `AddSource` step created.

use std::fmt::Write as _;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, InstanceName, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const ADD: &str = "demo.reg.AddSource";
const REGISTER: &str = "demo.reg.Register";
const ACCEPTED: &str = "demo.reg.Register/outcome/accepted";

const HEAD: &str = r#"
format: ess/30
system: demo
version: v1
domain: demo.reg
types:
  - {name: demo.reg.ServerId, kind: newtype, of: Uuid}
  - {name: demo.reg.SourceId, kind: newtype, of: Uuid}
entities:
  - name: demo.reg.Source
    identity: {name: id, type: demo.reg.SourceId}
    fields: []
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
  - name: demo.reg.Server
    identity: {name: id, type: demo.reg.ServerId}
    fields:
      - {name: sources, type: "FIELD_TYPE"}
    relations:
      - {name: sources, kind: references, target: demo.reg.Source, cardinality: CARDINALITY, via: sources}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
events:
  - name: demo.reg.SourceAdded
    fields: [{name: id, type: demo.reg.SourceId}]
  - name: demo.reg.Registered
    fields: [{name: id, type: demo.reg.ServerId}]
actors:
  - name: demo.reg.Operator
    may: [ACTOR_MAY]
commands:
"#;

const ADD_SOURCE: &str = r"
  - name: demo.reg.AddSource
    input: [{name: id, type: demo.reg.SourceId}]
    outcomes:
      - name: added
        creates: demo.reg.Source
        instance: id
        emits: [demo.reg.SourceAdded]
        payload: {demo.reg.SourceAdded: {id: input.id}}
";

const REGISTER_COMMAND: &str = r#"
  - name: demo.reg.Register
    input:
      - {name: id, type: demo.reg.ServerId}
      - {name: sources, type: "FIELD_TYPE"}
    outcomes:
      - name: accepted
        creates: demo.reg.Server
        instance: id
        sets: {sources: input.sources}
        emits: [demo.reg.Registered]
        payload: {demo.reg.Registered: {id: input.id}}
views:
  - name: demo.reg.Servers
    source: demo.reg.Server
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.reg.ServerId}
      - {name: sources, type: "FIELD_TYPE"}
"#;

/// The model: `many` is a `List` of identities, `one` a plain one; `route` keeps `AddSource`.
fn model(many: bool, route: bool) -> EssIr {
    let (field, cardinality) = if many {
        ("List<demo.reg.SourceId>", "many")
    } else {
        ("demo.reg.SourceId", "one")
    };
    let granted = if route {
        "demo.reg.AddSource, demo.reg.Register"
    } else {
        "demo.reg.Register"
    };
    let text = format!(
        "{}{}{}",
        HEAD,
        if route { ADD_SOURCE } else { "" },
        REGISTER_COMMAND
    )
    .replace("FIELD_TYPE", field)
    .replace("CARDINALITY", cardinality)
    .replace("ACTOR_MAY", granted);
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("reg.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn scenario<'a>(synthesis: &'a Synthesis, id: &str) -> Option<&'a ConformanceScenario> {
    synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

/// The `sources` input of the `Register` step of `scenario`, with the instances captured after
/// each earlier `AddSource` step: its `id` is what the capture binds.
fn register_sources(scenario: &ConformanceScenario) -> (ScenarioValue, Vec<InstanceName>) {
    let mut added: Vec<InstanceName> = Vec::new();
    let mut after_add = false;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                after_add = command.to_string() == ADD;
                if command.to_string() == REGISTER {
                    return (input["sources"].clone(), added);
                }
            }
            ScenarioStep::CaptureInstance { instance, .. } if after_add => {
                added.push(instance.clone());
            }
            _ => {}
        }
    }
    panic!("no Register step in {:#?}", scenario.steps);
}

fn instances(value: &ScenarioValue) -> Vec<InstanceName> {
    let items = match value {
        ScenarioValue::List { items } => items.as_slice(),
        single => std::slice::from_ref(single),
    };
    items
        .iter()
        .map(|item| match item {
            ScenarioValue::Instance { instance } => instance.clone(),
            other => panic!("an identity no step arranged: {other:#?}"),
        })
        .collect()
}

#[test]
fn an_accepted_scenario_arranges_every_referenced_list_member() {
    let synthesis = synthesize(&model(true, true));
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (sent, added) = register_sources(accepted);
    let named = instances(&sent);
    assert!(!named.is_empty(), "a non-empty list: {sent:#?}");
    for name in &named {
        assert!(
            added.contains(name),
            "{name:?} is the identity of no earlier AddSource step: {added:#?}"
        );
    }
}

#[test]
fn an_unarrangeable_reference_list_sends_an_empty_list() {
    let synthesis = synthesize(&model(true, false));
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (sent, added) = register_sources(accepted);
    assert_eq!(added, Vec::<InstanceName>::new(), "no AddSource step");
    assert_eq!(sent, ScenarioValue::literal(Node::Seq(Vec::new())));
}

#[test]
fn an_unarrangeable_one_cardinality_reference_is_refused_by_name() {
    let synthesis = synthesize(&model(false, false));
    assert!(
        scenario(&synthesis, ACCEPTED).is_none(),
        "no accepted scenario sends a reference to no row"
    );
    let refusal = synthesis
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(ACCEPTED)
        })
        .unwrap_or_else(|| panic!("no refusal of {ACCEPTED}: {:#?}", synthesis.refusals));
    assert!(
        refusal.to_string().contains("demo.reg.Server.sources"),
        "the refusal names the relation: {refusal}"
    );
}

#[test]
fn a_one_cardinality_reference_by_input_is_arranged_too() {
    let synthesis = synthesize(&model(false, true));
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (sent, added) = register_sources(accepted);
    let named = instances(&sent);
    assert_eq!(named.len(), 1, "one identity: {sent:#?}");
    assert!(
        added.contains(&named[0]),
        "{:?} is the identity of no earlier AddSource step: {added:#?}",
        named[0]
    );
}

#[test]
fn the_arranged_suites_run_on_the_interpreter() {
    for many in [true, false] {
        let model = model(many, true);
        let synthesis = synthesize(&model);
        assert!(
            synthesis.refusals.is_empty(),
            "nothing is refused: {:#?}",
            synthesis.refusals
        );
        let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
        let run = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(model));
        let failed: Vec<_> = run
            .scenarios
            .iter()
            .filter(|result| result.status != Status::Passed)
            .collect();
        assert!(failed.is_empty(), "{failed:#?}");
    }
}

/// Every identity a `Register` step sends in `sources` that no earlier step of its own scenario
/// created, over every scenario of the suite. An empty list names no row and is left out.
fn dangling_sources(synthesis: &Synthesis) -> Vec<String> {
    let mut found = Vec::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        let mut created: Vec<&InstanceName> = Vec::new();
        for step in &scenario.steps {
            match step {
                ScenarioStep::CaptureInstance { instance, .. } => created.push(instance),
                ScenarioStep::ExecuteCommand { command, input, .. }
                    if command.to_string() == REGISTER =>
                {
                    let sent = &input["sources"];
                    let items = match sent {
                        ScenarioValue::List { items } => items.as_slice(),
                        ScenarioValue::Literal {
                            value: Node::Seq(items),
                        } if items.is_empty() => continue,
                        single => std::slice::from_ref(single),
                    };
                    for item in items {
                        let arranged = matches!(item, ScenarioValue::Instance { instance }
                            if created.contains(&instance));
                        if !arranged {
                            found.push(format!("{id}: {item:?}"));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    found
}

#[test]
fn every_referenced_identity_names_a_row_an_earlier_step_of_its_scenario_created() {
    for (many, route) in [(true, true), (false, true), (true, false)] {
        let synthesis = synthesize(&model(many, route));
        let sending = synthesis.suite.scenarios.values().any(|scenario| {
            scenario.steps.iter().any(|step| {
                matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                    if command.to_string() == REGISTER)
            })
        });
        assert!(
            sending,
            "the suite sends Register (many {many}, route {route})"
        );
        assert_eq!(
            dangling_sources(&synthesis),
            Vec::<String>::new(),
            "many {many}, route {route}"
        );
    }
}

// ---- Adversary cases (turn 3): what the arrangement does beside the four acceptance models. ----

/// The reproduction's model head, with the `Source` lifecycle, extra entities and errors spliced in.
/// Only the events, errors and grants the commands name are declared.
fn adversary_text(source_lifecycle: &str, extra: &str, commands: &str) -> String {
    let events: String = [
        ("demo.reg.SourceAdded", "demo.reg.SourceId"),
        ("demo.reg.SourcePublished", "demo.reg.SourceId"),
        ("demo.reg.OriginAdded", "demo.reg.OriginId"),
        ("demo.reg.Registered", "demo.reg.ServerId"),
        ("demo.reg.Retired", "demo.reg.ServerId"),
    ]
    .iter()
    .filter(|(event, _)| commands.contains(&format!("emits: [{event}]")))
    .fold(String::new(), |mut events, (event, id)| {
        let _ = writeln!(
            events,
            "  - name: {event}\n    fields: [{{name: id, type: {id}}}]"
        );
        events
    });
    let errors = if commands.contains("error: demo.reg.Refused") {
        "errors:\n  - name: demo.reg.Refused\n"
    } else {
        ""
    };
    let may: Vec<&str> = commands
        .lines()
        .filter_map(|line| line.strip_prefix("  - name: "))
        .collect();
    let origin = if extra.contains("demo.reg.OriginId") || commands.contains("demo.reg.OriginId") {
        "  - {name: demo.reg.OriginId, kind: newtype, of: Uuid}\n"
    } else {
        ""
    };
    format!(
        r"
format: ess/30
system: demo
version: v1
domain: demo.reg
types:
  - {{name: demo.reg.ServerId, kind: newtype, of: Uuid}}
  - {{name: demo.reg.SourceId, kind: newtype, of: Uuid}}
{origin}entities:
  - name: demo.reg.Source
    identity: {{name: id, type: demo.reg.SourceId}}
    fields: []
    lifecycle: {source_lifecycle}
{extra}
events:
{events}{errors}actors:
  - name: demo.reg.Operator
    may: [{may}]
commands:
{commands}
",
        may = may.join(", ")
    )
}

const ADV_LIVE: &str = "{initial: Live, states: [Live], terminal: [Live]}";

const ADV_SERVER_MANY: &str = r#"  - name: demo.reg.Server
    identity: {name: id, type: demo.reg.ServerId}
    fields:
      - {name: sources, type: "List<demo.reg.SourceId>"}
    relations:
      - {name: sources, kind: references, target: demo.reg.Source, cardinality: many, via: sources}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}"#;

const ADV_SERVER_RETIRING: &str =
    "{initial: Live, states: [Live, Retired], terminal: [Retired], transitions: [{name: retire, from: [Live], to: Retired}]}";

const ADV_ADD_SOURCE: &str = r"  - name: demo.reg.AddSource
    input: [{name: id, type: demo.reg.SourceId}]
    outcomes:
      - name: added
        creates: demo.reg.Source
        instance: id
        emits: [demo.reg.SourceAdded]
        payload: {demo.reg.SourceAdded: {id: input.id}}";

const ADV_RETIRE: &str = r"  - name: demo.reg.Retire
    input: [{name: id, type: demo.reg.ServerId}]
    outcomes:
      - name: retired
        moves: demo.reg.Server.retire
        instance: id
        emits: [demo.reg.Retired]
        payload: {demo.reg.Retired: {id: input.id}}";

/// `Register`, its outcomes preceded by `before` (a refusal or nothing), `sources` typed `field`.
fn adversary_register(field: &str, before: &str, extra_input: &str, extra_sets: &str) -> String {
    format!(
        r#"  - name: demo.reg.Register
    input:
      - {{name: id, type: demo.reg.ServerId}}
      - {{name: sources, type: "{field}"}}
{extra_input}
    outcomes:
{before}
      - name: accepted
        creates: demo.reg.Server
        instance: id
        sets: {{sources: input.sources{extra_sets}}}
        emits: [demo.reg.Registered]
        payload: {{demo.reg.Registered: {{id: input.id}}}}"#
    )
}

fn adversary_compile(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("reg.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

/// A repository model, read from its path below the workspace root.
fn adversary_repository_model(relative: &str) -> EssIr {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(relative);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{relative}: {error}"));
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{relative}: {error}"));
    let mut sources = SourceMap::new();
    sources.insert(relative, text.clone());
    let spec = Specification::assemble([(Source::new(relative), raw)])
        .unwrap_or_else(|errors| panic!("{relative}: {errors}"));
    compile(&spec, &sources).unwrap_or_else(|error| panic!("{relative}: {error:?}"))
}

/// The synthesis of `model` and every scenario of it the interpreter does not pass, described.
fn adversary_interpreted(model: EssIr) -> (Synthesis, Vec<String>) {
    let synthesis = synthesize(&model);
    let failed = {
        let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
        let run = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(model));
        run.scenarios
            .iter()
            .filter(|result| result.status != Status::Passed)
            .map(|result| {
                let described = format!(
                    "{} {:?}: {:?}",
                    result.scenario,
                    result.status,
                    result.diagnostics().collect::<Vec<_>>()
                );
                described.chars().take(1500).collect::<String>()
            })
            .collect()
    };
    (synthesis, failed)
}

/// Every execution of `command` in `scenario`: its step index and its input.
fn adversary_executions<'a>(
    scenario: &'a ConformanceScenario,
    command: &str,
) -> Vec<(usize, &'a std::collections::BTreeMap<String, ScenarioValue>)> {
    scenario
        .steps
        .iter()
        .enumerate()
        .filter_map(|(at, step)| match step {
            ScenarioStep::ExecuteCommand {
                command: executed,
                input,
                ..
            } if executed.to_string() == command => Some((at, input)),
            _ => None,
        })
        .collect()
}

/// The instances captured right after a `creator` step, before step `before`.
fn adversary_captured(
    scenario: &ConformanceScenario,
    creator: &str,
    before: usize,
) -> Vec<InstanceName> {
    let mut captured = Vec::new();
    let mut after_creator = false;
    for step in &scenario.steps[..before] {
        match step {
            ScenarioStep::ExecuteCommand { command, .. } => {
                after_creator = command.to_string() == creator;
            }
            ScenarioStep::CaptureInstance { instance, .. } if after_creator => {
                captured.push(instance.clone());
            }
            _ => {}
        }
    }
    captured
}

/// Whether `value`, sent at step `at`, names only rows an earlier `creator` step created.
fn adversary_dangling(
    scenario: &ConformanceScenario,
    value: &ScenarioValue,
    creator: &str,
    at: usize,
) -> Option<String> {
    let captured = adversary_captured(scenario, creator, at);
    let items = match value {
        ScenarioValue::List { items } => items.as_slice(),
        ScenarioValue::Literal {
            value: Node::Seq(items),
        } if items.is_empty() => return Some("the empty list".to_owned()),
        single => std::slice::from_ref(single),
    };
    let dangling: Vec<String> = items
        .iter()
        .filter(|item| {
            !matches!(item, ScenarioValue::Instance { instance } if captured.contains(instance))
        })
        .map(|item| format!("{item:?}"))
        .collect();
    (!dangling.is_empty()).then(|| format!("{dangling:?}, arranged by {creator}: {captured:?}"))
}

fn adversary_count_model(guard: &str) -> EssIr {
    let refusal = format!(
        "      - name: refused\n        when: \"{guard}\"\n        error: demo.reg.Refused"
    );
    adversary_compile(&adversary_text(
        ADV_LIVE,
        ADV_SERVER_MANY,
        &format!(
            "{ADV_ADD_SOURCE}\n{}",
            adversary_register("List<demo.reg.SourceId>", &refusal, "", "")
        ),
    ))
}

/// A sibling refusal holds the list at four or more: the witness sends four, and the
/// arrangement must not cut it to its three-row cap, or the refusal answers instead.
#[test]
fn adversary_a_list_held_above_three_by_a_sibling_refusal_keeps_its_length() {
    let (synthesis, failed) = adversary_interpreted(adversary_count_model("sources.count < 4"));
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (_, input) = adversary_executions(accepted, REGISTER)[0];
    assert!(
        failed.iter().all(|line| !line.starts_with(ACCEPTED)),
        "the accepted scenario fails the interpreter, sent {:#?}: {failed:#?}",
        input["sources"]
    );
}

/// A sibling refusal admits only the empty list: the witness sends `[]`, and the arrangement
/// must not raise it to one row, or the refusal answers instead.
#[test]
fn adversary_a_list_held_empty_by_a_sibling_refusal_stays_empty() {
    let (synthesis, failed) = adversary_interpreted(adversary_count_model("sources.count > 0"));
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (_, input) = adversary_executions(accepted, REGISTER)[0];
    assert!(
        failed.iter().all(|line| !line.starts_with(ACCEPTED)),
        "the accepted scenario fails the interpreter, sent {:#?}: {failed:#?}",
        input["sources"]
    );
}

/// `Source` is created straight into `Live` (`into:`), so a route creates it although none
/// reaches its `initial`: the `one` reference is arrangeable, and no refusal claims otherwise.
#[test]
fn adversary_a_reference_created_into_a_later_state_is_still_arranged() {
    let lifecycle = "{initial: Draft, states: [Draft, Live], terminal: [Live], transitions: [{name: publish, from: [Draft], to: Live}]}";
    let server = ADV_SERVER_MANY
        .replace("List<demo.reg.SourceId>", "demo.reg.SourceId")
        .replace("cardinality: many", "cardinality: one");
    let add_source = ADV_ADD_SOURCE.replace(
        "creates: demo.reg.Source\n",
        "creates: demo.reg.Source\n        into: Live\n",
    );
    let publish = r"  - name: demo.reg.Publish
    input: [{name: id, type: demo.reg.SourceId}]
    outcomes:
      - name: published
        moves: demo.reg.Source.publish
        instance: id
        emits: [demo.reg.SourcePublished]
        payload: {demo.reg.SourcePublished: {id: input.id}}";
    let model = adversary_compile(&adversary_text(
        lifecycle,
        &server,
        &format!(
            "{add_source}\n{publish}\n{}",
            adversary_register("demo.reg.SourceId", "", "", "")
        ),
    ));
    let synthesis = synthesize(&model);
    let accepted = scenario(&synthesis, ACCEPTED).unwrap_or_else(|| {
        panic!(
            "no accepted scenario although AddSource creates a Source: {:#?}",
            synthesis.refusals
        )
    });
    let (at, input) = adversary_executions(accepted, REGISTER)[0];
    assert_eq!(
        adversary_dangling(accepted, &input["sources"], ADD, at),
        None,
        "the reference names an arranged row"
    );
}

/// The referenced `Source` itself references an `Origin`: the row arranged for `Register` is
/// created naming an `Origin` an earlier step created, not a dangling one.
#[test]
fn adversary_a_referenced_row_that_references_another_is_arranged_with_it() {
    let origin_and_source = format!(
        "{ADV_SERVER_MANY}\n  - name: demo.reg.Origin\n    identity: {{name: id, type: demo.reg.OriginId}}\n    fields: []\n    lifecycle: {ADV_LIVE}"
    );
    let add_origin = r"  - name: demo.reg.AddOrigin
    input: [{name: id, type: demo.reg.OriginId}]
    outcomes:
      - name: added
        creates: demo.reg.Origin
        instance: id
        emits: [demo.reg.OriginAdded]
        payload: {demo.reg.OriginAdded: {id: input.id}}";
    let add_source = ADV_ADD_SOURCE
        .replace(
            "input: [{name: id, type: demo.reg.SourceId}]",
            "input: [{name: id, type: demo.reg.SourceId}, {name: origin, type: demo.reg.OriginId}]",
        )
        .replace(
            "instance: id\n",
            "instance: id\n        sets: {origin: input.origin}\n",
        );
    let text = adversary_text(
        ADV_LIVE,
        &origin_and_source,
        &format!(
            "{add_origin}\n{add_source}\n{}",
            adversary_register("List<demo.reg.SourceId>", "", "", "")
        ),
    )
    .replace(
        "  - name: demo.reg.Source\n    identity: {name: id, type: demo.reg.SourceId}\n    fields: []\n",
        "  - name: demo.reg.Source\n    identity: {name: id, type: demo.reg.SourceId}\n    fields:\n      - {name: origin, type: demo.reg.OriginId}\n    relations:\n      - {name: origin, kind: references, target: demo.reg.Origin, cardinality: one, via: origin}\n",
    );
    let synthesis = synthesize(&adversary_compile(&text));
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let executions = adversary_executions(accepted, ADD);
    assert!(
        !executions.is_empty(),
        "a Source is arranged: {:#?}",
        accepted.steps
    );
    for (at, input) in executions {
        assert_eq!(
            adversary_dangling(accepted, &input["origin"], "demo.reg.AddOrigin", at),
            None,
            "the arranged Source names an Origin no step created"
        );
    }
}

/// A later command on the server arranges it through `Register`: that arranging step sends
/// `sources` naming rows the scenario created too, or a target resolving them refuses the setup.
#[test]
#[ignore = "backlog: later-command-rows"]
fn adversary_a_server_arranged_for_a_later_command_names_arranged_sources() {
    let server = ADV_SERVER_MANY.replace(
        "    lifecycle: {initial: Live, states: [Live], terminal: [Live]}",
        &format!("    lifecycle: {ADV_SERVER_RETIRING}"),
    );
    let model = adversary_compile(&adversary_text(
        ADV_LIVE,
        &server,
        &format!(
            "{ADV_ADD_SOURCE}\n{}\n{ADV_RETIRE}",
            adversary_register("List<demo.reg.SourceId>", "", "", "")
        ),
    ));
    let synthesis = synthesize(&model);
    let retired = scenario(&synthesis, "demo.reg.Retire/outcome/retired")
        .expect("the retired scenario exists");
    let (at, input) = adversary_executions(retired, REGISTER)[0];
    assert_eq!(
        adversary_dangling(retired, &input["sources"], ADD, at),
        None,
        "the Server arranged for Retire names Sources no step created"
    );
}

/// The same gap in a repository model: `ChangeTier/outcome/changed` arranges its `Seat` through
/// `Issue`, which writes `input.tier_id` into the `tier` reference.
#[test]
#[ignore = "backlog: later-command-rows"]
fn adversary_a_repository_seat_arranged_for_an_update_names_an_arranged_tier() {
    let model = adversary_repository_model(
        "crates/verify/ess-conformance/tests/fixtures/related-guard-unknown-instance.yaml",
    );
    let synthesis = synthesize(&model);
    let changed = scenario(&synthesis, "demo.seats.ChangeTier/outcome/changed")
        .expect("the changed scenario exists");
    let (at, input) = adversary_executions(changed, "demo.seats.Issue")[0];
    assert_eq!(
        adversary_dangling(changed, &input["tier_id"], "demo.seats.DefineTier", at),
        None,
        "the Seat arranged for ChangeTier names a Tier no step created: {:#?}",
        changed.steps
    );
}

/// A command with a related guard on another field is skipped whole: its accepted scenario
/// still sends a `sources` list no step created (reported, not part of this unit's acceptance).
#[test]
#[ignore = "backlog: guarded-command"]
fn adversary_a_related_guarded_command_still_sends_dangling_sources() {
    let server = ADV_SERVER_MANY.replace(
        "      - {name: sources, type: \"List<demo.reg.SourceId>\"}\n    relations:\n",
        "      - {name: sources, type: \"List<demo.reg.SourceId>\"}\n      - {name: primary, type: demo.reg.SourceId}\n    relations:\n      - {name: primary, kind: references, target: demo.reg.Source, cardinality: one, via: primary}\n",
    );
    let guard = "      - name: no-primary\n        when_related: {via: input.primary, exists: false}\n        error: demo.reg.Refused";
    let model = adversary_compile(&adversary_text(
        ADV_LIVE,
        &server,
        &format!(
            "{ADV_ADD_SOURCE}\n{}",
            adversary_register(
                "List<demo.reg.SourceId>",
                guard,
                "      - {name: primary, type: demo.reg.SourceId}",
                ", primary: input.primary"
            )
        ),
    ));
    let synthesis = synthesize(&model);
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (at, input) = adversary_executions(accepted, REGISTER)[0];
    assert_eq!(
        adversary_dangling(accepted, &input["sources"], ADD, at),
        None,
        "the related-guarded Register sends Sources no step created"
    );
}

/// An `Optional` reference: every run that sends it present names an arranged row, and the
/// suite passes the interpreter.
#[test]
fn adversary_an_optional_reference_is_arranged_when_present() {
    let server = ADV_SERVER_MANY
        .replace("List<demo.reg.SourceId>", "Optional<demo.reg.SourceId>")
        .replace("cardinality: many", "cardinality: one");
    let model = adversary_compile(&adversary_text(
        ADV_LIVE,
        &server,
        &format!(
            "{ADV_ADD_SOURCE}\n{}",
            adversary_register("Optional<demo.reg.SourceId>", "", "", "")
        ),
    ));
    let (synthesis, failed) = adversary_interpreted(model);
    assert!(failed.is_empty(), "{failed:#?}");
    let mut present = 0;
    for (id, scenario) in &synthesis.suite.scenarios {
        for (at, input) in adversary_executions(scenario, REGISTER) {
            match input.get("sources") {
                None | Some(ScenarioValue::Literal { value: Node::Null }) => {}
                Some(value) => {
                    present += 1;
                    assert_eq!(adversary_dangling(scenario, value, ADD, at), None, "{id}");
                }
            }
        }
    }
    assert!(present > 0, "some run sends the reference present");
}

/// The five repository models whose suites moved still pass the interpreter in full, but for
/// `28.yaml`'s `gap.demo.TakeDefault`: a `system: true` actor issues it, so the runner awaits a
/// command the interpreter never issues, and the arrangement skips it (a related guard).
#[test]
fn adversary_the_moved_repository_models_pass_the_interpreter() {
    let mut failures = Vec::new();
    for relative in [
        "crates/generate/ess-synth/tests/fixtures/generation-by-name/28.yaml",
        "crates/generate/ess-synth/tests/fixtures/generation-by-name/29.yaml",
        "crates/specify/ess-compiler/tests/fixtures/precedence-rule-format-a.yaml",
        "crates/verify/ess-conformance/tests/fixtures/guard-pair-corpus.yaml",
        "crates/verify/ess-conformance/tests/fixtures/related-guard-unknown-instance.yaml",
    ] {
        let (_, failed) = adversary_interpreted(adversary_repository_model(relative));
        failures.extend(
            failed
                .into_iter()
                .filter(|line| {
                    !line.contains("by/gap.demo.TakeDefault/")
                        && !line.starts_with("gap.demo.TakeDefault/")
                })
                .map(|line| format!("{relative}: {line}")),
        );
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
