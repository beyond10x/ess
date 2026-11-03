//! A view an actor's `may:` names is read-granted, witnessed per view (beyond10x/ess#286).
//!
//! Where the model serves a component (`reached_by: network`), synthesis files
//! `<view>/grant/read/denied` for each read-granted view some declared actor lacks the grant for:
//! the view read as that actor, and the standard refusal required. Each actor granted the view
//! reads it in `<view>/grant/read/admitted/<actor>`, and every other read of it in the suite is
//! sent as an actor granted it. A view no actor names stays open and owes none of this. A target
//! that serves a read no grant admits fails exactly the denied scenarios.

use std::collections::BTreeSet;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::SuiteFormat;
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioId, ScenarioStep};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("../../../../docs/design/view-grants.example.yaml");

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn read_grant_ids(suite: &ConformanceSuite) -> BTreeSet<String> {
    suite
        .scenarios
        .keys()
        .filter(|id| {
            matches!(
                id,
                ScenarioId::ReadGrant { .. } | ScenarioId::ReadGrantAdmitted { .. }
            )
        })
        .map(ToString::to_string)
        .collect()
}

#[test]
fn a_read_granted_view_owes_a_denied_read_and_an_admitted_read_per_granted_actor() {
    let synthesis = ess_conformance::synthesize(&compiled(MODEL));
    assert_eq!(
        read_grant_ids(&synthesis.suite),
        [
            "desk.tickets.Board/grant/read/admitted/desk.tickets.Clerk",
            "desk.tickets.Board/grant/read/denied",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>(),
        "`Titles` is named by no actor, so it is open and owes neither"
    );
    let denied: ScenarioId = "desk.tickets.Board/grant/read/denied".parse().unwrap();
    let scenario = synthesis.suite.scenario(&denied).expect("filed");
    let refused = scenario
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ExpectNotGranted { .. }))
        .expect("the refusal is required");
    assert!(
        matches!(&scenario.steps[refused - 1], ScenarioStep::QueryView { view, .. }
            if view.to_string() == "desk.tickets.Board"),
        "the refused read comes just before: {:?}",
        scenario.steps
    );
    assert!(
        matches!(&scenario.steps[refused - 2], ScenarioStep::ReadAs { actor: Some(actor) }
            if actor.to_string() == "desk.tickets.Watcher"),
        "read as the actor no grant admits: {:?}",
        scenario.steps
    );
    assert!(
        matches!(&scenario.steps[refused], ScenarioStep::ExpectNotGranted { actor: Some(actor), .. }
            if actor.to_string() == "desk.tickets.Watcher"),
        "{:?}",
        scenario.steps
    );
    // Then the same read as no actor at all, refused naming none (security review F2).
    assert!(
        matches!(
            &scenario.steps[refused + 1..],
            [
                ScenarioStep::ReadAs { actor: None },
                ScenarioStep::QueryView { view, .. },
                ScenarioStep::ExpectNotGranted { actor: None, .. },
            ] if view.to_string() == "desk.tickets.Board"
        ),
        "the read as no actor follows: {:?}",
        scenario.steps
    );
}

/// A view every declared actor may read still owes the read as no actor (security review F2).
#[test]
fn a_view_every_actor_may_read_is_still_refused_to_no_actor() {
    let every = MODEL.replace(
        "    may:\n      - desk.tickets.OpenTicket\ncomponents:",
        "    may:\n      - desk.tickets.OpenTicket\n      - desk.tickets.Board\ncomponents:",
    );
    assert_ne!(every, MODEL, "the Watcher is granted the Board too");
    let synthesis = ess_conformance::synthesize(&compiled(&every));
    let denied: ScenarioId = "desk.tickets.Board/grant/read/denied".parse().unwrap();
    let scenario = synthesis.suite.scenario(&denied).expect("still filed");
    assert!(
        matches!(
            scenario.steps.as_slice(),
            [
                ScenarioStep::ReadAs { actor: None },
                ScenarioStep::QueryView { .. },
                ScenarioStep::ExpectNotGranted { actor: None, .. },
            ]
        ),
        "{:?}",
        scenario.steps
    );
}

