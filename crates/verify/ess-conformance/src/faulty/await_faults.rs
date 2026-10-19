//! Targets wrong about a command the system issues itself, in exactly one way each
//! (beyond10x/ess#542, `docs/design/system-issued-commands.md`, *How each faulty target fails*).
//!
//! A family of its own rather than rows of [`Fault`](super::Fault): that matrix runs each fault
//! against a reference specification with a designated scenario, and none of those specifications
//! declares a system actor. Each [`AwaitFault`] perturbs only the await answer, and what a later
//! observation of the same scenario reads; every other call is forwarded unchanged, so a wrapper
//! cannot hide a capability the target underneath has.
//!
//! | fault | caught at |
//! |---|---|
//! | [`NeverTakesStep`](AwaitFault::NeverTakesStep) | `await_outcome`: nothing within the bound |
//! | [`TooLate`](AwaitFault::TooLate) | `await_outcome`: a reading above `within` |
//! | [`OtherInstance`](AwaitFault::OtherInstance) | `await_outcome`: the identity input |
//! | [`OtherBranch`](AwaitFault::OtherBranch) | `expect_outcome` |
//! | [`DisagreeingInput`](AwaitFault::DisagreeingInput) | the expectation reading `{kind: awaited}` |
//! | [`WrongInputType`](AwaitFault::WrongInputType) | `await_outcome`, `ESS-CF-PAYLOAD` |
//! | [`TakesRefusedStep`](AwaitFault::TakesRefusedStep) | `expect_not_issued`: an outcome reached |
//! | [`PublishesSilently`](AwaitFault::PublishesSilently) | `expect_not_issued`'s `unpublished` |
//! | [`AnswersAtOnce`](AwaitFault::AnswersAtOnce) | with `within`, the short reading; without, never believed before the runner's deadline |

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_primitives::node::Node;

use crate::scenario::OutcomeRef;
use crate::target::{
    AbsentInputRequest, AwaitAnswer, AwaitRequest, ConformanceTarget, ElapsedObservation,
    ElapsedObservationRequest, EntitySetupRequest, EventDeliveryRequest, EventObservationRequest,
    ExternalOutcomeControl, ImplementationIdentity, InstantMark, InvocationObservationRequest,
    ObservedEvent, ObservedInvocation, OrderedScan, OrderedScanRequest, RecordedCommandCompletion,
    RedeliveryRequest, ScenarioContext, SemanticCommandRequest, SemanticCommandResult,
    SemanticViewRequest, SemanticViewResult, TargetError,
};

/// One way a target can be wrong about a command its system issues itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AwaitFault {
    /// The system never takes the step.
    NeverTakesStep,
    /// It takes the step, later than the declared bound.
    TooLate,
    /// It reports the step on another instance than the one awaited.
    OtherInstance,
    /// It takes another branch than the one the scenario requires.
    OtherBranch,
    /// It reports an input that disagrees with what it published and stored.
    DisagreeingInput,
    /// It reports an input of the wrong type.
    WrongInputType,
    /// It takes the step from a state the specification says refuses it.
    TakesRefusedStep,
    /// It reports nothing, and publishes the command's event anyway.
    PublishesSilently,
    /// It answers "nothing" at once, without holding the window open.
    AnswersAtOnce,
}

impl AwaitFault {
    /// Every await fault, in declaration order.
    pub const ALL: &'static [Self] = &[
        Self::NeverTakesStep,
        Self::TooLate,
        Self::OtherInstance,
        Self::OtherBranch,
        Self::DisagreeingInput,
        Self::WrongInputType,
        Self::TakesRefusedStep,
        Self::PublishesSilently,
        Self::AnswersAtOnce,
    ];

    /// How it is written, in the identity a faulty target answers with.
    pub fn written(self) -> &'static str {
        match self {
            Self::NeverTakesStep => "never-takes-step",
            Self::TooLate => "too-late",
            Self::OtherInstance => "other-instance",
            Self::OtherBranch => "other-branch",
            Self::DisagreeingInput => "disagreeing-input",
            Self::WrongInputType => "wrong-input-type",
            Self::TakesRefusedStep => "takes-refused-step",
            Self::PublishesSilently => "publishes-silently",
            Self::AnswersAtOnce => "answers-at-once",
        }
    }
}

