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

// ---- Adversary pass 2 (turn 3): the fix commit c7026095ea. ----

/// The refusal of `ACCEPTED`, where synthesis recorded one.
fn adversary2_refusal(synthesis: &Synthesis) -> Option<String> {
    synthesis
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
        .map(ToString::to_string)
}

/// Either the accepted scenario passes the interpreter, or it is refused naming the relation (the
/// story's design rule: no route and a list that must be non-empty is refused by name).
fn adversary2_accepted_passes_or_is_refused_by_name(model: EssIr) {
    let (synthesis, failed) = adversary_interpreted(model);
    if let Some(accepted) = scenario(&synthesis, ACCEPTED) {
        let (_, input) = adversary_executions(accepted, REGISTER)[0];
        assert!(
            failed.iter().all(|line| !line.starts_with(ACCEPTED)),
            "the accepted scenario fails the interpreter, sent {:#?}: {failed:#?}",
            input["sources"]
        );
    } else {
        let refusal = adversary2_refusal(&synthesis);
        assert!(
            refusal
                .as_deref()
                .is_some_and(|text| text.contains("demo.reg.Server.sources")),
            "no accepted scenario and no refusal naming the relation: {refusal:?}"
        );
    }
}

/// No route creates a `Source` and a sibling refusal answers an empty list: the witness sends one
/// identity, the arrangement replaces it with `[]`, and the refusal answers the accepted scenario.
#[test]
fn adversary2_an_unarrangeable_list_a_sibling_refusal_holds_non_empty_is_not_sent_empty() {
    let refusal =
        "      - name: refused\n        when: \"sources.count < 1\"\n        error: demo.reg.Refused";
    adversary2_accepted_passes_or_is_refused_by_name(adversary_compile(&adversary_text(
        ADV_LIVE,
        ADV_SERVER_MANY,
        &adversary_register("List<demo.reg.SourceId>", refusal, "", ""),
    )));
}

/// The same with an entity invariant holding the list non-empty instead of a sibling refusal.
#[test]
fn adversary2_an_unarrangeable_list_an_invariant_holds_non_empty_is_not_sent_empty() {
    let server = ADV_SERVER_MANY.replace(
        "    lifecycle:",
        "    invariants: [\"sources.count >= 1\"]\n    lifecycle:",
    );
    adversary2_accepted_passes_or_is_refused_by_name(adversary_compile(&adversary_text(
        ADV_LIVE,
        &server,
        &adversary_register("List<demo.reg.SourceId>", "", "", ""),
    )));
}

/// The accepting branch's own guard reads only the list's length (`sources.count > 0`). The
/// arrangement keeps the witness length, so nothing the guard reads moves, yet the branch is
/// skipped whole and the accepted scenario sends identities no step created.
#[test]
fn adversary2_an_accepting_branch_guarded_on_the_list_count_names_arranged_rows() {
    let register = r#"  - name: demo.reg.Register
    input:
      - {name: id, type: demo.reg.ServerId}
      - {name: sources, type: "List<demo.reg.SourceId>"}
    outcomes:
      - name: accepted
        when: sources.count > 0
        creates: demo.reg.Server
        instance: id
        sets: {sources: input.sources}
        emits: [demo.reg.Registered]
        payload: {demo.reg.Registered: {id: input.id}}
      - name: refused
        error: demo.reg.Refused"#;
    let model = adversary_compile(&adversary_text(
        ADV_LIVE,
        ADV_SERVER_MANY,
        &format!("{ADV_ADD_SOURCE}\n{register}"),
    ));
    let synthesis = synthesize(&model);
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (at, input) = adversary_executions(accepted, REGISTER)[0];
    assert_eq!(
        adversary_dangling(accepted, &input["sources"], ADD, at),
        None,
        "the accepted Register sends Sources no step created"
    );
}

/// Whether `value` is an Optional sent absent.
fn adversary2_absent(value: Option<&ScenarioValue>) -> bool {
    matches!(
        value,
        None | Some(ScenarioValue::Literal { value: Node::Null })
    )
}

