//! A refusal-selected failure policy, resolved (ess/22, beyond10x/ess#269).
//!
//! `docs/design/conditional-binding-failure-policies.md`, "Typed representation and consumers":
//! one typed table, every declared refusal of the invoked command in the command's declaration
//! order mapped to the concrete policy its aliases resolved to, plus the explicit fallback for a
//! failure that carries no declared outcome. Ordered collections only. Read through
//! [`ResolvedFailure::ByRefusal`](super::ResolvedFailure::ByRefusal); the binding's universal
//! fields are a view [`agrees_with`](ResolvedRefusalPolicy::agrees_with) checks, never a policy for
//! every refusal.

use ess_domain::binding::refusal::RefusalPolicy;
use ess_domain::binding::Failure;
use ess_domain::command::{CommandSpec, OutcomeName};

use super::{EventHandle, ResolvedBinding, ResolvedRetryBound};

/// What one refusal, or the fallback, is answered with.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "policy", rename_all = "snake_case")]
pub enum ResolvedRefusalAction {
    /// Give up: the work is lost, and nothing is published.
    Drop,
    /// Try again: with no bound, on whatever schedule the transport provides; with one, until the
    /// occurrence's total invocations reach `attempts` or a `final` refusal answers.
    Retry {
        /// The bound, where the retry states one.
        #[serde(skip_serializing_if = "Option::is_none")]
        bound: Option<ResolvedRetryBound>,
    },
    /// Publish this event once, from the failed attempt's actual input, and stop.
    Escalate {
        /// The event the escalation emits.
        emits: EventHandle,
    },
}

impl ResolvedRefusalAction {
    /// The word a document writes for this policy.
    pub fn word(&self) -> Failure {
        match self {
            Self::Drop => Failure::Drop,
            Self::Retry { .. } => Failure::Retry,
            Self::Escalate { .. } => Failure::Escalate,
        }
    }
}

/// One declared refusal and the policy that answers it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ResolvedRefusalRule {
    /// The invoked command's outcome — one that carries an `error:`.
    pub outcome: OutcomeName,
    /// What answers it.
    #[serde(flatten)]
    pub action: ResolvedRefusalAction,
}

/// A binding's failure policy, selected per refusal of the invoked command.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ResolvedRefusalPolicy {
    /// Every declared refusal of the invoked command, in the command's declaration order.
    pub refusals: Vec<ResolvedRefusalRule>,
    /// What answers a failure of the invoked command port that carries no declared outcome.
    pub fallback: ResolvedRefusalAction,
}

impl ResolvedRefusalPolicy {
    /// The table an admitted policy resolves to against `command`, with the binding's resolved
    /// escalation event; `None` where the policy was not admitted against it.
    pub(crate) fn resolve(
        policy: &RefusalPolicy,
        command: &CommandSpec,
        escalation: Option<&EventHandle>,
    ) -> Option<Self> {
        let assignment = policy.assign(command)?;
        let action = |index: usize| -> Option<ResolvedRefusalAction> {
            let entry = policy.entries.get(index)?;
            Some(match entry.policy {
                Failure::Drop => ResolvedRefusalAction::Drop,
                Failure::Retry => ResolvedRefusalAction::Retry {
                    bound: entry.attempts.map(|attempts| ResolvedRetryBound {
                        attempts,
                        final_outcomes: policy
                            .finals(index, command)
                            .into_iter()
                            .cloned()
                            .collect(),
                    }),
                },
                Failure::Escalate => ResolvedRefusalAction::Escalate {
                    emits: escalation?.clone(),
                },
            })
        };
        let refusals = assignment
            .refusals
            .iter()
            .map(|(outcome, index)| {
                Some(ResolvedRefusalRule {
                    outcome: (*outcome).clone(),
                    action: action(*index)?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            refusals,
            fallback: action(assignment.fallback)?,
        })
    }

    /// What answers a failed attempt: its declared refusal's policy, or the fallback for a failure
    /// that carries no declared outcome. An outcome outside the table — which an admitted policy
    /// cannot meet, because the table holds every declared refusal — is answered by the fallback.
    pub fn select(&self, outcome: Option<&OutcomeName>) -> &ResolvedRefusalAction {
        outcome
            .and_then(|outcome| self.rule(outcome))
            .unwrap_or(&self.fallback)
    }

    /// The policy a declared refusal is answered with.
    pub fn rule(&self, outcome: &OutcomeName) -> Option<&ResolvedRefusalAction> {
        self.refusals
            .iter()
            .find(|rule| &rule.outcome == outcome)
            .map(|rule| &rule.action)
    }

    /// Every policy the table uses, the fallback included.
    pub fn actions(&self) -> impl Iterator<Item = &ResolvedRefusalAction> {
        self.refusals
            .iter()
            .map(|rule| &rule.action)
            .chain(std::iter::once(&self.fallback))
    }

    /// The event the table's `escalate` publishes, where it has one. One `escalate` per policy, so
    /// at most one event.
    pub fn escalation(&self) -> Option<&EventHandle> {
        self.actions().find_map(|action| match action {
            ResolvedRefusalAction::Escalate { emits } => Some(emits),
            _ => None,
        })
    }

    /// The bound the table's `retry` states, where it states one.
    pub fn bound(&self) -> Option<&ResolvedRetryBound> {
        self.actions().find_map(|action| match action {
            ResolvedRefusalAction::Retry { bound } => bound.as_ref(),
            _ => None,
        })
    }

    /// Whether `binding`'s universal fields are the view of this table the compiler derives: the
    /// fallback's word, the table's escalation event and the table's retry bound.
    pub fn agrees_with(&self, binding: &ResolvedBinding) -> bool {
        binding.failure == self.fallback.word()
            && binding.escalation.as_ref() == self.escalation()
            && binding.retry.as_ref() == self.bound()
    }
}
