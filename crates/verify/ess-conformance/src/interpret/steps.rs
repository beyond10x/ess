//! A component's steps (ess/38, `docs/design/component-steps.md`), run by the interpreter after
//! each command a scenario sends with its input (a command refused for an absent input changes no
//! store and reads no clock, so it starts no step).
//!
//! The time steps run first, once for each whole period of the scenario's clock that elapsed
//! since the scenario's first command decision and has not run yet; a time step needs the
//! clock, and a command of a model that declares one, decided without it, is refused by name.
//! Then the level steps run in rounds: each round runs every level step in the order the
//! components (by name) and their steps are declared, its command once for each row its view
//! answers, each input read from the row's field of the same name. The rounds end at the first
//! round in which no level step's view answered a row, or at [`STEP_ROUNDS`], which is a refusal
//! naming the bound and never a silent stop.
//!
//! A command a step runs is a decision like any other: granted to the step's actor, deciding
//! against the store as it is, reading the clock once, dispatching the bindings its events
//! trigger. It does not start the steps again; the rounds do.

use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedComponentStep};
use ess_domain::component::StepForm;
use ess_primitives::consistency::QueryConsistency;
use ess_primitives::ids::CorrelationId;
use ess_primitives::time::Timestamp;

use super::Interpreted;
use crate::occurrence_clock::DecisionInstant;
use crate::scenario::{ActorRef, CommandRef, ViewRef};
use crate::target::{Deadline, SemanticCommandRequest, SemanticViewRequest, TargetError};

/// The most rounds of level steps one command starts. A level step whose command leaves its
/// view's rows in place ends here, in a refusal naming this bound.
pub const STEP_ROUNDS: u32 = 64;

/// Every step of the model, in the order the rounds run them.
fn declared(model: &EssIr) -> Vec<&ResolvedComponentStep> {
    model
        .components()
        .values()
        .flat_map(|component| component.steps.iter())
        .collect()
}

impl Interpreted {
    /// Runs the model's steps after a command a scenario sent, decided at `instant`.
    pub(super) fn run_steps(
        &self,
        instant: Option<DecisionInstant>,
        correlation: &CorrelationId,
    ) -> Result<(), TargetError> {
        let model = self.model("running the component steps")?;
        let steps = declared(model);
        if steps.is_empty() {
            return Ok(());
        }
        self.run_time_steps(&steps, instant, correlation)?;
        self.run_level_steps(model, &steps, correlation)
    }