/// `Source` references an `Origin`, and `Origin` references back an Optional `Source`: the chain
/// stops at the cycle, and the `Origin` arranged for the `Source` is created naming a `Source` no
/// step created, where leaving the Optional out names no missing row.
#[test]
fn adversary2_a_reference_cycle_sends_no_dangling_back_reference() {
    let origin = format!(
        "{ADV_SERVER_MANY}\n  - name: demo.reg.Origin\n    identity: {{name: id, type: demo.reg.OriginId}}\n    fields:\n      - {{name: source, type: \"Optional<demo.reg.SourceId>\"}}\n    relations:\n      - {{name: source, kind: references, target: demo.reg.Source, cardinality: one, via: source}}\n    lifecycle: {ADV_LIVE}"
    );
    let add_origin = r#"  - name: demo.reg.AddOrigin
    input:
      - {name: id, type: demo.reg.OriginId}
      - {name: source, type: "Optional<demo.reg.SourceId>"}
    outcomes:
      - name: added
        creates: demo.reg.Origin
        instance: id
        sets: {source: input.source}
        emits: [demo.reg.OriginAdded]
        payload: {demo.reg.OriginAdded: {id: input.id}}"#;
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
        &origin,
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
    let executions = adversary_executions(accepted, "demo.reg.AddOrigin");
    assert!(
        !executions.is_empty(),
        "an Origin is arranged: {:#?}",
        accepted.steps
    );
    for (at, input) in executions {
        if adversary2_absent(input.get("source")) {
            continue;
        }
        assert_eq!(
            adversary_dangling(accepted, &input["source"], ADD, at),
            None,
            "the Origin arranged for a Source names a Source no step created"
        );
    }
}

/// `Source` references an Optional parent `Source`: the row arranged for `Register` is created
/// naming a parent no step created, where leaving the Optional out names no missing row.
#[test]
fn adversary2_a_self_referencing_arranged_row_sends_no_dangling_parent() {
    let add_source = ADV_ADD_SOURCE.replace(
        "    input: [{name: id, type: demo.reg.SourceId}]\n",
        "    input:\n      - {name: id, type: demo.reg.SourceId}\n      - {name: parent, type: \"Optional<demo.reg.SourceId>\"}\n",
    ).replace(
        "instance: id\n",
        "instance: id\n        sets: {parent: input.parent}\n",
    );
    let text = adversary_text(
        ADV_LIVE,
        ADV_SERVER_MANY,
        &format!(
            "{add_source}\n{}",
            adversary_register("List<demo.reg.SourceId>", "", "", "")
        ),
    )
    .replace(
        "  - name: demo.reg.Source\n    identity: {name: id, type: demo.reg.SourceId}\n    fields: []\n",
        "  - name: demo.reg.Source\n    identity: {name: id, type: demo.reg.SourceId}\n    fields:\n      - {name: parent, type: \"Optional<demo.reg.SourceId>\"}\n    relations:\n      - {name: parent, kind: references, target: demo.reg.Source, cardinality: one, via: parent}\n",
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
        if adversary2_absent(input.get("parent")) {
            continue;
        }
        assert_eq!(
            adversary_dangling(accepted, &input["parent"], ADD, at),
            None,
            "the Source arranged for Register names a parent no step created"
        );
    }
}

/// Held (probe): two relations to `Source` (a list and a plain one) on one command, and a
/// `Source` creator with several branches where only a guarded one creates. Every scenario
/// passes the interpreter, both inputs name arranged rows, and no instance is captured twice.
#[test]
fn adversary2_two_relations_to_a_guarded_creator_name_distinct_arranged_rows() {
    let server = ADV_SERVER_MANY.replace(
        "    relations:\n",
        "      - {name: primary, type: demo.reg.SourceId}\n    relations:\n      - {name: primary, kind: references, target: demo.reg.Source, cardinality: one, via: primary}\n",
    );
    let add_source = r"  - name: demo.reg.AddSource
    input: [{name: id, type: demo.reg.SourceId}, {name: score, type: Integer}]
    outcomes:
      - name: added
        when: score > 50
        creates: demo.reg.Source
        instance: id
        emits: [demo.reg.SourceAdded]
        payload: {demo.reg.SourceAdded: {id: input.id}}
      - name: low
        error: demo.reg.Refused";
    let refusal =
        "      - name: refused\n        when: \"sources.count < 3\"\n        error: demo.reg.Refused";
    let model = adversary_compile(&adversary_text(
        ADV_LIVE,
        &server,
        &format!(
            "{add_source}\n{}",
            adversary_register(
                "List<demo.reg.SourceId>",
                refusal,
                "      - {name: primary, type: demo.reg.SourceId}",
                ", primary: input.primary"
            )
        ),
    ));
    let (synthesis, failed) = adversary_interpreted(model);
    assert!(failed.is_empty(), "{failed:#?}");
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (at, input) = adversary_executions(accepted, REGISTER)[0];
    for field in ["sources", "primary"] {
        assert_eq!(
            adversary_dangling(accepted, &input[field], ADD, at),
            None,
            "{field}"
        );
    }
    for (id, scenario) in &synthesis.suite.scenarios {
        let mut seen = Vec::new();
        for step in &scenario.steps {
            if let ScenarioStep::CaptureInstance { instance, .. } = step {
                assert!(
                    !seen.contains(instance),
                    "{id}: {instance:?} captured twice"
                );
                seen.push(instance.clone());
            }
        }
    }
}

