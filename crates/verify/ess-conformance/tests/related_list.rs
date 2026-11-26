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

/// From `ess/38` (beyond10x/ess#640) an `instance:` and an `each: {in:}` name a member of a struct
/// input: synthesis sends the arranged row, or a fresh identity for `unknown_instance:`, inside the
/// struct, the interpreted target reads it there, and the suite format does not move.
mod struct_input_member_addressed {
    use std::collections::BTreeMap;

    use ess_compiler::ir::EssIr;
    use ess_conformance::synthesize::Synthesis;
    use ess_conformance::{ConformanceScenario, ScenarioStep, ScenarioValue};

    /// A request names the parcel it acts on inside a struct; a labelling names its parcels in a
    /// list inside its batch.
    const PARCEL: &str = include_str!("fixtures/struct-input-member-addressed-rows.yaml.in");

    /// A route's settings name its depots, a primary and a backup (s1's model).
    const ROUTE: &str = include_str!("fixtures/struct-input-member-related-rows.yaml.in");

    /// A check reads the bin its settings name first, and a note updates that bin.
    const STOCK: &str = include_str!("fixtures/struct-input-member-served.yaml.in");

    fn synthesized(text: &str) -> (EssIr, Synthesis) {
        let model = super::ir(text);
        let synthesis = ess_conformance::synthesize::synthesize(&model);
        (model, synthesis)
    }