    fn run_time_steps(
        &self,
        steps: &[&ResolvedComponentStep],
        instant: Option<DecisionInstant>,
        correlation: &CorrelationId,
    ) -> Result<(), TargetError> {
        let timed: Vec<(usize, &ResolvedComponentStep)> = steps
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, step)| step.form == StepForm::Time)
            .collect();
        if timed.is_empty() {
            return Ok(());
        }
        let Some(now) = instant.map(DecisionInstant::instant) else {
            let (_, step) = timed[0];
            return Err(TargetError::unsupported(
                format!("running the time step of `{}`", step.command.name()),
                "a time step runs once per period of the scenario's clock, and this command was \
                 decided with no clock",
            ));
        };
        let origin = *self.scenario.borrow_mut().step_origin.get_or_insert(now);
        let elapsed = u64::try_from(now.epoch_seconds() - origin.epoch_seconds()).unwrap_or(0);
        for (index, step) in timed {
            let Some(every) = step.every else {
                continue;
            };
            let due = elapsed / u64::from(every.seconds());
            let done = self
                .scenario
                .borrow()
                .time_runs
                .get(&index)
                .copied()
                .unwrap_or(0);
            for _ in done..due {
                self.run_step(step, BTreeMap::new(), correlation)?;
            }
            self.scenario.borrow_mut().time_runs.insert(index, due);
        }
        Ok(())
    }

    fn run_level_steps(
        &self,
        model: &EssIr,
        steps: &[&ResolvedComponentStep],
        correlation: &CorrelationId,
    ) -> Result<(), TargetError> {
        let level: Vec<&ResolvedComponentStep> = steps
            .iter()
            .copied()
            .filter(|step| step.form == StepForm::Level)
            .collect();
        if level.is_empty() {
            return Ok(());
        }
        for _ in 0..STEP_ROUNDS {
            let mut answered = false;
            for step in &level {
                let Some(view) = &step.view else {
                    continue;
                };
                let rows = {
                    let scenario = self.scenario.borrow();
                    super::views::query(
                        model,
                        &scenario.store,
                        &SemanticViewRequest {
                            view: ViewRef::new(view.name().clone()),
                            params: BTreeMap::new(),
                            consistency: QueryConsistency::Current,
                            correlation: correlation.clone(),
                            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
                        },
                        None,
                    )?
                    .rows
                };
                let Some(command) = model.commands().get(step.command.name()) else {
                    continue;
                };
                for row in rows {
                    answered = true;
                    let input = command
                        .input
                        .iter()
                        .filter_map(|field| {
                            row.get(field.name.as_str())
                                .map(|value| (field.name.clone(), value.clone()))
                        })
                        .collect();
                    self.run_step(step, input, correlation)?;
                }
            }
            if !answered {
                return Ok(());
            }
        }
        let names: Vec<String> = level
            .iter()
            .filter_map(|step| step.view.as_ref().map(|view| format!("`{}`", view.name())))
            .collect();
        Err(TargetError::unsupported(
            "running the level steps",
            format!(
                "the level steps ran {STEP_ROUNDS} rounds (STEP_ROUNDS) and {} still answer rows",
                names.join(", ")
            ),
        ))
    }

    fn run_step(
        &self,
        step: &ResolvedComponentStep,
        input: BTreeMap<String, ess_primitives::node::Node>,
        correlation: &CorrelationId,
    ) -> Result<(), TargetError> {
        let request = SemanticCommandRequest {
            command: CommandRef::new(step.command.name().clone()),
            actor: Some(ActorRef::new(step.actor.name().clone())),
            caller: None,
            input,
            correlation: correlation.clone(),
        };
        let mut instant = None;
        self.decide(&request, &mut instant).map(drop)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use ess_primitives::node::Node;

    use super::*;
    use crate::occurrence_clock::CommandClock;
    use crate::target::{ConformanceTarget, ScenarioContext};

    /// A queue a worker leases from while its view answers rows, and a sweep run every minute.
    const JOBS: &str = "\
format: ess/38
system: demo
version: v1
domain: demo.jobs
summary: A queue worked by its own worker.
types:
  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}
  - {name: demo.jobs.SweepId, kind: newtype, of: Uuid}
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: demo.jobs.JobId}
    fields:
      - {name: title, type: String}
    lifecycle:
      initial: Pending
      states: [Pending, Leased]
      terminal: [Leased]
      transitions:
        - {name: lease, from: [Pending], to: Leased}
  - name: demo.jobs.Sweep
    identity: {name: sweep_id, type: demo.jobs.SweepId}
    fields: []
    lifecycle: {initial: Done, states: [Done], terminal: [Done]}
errors:
  - {name: demo.jobs.NotPending, summary: The job is not pending., fields: []}
events:
  - name: demo.jobs.JobSubmitted
    fields:
      - {name: job_id, type: demo.jobs.JobId}
  - name: demo.jobs.JobLeased
    fields:
      - {name: job_id, type: demo.jobs.JobId}
  - name: demo.jobs.Swept
    fields:
      - {name: sweep_id, type: demo.jobs.SweepId}
actors:
  - name: demo.jobs.Client
    may: [demo.jobs.Submit, demo.jobs.PendingJobs, demo.jobs.LeasedJobs, demo.jobs.Sweeps]
  - name: demo.jobs.Worker
    system: true
    may: [demo.jobs.Lease, demo.jobs.RecordSweep, demo.jobs.PendingJobs]