/// A chain `Server -> A1 -> A2 -> A3 -> A4`, `A2` created straight into `Live`: the model the
/// depth and `into:` probes run on.
fn adversary2_chain_model() -> EssIr {
    let mut types = String::from("  - {name: demo.reg.ServerId, kind: newtype, of: Uuid}\n");
    let mut entities = String::new();
    let mut events = String::from(
        "  - name: demo.reg.Registered\n    fields: [{name: id, type: demo.reg.ServerId}]\n",
    );
    let mut commands = String::new();
    let mut may = Vec::new();
    for level in 1..=4 {
        let next = level + 1;
        let (fields, relations, input, sets) = if level < 4 {
            (
                format!("    fields:\n      - {{name: next, type: demo.reg.A{next}Id}}\n"),
                format!("    relations:\n      - {{name: next, kind: references, target: demo.reg.A{next}, cardinality: one, via: next}}\n"),
                format!(", {{name: next, type: demo.reg.A{next}Id}}"),
                "        sets: {next: input.next}\n".to_owned(),
            )
        } else {
            (
                "    fields: []\n".to_owned(),
                String::new(),
                String::new(),
                String::new(),
            )
        };
        let (lifecycle, into) = if level == 2 {
            (
                "{initial: Draft, states: [Draft, Live], terminal: [Live], transitions: [{name: publish, from: [Draft], to: Live}]}",
                "        into: Live\n",
            )
        } else {
            ("{initial: Live, states: [Live], terminal: [Live]}", "")
        };
        let _ = writeln!(
            types,
            "  - {{name: demo.reg.A{level}Id, kind: newtype, of: Uuid}}"
        );
        let _ = write!(
            entities,
            "  - name: demo.reg.A{level}\n    identity: {{name: id, type: demo.reg.A{level}Id}}\n{fields}{relations}    lifecycle: {lifecycle}\n"
        );
        let _ = write!(
            events,
            "  - name: demo.reg.A{level}Added\n    fields: [{{name: id, type: demo.reg.A{level}Id}}]\n"
        );
        let _ = write!(
            commands,
            "  - name: demo.reg.AddA{level}\n    input: [{{name: id, type: demo.reg.A{level}Id}}{input}]\n    outcomes:\n      - name: added\n        creates: demo.reg.A{level}\n{into}        instance: id\n{sets}        emits: [demo.reg.A{level}Added]\n        payload: {{demo.reg.A{level}Added: {{id: input.id}}}}\n"
        );
        may.push(format!("demo.reg.AddA{level}"));
        if level == 2 {
            events.push_str(
                "  - name: demo.reg.A2Published\n    fields: [{name: id, type: demo.reg.A2Id}]\n",
            );
            commands.push_str("  - name: demo.reg.PublishA2\n    input: [{name: id, type: demo.reg.A2Id}]\n    outcomes:\n      - name: published\n        moves: demo.reg.A2.publish\n        instance: id\n        emits: [demo.reg.A2Published]\n        payload: {demo.reg.A2Published: {id: input.id}}\n");
            may.push("demo.reg.PublishA2".to_owned());
        }
    }
    may.push(REGISTER.to_owned());
    let text = format!(
        r"
format: ess/30
system: demo
version: v1
domain: demo.reg
types:
{types}entities:
{entities}  - name: demo.reg.Server
    identity: {{name: id, type: demo.reg.ServerId}}
    fields:
      - {{name: first, type: demo.reg.A1Id}}
    relations:
      - {{name: first, kind: references, target: demo.reg.A1, cardinality: one, via: first}}
    lifecycle: {{initial: Live, states: [Live], terminal: [Live]}}
events:
{events}actors:
  - name: demo.reg.Operator
    may: [{may}]
commands:
{commands}  - name: demo.reg.Register
    input: [{{name: id, type: demo.reg.ServerId}}, {{name: first, type: demo.reg.A1Id}}]
    outcomes:
      - name: accepted
        creates: demo.reg.Server
        instance: id
        sets: {{first: input.first}}
        emits: [demo.reg.Registered]
        payload: {{demo.reg.Registered: {{id: input.id}}}}
",
        may = may.join(", ")
    );
    adversary_compile(&text)
}

