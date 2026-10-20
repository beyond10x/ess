//! The interpreted model answering an await of a command the system issues itself (ess/27,
//! beyond10x/ess#542, `docs/design/system-issued-commands.md`, *Every lane*).
//!
//! # The hold
//!
//! The interpreter issues a system-issued command only here, when a scenario awaits it, so it holds
//! its system by construction: nothing a scenario does before the await takes the step, and a
//! binding never invokes such a command (validation refuses one). An answer that nothing was
//! reached changes nothing, so asking again — as the runner does until its deadline when the
//! command declares no bound — answers the same.
//!
//! # Which input the system issues it with
//!
//! The awaited instance is the identity input: the field every outcome that addresses a record
//! names as its `instance:`. Every other input is the system's to decide, and is decided by
//! candidates in a fixed order, field by field in declared order:
//!
//! | input | candidates, in order |
//! |---|---|
//! | read by a `when_related:` guard through the input | the identities of held rows of that entity, in canonical order |
//! | any other | one value minted from the scenario's counter, as every value the model leaves to the implementation is (`interpret.rs`, *Where the identifiers come from*) |
//! | either, declared `Optional` | absent first |
//!
//! Every combination is executed against the store as it was before the await, in that order. A
//! combination under which the model selects one accepting branch, or the external branch a
//! scenario forced, is *reached*; the first reached is the step the system takes. One that selects a
//! refusal — decided by the record (`wrong_state:`, `unknown_instance:`, a held-state guard) or by
//! the input (a `when:` guard, a missing or failing related row) — is not. None reached is
//! [`AwaitAnswer::NotIssued`].
//!
//! # Nothing is chosen here
//!
//! As for an invoked command, the interpreter refuses where the model leaves the answer open:
//!
//! - combinations reaching different accepting branches: which the system takes is not in the
//!   model;
//! - a compared input the model does not determine: the identity input is the instance, and an
//!   input a `when_related:` guard reads is determined when every reached combination carries the
//!   same value of it; a minted value never is, because it is this interpreter's own;
//! - a model that leaves one combination's outcome open, or a construct it does not execute.
//!
//! Each is [`TargetError::Unsupported`], never a guess.
//!
//! # The bound
//!
//! Answered as [`observe_elapsed`](crate::target::ConformanceTarget::observe_elapsed) is, on the scenario's
//! logical clock: a reached step is taken at once (a reading of `0`), and for nothing reached
//! within a declared bound the window is held for the whole bound, which advances the clock by it.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{EssIr, ResolvedCommand, ResolvedCondition, ResolvedRelatedVia};
use ess_domain::name::QualifiedName;
use ess_primitives::node::Node;

use super::execute::{self, Externals, Step, Store};
use super::{refusal, response, Interpreted};
use crate::system_issued::Unaddressed;
use crate::target::{AwaitAnswer, AwaitRequest, TargetError};

/// The most combinations of decided inputs one await executes. Beyond it the answer is
/// unsupported rather than slow: a model whose system chooses among that many is not one this
/// interpreter decides.
const MOST_CANDIDATES: usize = 4_096;

/// One combination the model reaches an accepting, or forced external, branch under.
struct Candidate {
    input: BTreeMap<String, Node>,
    step: Step,
    responses: response::Authority,
}