components:
  - component: worker
    owns: {domains: [demo.jobs]}
    accepts:
      commands: [demo.jobs.Submit, demo.jobs.Lease, demo.jobs.RecordSweep]
    publishes:
      events: [demo.jobs.JobSubmitted, demo.jobs.JobLeased, demo.jobs.Swept]
    reached_by: network
    steps:
      - {form: level, view: demo.jobs.PendingJobs, command: demo.jobs.Lease, every: PT5S}
      - {form: time, command: demo.jobs.RecordSweep, every: PT60S}
commands:
  - name: demo.jobs.Submit
    input:
      - {name: title, type: String}
    outcomes:
      - name: submitted
        creates: demo.jobs.Job
        instance: job_id
        sets: {title: input.title}
        emits: [demo.jobs.JobSubmitted]
        payload:
          demo.jobs.JobSubmitted: {job_id: {generated: true}}
  - name: demo.jobs.Lease
    input:
      - {name: job_id, type: demo.jobs.JobId}
    outcomes:
      - name: leased
        moves: demo.jobs.Job.lease
        instance: job_id
        emits: [demo.jobs.JobLeased]
        payload:
          demo.jobs.JobLeased: {job_id: input.job_id}
      - {name: not-pending, wrong_state: true, error: demo.jobs.NotPending}
  - name: demo.jobs.RecordSweep
    input: []
    outcomes:
      - name: swept
        creates: demo.jobs.Sweep
        instance: sweep_id
        emits: [demo.jobs.Swept]
        payload:
          demo.jobs.Swept: {sweep_id: {generated: true}}
views:
  - name: demo.jobs.PendingJobs
    source: demo.jobs.Job
    consistency: read_your_writes
    filter: state == Pending
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: title, type: String}
  - name: demo.jobs.LeasedJobs
    source: demo.jobs.Job
    consistency: read_your_writes
    filter: state == Leased
    fields:
      - {name: job_id, type: demo.jobs.JobId}
      - {name: title, type: String}
  - name: demo.jobs.Sweeps
    source: demo.jobs.Sweep
    consistency: read_your_writes
    fields:
      - {name: sweep_id, type: demo.jobs.SweepId}