/// Held (probe): every reference down the chain, through the `into:` creator, names an arranged
/// row, four rows deep (the design page says three), and the suite passes the interpreter.
#[test]
fn adversary2_a_four_row_chain_through_an_into_creator_names_arranged_rows() {
    let (synthesis, failed) = adversary_interpreted(adversary2_chain_model());
    assert!(failed.is_empty(), "{failed:#?}");
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (at, input) = adversary_executions(accepted, REGISTER)[0];
    assert_eq!(
        adversary_dangling(accepted, &input["first"], "demo.reg.AddA1", at),
        None,
        "Register.first"
    );
    for level in 1..4 {
        let next = level + 1;
        for (at, input) in adversary_executions(accepted, &format!("demo.reg.AddA{level}")) {
            assert_eq!(
                adversary_dangling(
                    accepted,
                    &input["next"],
                    &format!("demo.reg.AddA{next}"),
                    at
                ),
                None,
                "A{level}.next: {:#?}",
                accepted.steps
            );
        }
    }
}

// ---- Adversary pass 3 (turn 3): the fix commit 8d773d68c3. ----

/// `Server` with an `Integer` field `min` and the invariant `sources.count >= min`.
fn adversary3_min_server() -> String {
    ADV_SERVER_MANY
        .replace(
            "    relations:\n",
            "      - {name: min, type: Integer}\n    relations:\n",
        )
        .replace(
            "    lifecycle:",
            "    invariants: [\"sources.count >= min\"]\n    lifecycle:",
        )
}

/// The invariant `sources.count >= min` holds the list non-empty, but `min` is set from a literal
/// (`min: "1"`), so `keeps_branch` does not see the invariant (only invariants every read of which
/// lands on a copied input are checked). With a route the arranged scenario passes, so the witness
/// list holds a row; with none it is sent `[]`, and the interpreter finds the invariant broken.
#[test]
fn adversary3_an_unarrangeable_list_a_literal_bounded_invariant_holds_non_empty_is_not_sent_empty()
{
    let register = adversary_register("List<demo.reg.SourceId>", "", "", ", min: \"1\"");
    let (_, failed) = adversary_interpreted(adversary_compile(&adversary_text(
        ADV_LIVE,
        &adversary3_min_server(),
        &format!("{ADV_ADD_SOURCE}\n{register}"),
    )));
    assert!(
        failed.iter().all(|line| !line.starts_with(ACCEPTED)),
        "with a route the accepted scenario passes: {failed:#?}"
    );
    adversary2_accepted_passes_or_is_refused_by_name(adversary_compile(&adversary_text(
        ADV_LIVE,
        &adversary3_min_server(),
        &register,
    )));
}

/// `accepted` (`when: flag == true`) is declared before the accepting `bare`
/// (`when: sources.count == 0`): with no route, `[]` and `flag: true` still select `accepted` by
/// declaration order, so the empty list keeps the branch and the scenario is not refused.
#[test]
fn adversary3_a_later_accepting_branch_reading_the_count_does_not_refuse_the_empty_list() {
    let register = r#"  - name: demo.reg.Register
    input:
      - {name: id, type: demo.reg.ServerId}
      - {name: sources, type: "List<demo.reg.SourceId>"}
      - {name: flag, type: Boolean}
    outcomes:
      - name: accepted
        when: flag == true
        creates: demo.reg.Server
        instance: id
        sets: {sources: input.sources}
        emits: [demo.reg.Registered]
        payload: {demo.reg.Registered: {id: input.id}}
      - name: bare
        when: sources.count == 0
        creates: demo.reg.Server
        instance: id
        sets: {sources: input.sources}
        emits: [demo.reg.Registered]
        payload: {demo.reg.Registered: {id: input.id}}
      - name: refused
        error: demo.reg.Refused"#;
    let (synthesis, failed) = adversary_interpreted(adversary_compile(&adversary_text(
        ADV_LIVE,
        ADV_SERVER_MANY,
        register,
    )));
    assert!(
        scenario(&synthesis, ACCEPTED).is_some(),
        "`[]` with `flag: true` still selects accepted, yet it is refused: {:?}",
        adversary2_refusal(&synthesis)
    );
    assert!(
        failed.iter().all(|line| !line.starts_with(ACCEPTED)),
        "{failed:#?}"
    );
}