/// A target whose await answers are wrong in exactly the way its [`AwaitFault`] says.
#[derive(Debug)]
pub struct AwaitFaulty<T> {
    inner: T,
    fault: AwaitFault,
    /// What the last reached answer published: what a silent step publishes again.
    reached: RefCell<Vec<ObservedEvent>>,
    /// Occurrences published without an answer saying so ([`AwaitFault::PublishesSilently`]).
    silent: RefCell<Vec<ObservedEvent>>,
}

impl<T> AwaitFaulty<T> {
    /// Wraps `inner` so that `fault` is injected into its await answers.
    pub fn new(inner: T, fault: AwaitFault) -> Self {
        Self {
            inner,
            fault,
            reached: RefCell::new(Vec::new()),
            silent: RefCell::new(Vec::new()),
        }
    }

    /// The target underneath.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// The fault it carries.
    pub fn fault(&self) -> AwaitFault {
        self.fault
    }
}

/// Every input value other than the awaited instance, changed by `change`.
fn reported(
    input: BTreeMap<String, Node>,
    instance: &Node,
    change: impl Fn(Node) -> Node,
) -> BTreeMap<String, Node> {
    input
        .into_iter()
        .map(|(field, value)| {
            if &value == instance {
                (field, value)
            } else {
                (field, change(value))
            }
        })
        .collect()
}

fn outcome(request: &AwaitRequest, name: &str) -> OutcomeRef {
    OutcomeRef::new(
        request.command.clone(),
        name.parse().expect("a well-formed outcome name"),
    )
}

impl<T: ConformanceTarget> AwaitFaulty<T> {
    fn faulted(&self, request: &AwaitRequest, answer: AwaitAnswer) -> AwaitAnswer {
        let within_ms = request.within.map(crate::scenario::Elapsed::millis);
        match (self.fault, answer) {
            (AwaitFault::NeverTakesStep, AwaitAnswer::Reached { .. }) => AwaitAnswer::NotIssued {
                elapsed_ms: within_ms,
            },
            (AwaitFault::TooLate, AwaitAnswer::Reached { result, input, .. }) => {
                AwaitAnswer::Reached {
                    result,
                    input,
                    elapsed_ms: Some(within_ms.unwrap_or(0) + 1_000),
                }
            }
            (
                AwaitFault::OtherInstance,
                AwaitAnswer::Reached {
                    result,
                    input,
                    elapsed_ms,
                },
            ) => AwaitAnswer::Reached {
                result,
                input: input
                    .into_iter()
                    .map(|(field, value)| {
                        if value == request.instance {
                            (field, Node::Text("another-instance".to_owned()))
                        } else {
                            (field, value)
                        }
                    })
                    .collect(),
                elapsed_ms,
            },
            (
                AwaitFault::OtherBranch,
                AwaitAnswer::Reached {
                    mut result,
                    input,
                    elapsed_ms,
                },
            ) => {
                result.outcome = Some(outcome(request, "another-branch"));
                AwaitAnswer::Reached {
                    result,
                    input,
                    elapsed_ms,
                }
            }
            (
                AwaitFault::DisagreeingInput,
                AwaitAnswer::Reached {
                    result,
                    input,
                    elapsed_ms,
                },
            ) => AwaitAnswer::Reached {
                result,
                input: reported(input, &request.instance, |value| match value {
                    Node::Text(text) => Node::Text(format!("{text}-reported")),
                    other => other,
                }),
                elapsed_ms,
            },
            (
                AwaitFault::WrongInputType,
                AwaitAnswer::Reached {
                    result,
                    input,
                    elapsed_ms,
                },
            ) => AwaitAnswer::Reached {
                result,
                input: reported(input, &request.instance, |_| Node::Bool(true)),
                elapsed_ms,
            },
            (AwaitFault::TakesRefusedStep, AwaitAnswer::NotIssued { .. }) => AwaitAnswer::Reached {
                result: SemanticCommandResult::took(outcome(request, "taken-anyway")),
                input: BTreeMap::new(),
                elapsed_ms: Some(0),
            },
            (AwaitFault::PublishesSilently, answer @ AwaitAnswer::NotIssued { .. }) => {
                let again = self.reached.borrow().clone();
                self.silent.borrow_mut().extend(again);
                answer
            }
            (AwaitFault::AnswersAtOnce, AwaitAnswer::NotIssued { .. }) => AwaitAnswer::NotIssued {
                elapsed_ms: Some(0),
            },
            (_, answer) => answer,
        }
    }
}