/// The reader each read of `view` in `scenario` is sent as.
fn readers_of(scenario: &ess_conformance::ConformanceScenario, view: &str) -> Vec<Option<String>> {
    let mut reader = None;
    let mut out = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ReadAs { actor } => reader = actor.as_ref().map(ToString::to_string),
            ScenarioStep::QueryView { view: read, .. }
            | ScenarioStep::EventuallyView { view: read, .. }
                if read.to_string() == view =>
            {
                out.push(reader.clone());
            }
            _ => {}
        }
    }
    out
}

#[test]
fn every_other_read_of_a_read_granted_view_is_sent_as_an_actor_granted_it() {
    let synthesis = ess_conformance::synthesize(&compiled(MODEL));
    let mut reads = 0;
    for (id, scenario) in &synthesis.suite.scenarios {
        if matches!(id, ScenarioId::ReadGrant { .. }) {
            continue;
        }
        for reader in readers_of(scenario, "desk.tickets.Board") {
            reads += 1;
            assert_eq!(
                reader.as_deref(),
                Some("desk.tickets.Clerk"),
                "`{id}` reads the Board: {:?}",
                scenario.steps
            );
        }
    }
    assert!(reads >= 2, "ordinary scenarios read the Board too: {reads}");
}

#[test]
fn the_interpreter_passes_every_scenario_of_a_model_with_a_read_grant() {
    let ir = compiled(MODEL);
    let suite = ess_conformance::synthesize(&ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    let report = Runner::for_suite(&suite).run_admitted(&admitted, &Interpreted::for_model(ir));
    let failures: Vec<_> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .collect();
    assert_eq!(failures.len(), 0, "{failures:#?}");
    assert!(report
        .scenarios
        .iter()
        .any(|scenario| scenario.scenario.to_string() == "desk.tickets.Board/grant/read/denied"));
}

/// The interpreter, serving every read whoever it is read as: a surface that checks no read grant.
struct ServesEveryRead(Interpreted);

impl ConformanceTarget for ServesEveryRead {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("serves-every-read", "1"))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
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
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
}

#[test]
fn a_target_that_serves_a_denied_read_fails_exactly_the_denied_scenario() {
    let ir = compiled(MODEL);
    let suite = ess_conformance::synthesize(&ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &ServesEveryRead(Interpreted::for_model(ir)));
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    assert_eq!(failed, ["desk.tickets.Board/grant/read/denied"]);
}

#[test]
fn a_model_that_names_no_view_keeps_its_suite() {
    let open = MODEL.replace("      - desk.tickets.Board\n", "");
    let synthesis = ess_conformance::synthesize(&compiled(&open));
    assert_eq!(read_grant_ids(&synthesis.suite).len(), 0);
    let json = synthesis.suite.to_canonical_json().expect("admits");
    assert!(!json.contains("read_as"), "{json}");
    assert!(!json.contains("/grant/read/"), "{json}");
}

#[test]
fn a_read_grant_id_round_trips_through_its_rendered_form() {
    for rendered in [
        "desk.tickets.Board/grant/read/denied",
        "desk.tickets.Board/grant/read/admitted/desk.tickets.Clerk",
    ] {
        let id: ScenarioId = rendered.parse().expect("parses");
        assert!(
            matches!(
                id,
                ScenarioId::ReadGrant { .. } | ScenarioId::ReadGrantAdmitted { .. }
            ),
            "{id:?}"
        );
        assert_eq!(id.to_string(), rendered);
    }
}