/// `Register` whose accepting branch is guarded by `guard`, with an `Integer` input `n` and a
/// plain `SourceId` input `primary` (no relation) the guard may read; a default refusal below.
fn adversary3_guarded_register(guard: &str) -> String {
    format!(
        r#"  - name: demo.reg.Register
    input:
      - {{name: id, type: demo.reg.ServerId}}
      - {{name: sources, type: "List<demo.reg.SourceId>"}}
      - {{name: n, type: Integer}}
      - {{name: primary, type: demo.reg.SourceId}}
    outcomes:
      - name: accepted
        when: {guard}
        creates: demo.reg.Server
        instance: id
        sets: {{sources: input.sources}}
        emits: [demo.reg.Registered]
        payload: {{demo.reg.Registered: {{id: input.id}}}}
      - name: refused
        error: demo.reg.Refused"#
    )
}

/// Held (probe): `sources.count == n` compared with another input. With a route the list names
/// arranged rows and passes; with none it passes or is refused by name.
#[test]
fn adversary3_a_count_compared_with_an_input_is_arranged_or_refused_by_name() {
    let register = adversary3_guarded_register("\"sources.count == n\"");
    let (synthesis, failed) = adversary_interpreted(adversary_compile(&adversary_text(
        ADV_LIVE,
        ADV_SERVER_MANY,
        &format!("{ADV_ADD_SOURCE}\n{register}"),
    )));
    assert!(failed.is_empty(), "{failed:#?}");
    let accepted = scenario(&synthesis, ACCEPTED).expect("the accepted scenario exists");
    let (at, input) = adversary_executions(accepted, REGISTER)[0];
    if !matches!(&input["sources"], ScenarioValue::Literal { value: Node::Seq(items) } if items.is_empty())
    {
        assert_eq!(
            adversary_dangling(accepted, &input["sources"], ADD, at),
            None,
            "sources"
        );
    }
    adversary2_accepted_passes_or_is_refused_by_name(adversary_compile(&adversary_text(
        ADV_LIVE,
        ADV_SERVER_MANY,
        &register,
    )));
}

/// Held (probe): a guard reading the count and an element (`exists ... s == primary`) is left
/// alone, route or none: the accepted scenario exists and passes, so the witness list that holds
/// `primary` is not replaced by arranged rows, nor refused.
#[test]
fn adversary3_a_guard_reading_the_count_and_an_element_is_left_alone() {
    let register = adversary3_guarded_register(
        "{all: [\"sources.count > 0\", {exists: {in: sources, as: s, that: \"s == primary\"}}]}",
    );
    for commands in [format!("{ADV_ADD_SOURCE}\n{register}"), register.clone()] {
        let (synthesis, failed) = adversary_interpreted(adversary_compile(&adversary_text(
            ADV_LIVE,
            ADV_SERVER_MANY,
            &commands,
        )));
        assert!(
            scenario(&synthesis, ACCEPTED).is_some(),
            "refused: {:?}",
            adversary2_refusal(&synthesis)
        );
        assert!(
            failed.iter().all(|line| !line.starts_with(ACCEPTED)),
            "{failed:#?}"
        );
    }
}

