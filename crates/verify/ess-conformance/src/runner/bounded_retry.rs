//! The runner's half of a bounded retry (suite/26, [`crate::bounded_retry`]): an outcome forced on
//! the next `times` invocations, and an exact invocation count.
//!
//! The count is observed for the step's whole eventual window (§15), asked of the target until the
//! run's deadline passes, never after a fixed delay. It passes only when the matching invocations
//! reach **exactly** the count and are still exactly the count when the deadline passes; a count
//! above it at any observation fails at once. A sender that makes fewer attempts fails at the
//! deadline; one that makes more fails as soon as the extra attempt is visible within the window.
//! The price is that a passing count always waits out the window.

use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU32;

use ess_primitives::node::Node;

use super::{
    matches, quote, quote_input, target_failure, BindingRef, CheckCode, CheckResult, Clock,
    CommandRef, ConformanceTarget, Diagnostic, ExternalOutcomeControl, Flow,
    InvocationObservationRequest, OutcomeRef, Run, Runner, ScenarioValue,
};

impl<C: Clock> Runner<C> {
    /// Requires exactly `count` invocations of `command` by `binding` carrying `input`.
    pub(super) fn expect_invocation_count<T: ConformanceTarget>(
        &mut self,
        binding: &BindingRef,
        command: &CommandRef,
        input: &BTreeMap<String, ScenarioValue>,
        count: NonZeroU32,
        run: &mut Run,
        target: &T,
    ) -> Flow {
        let Some((wanted, absent)) = expected(binding, input, run) else {
            return Flow::Stop;
        };
        let wanted_count = usize::try_from(count.get()).unwrap_or(usize::MAX);
        let about = format!("`{binding}` invokes `{command}` exactly {count} time(s)");
        let deadline = self.deadline();
        let mut asks = 0_u32;
        loop {
            asks += 1;
            let request = InvocationObservationRequest {
                binding: binding.clone(),
                command: command.clone(),
                correlation: run.context.correlation.clone(),
                deadline,
            };
            let invocations = match target.observe_invocations(request) {
                Ok(invocations) => invocations,
                Err(error) if error.is_unsupported() => {
                    run.record(CheckResult::unsupported(
                        about,
                        Diagnostic::new(CheckCode::Invocation, run.id.clone())
                            .declared_by(binding.clone())
                            .declared_by(command.clone())
                            .expected("the target exposes the commands its bindings invoke (§16)")
                            .observed(error.to_string()),
                    ));
                    return Flow::Continue;
                }
                Err(error) => {
                    run.record(target_failure(
                        &run.id,
                        &format!("observing what `{binding}` invoked"),
                        &error,
                    ));
                    return Flow::Stop;
                }
            };
            let matching: Vec<_> = invocations
                .iter()
                .filter(|invocation| {
                    &invocation.command == command
                        && matches(&invocation.input, &wanted)
                        && absent
                            .iter()
                            .all(|field| !invocation.input.contains_key(field))
                })
                .collect();
            // Observed for the whole window: a count reached early may still be exceeded by an
            // attempt the target makes visible later. Above the count fails at once.
            if matching.len() <= wanted_count && !deadline.has_passed(self.clock.now()) {
                continue;
            }
            if matching.len() == wanted_count {
                run.record(CheckResult::passed(CheckCode::Invocation, about));
            } else {
                let mut diagnostic = Diagnostic::new(CheckCode::Invocation, run.id.clone())
                    .declared_by(binding.clone())
                    .declared_by(command.clone())
                    .expected(format!(
                        "exactly {count} invocation(s) of `{command}` by `{binding}`"
                    ));
                for (field, value) in &wanted {
                    diagnostic =
                        diagnostic.expected(format!("{command}.{field} = {}", quote(value)));
                }
                let seen = invocations
                    .iter()
                    .map(|invocation| {
                        quote_input(&invocation.command.to_string(), &invocation.input)
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                run.record(CheckResult::failed(
                    about,
                    diagnostic.observed(format!(
                        "{} matching invocation(s) after {asks} observation(s): [{seen}]",
                        matching.len()
                    )),
                ));
            }
            return Flow::Continue;
        }
    }
}

/// Forces `force` on the next `times` invocations of its command.
pub(super) fn configure_repeated_external<T: ConformanceTarget>(
    force: &OutcomeRef,
    times: NonZeroU32,
    run: &mut Run,
    target: &T,
) -> Flow {
    let request = ExternalOutcomeControl {
        force: force.clone(),
        correlation: run.context.correlation.clone(),
    };
    match target.configure_external_outcome_repeatedly(request, times) {
        Ok(()) => Flow::Continue,
        Err(error) => {
            run.record(target_failure(
                &run.id,
                &format!("forcing the external outcome `{force}` on the next {times} invocations"),
                &error,
            ));
            Flow::Stop
        }
    }
}

/// The values each named input must carry, and the inputs that must be absent, or `None` after
/// recording why an expected value could not be resolved.
pub(super) fn expected(
    binding: &BindingRef,
    input: &BTreeMap<String, ScenarioValue>,
    run: &mut Run,
) -> Option<(BTreeMap<String, Node>, BTreeSet<String>)> {
    let mut wanted = BTreeMap::new();
    let mut absent = BTreeSet::new();
    for (field, value) in input {
        match run.resolve_expected(value) {
            Ok(crate::accessor::Expected::Present(node)) => {
                wanted.insert(field.clone(), node);
            }
            Ok(crate::accessor::Expected::Absent) => {
                absent.insert(field.clone());
            }
            Err(reason) => {
                run.record(CheckResult::errored(
                    format!("the mapping of `{field}` by `{binding}`"),
                    Diagnostic::new(CheckCode::Suite, run.id.clone())
                        .declared_by(binding.clone())
                        .expected(format!(
                            "`{field}` refers to something an earlier step observed"
                        ))
                        .observed(reason),
                ));
                return None;
            }
        }
    }
    Some((wanted, absent))
}