#[test]
fn a_suite_pinned_below_the_read_grant_format_refuses_a_read_as() {
    let mut suite = ess_conformance::synthesize(&compiled(MODEL)).suite;
    suite.provenance.suite_version = SuiteFormat::parse("ess-conformance/33").expect("a format");
    let error = suite
        .to_canonical_json()
        .expect_err("suite/33 has no read grants");
    assert!(error.to_string().contains("suite/34"), "{error}");
}

/// Two domains, each with a `Client` neither of which the view `demo.tasks.AllTasks` names, and a
/// granted `demo.tasks.Agent` (a2a wave-1 gaps, M6). [`two_domains`] compiles them.
const TWO_SYSTEM: &str = "format: ess/32
system: demo
version: v1
domains:
  - demo.public
  - demo.tasks
";
const TWO_COMPONENTS: &str = "components:
  - component: demo-service
    owns: {domains: [demo.public, demo.tasks]}
    accepts: {commands: [demo.tasks.OpenTask]}
    publishes: {events: [demo.tasks.TaskOpened]}
    reached_by: network
";
const TWO_PUBLIC: &str = "domain: demo.public

actors:
  - name: demo.public.Client
    may: [demo.tasks.OpenTask]
";
const TWO_TASKS: &str = "domain: demo.tasks

entities:
  - name: demo.tasks.Task
    identity: {name: id, type: String}
    fields:
      - {name: title, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}

events:
  - name: demo.tasks.TaskOpened
    fields:
      - {name: id, type: String}

commands:
  - name: demo.tasks.OpenTask
    input:
      - {name: id, type: String}
      - {name: title, type: String}
    outcomes:
      - name: opened
        creates: demo.tasks.Task
        instance: id
        sets: {title: input.title}
        emits: [demo.tasks.TaskOpened]
        payload:
          demo.tasks.TaskOpened: {id: input.id}

views:
  - name: demo.tasks.AllTasks
    source: demo.tasks.Task
    consistency: read_your_writes
    fields:
      - {name: id, type: String}
      - {name: title, type: String}

actors:
  - name: demo.tasks.Agent
    may: [demo.tasks.OpenTask, demo.tasks.AllTasks]
  - name: demo.tasks.Client
    may: [demo.tasks.OpenTask]
";

const ALL_TASKS_DENIED: &str = "demo.tasks.AllTasks/grant/read/denied";

