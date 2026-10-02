//! Source-derived observation authority shared by generated and authored producers.
use super::{EventAuthority, EventWindow, Origin, Response, Trace};
use crate::{
    scenario::{CommandRef, EventRef, OutcomeRef},
    ConformanceScenario, ScenarioStep,
};
use ess_compiler::EssIr;
use std::collections::BTreeSet;

pub(crate) fn attach(ir: &EssIr, scenario: &mut ConformanceScenario) -> Result<(), String> {
    let invoked: BTreeSet<_> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { command, .. } => Some(command),
            _ => None,
        })
        .collect();
    let selected: BTreeSet<_> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectOutcome { outcome } => Some(outcome.clone()),
            _ => None,
        })
        .collect();
    let mut origins = Vec::new();
    let mut required_origins = Vec::new();
    for command in ir.commands().values() {
        let reference = CommandRef::new(command.name.clone());
        if !invoked.contains(&reference) {
            continue;
        }
        for outcome in &command.outcomes {
            if outcome.one_time_response.is_empty() {
                continue;
            }
            let outcome_ref = OutcomeRef::new(reference.clone(), outcome.name.clone());
            if selected.contains(&outcome_ref) {
                required_origins.push(outcome_ref.clone());
            }
            origins.push(Origin {
                command: reference.clone(),
                outcome: outcome_ref,
                response: Response::of(ir, command)?,
                fields: outcome.one_time_response.clone(),
            });
        }
    }
    if required_origins.is_empty() {
        return Ok(());
    }
    for origin in &origins {
        scenario.source.insert(origin.command.clone().into());
        scenario.source.insert(origin.outcome.clone().into());
    }
    let within_ms = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectNotBefore { elapsed, .. }
            | ScenarioStep::ExpectWithin { elapsed, .. }
            | ScenarioStep::ExpectQuiet { elapsed, .. } => Some(elapsed.millis()),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    let events: Vec<_> = ir
        .events()
        .values()
        .map(|event| EventAuthority {
            event: EventRef::new(event.name.clone()),
            within_ms,
        })
        .collect();
    let mut event_windows = Vec::new();
    for authority in &events {
        scenario.source.insert(authority.event.clone().into());
        for (after_step, step) in scenario.steps.iter().enumerate() {
            if matches!(
                step,
                ScenarioStep::ExecuteCommand { .. }
                    | ScenarioStep::ExecuteCommandWithoutInput { .. }
                    | ScenarioStep::QueryView { .. }
                    | ScenarioStep::EventuallyView { .. }
            ) {
                for within_ms in BTreeSet::from([0, authority.within_ms]) {
                    event_windows.push(EventWindow {
                        event: authority.event.clone(),
                        after_step,
                        within_ms,
                    });
                }
            }
        }
    }
    let trace = Trace {
        origins,
        required_origins,
        events,
        event_windows,
    };
    trace.validate(scenario)?;
    scenario.one_time_response = Some(trace);
    Ok(())
}