/// Held (probe): with no route, `sources.count <= limit` (`limit` copied from the input) keeps
/// the empty list, which is sent; so does `forall` over the list, which the arrangement does not
/// refuse.
#[test]
fn adversary3_invariants_over_the_list_decide_the_empty_list() {
    let server = ADV_SERVER_MANY
        .replace(
            "    relations:\n",
            "      - {name: limit, type: Integer}\n      - {name: primary, type: demo.reg.SourceId}\n    relations:\n",
        );
    let extra_input =
        "      - {name: limit, type: Integer}\n      - {name: primary, type: demo.reg.SourceId}";
    let extra_sets = ", limit: input.limit, primary: input.primary";
    let register = adversary_register("List<demo.reg.SourceId>", "", extra_input, extra_sets);
    let bounded = server.replace(
        "    lifecycle:",
        "    invariants: [\"sources.count <= limit\"]\n    lifecycle:",
    );
    let model = adversary_compile(&adversary_text(ADV_LIVE, &bounded, &register));
    let (synthesis, failed) = adversary_interpreted(model);
    let accepted = scenario(&synthesis, ACCEPTED).unwrap_or_else(|| {
        panic!(
            "the empty list keeps `sources.count <= limit`, yet: {:?}",
            adversary2_refusal(&synthesis)
        )
    });
    let (_, input) = adversary_executions(accepted, REGISTER)[0];
    assert_eq!(
        input["sources"],
        ScenarioValue::literal(Node::Seq(Vec::new()))
    );
    assert!(
        failed.iter().all(|line| !line.starts_with(ACCEPTED)),
        "{failed:#?}"
    );
    // `exists` over the list is not probed: its witness is refused before any arrangement
    // (ESS-SYNTH-003, "no candidate of the 2 tried satisfies"), which is not this unit's.
    let forall = server.replace(
        "    lifecycle:",
        "    invariants: [{forall: {in: sources, as: s, that: \"s != primary\"}}]\n    lifecycle:",
    );
    let (synthesis, failed) = adversary_interpreted(adversary_compile(&adversary_text(
        ADV_LIVE, &forall, &register,
    )));
    let refusal = adversary2_refusal(&synthesis);
    assert!(
        !refusal
            .as_deref()
            .is_some_and(|text| text.contains("would not keep the branch")),
        "`forall` holds the empty list vacuously, yet the arrangement refuses: {refusal:?}"
    );
    assert!(
        failed.iter().all(|line| !line.starts_with(ACCEPTED)),
        "{failed:#?}"
    );
}

/// Held (probe): an `Optional` `one` reference with no route, a sibling refusal requiring it
/// present (`not defined(sources)`): the accepted scenario passes or is refused by name, never
/// sent absent into the refusal.
#[test]
fn adversary3_an_unarrangeable_optional_reference_a_refusal_requires_is_refused_by_name() {
    let server = ADV_SERVER_MANY
        .replace("List<demo.reg.SourceId>", "Optional<demo.reg.SourceId>")
        .replace("cardinality: many", "cardinality: one");
    let refusal =
        "      - name: refused\n        when: 'not defined(sources)'\n        error: demo.reg.Refused";
    adversary2_accepted_passes_or_is_refused_by_name(adversary_compile(&adversary_text(
        ADV_LIVE,
        &server,
        &adversary_register("Optional<demo.reg.SourceId>", refusal, "", ""),
    )));
}

// ---- Adversary pass 4 (turn 3): the fix commit 6e73807365, `empty_keeps_branch`. ----

const RECONFIGURED: &str = "demo.reg.Reconfigure/outcome/accepted";

/// The refusal of scenario `id`, where synthesis recorded one.
fn adversary4_refusal(synthesis: &Synthesis, id: &str) -> Option<String> {
    synthesis
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(id)
        })
        .map(ToString::to_string)
}

/// Scenario `id` exists and passes the interpreter: neither refused nor failed.
fn adversary4_exists_and_passes(model: EssIr, id: &str) -> Synthesis {
    let (synthesis, failed) = adversary_interpreted(model);
    assert!(
        scenario(&synthesis, id).is_some(),
        "{id} is refused although `[]` passes the interpreter: {:?}",
        adversary4_refusal(&synthesis, id)
    );
    assert!(
        failed.iter().all(|line| !line.starts_with(id)),
        "{failed:#?}"
    );
    synthesis
}

/// Scenario `id` passes the interpreter, or is refused naming `demo.reg.Server.sources`.
fn adversary4_passes_or_refused_by_name(model: EssIr, id: &str, command: &str) {
    let (synthesis, failed) = adversary_interpreted(model);
    if let Some(found) = scenario(&synthesis, id) {
        let (_, input) = adversary_executions(found, command)[0];
        assert!(
            failed.iter().all(|line| !line.starts_with(id)),
            "{id} fails the interpreter, sent {:#?}: {failed:#?}",
            input["sources"]
        );
    } else {
        let refusal = adversary4_refusal(&synthesis, id);
        assert!(
            refusal
                .as_deref()
                .is_some_and(|text| text.contains("demo.reg.Server.sources")),
            "no {id} scenario and no refusal naming the relation: {refusal:?}"
        );
    }
}