/// The two-domain model; `tasks_client_reads` grants `demo.tasks.Client` the view too.
fn two_domains(tasks_client_reads: bool) -> EssIr {
    let tasks = if tasks_client_reads {
        TWO_TASKS.replace(
            "  - name: demo.tasks.Client\n    may: [demo.tasks.OpenTask]\n",
            "  - name: demo.tasks.Client\n    may: [demo.tasks.OpenTask, demo.tasks.AllTasks]\n",
        )
    } else {
        TWO_TASKS.to_owned()
    };
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in [
        ("system.yaml", TWO_SYSTEM.to_owned()),
        ("components.yaml", TWO_COMPONENTS.to_owned()),
        ("domains/public.yaml", TWO_PUBLIC.to_owned()),
        ("domains/tasks.yaml", tasks),
    ] {
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.insert(label.to_owned(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("demo: {errors}"));
    compile(&specification, &sources).unwrap_or_else(|errors| panic!("demo: {errors}"))
}

/// The actors the denied scenario of `AllTasks` reads as, in order, `None` for no actor.
fn denied_readers(suite: &ConformanceSuite) -> Vec<Option<String>> {
    let id: ScenarioId = ALL_TASKS_DENIED.parse().unwrap();
    let scenario = suite.scenario(&id).expect("the denied read is filed");
    readers_of(scenario, "demo.tasks.AllTasks")
}

/// story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain: the denied read is sent as
/// an ungranted actor of the view's own domain, not the lowest-named across domains.
#[test]
fn the_denied_read_prefers_an_actor_of_the_views_domain() {
    let synthesis = ess_conformance::synthesize(&two_domains(false));
    assert_eq!(
        denied_readers(&synthesis.suite),
        [Some("demo.tasks.Client".to_owned()), None],
        "story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain"
    );
}

/// A target that admits `demo.tasks.Client` to `AllTasks` fails the denied scenario; the honest
/// one passes every scenario.
#[test]
fn a_target_admitting_an_ungranted_actor_of_the_views_domain_fails() {
    let suite = ess_conformance::synthesize(&two_domains(false)).suite;
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    let verdicts = |model: EssIr| -> Vec<(String, Status)> {
        Runner::for_suite(&suite)
            .run_admitted(&admitted, &Interpreted::for_model(model))
            .into_report()
            .scenarios
            .iter()
            .map(|result| (result.scenario.to_string(), result.status))
            .collect()
    };
    let honest = verdicts(two_domains(false));
    assert!(
        honest.iter().all(|(_, status)| *status == Status::Passed),
        "{honest:#?}"
    );
    let leaking = verdicts(two_domains(true));
    let failed: Vec<&str> = leaking
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect();
    assert_eq!(
        failed,
        [ALL_TASKS_DENIED],
        "story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain: {leaking:#?}"
    );
}

/// Where the view's domain has no ungranted actor, the denied read is today's: the lowest-named
/// declared actor the grant does not name, then no actor, under today's purpose.
#[test]
fn the_denied_read_falls_back_to_the_lowest_named_actor() {
    let synthesis = ess_conformance::synthesize(&two_domains(true));
    let id: ScenarioId = ALL_TASKS_DENIED.parse().unwrap();
    let scenario = synthesis
        .suite
        .scenario(&id)
        .expect("the denied read is filed");
    assert!(
        matches!(
            scenario.steps.as_slice(),
            [
                ScenarioStep::ReadAs { actor: Some(first) },
                ScenarioStep::QueryView { .. },
                ScenarioStep::ExpectNotGranted { actor: Some(refused), .. },
                ScenarioStep::ReadAs { actor: None },
                ScenarioStep::QueryView { .. },
                ScenarioStep::ExpectNotGranted { actor: None, .. },
            ] if first.to_string() == "demo.public.Client" && refused == first
        ),
        "{:?}",
        scenario.steps
    );
    assert_eq!(
        scenario.purpose.to_string(),
        "`demo.tasks.AllTasks` read as `demo.public.Client`, whose grant does not name it, and as \
         no actor is refused before anything is read"
    );
}

/// Every ungranted actor of the view's domain is read as, lowest-named first, then no actor; an
/// ungranted actor of another domain is not.
#[test]
fn every_ungranted_actor_of_the_views_domain_is_refused() {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    let tasks = TWO_TASKS.replace(
        "  - name: demo.tasks.Client\n",
        "  - name: demo.tasks.Guest\n    may: [demo.tasks.OpenTask]\n  - name: demo.tasks.Client\n",
    );
    for (label, text) in [
        ("system.yaml", TWO_SYSTEM.to_owned()),
        ("components.yaml", TWO_COMPONENTS.to_owned()),
        ("domains/public.yaml", TWO_PUBLIC.to_owned()),
        ("domains/tasks.yaml", tasks),
    ] {
        let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.insert(label.to_owned(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("demo: {errors}"));
    let ir = compile(&specification, &sources).unwrap_or_else(|errors| panic!("demo: {errors}"));
    let synthesis = ess_conformance::synthesize(&ir);
    assert_eq!(
        denied_readers(&synthesis.suite),
        [
            Some("demo.tasks.Client".to_owned()),
            Some("demo.tasks.Guest".to_owned()),
            None
        ],
        "story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain"
    );
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("admits");
    let report =
        Runner::for_suite(&synthesis.suite).run_admitted(&admitted, &Interpreted::for_model(ir));
    assert!(
        report
            .scenarios
            .iter()
            .all(|scenario| scenario.status == Status::Passed),
        "{:#?}",
        report.scenarios
    );
}