impl<T: ConformanceTarget> ConformanceTarget for AwaitFaulty<T> {
    fn fixture_values(
        &self,
        scenario: &ScenarioContext,
        contract: &crate::fixtures::Contract,
    ) -> Result<BTreeMap<String, Node>, TargetError> {
        self.inner.fixture_values(scenario, contract)
    }

    fn open_periodic(
        &self,
        request: crate::periodic::Open,
    ) -> Result<crate::periodic::Opened, TargetError> {
        self.inner.open_periodic(request)
    }

    fn observe_periodic(
        &self,
        request: crate::periodic::Observe,
    ) -> Result<crate::periodic::Observation, TargetError> {
        self.inner.observe_periodic(request)
    }

    fn close_periodic(
        &self,
        scope: crate::periodic::Scope,
    ) -> Result<crate::periodic::Closed, TargetError> {
        self.inner.close_periodic(scope)
    }

    fn observe_clock_reading(
        &self,
        request: crate::reading::ReadingObservationRequest,
    ) -> Result<ess_domain::reading::ReadingEvidence, TargetError> {
        self.inner.observe_clock_reading(request)
    }

    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }

    /// The implementation underneath, named for the defect it carries.
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        let inner = self.inner.identity()?;
        Ok(ImplementationIdentity::new(
            format!("{}-{}", inner.name, self.fault.written()),
            inner.version,
        ))
    }

    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.reached.borrow_mut().clear();
        self.silent.borrow_mut().clear();
        self.inner.begin_scenario(scenario)
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }

    fn execute_command_recorded(
        &self,
        request: SemanticCommandRequest,
    ) -> RecordedCommandCompletion {
        self.inner.execute_command_recorded(request)
    }

    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }

    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &crate::scenario::ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view_as(request, reader)
    }

    fn query_view_anonymous(
        &self,
        request: SemanticViewRequest,
    ) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view_anonymous(request)
    }

    /// What the target underneath published, and what a silent step published beside it.
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let event = request.event.clone();
        let mut observed = self.inner.observe_events(request)?;
        observed.extend(
            self.silent
                .borrow()
                .iter()
                .filter(|seen| seen.event == event)
                .cloned(),
        );
        Ok(observed)
    }

    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }

    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        self.inner
            .configure_external_outcome_repeatedly(request, times)
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }

    fn deliver_event(&self, request: EventDeliveryRequest) -> Result<(), TargetError> {
        self.inner.deliver_event(request)
    }

    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }

    fn mark_instant(&self, request: InstantMark) -> Result<(), TargetError> {
        self.inner.mark_instant(request)
    }

    fn observe_elapsed(
        &self,
        request: ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        self.inner.observe_elapsed(request)
    }

    fn scan_view(&self, request: OrderedScanRequest) -> Result<OrderedScan, TargetError> {
        self.inner.scan_view(request)
    }

    fn scan_view_as(
        &self,
        request: OrderedScanRequest,
        reader: &crate::scenario::ActorRef,
    ) -> Result<OrderedScan, TargetError> {
        self.inner.scan_view_as(request, reader)
    }

    fn scan_view_anonymous(&self, request: OrderedScanRequest) -> Result<OrderedScan, TargetError> {
        self.inner.scan_view_anonymous(request)
    }

    fn await_outcome(&self, request: AwaitRequest) -> Result<AwaitAnswer, TargetError> {
        let answer = self.inner.await_outcome(request.clone())?;
        if let AwaitAnswer::Reached { result, .. } = &answer {
            self.reached.borrow_mut().clone_from(&result.direct_events);
        }
        Ok(self.faulted(&request, answer))
    }

    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}