/// `Reconfigure`, an update of `Server` copying `sources`, with an input `flag`, its outcomes
/// `outcomes` (each `updates` branch written by [`adversary4_update`]).
fn adversary4_reconfigure(outcomes: &str) -> String {
    format!(
        r#"  - name: demo.reg.Reconfigure
    input:
      - {{name: id, type: demo.reg.ServerId}}
      - {{name: sources, type: "List<demo.reg.SourceId>"}}
      - {{name: flag, type: Boolean}}
    outcomes:
{outcomes}"#
    )
}

/// An accepting `updates: Server` branch named `name`, guarded by `when` where given.
fn adversary4_update(name: &str, when: &str, sets: &str) -> String {
    let when = if when.is_empty() {
        String::new()
    } else {
        format!("        when: {when}\n")
    };
    format!(
        "      - name: {name}\n{when}        updates: demo.reg.Server\n        instance: id\n        \
         sets: {{sources: input.sources{sets}}}\n        emits: [demo.reg.Registered]\n        \
         payload: {{demo.reg.Registered: {{id: input.id}}}}\n"
    )
}

const ADV4_FROZEN: &str =
    "      - name: frozen\n        when_subject_state: Retired\n        error: demo.reg.Refused\n";

const ADV4_REFUSED: &str = "      - name: refused\n        error: demo.reg.Refused";

/// A `Server` that retires, so `Reconfigure` carries a held-state guard.
fn adversary4_retiring_server() -> String {
    ADV_SERVER_MANY.replace(ADV_LIVE, ADV_SERVER_RETIRING)
}

/// Held (probe): the fallback path. A held-state refusal makes `selects_branch` err, so
/// `keeps_branch` decides; a refusal reading the count (`sources.count < 1`) is answered before
/// `accepted`, and `[]` is refused by name, never sent into the refusal. (A later accepting branch
/// reading the count beside a held-state guard does not compile: `non_exhaustive_branches`.)
#[test]
fn adversary4_a_held_state_guard_and_a_refusal_reading_the_count_refuse_by_name() {
    let outcomes = format!(
        "      - name: short\n        when: \"sources.count < 1\"\n        error: \
         demo.reg.Refused\n{ADV4_FROZEN}{}",
        adversary4_update("accepted", "", ""),
    );
    let commands = format!(
        "{}\n{ADV_RETIRE}\n{}",
        adversary_register("List<demo.reg.SourceId>", "", "", ""),
        adversary4_reconfigure(&outcomes)
    );
    adversary4_passes_or_refused_by_name(
        adversary_compile(&adversary_text(
            ADV_LIVE,
            &adversary4_retiring_server(),
            &commands,
        )),
        RECONFIGURED,
        "demo.reg.Reconfigure",
    );
}

/// The branch copies `input.sources` into `sources` and into `backup`, a plain list with the
/// invariant `backup.count >= 1`. Only invariants rooted at `sources` are evaluated over the
/// stored row, so `[]` is sent and `backup` breaks its invariant in the interpreter.
#[test]
fn adversary4_an_invariant_over_a_second_copy_of_the_list_is_kept() {
    let server = ADV_SERVER_MANY
        .replace(
            "    relations:\n",
            "      - {name: backup, type: \"List<demo.reg.SourceId>\"}\n    relations:\n",
        )
        .replace(
            "    lifecycle:",
            "    invariants: [\"backup.count >= 1\"]\n    lifecycle:",
        );
    adversary2_accepted_passes_or_is_refused_by_name(adversary_compile(&adversary_text(
        ADV_LIVE,
        &server,
        &adversary_register("List<demo.reg.SourceId>", "", "", ", backup: input.sources"),
    )));
}