";

    fn model(text: &str) -> EssIr {
        let specification = ess_domain::Specification::assemble([(
            ess_domain::system::Source::new("jobs.yaml"),
            ess_domain::spec::RawSpecFile::parse(text).unwrap(),
        )])
        .unwrap_or_else(|errors| panic!("{errors}"));
        ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
            .unwrap()
    }

    /// A clock the test moves: every decision reads the instant last set.
    #[derive(Clone)]
    struct Moved(Arc<Mutex<DecisionInstant>>);

    impl Moved {
        fn at(text: &str) -> Self {
            Self(Arc::new(Mutex::new(DecisionInstant::parse(text).unwrap())))
        }

        fn set(&self, text: &str) {
            *self.0.lock().unwrap() = DecisionInstant::parse(text).unwrap();
        }
    }

    impl CommandClock for Moved {
        fn read(&self) -> Option<DecisionInstant> {
            Some(*self.0.lock().unwrap())
        }
    }

    fn correlation() -> CorrelationId {
        CorrelationId::new("steps").unwrap()
    }

    fn begun(target: &Interpreted) {
        target
            .begin_scenario(&ScenarioContext::new(
                "demo.jobs.Submit/outcome/submitted".parse().unwrap(),
                correlation(),
            ))
            .unwrap();
    }

    fn submit(target: &Interpreted, title: &str) -> Result<(), TargetError> {
        target
            .execute_command(SemanticCommandRequest {
                command: CommandRef::new("demo.jobs.Submit".parse().unwrap()),
                actor: Some(ActorRef::new("demo.jobs.Client".parse().unwrap())),
                caller: None,
                input: BTreeMap::from([("title".to_owned(), Node::Text(title.to_owned()))]),
                correlation: correlation(),
            })
            .map(drop)
    }

    fn rows(target: &Interpreted, view: &str, field: &str) -> Vec<Node> {
        target
            .query_view(SemanticViewRequest {
                view: ViewRef::new(view.parse().unwrap()),
                params: BTreeMap::new(),
                consistency: QueryConsistency::Current,
                correlation: correlation(),
                deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
            })
            .unwrap()
            .rows
            .into_iter()
            .map(|row| row[field].clone())
            .collect()
    }

    #[test]
    fn a_level_step_runs_its_command_for_each_row_until_its_view_answers_none() {
        let clock = Moved::at("2026-01-01T00:00:00Z");
        let target = Interpreted::for_model(model(JOBS)).with_command_clock(clock);
        begun(&target);
        submit(&target, "a").unwrap();
        submit(&target, "b").unwrap();
        assert!(rows(&target, "demo.jobs.PendingJobs", "title").is_empty());
        let mut leased = rows(&target, "demo.jobs.LeasedJobs", "title");
        leased.sort_by_key(|title| format!("{title:?}"));
        assert_eq!(
            leased,
            vec![Node::Text("a".to_owned()), Node::Text("b".to_owned())]
        );
    }

    #[test]
    fn a_time_step_runs_once_per_period_of_the_scenario_clock() {
        let clock = Moved::at("2026-01-01T00:00:00Z");
        let target = Interpreted::for_model(model(JOBS)).with_command_clock(clock.clone());
        begun(&target);
        submit(&target, "a").unwrap();
        assert_eq!(rows(&target, "demo.jobs.Sweeps", "sweep_id").len(), 0);
        clock.set("2026-01-01T00:00:59Z");
        submit(&target, "b").unwrap();
        assert_eq!(rows(&target, "demo.jobs.Sweeps", "sweep_id").len(), 0);
        clock.set("2026-01-01T00:02:30Z");
        submit(&target, "c").unwrap();
        assert_eq!(rows(&target, "demo.jobs.Sweeps", "sweep_id").len(), 2);
        clock.set("2026-01-01T00:02:59Z");
        submit(&target, "d").unwrap();
        assert_eq!(rows(&target, "demo.jobs.Sweeps", "sweep_id").len(), 2);
    }

    #[test]
    fn a_time_step_decided_with_no_clock_is_refused_by_name() {
        let target = Interpreted::for_model(model(JOBS));
        begun(&target);
        let refused = submit(&target, "a").unwrap_err().to_string();
        assert!(refused.contains("time step"), "{refused}");
        assert!(refused.contains("no clock"), "{refused}");
    }

    #[test]
    fn synthesis_names_a_refusal_for_each_step() {
        let synthesis = crate::synthesize::synthesize(&model(JOBS));
        let named: Vec<&str> = synthesis
            .refusals
            .iter()
            .filter_map(|refusal| match &refusal.cause {
                crate::synthesize::RefusalCause::NotSynthesisedYet {
                    construct,
                    sections,
                } if sections.contains("component-steps") => Some(*construct),
                _ => None,
            })
            .collect();
        assert_eq!(
            named,
            ["a level step of a component", "a time step of a component"]
        );
    }

    #[test]
    fn a_level_step_whose_view_never_empties_ends_in_the_named_bound() {
        let unfiltered = JOBS.replacen("    filter: state == Pending\n", "", 1);
        let clock = Moved::at("2026-01-01T00:00:00Z");
        let target = Interpreted::for_model(model(&unfiltered)).with_command_clock(clock);
        begun(&target);
        let refused = submit(&target, "a").unwrap_err().to_string();
        assert!(refused.contains("STEP_ROUNDS"), "{refused}");
        assert!(refused.contains("demo.jobs.PendingJobs"), "{refused}");
    }
}