    /// The refusals under `prefix`, printed.
    fn refused(synthesis: &Synthesis, prefix: &str) -> Vec<String> {
        synthesis
            .refusals
            .iter()
            .filter(|refusal| format!("{refusal:?}").contains(prefix))
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

    /// The last run of the scenario's own command, `<command>/outcome/<name>`, with its answer;
    /// the refusals under the command printed when there is no such scenario.
    fn last_run(synthesis: &Synthesis, id: &str) -> (BTreeMap<String, ScenarioValue>, String) {
        let command = id.split('/').next().unwrap_or(id);
        let Some(scenario) = synthesis
            .suite
            .scenarios
            .iter()
            .find(|(key, _)| key.to_string() == id)
            .map(|(_, scenario)| scenario)
        else {
            panic!(
                "no scenario {id}; refused: {:#?}",
                refused(synthesis, command)
            );
        };
        let runs = sent(scenario, command);
        let Some(last) = runs.last() else {
            panic!("{id} sends {command}: {:#?}", scenario.steps);
        };
        assert!(
            last.0.keys().all(|key| !key.contains('.')),
            "{id}: no input is keyed by a path: {:#?}",
            last.0
        );
        last.clone()
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

    /// `id`'s last run sends the arranged row at `path`, inside a `Members` struct.
    fn sends_the_arranged_row(synthesis: &Synthesis, id: &str, path: &str) {
        let (input, _) = last_run(synthesis, id);
        let root = path.split('.').next().unwrap_or(path);
        assert!(
            matches!(input.get(root), Some(ScenarioValue::Members { .. })),
            "{id}: `{root}` carries an arranged row: {input:#?}"
        );
        assert!(
            matches!(member(&input, path), Some(ScenarioValue::Instance { .. })),
            "{id}: the arranged row is sent at `{path}`: {input:#?}"
        );
    }

    /// `id`'s last run sends at `path` an identity no row carries, and expects `error`.
    fn sends_a_fresh_identity(synthesis: &Synthesis, id: &str, path: &str, error: &str) {
        let (input, _) = last_run(synthesis, id);
        assert!(
            matches!(member(&input, path), Some(ScenarioValue::Literal { .. })),
            "{id}: a fresh identity is sent at `{path}`: {input:#?}"
        );
        let expected = errors(synthesis, id);
        assert!(
            expected.iter().any(|expected| expected == error),
            "{id} expects `{error}`: {expected:?}"
        );
    }

    /// The errors the scenario `id` expects, in step order.
    fn errors(synthesis: &Synthesis, id: &str) -> Vec<String> {
        super::scenario(&synthesis.suite, id)
            .steps
            .iter()
            .filter_map(|step| match step {
                ScenarioStep::ExpectError { error, .. } => Some(error.to_string()),
                _ => None,
            })
            .collect()
    }

    /// Every run of `command` in the suite that the scenario expects to answer `outcome`.
    fn runs_answering(
        synthesis: &Synthesis,
        command: &str,
        outcome: &str,
    ) -> Vec<(String, BTreeMap<String, ScenarioValue>)> {
        synthesis
            .suite
            .scenarios
            .iter()
            .flat_map(|(id, scenario)| {
                sent(scenario, command)
                    .into_iter()
                    .filter(|(_, answer)| answer == outcome)
                    .map(move |(input, _)| (id.to_string(), input))
            })
            .collect()
    }

    /// `Hold`'s `wrong_state:` branch, which the illegal-move family witnesses, is sent the
    /// arranged parcel inside `request`.
    fn a_held_parcel_is_named_inside_the_request(synthesis: &Synthesis) {
        let runs = runs_answering(synthesis, "shop.parcel.Hold", "shop.parcel.Hold/not-open");
        assert!(!runs.is_empty(), "a run answers `not-open`");
        for (id, input) in &runs {
            assert!(
                matches!(
                    member(input, "request.parcel_id"),
                    Some(ScenarioValue::Instance { .. })
                ),
                "{id}: the arranged parcel is sent inside `request`: {input:#?}"
            );
        }
    }

    /// `Register/outcome/taken` sends at `target.parcel_id` the identity an earlier `Register` of
    /// the same scenario created its row with, answered `registered`, and expects `Taken`.
    fn the_taken_row_is_named_inside_the_target(synthesis: &Synthesis) {
        const REGISTER: &str = "shop.parcel.Register";
        let id = "shop.parcel.Register/outcome/taken";
        let runs = sent(super::scenario(&synthesis.suite, id), REGISTER);
        let Some(at) = runs
            .iter()
            .position(|(_, answer)| answer == "shop.parcel.Register/taken")
        else {
            panic!("{id} sends {REGISTER} answered `taken`: {runs:#?}");
        };
        let (again, _) = &runs[at];
        let Some((created, _)) = runs[..at]
            .iter()
            .rev()
            .find(|(_, answer)| answer == "shop.parcel.Register/registered")
        else {
            panic!("{id}: a `registered` send comes first: {runs:#?}");
        };
        let identity = member(created, "target.parcel_id");
        assert!(
            matches!(identity, Some(ScenarioValue::Literal { .. }))
                && member(again, "target.parcel_id") == identity,
            "{id}: the second send names the row the first created, inside `target`: {runs:#?}"
        );
        assert!(
            again.keys().all(|key| !key.contains('.')),
            "{id}: no input is keyed by a path: {again:#?}"
        );
        assert_eq!(errors(synthesis, id), vec!["shop.parcel.Taken".to_owned()]);
    }

    const UNKNOWN_PARCEL: &str = "shop.parcel.UnknownParcel";

    /// Test 1: `Relabel` (`updates:`) sends the arranged parcel inside `target`, and a fresh
    /// identity there for `unknown_instance:`.
    fn the_update_names_its_rows(synthesis: &Synthesis) {
        sends_the_arranged_row(
            synthesis,
            "shop.parcel.Relabel/outcome/relabelled",
            "target.parcel_id",
        );
        sends_a_fresh_identity(
            synthesis,
            "shop.parcel.Relabel/outcome/unknown",
            "target.parcel_id",
            UNKNOWN_PARCEL,
        );
    }

    /// Test 2: `Hold` (`moves:`) through the required struct input `request`.
    fn the_move_names_its_rows(synthesis: &Synthesis) {
        sends_the_arranged_row(
            synthesis,
            "shop.parcel.Hold/outcome/held",
            "request.parcel_id",
        );
        a_held_parcel_is_named_inside_the_request(synthesis);
        sends_a_fresh_identity(
            synthesis,
            "shop.parcel.Hold/outcome/unknown",
            "request.parcel_id",
            UNKNOWN_PARCEL,
        );
    }

    /// Test 3: `Drop` (`deletes:`).
    fn the_delete_names_its_rows(synthesis: &Synthesis) {
        sends_the_arranged_row(
            synthesis,
            "shop.parcel.Drop/outcome/dropped",
            "target.parcel_id",
        );
        sends_a_fresh_identity(
            synthesis,
            "shop.parcel.Drop/outcome/unknown",
            "target.parcel_id",
            UNKNOWN_PARCEL,
        );
    }

    /// Test 4: `Register` (`existing_instance:`).
    fn the_existing_instance_names_its_row(synthesis: &Synthesis) {
        the_taken_row_is_named_inside_the_target(synthesis);
        let (input, _) = last_run(synthesis, "shop.parcel.Register/outcome/registered");
        assert!(
            matches!(
                member(&input, "target.parcel_id"),
                Some(ScenarioValue::Literal { .. })
            ),
            "a registration sends an identity no row carries: {input:#?}"
        );
    }

    /// Tests 1-4's statements on `PARCEL`, for every `updates:`, `moves:`, `deletes:` and
    /// `existing_instance:` branch.
    fn every_instance_branch_names_its_row(synthesis: &Synthesis) {
        the_update_names_its_rows(synthesis);
        the_move_names_its_rows(synthesis);
        the_delete_names_its_rows(synthesis);
        the_existing_instance_names_its_row(synthesis);
    }

    /// `PARCEL` with the labelled list moved to the top-level input `lines`.
    fn parcel_control() -> String {
        let line = "      - {name: batch, type: shop.parcel.Batch}\n";
        let control = PARCEL
            .replacen(
                line,
                &format!("{line}      - {{name: lines, type: \"List<shop.parcel.Line>\"}}\n"),
                1,
            )
            .replacen("in: input.batch.lines", "in: input.lines", 1);
        assert!(
            control.contains("name: lines, type: \"List<shop.parcel.Line>\"}\n")
                && control.contains("in: input.lines,"),
            "{control}"
        );
        control
    }

    /// `ROUTE` with `Send`'s depots moved to the top-level input `depots`.
    fn route_control() -> String {
        let send = "  - name: shop.route.Send\n    input:\n      - {name: settings, type: shop.route.Settings}\n";
        let control = ROUTE
            .replacen(
                send,
                &format!("{send}      - {{name: depots, type: \"List<shop.route.DepotId>\"}}\n"),
                1,
            )
            .replace("input.settings.depots", "input.depots");
        assert!(
            control.contains("{name: depots, type: \"List<shop.route.DepotId>\"}\n    outcomes:")
                && !control.contains("input.settings.depots"),
            "{control}"
        );
        control
    }

    /// The `ExecuteCommand` inputs of `command` across every scenario whose id starts with it.
    fn label_runs(
        synthesis: &Synthesis,
        command: &str,
    ) -> BTreeMap<String, Vec<BTreeMap<String, ScenarioValue>>> {
        synthesis
            .suite
            .scenarios
            .iter()
            .filter(|(id, _)| id.to_string().starts_with(command))
            .map(|(id, scenario)| {
                (
                    id.to_string(),
                    sent(scenario, command)
                        .into_iter()
                        .map(|(input, _)| input)
                        .collect(),
                )
            })
            .collect()
    }

    #[test]
    fn an_update_through_a_member_names_the_arranged_row_and_a_fresh_one() {
        the_update_names_its_rows(&synthesized(PARCEL).1);
    }

    #[test]
    fn a_move_through_a_struct_member_names_the_arranged_row() {
        the_move_names_its_rows(&synthesized(PARCEL).1);
    }

    #[test]
    fn a_delete_through_a_member_names_the_arranged_row() {
        the_delete_names_its_rows(&synthesized(PARCEL).1);
    }

    #[test]
    fn existing_instance_through_a_member_names_the_taken_row() {
        the_existing_instance_names_its_row(&synthesized(PARCEL).1);
    }

    #[test]
    fn an_each_over_a_list_member_is_witnessed_as_a_top_level_each_is() {
        const LABEL: &str = "shop.parcel.Label";
        let (_, path) = synthesized(PARCEL);
        let (_, control) = synthesized(&parcel_control());
        let refusals = refused(&path, LABEL);
        assert!(refusals.is_empty(), "{refusals:#?}");
        let path_runs = label_runs(&path, LABEL);
        let control_runs = label_runs(&control, LABEL);
        assert_eq!(
            path_runs.keys().collect::<Vec<_>>(),
            control_runs.keys().collect::<Vec<_>>(),
            "the same `Label` scenarios"
        );
        for (id, runs) in &path_runs {
            let controls = &control_runs[id];
            assert_eq!(runs.len(), controls.len(), "{id}: {runs:#?} {controls:#?}");
            for (sent, control) in runs.iter().zip(controls) {
                assert_eq!(
                    member(sent, "batch.lines"),
                    control.get("lines").cloned(),
                    "{id}: `batch.lines` carries what the control sends at `lines`"
                );
            }
        }
    }

    #[test]
    fn the_addressed_suite_has_no_refusals_and_passes_on_the_interpreter() {
        let (model, synthesis) = synthesized(PARCEL);
        let refusals = refused(&synthesis, "shop.parcel.");
        assert!(refusals.is_empty(), "{refusals:#?}");
        super::passes_on_the_interpreter(model, &synthesis.suite);
    }

    #[test]
    fn parent_acceptance_one_holds_on_both_models() {
        let (_, route) = synthesized(ROUTE);
        let refusals = refused(&route, "shop.route.");
        assert!(refusals.is_empty(), "{refusals:#?}");
        let (input, _) = last_run(&route, "shop.route.Send/outcome/unknown-depot");
        let Some(ScenarioValue::List { items }) = member(&input, "settings.depots") else {
            panic!("the depots inside the settings are a list of values: {input:#?}");
        };
        assert!(
            matches!(
                items.as_slice(),
                [
                    ScenarioValue::Instance { .. },
                    ScenarioValue::Literal { .. }
                ]
            ),
            "a created row first, then an identity no row carries: {items:#?}"
        );
        let runs = sent(
            super::scenario(&route.suite, "shop.route.Send/outcome/sent"),
            "shop.route.Send",
        );
        let Some((input, _)) = runs.first() else {
            panic!("the accepting branch sends shop.route.Send");
        };
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
        let (_, parcel) = synthesized(PARCEL);
        every_instance_branch_names_its_row(&parcel);
    }

    #[test]
    fn no_suite_format_moves() {
        let pairs = [
            (PARCEL, parcel_control(), "shop.parcel."),
            (ROUTE, route_control(), "shop.route."),
        ];
        for (text, control, prefix) in &pairs {
            let (_, synthesis) = synthesized(text);
            let (_, control) = synthesized(control);
            let refusals = refused(&synthesis, prefix);
            assert!(refusals.is_empty(), "{refusals:#?}");
            assert_eq!(
                synthesis.suite.provenance.suite_version.major(),
                control.suite.provenance.suite_version.major(),
                "{prefix}: the member address moves no suite major"
            );
            for (id, scenario) in &synthesis.suite.scenarios {
                for step in &scenario.steps {
                    if let ScenarioStep::ExecuteCommand { input, .. } = step {
                        assert!(
                            input.keys().all(|key| !key.contains('.')),
                            "{id}: no input is keyed by a path: {input:#?}"
                        );
                    }
                }
            }
        }
        assert_eq!(
            ess_conformance::scenario::SUPPORTED_SUITE_FORMATS.last(),
            Some(&67),
            "no suite major was added for `ess/38`"
        );
    }

    #[test]
    fn the_minimal_model_synthesizes_without_refusals_and_passes_on_the_interpreter() {
        let (model, synthesis) = synthesized(STOCK);
        let refusals = refused(&synthesis, "shop.stock.");
        assert!(refusals.is_empty(), "{refusals:#?}");
        sends_the_arranged_row(
            &synthesis,
            "shop.stock.Note/outcome/noted",
            "settings.primary",
        );
        for path in ["settings.primary", "settings.backup"] {
            sends_the_arranged_row(&synthesis, "shop.stock.Note/outcome/backup-closed", path);
        }
        for id in [
            "shop.stock.Check/outcome/checked",
            "shop.stock.Check/outcome/closed",
        ] {
            sends_the_arranged_row(&synthesis, id, "settings.primary");
        }
        super::passes_on_the_interpreter(model, &synthesis.suite);
    }
}