/// An update leaves `note` (set `hello` by `Register`) in place, but the stored row binds only
/// what the update sets, so `note` reads absent and `not defined(note)` holds. With `open: false`
/// the real row (`note` present, `sources: []`, `open: false`) breaks
/// `any: [not defined(note), sources.count >= 1, open == true]`, yet `[]` is sent.
#[test]
fn adversary4_an_optional_field_the_update_leaves_in_place_is_not_read_absent() {
    let server = ADV_SERVER_MANY
        .replace(
            "    relations:\n",
            "      - {name: note, type: Optional<String>}\n      - {name: open, type: Boolean}\n    \
             relations:\n",
        )
        .replace(
            "    lifecycle:",
            "    invariants:\n      - any: [{not: \"defined(note)\"}, \"sources.count >= 1\", \
             open == true]\n    lifecycle:",
        );
    let register = adversary_register(
        "List<demo.reg.SourceId>",
        "",
        "",
        ", note: hello, open: \"true\"",
    );
    let reconfigure = format!(
        r#"  - name: demo.reg.Reconfigure
    input:
      - {{name: id, type: demo.reg.ServerId}}
      - {{name: sources, type: "List<demo.reg.SourceId>"}}
      - {{name: open, type: Boolean}}
    outcomes:
      - name: accepted
        when: open == false
        updates: demo.reg.Server
        instance: id
        sets: {{sources: input.sources, open: input.open}}
        emits: [demo.reg.Registered]
        payload: {{demo.reg.Registered: {{id: input.id}}}}
{ADV4_REFUSED}"#
    );
    adversary4_passes_or_refused_by_name(
        adversary_compile(&adversary_text(
            ADV_LIVE,
            &server,
            &format!("{register}\n{reconfigure}"),
        )),
        RECONFIGURED,
        "demo.reg.Reconfigure",
    );
}

/// `limit` is stored from an `Optional` input with a default (`{input: limit, else: 8}`), the
/// invariant `sources.count <= limit`: `[]` keeps it whatever `limit` holds, yet the defaulted
/// copy is not part of the stored row, the invariant is undecided and the scenario is refused.
#[test]
fn adversary4_a_defaulted_copy_bounding_the_list_keeps_the_empty_list() {
    let server = ADV_SERVER_MANY
        .replace(
            "    relations:\n",
            "      - {name: limit, type: Integer}\n    relations:\n",
        )
        .replace(
            "    lifecycle:",
            "    invariants: [\"sources.count <= limit\"]\n    lifecycle:",
        );
    let register = adversary_register(
        "List<demo.reg.SourceId>",
        "",
        "      - {name: limit, type: Optional<Integer>}",
        ", limit: {input: limit, else: 8}",
    );
    adversary4_exists_and_passes(
        adversary_compile(&adversary_text(ADV_LIVE, &server, &register)),
        ACCEPTED,
    );
}

/// The invariant `any: [state == Live, sources.count >= 1]` (a retired server names a source):
/// `Register` creates the row in `Live`, so `[]` keeps it, yet the stored row carries no state,
/// the invariant is undecided and the scenario is refused.
#[test]
fn adversary4_an_invariant_reading_the_created_state_keeps_the_empty_list() {
    let server = adversary4_retiring_server().replace(
        "    lifecycle:",
        "    invariants:\n      - any: [state == Live, \"sources.count >= 1\"]\n    lifecycle:",
    );
    adversary4_exists_and_passes(
        adversary_compile(&adversary_text(
            ADV_LIVE,
            &server,
            &format!(
                "{}\n{ADV_RETIRE}",
                adversary_register("List<demo.reg.SourceId>", "", "", "")
            ),
        )),
        ACCEPTED,
    );
}

/// Held (probe): the precedence order with an earlier accepting `bare` (`sources.count == 0`)
/// and the default `accepted`: `[]` would be claimed by `bare`, so `accepted` passes or is refused
/// by name.
#[test]
fn adversary4_a_default_branch_behind_an_accepting_count_guard_refuses_by_name() {
    let register = r#"  - name: demo.reg.Register
    input:
      - {name: id, type: demo.reg.ServerId}
      - {name: sources, type: "List<demo.reg.SourceId>"}
    outcomes:
      - name: bare
        when: sources.count == 0
        creates: demo.reg.Server
        instance: id
        sets: {sources: input.sources}
        emits: [demo.reg.Registered]
        payload: {demo.reg.Registered: {id: input.id}}
      - name: accepted
        creates: demo.reg.Server
        instance: id
        sets: {sources: input.sources}
        emits: [demo.reg.Registered]
        payload: {demo.reg.Registered: {id: input.id}}"#;
    adversary2_accepted_passes_or_is_refused_by_name(adversary_compile(&adversary_text(
        ADV_LIVE,
        ADV_SERVER_MANY,
        register,
    )));
}
