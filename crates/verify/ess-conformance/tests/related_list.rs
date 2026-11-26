//! A guard over the rows a `List` of references names (`ess/28`, beyond10x/ess#558,
//! `docs/design/generic-language-features.md` § G2, quantifiers): `when_related: {via:
//! input.<list>, exists: false}` refuses a list naming a missing row, `{missing: input.<list>}`
//! names it, and a list whose every element names a row — the empty one included — is accepted.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// An incident names the services it affects; one naming a service no row carries is refused.
const LEDGER: &str = include_str!("fixtures/list-of-references.yaml");

const OPEN: &str = "ledger.desk.OpenIncident";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(model: &EssIr) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(model);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id} among {:?}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// The list each `OpenIncident` the scenario sends carries, in step order, with the outcome or
/// error expected right after it.
fn sends(scenario: &ConformanceScenario) -> Vec<(ScenarioValue, String)> {
    let mut found = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != OPEN {
            continue;
        }
        let answer = match steps.peek() {
            Some(ScenarioStep::ExpectOutcome { outcome }) => outcome.to_string(),
            other => format!("{other:?}"),
        };
        found.push((input["services"].clone(), answer));
    }
    found
}

fn passes_on_the_interpreter(model: EssIr, suite: &ConformanceSuite) {
    let admitted = AdmittedSuite::from_suite(suite).expect("the suite is admitted");
    let run = Runner::for_suite(admitted.suite()).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(model),
    );
    assert_ne!(run.scenarios.len(), 0);
    let failed: Vec<_> = run
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn a_missing_list_element_takes_the_refusal() {
    let model = ir(LEDGER);
    let suite = suite(&model);
    let refused = scenario(&suite, "ledger.desk.OpenIncident/outcome/unknown-service");
    let sent = sends(refused);
    let Some((ScenarioValue::List { items }, _)) = sent.last() else {
        panic!("the refusal sends a list: {sent:#?}");
    };
    let [ScenarioValue::Instance { .. }, ScenarioValue::Literal { value: missing }] =
        items.as_slice()
    else {
        panic!("a created row first, then an identity no row carries: {items:#?}");
    };
    let asserted: Vec<BTreeMap<String, Node>> = refused
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectError { fields, .. } => Some(fields.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        asserted,
        vec![BTreeMap::from([("service_id".to_owned(), missing.clone())])],
        "the refusal names the second element, the one naming no row"
    );
    passes_on_the_interpreter(model, &suite);
}

#[test]
fn every_element_present_takes_the_accepting_branch() {
    let model = ir(LEDGER);
    let suite = suite(&model);
    let opened = scenario(&suite, "ledger.desk.OpenIncident/outcome/opened");
    let sent = sends(opened);
    let Some((ScenarioValue::List { items }, answer)) = sent.first() else {
        panic!("the accepting branch sends a list: {sent:#?}");
    };
    assert!(
        items.len() == 2
            && items
                .iter()
                .all(|item| matches!(item, ScenarioValue::Instance { .. })),
        "two rows the scenario created: {items:#?}"
    );
    assert_eq!(answer, "ledger.desk.OpenIncident/opened");
    passes_on_the_interpreter(model, &suite);
}

#[test]
fn an_empty_list_takes_the_accepting_branch() {
    let model = ir(LEDGER);
    let suite = suite(&model);
    let empty = ScenarioValue::Literal {
        value: Node::Seq(Vec::new()),
    };
    let witnessed: Vec<String> = suite
        .scenarios
        .iter()
        .filter(|(_, scenario)| {
            sends(scenario)
                .iter()
                .any(|(list, answer)| *list == empty && answer == "ledger.desk.OpenIncident/opened")
        })
        .map(|(id, _)| id.to_string())
        .collect();
    assert!(
        !witnessed.is_empty(),
        "some scenario sends an empty list and requires it accepted: {:#?}",
        suite
            .scenarios
            .iter()
            .map(|(id, scenario)| (id.to_string(), sends(scenario)))
            .collect::<Vec<_>>()
    );
    passes_on_the_interpreter(model, &suite);
}

/// From `ess/38` (beyond10x/ess#640) a `when_related:` guard names its row through a member of a
/// struct input: synthesis sends the arranged identity inside the struct, and the interpreted
/// target reads it there.
mod struct_input_member {
    use std::collections::{BTreeMap, BTreeSet};

    use ess_conformance::interpret::Interpreted;
    use ess_conformance::report::Status;
    use ess_conformance::synthesize::Synthesis;
    use ess_conformance::target::*;
    use ess_conformance::{
        AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue,
    };
    use ess_primitives::node::Node;

    /// A shipment names its depots, its primary, its backup and an optional spare inside its
    /// settings.
    const ROUTE: &str = include_str!("fixtures/struct-input-member-related-rows.yaml.in");

    const SEND: &str = "shop.route.Send";

    fn synthesized() -> (ess_compiler::ir::EssIr, Synthesis) {
        let model = super::ir(ROUTE);
        let synthesis = ess_conformance::synthesize::synthesize(&model);
        (model, synthesis)
    }

