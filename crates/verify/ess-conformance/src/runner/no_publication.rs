//! The runner's half of an event that must be published an exact number of times — none, or
//! `count` (suite/36, ess/22, beyond10x/ess#269, [`crate::refusal_policy`]).
//!
//! [`ScenarioStep::ExpectNoPublication`](crate::ScenarioStep::ExpectNoPublication) and
//! [`ScenarioStep::ExpectPublicationCount`](crate::ScenarioStep::ExpectPublicationCount) ask the
//! target for the event's occurrences under the scenario's correlation until the run's deadline
//! passes. More occurrences than allowed, at any ask, fail at once, including ones made visible
//! only late. The allowed number passes only once the deadline has passed. A target that cannot
//! observe the event leaves the check unsupported.

use std::num::NonZeroU32;

use super::{
    target_failure, CheckCode, CheckResult, Clock, ConformanceTarget, Diagnostic, Flow, Run, Runner,
};
use crate::scenario::EventRef;
use crate::target::EventObservationRequest;

impl<C: Clock> Runner<C> {
    /// Requires that `event` is published exactly `count` times under this scenario — no times
    /// where `count` is `None` — through the deadline.
    pub(super) fn expect_publications<T: ConformanceTarget>(
        &mut self,
        event: &EventRef,
        count: Option<NonZeroU32>,
        run: &mut Run,
        target: &T,
    ) -> Flow {
        let wanted = count.map_or(0, |count| {
            usize::try_from(count.get()).unwrap_or(usize::MAX)
        });
        let about = format!("`{event}` is published exactly {wanted} time(s) for the whole window");
        let deadline = self.deadline();
        let mut asks = 0_u32;
        loop {
            asks += 1;
            let request = EventObservationRequest {
                event: event.clone(),
                correlation: run.context.correlation.clone(),
                deadline,
            };
            let published = match target.observe_events(request) {
                Ok(published) => published,
                Err(error) if error.is_unsupported() => {
                    run.record(CheckResult::unsupported(
                        about,
                        Diagnostic::new(CheckCode::Event, run.id.clone())
                            .declared_by(event.clone())
                            .expected("the target exposes the events the system publishes (§13)")
                            .observed(error.to_string()),
                    ));
                    return Flow::Continue;
                }
                Err(error) => {
                    run.record(target_failure(
                        &run.id,
                        &format!("observing how often `{event}` was published"),
                        &error,
                    ));
                    return Flow::Stop;
                }
            };
            let seen = published
                .iter()
                .filter(|occurrence| &occurrence.event == event)
                .count();
            if seen <= wanted && !deadline.has_passed(self.clock.now()) {
                continue;
            }
            if seen == wanted {
                run.record(CheckResult::passed(CheckCode::Event, about));
            } else {
                run.record(CheckResult::failed(
                    about,
                    Diagnostic::new(CheckCode::Event, run.id.clone())
                        .declared_by(event.clone())
                        .expected(format!(
                            "`{event}` is published exactly {wanted} time(s) through the deadline"
                        ))
                        .observed(format!("{seen} occurrence(s) at observation {asks}")),
                ));
            }
            return Flow::Continue;
        }
    }
}