impl Interpreted {
    /// [`crate::target::ConformanceTarget::await_outcome`] for the interpreted model.
    pub(super) fn await_system_outcome(
        &self,
        request: &AwaitRequest,
    ) -> Result<AwaitAnswer, TargetError> {
        let observation = format!("awaiting `{}` on an instance", request.command);
        let model = self.model(observation.clone())?;
        self.scenario.borrow().facts.check(&request.correlation)?;
        let (command, identity) = awaited(model, request, &observation)?;
        let related = related_inputs(command);

        let forced = self.forced_for(&request.command);
        let externals = forced
            .clone()
            .map_or(Externals::Withheld, Externals::Forced);

        let (mut base, issued) = {
            let scenario = self.scenario.borrow();
            (scenario.store.clone(), scenario.issued.clone())
        };
        let choices = choices(
            model,
            command,
            (identity, &request.instance),
            &related,
            &mut base,
        )
        .map_err(|why| refusal(observation.clone(), &why))?;
        let combinations = combinations(&choices).ok_or_else(|| {
            TargetError::unsupported(
                &observation,
                format!(
                    "the system decides among more than {MOST_CANDIDATES} combinations of \
                         input, which the interpreter does not enumerate"
                ),
            )
        })?;

        // The decision edge, read once for every combination: they are alternatives of one
        // decision, not a sequence of them.
        let decision_time = self.clock.as_ref().and_then(|clock| clock.0.read());
        let mut reached: Vec<Candidate> = Vec::new();
        for input in combinations {
            let mut responses = response::Authority::native(&issued);
            let mut steps = execute::responding(
                model,
                &base,
                &command.name,
                &execute::caller::Invocation {
                    input: &input,
                    caller: None,
                },
                &externals,
                &execute::Generated::Counter,
                &mut responses,
                decision_time,
            )
            .map_err(|why| refusal(observation.clone(), &why))?;
            if steps.len() != 1 {
                return Err(TargetError::unsupported(
                    observation,
                    format!(
                        "the model leaves the outcome open over the input {} and the interpreter \
                         does not choose",
                        crate::report::quote_input(&command.name.to_string(), &input)
                    ),
                ));
            }
            let step = steps.remove(0);
            if reaches(command, &step, forced.as_ref()) {
                reached.push(Candidate {
                    input,
                    step,
                    responses,
                });
            }
        }

        if reached.is_empty() {
            return self.nothing_reached(model, request, &observation);
        }
        if let Err(why) = chosen(&reached, &request.compared, identity, &related) {
            return Err(TargetError::unsupported(observation, why));
        }

        let Candidate {
            input,
            step,
            responses,
        } = reached.swap_remove(0);
        if forced.is_some() {
            self.consume_forced();
        }
        let prepared = responses
            .finish(model, &step)
            .map_err(|why| refusal(observation, &why))?;
        let result =
            self.complete_command(&request.command, &request.correlation, step, prepared)?;
        Ok(AwaitAnswer::Reached {
            result,
            input,
            elapsed_ms: Some(0),
        })
    }

    /// The external outcome a scenario forced on `command`, where it forced one.
    fn forced_for(
        &self,
        command: &crate::scenario::CommandRef,
    ) -> Option<ess_domain::command::OutcomeName> {
        match &self.scenario.borrow().forced {
            Some((forced, _)) if forced.command == *command => Some(forced.outcome.clone()),
            _ => None,
        }
    }

    /// The forced control lapses with the step it forced, as it does for an invoked command.
    fn consume_forced(&self) {
        let mut scenario = self.scenario.borrow_mut();
        if let Some((outcome, remaining)) = scenario.forced.take() {
            scenario.forced = std::num::NonZeroU32::new(remaining.get() - 1)
                .map(|remaining| (outcome, remaining));
        }
    }

    /// Nothing reached: within a declared bound the window is held for all of it.
    fn nothing_reached(
        &self,
        model: &EssIr,
        request: &AwaitRequest,
        observation: &str,
    ) -> Result<AwaitAnswer, TargetError> {
        let Some(within) = request.within else {
            return Ok(AwaitAnswer::NotIssued { elapsed_ms: None });
        };
        if model
            .bindings()
            .values()
            .any(|binding| binding.cause.periodic().is_some())
        {
            return Err(TargetError::unsupported(
                observation,
                "holding the window for the declared bound requires the model's periodic host \
                 facts",
            ));
        }
        let mut scenario = self.scenario.borrow_mut();
        scenario.facts.hold(within.millis())?;
        Ok(AwaitAnswer::NotIssued {
            elapsed_ms: Some(within.millis()),
        })
    }
}

/// Whether the reached combinations leave the interpreter nothing to choose: one branch, and every
/// compared input determined. The identity input is the instance; an input a `when_related:`
/// guard reads is determined when every reached combination carries one value of it.
fn chosen(
    reached: &[Candidate],
    compared: &BTreeSet<String>,
    identity: &str,
    related: &BTreeMap<&str, &QualifiedName>,
) -> Result<(), String> {
    let branches: BTreeSet<String> = reached
        .iter()
        .filter_map(|candidate| candidate.step.outcome.as_ref().map(ToString::to_string))
        .collect();
    if branches.len() > 1 {
        return Err(format!(
            "the system may reach any of {} depending on the input it decides, and the \
             interpreter does not choose",
            branches.into_iter().collect::<Vec<_>>().join(", ")
        ));
    }
    let Some(first) = reached.first() else {
        return Ok(());
    };
    for field in compared {
        let determined = field == identity
            || (related.contains_key(field.as_str())
                && reached
                    .iter()
                    .all(|candidate| candidate.input.get(field) == first.input.get(field)));
        if !determined {
            return Err(format!(
                "the scenario compares input `{field}`, which the system decides and the model \
                 does not determine"
            ));
        }
    }
    Ok(())
}