    /// The refusals under `shop.route.`, printed.
    fn refused(synthesis: &Synthesis) -> Vec<String> {
        synthesis
            .refusals
            .iter()
            .filter(|refusal| format!("{refusal:?}").contains("shop.route."))
            .map(|refusal| {
                format!(
                    "{} [{}]: {}",
                    refusal.cause.code(),
                    refusal
                        .scenario
                        .as_ref()
                        .map_or_else(|| format!("{:?}", refusal.subject), ToString::to_string),
                    refusal.cause
                )
            })
            .collect()
    }

    /// The input of each run of `command` the scenario sends, in step order, with the outcome or
    /// error expected right after it.
    fn sent(
        scenario: &ConformanceScenario,
        command: &str,
    ) -> Vec<(BTreeMap<String, ScenarioValue>, String)> {
        let mut found = Vec::new();
        let mut steps = scenario.steps.iter().peekable();
        while let Some(step) = steps.next() {
            let ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } = step
            else {
                continue;
            };
            if sent.to_string() != command {
                continue;
            }
            let answer = match steps.peek() {
                Some(ScenarioStep::ExpectOutcome { outcome }) => outcome.to_string(),
                Some(ScenarioStep::ExpectError { error, .. }) => error.to_string(),
                other => format!("{other:?}"),
            };
            found.push((input.clone(), answer));
        }
        found
    }

    /// The last run of `command` the scenario `id` sends.
    fn last_sent(
        synthesis: &Synthesis,
        id: &str,
        command: &str,
    ) -> BTreeMap<String, ScenarioValue> {
        let scenario = super::scenario(&synthesis.suite, id);
        let runs = sent(scenario, command);
        let Some((input, _)) = runs.last() else {
            panic!("{id} sends {command}: {:#?}", scenario.steps);
        };
        assert!(
            input.keys().all(|key| !key.contains('.')),
            "{id}: no input is keyed by a path: {input:#?}"
        );
        input.clone()
    }

    /// The value at the dotted `path` of a sent input: through `Members`, or through a literal
    /// mapping.
    fn member(input: &BTreeMap<String, ScenarioValue>, path: &str) -> Option<ScenarioValue> {
        let mut segments = path.split('.');
        let mut value = input.get(segments.next()?)?.clone();
        for segment in segments {
            value = match value {
                ScenarioValue::Members { members } => members.get(segment)?.clone(),
                ScenarioValue::Literal { value } => ScenarioValue::Literal {
                    value: value.as_map()?.get(segment)?.clone(),
                },
                _ => return None,
            };
        }
        Some(value)
    }

    #[test]
    fn a_list_member_naming_a_missing_row_takes_the_refusal_and_names_it() {
        let (_, synthesis) = synthesized();
        let id = "shop.route.Send/outcome/unknown-depot";
        let refusals = refused(&synthesis);
        assert!(
            refusals.iter().all(|refusal| !refusal.contains(SEND)),
            "{refusals:#?}"
        );
        let input = last_sent(&synthesis, id, SEND);
        assert!(
            matches!(input.get("settings"), Some(ScenarioValue::Members { .. })),
            "the settings carry an arranged row: {input:#?}"
        );
        let Some(ScenarioValue::List { items }) = member(&input, "settings.depots") else {
            panic!("the depots inside the settings are a list of values: {input:#?}");
        };
        let [ScenarioValue::Instance { .. }, ScenarioValue::Literal { value: missing }] =
            items.as_slice()
        else {
            panic!("a created row first, then an identity no row carries: {items:#?}");
        };
        let asserted: Vec<BTreeMap<String, Node>> = super::scenario(&synthesis.suite, id)
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::ExpectError { fields, .. } => Some(fields.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(
            asserted,
            vec![BTreeMap::from([("depot_id".to_owned(), missing.clone())])],
            "the refusal names the second element, the one naming no row"
        );
    }

    #[test]
    fn every_element_present_takes_the_accepting_branch() {
        let (_, synthesis) = synthesized();
        // The branch's own run; the run after its assertions sends the empty list.
        let runs = sent(
            super::scenario(&synthesis.suite, "shop.route.Send/outcome/sent"),
            SEND,
        );
        let Some((input, _)) = runs.first() else {
            panic!("the accepting branch sends {SEND}");
        };
        assert!(
            input.keys().all(|key| !key.contains('.')),
            "no input is keyed by a path: {input:#?}"
        );
        let Some(ScenarioValue::List { items }) = member(input, "settings.depots") else {
            panic!("the depots inside the settings are a list of values: {input:#?}");
        };
        assert!(
            !items.is_empty()
                && items
                    .iter()
                    .all(|item| matches!(item, ScenarioValue::Instance { .. })),
            "every depot sent names an arranged row: {items:#?}"
        );
    }

    #[test]
    fn a_scalar_member_names_the_arranged_row() {
        let (_, synthesis) = synthesized();
        for id in [
            "shop.route.Check/outcome/closed",
            "shop.route.Check/outcome/checked",
        ] {
            let input = last_sent(&synthesis, id, "shop.route.Check");
            assert!(
                matches!(
                    member(&input, "settings.primary"),
                    Some(ScenarioValue::Instance { .. })
                ),
                "{id} sends the arranged depot inside the settings: {input:#?}"
            );
        }
        let input = last_sent(
            &synthesis,
            "shop.route.Check/outcome/no-depot",
            "shop.route.Check",
        );
        assert!(
            matches!(
                member(&input, "settings.primary"),
                Some(ScenarioValue::Literal { .. })
            ),
            "the missing depot is an identity no row carries: {input:#?}"
        );
        let input = last_sent(
            &synthesis,
            "shop.route.Pair/outcome/both-closed",
            "shop.route.Pair",
        );
        for path in ["settings.primary", "settings.backup"] {
            assert!(
                matches!(member(&input, path), Some(ScenarioValue::Instance { .. })),
                "both rows of the `all:` branch are arranged, {path}: {input:#?}"
            );
        }
    }

    #[test]
    fn a_route_through_an_optional_member_is_left_out_for_its_absence() {
        let (_, synthesis) = synthesized();
        let probes: Vec<_> = synthesis
            .suite
            .scenarios
            .values()
            .flat_map(|scenario| sent(scenario, "shop.route.Probe"))
            .collect();
        assert!(
            probes.iter().any(|(input, answer)| {
                answer == "shop.route.Probe/probed"
                    && member(input, "settings").is_some()
                    && member(input, "settings.extra").is_none()
            }),
            "a probe without the optional `extra` is accepted: {probes:#?}"
        );
        let input = last_sent(
            &synthesis,
            "shop.route.Probe/outcome/no-spare",
            "shop.route.Probe",
        );
        assert!(
            matches!(
                member(&input, "settings.extra.spare"),
                Some(ScenarioValue::Literal { .. })
            ),
            "the missing spare is an identity no row carries: {input:#?}"
        );
    }

    #[test]
    fn the_suite_has_no_refusals_and_passes_on_the_interpreter() {
        let (model, synthesis) = synthesized();
        let refusals = refused(&synthesis);
        assert!(refusals.is_empty(), "{refusals:#?}");
        super::passes_on_the_interpreter(model, &synthesis.suite);
    }

    #[test]
    fn a_target_reading_a_top_level_list_fails_the_missing_row() {
        let (model, synthesis) = synthesized();
        let admitted =
            AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
        let statuses: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &TopLevelDepots(Interpreted::for_model(model)))
            .into_report()
            .scenarios
            .into_iter()
            .map(|run| (run.scenario.to_string(), run.status))
            .collect();
        let id = "shop.route.Send/outcome/unknown-depot";
        assert!(
            statuses
                .get(id)
                .is_some_and(|status| *status != Status::Passed),
            "a target reading a top-level `depots` fails {id}: {statuses:#?}"
        );
    }

    /// A target that reads the depots from a top-level `depots`, which the input does not carry,
    /// in place of `settings.depots`.
    struct TopLevelDepots(Interpreted);

    impl ConformanceTarget for TopLevelDepots {
        fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
            self.0.identity()
        }
        fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
            self.0.begin_scenario(context)
        }
        fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
            self.0.end_scenario(context)
        }
        fn execute_command(
            &self,
            mut request: SemanticCommandRequest,
        ) -> Result<SemanticCommandResult, TargetError> {
            if request.command.to_string() == SEND {
                let top = request
                    .input
                    .get("depots")
                    .cloned()
                    .unwrap_or(Node::Seq(Vec::new()));
                if let Some(Node::Map(settings)) = request.input.get_mut("settings") {
                    settings.insert("depots".to_owned(), top);
                }
            }
            self.0.execute_command(request)
        }
        fn query_view(
            &self,
            request: SemanticViewRequest,
        ) -> Result<SemanticViewResult, TargetError> {
            self.0.query_view(request)
        }
        fn observe_events(
            &self,
            request: EventObservationRequest,
        ) -> Result<Vec<ObservedEvent>, TargetError> {
            self.0.observe_events(request)
        }
        fn configure_external_outcome(
            &self,
            control: ExternalOutcomeControl,
        ) -> Result<(), TargetError> {
            self.0.configure_external_outcome(control)
        }
        fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
            self.0.redeliver_event(request)
        }
    }

    #[test]
    fn every_branch_of_the_model_has_its_scenario() {
        let (model, synthesis) = synthesized();
        let declared: BTreeSet<String> = model
            .commands()
            .values()
            .filter(|command| command.name.to_string().starts_with("shop.route."))
            .flat_map(|command| {
                command
                    .outcomes
                    .iter()
                    .map(move |outcome| format!("{}/outcome/{}", command.name, outcome.name))
            })
            .collect();
        let ids: BTreeSet<String> = synthesis
            .suite
            .scenarios
            .keys()
            .map(ToString::to_string)
            .collect();
        let missing: Vec<_> = declared.difference(&ids).collect();
        assert!(
            missing.is_empty(),
            "branches with no scenario: {missing:#?}"
        );
    }
}
