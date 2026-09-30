//! The scenario for an `input_absent:` branch (ess/16, beyond10x/ess#170): the command, sent with
//! no input at all, answering the declared error and publishing nothing.
//!
//! It arranges nothing. The branch is decided before any field is read, so no world the arrangement
//! could build selects another one, and the step that sends the command is
//! [`ExecuteCommandWithoutInput`](ScenarioStep::ExecuteCommandWithoutInput) — never
//! `execute_command` with `input: {}`, which an implementation may answer differently.

use super::{
    clipped, insert, not_emitted, ActorRef, BTreeMap, BTreeSet, CommandRef, ConformanceScenario,
    ConformanceSuite, ErrorRef, EssIr, EssSemanticRef, OutcomeRef, QualifiedName, Refusal,
    RefusalCause, ResolvedCommand, ResolvedCondition, ResolvedOutcome, ScenarioId, ScenarioStep,
};

/// One scenario per command declaring an `input_absent:` branch, filed under that branch's own
/// outcome id.
pub(super) fn absent_inputs(
    ir: &EssIr,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    suite: &mut ConformanceSuite,
    refusals: &mut Vec<Refusal>,
) {
    for command in ir.commands().values() {
        let Some(declared) = command
            .outcomes
            .iter()
            .find(|outcome| outcome.condition == ResolvedCondition::InputAbsent)
        else {
            continue;
        };
        let id = ScenarioId::Outcome {
            outcome: OutcomeRef::new(CommandRef::new(command.name.clone()), declared.name.clone()),
        };
        match scenario(ir, command, declared, actors) {
            Ok(scenario) => insert(suite, id, scenario, refusals),
            Err(cause) => refusals.push(Refusal::about(&id, cause)),
        }
    }
}

/// The scenario itself.
fn scenario(
    ir: &EssIr,
    command: &ResolvedCommand,
    declared: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<ConformanceScenario, RefusalCause> {
    let Some(error) = declared.error.as_ref() else {
        return Err(RefusalCause::StrategyWithoutGuard {
            strategy: declared.test_strategy,
        });
    };
    let command_ref = CommandRef::new(command.name.clone());
    let branch = OutcomeRef::new(command_ref.clone(), declared.name.clone());
    let named = ErrorRef::from(error);
    let mut steps = vec![
        ScenarioStep::ExecuteCommandWithoutInput {
            caller: std::collections::BTreeMap::new(),
            command: command_ref.clone(),
            actor: actors.get(&command.name).cloned(),
        },
        ScenarioStep::ExpectOutcome {
            outcome: branch.clone(),
        },
        // Sent without input and read before any row, so only a literal source is compared.
        super::expect_error(ir, declared, error, &BTreeMap::new(), &BTreeMap::new()),
    ];
    let mut source: BTreeSet<EssSemanticRef> = BTreeSet::new();
    source.insert(command_ref.into());
    source.insert(branch.into());
    source.insert(named.clone().into());
    if let Some(actor) = actors.get(&command.name) {
        source.insert(actor.clone().into());
    }
    let forbidden = not_emitted(ir, &[]);
    for event in &forbidden {
        steps.push(ScenarioStep::ExpectNoEvent {
            event: event.clone(),
        });
    }
    source.extend(forbidden.into_iter().map(EssSemanticRef::from));
    let text = format!(
        "`{}` with no input at all takes `{}` and reports `{named}`",
        command.name, declared.name
    );
    Ok(ConformanceScenario::new(clipped(&text), steps, source))
}
