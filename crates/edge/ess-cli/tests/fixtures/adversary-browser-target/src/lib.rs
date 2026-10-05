//! Adversary installation for the browser product's error paths. It issues a one-time secret
//! and then fails in a way the healthy fixtures never do: a trap that carries the secret in its
//! panic message, a factory that panics with it, and a target that never returns.
use ess_conformance::{
    AdvancingClock, RunnerConfig,
    scenario::OutcomeRef,
    target::*,
    web_execution::{Installation, Installed, RunContext},
};
use ess_primitives::node::Node;
use std::{cell::Cell, collections::BTreeMap};

/// The issued plaintext every browser channel is scanned for.
pub const SENTINEL: &str = "adv-sentinel-4f1c9e2a7b";

/// 0 panics on the second command while holding the issued secret;
/// 1 panics inside `Installation::create` with the secret in its message;
/// 2 issues the secret, then never returns from the second command;
/// 3 fails `Installation::create` with a typed product error;
/// 4 issues a fresh secret on every command and completes.
pub struct AdversaryInstallation<const MODE: u8>;
pub struct AdversaryTarget<const MODE: u8> {
    calls: Cell<usize>,
}
impl<const MODE: u8> Installation for AdversaryInstallation<MODE> {
    type Target = AdversaryTarget<MODE>;
    type Clock = AdvancingClock;
    fn create(
        _: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        if MODE == 1 {
            panic!("factory failed while holding {SENTINEL}");
        }
        if MODE == 3 {
            return Err(ess_conformance::web_execution::Error::ExecutionError);
        }
        Ok(Installed {
            target: AdversaryTarget {
                calls: Cell::new(0),
            },
            clock: AdvancingClock::default(),
            config: RunnerConfig::default(),
        })
    }
}
impl<const MODE: u8> ConformanceTarget for AdversaryTarget<MODE> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-controls", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let call = self.calls.get();
        self.calls.set(call + 1);
        if call > 0 {
            match MODE {
                0 => panic!("target crashed while holding {SENTINEL}"),
                2 => loop {
                    std::hint::black_box(call);
                },
                _ => {}
            }
        }
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            request.command,
            "issued".parse().unwrap(),
        ));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("write-1").unwrap());
        result.response = Some(BTreeMap::from([(
            "secret".into(),
            Node::Text(format!("{SENTINEL}-{call}")),
        )]));
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("view", "not offered"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external outcome", "not offered"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "not offered"))
    }
}