/// The system-issued command awaited, and its identity input. Awaiting a command a caller invokes
/// is a suite defect; one whose instance this interpreter cannot name is unsupported.
fn awaited<'m>(
    model: &'m EssIr,
    request: &AwaitRequest,
    observation: &str,
) -> Result<(&'m ResolvedCommand, &'m str), TargetError> {
    let command = model
        .commands()
        .get(request.command.name())
        .ok_or_else(|| TargetError::unavailable(observation, "command is not declared"))?;
    if !command.system_issued {
        return Err(TargetError::unavailable(
            observation,
            "a suite awaits only a command the system issues itself; this one is invoked by a \
             caller",
        ));
    }
    let identity =
        identity_input(command).map_err(|why| TargetError::unsupported(observation, why))?;
    Ok((command, identity))
}

/// The input every outcome that addresses an existing record names as its `instance:`
/// ([`crate::system_issued::identity_input`], the rule synthesis writes the await by).
fn identity_input(command: &ResolvedCommand) -> Result<&str, String> {
    crate::system_issued::identity_input(command).map_err(|unaddressed| match unaddressed {
        Unaddressed::Creates { outcome } => format!(
            "outcome `{outcome}` creates a record, and awaiting a system-issued command that \
             creates one is not interpreted (stage 1 awaits a step on an existing record)"
        ),
        Unaddressed::NoInstance => "no outcome addresses an existing record through `instance:`, \
                                    so there is no instance to await"
            .to_owned(),
        Unaddressed::SeveralInputs { .. } => "its outcomes address records through different \
                                              inputs, so the awaited instance names no one input"
            .to_owned(),
    })
}

/// Each input a `when_related:` guard reads through the input, with the entity whose identity it
/// carries.
fn related_inputs(command: &ResolvedCommand) -> BTreeMap<&str, &QualifiedName> {
    command
        .outcomes
        .iter()
        .filter_map(|outcome| match &outcome.condition {
            ResolvedCondition::Related {
                via: ResolvedRelatedVia::Input { field, .. },
                entity,
                ..
            } => Some((field.as_str(), entity.name())),
            _ => None,
        })
        .collect()
}

/// Each declared input with its candidates in order: `None` for absent.
type Choices = Vec<(String, Vec<Option<Node>>)>;

/// Each declared input with its candidates in order.
fn choices(
    model: &EssIr,
    command: &ResolvedCommand,
    (identity, instance): (&str, &Node),
    related: &BTreeMap<&str, &QualifiedName>,
    base: &mut Store,
) -> Result<Choices, execute::Undetermined> {
    let mut choices = Vec::with_capacity(command.input.len());
    for field in &command.input {
        let mut values = Vec::new();
        if field.name == identity {
            values.push(Some(instance.clone()));
        } else {
            if field.type_ref.is_optional() {
                values.push(None);
            }
            if let Some(entity) = related.get(field.name.as_str()) {
                values.extend(
                    base.instances_of(entity)
                        .map(|(identity, _)| Some(identity.clone())),
                );
            } else {
                values.push(Some(base.mint(model, &field.type_ref)?));
            }
        }
        choices.push((field.name.clone(), values));
    }
    Ok(choices)
}

/// Every combination of the choices, the first field varying slowest; `None` beyond
/// [`MOST_CANDIDATES`]. A field with no candidate leaves no combination.
fn combinations(choices: &Choices) -> Option<Vec<BTreeMap<String, Node>>> {
    let mut total: usize = 1;
    for (_, values) in choices {
        total = total.checked_mul(values.len())?;
        if total > MOST_CANDIDATES {
            return None;
        }
    }
    let mut out = Vec::with_capacity(total);
    for mut index in 0..total {
        let mut input = BTreeMap::new();
        for (field, values) in choices.iter().rev() {
            let value = &values[index % values.len()];
            index /= values.len();
            if let Some(value) = value {
                input.insert(field.clone(), value.clone());
            }
        }
        out.push(input);
    }
    Some(out)
}

/// Whether the step is the system taking its own step: an accepting branch, or the external branch
/// a scenario forced. A refusal the record or the input decides is the system not issuing it.
fn reaches(
    command: &ResolvedCommand,
    step: &Step,
    forced: Option<&ess_domain::command::OutcomeName>,
) -> bool {
    let Some(taken) = &step.outcome else {
        return false;
    };
    if forced == Some(&taken.outcome) {
        return true;
    }
    command
        .outcomes
        .iter()
        .find(|outcome| outcome.name == taken.outcome)
        .is_some_and(|outcome| outcome.error.is_none())
}
