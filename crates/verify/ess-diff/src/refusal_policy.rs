//! The typed, normalized content of a refusal-selected failure policy, as a delta carries it
//! (`ess-diff/14`, ess/22, beyond10x/ess#269).
//!
//! The complete resolved table: every declared refusal of the invoked command with the policy that
//! answers it, keyed by outcome name (in name order, `final` too), and the fallback for a failure
//! carrying no declared outcome. Aliases are already resolved and the command's own declaration
//! order is not part of it, so two revisions that name the same refusals differently, or declare
//! them in another order, carry equal content and compare equal: a reorder is the command's
//! change, not the binding's. Identities are the declared names, never an IR's handles, so a
//! delta reads without either compiled model.

use ess_compiler::ir::{ResolvedRefusalAction, ResolvedRefusalPolicy};
use ess_conformance::scenario::EventRef;

/// What one refusal, or the fallback, is answered with.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "policy", rename_all = "snake_case")]
pub enum RefusalAction {
    /// Give up.
    Drop,
    /// Try again, bounded or not.
    Retry {
        /// Total invocations for one occurrence, where the retry is bounded.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attempts: Option<u32>,
        /// The refusals that end it at once, in name order.
        #[serde(default, rename = "final", skip_serializing_if = "Vec::is_empty")]
        finals: Vec<String>,
    },
    /// Publish this event once, and stop.
    Escalate {
        /// The event.
        emits: EventRef,
    },
}

impl RefusalAction {
    fn of(action: &ResolvedRefusalAction) -> Self {
        match action {
            ResolvedRefusalAction::Drop => Self::Drop,
            ResolvedRefusalAction::Retry { bound } => Self::Retry {
                attempts: bound.as_ref().map(|bound| bound.attempts),
                finals: bound
                    .iter()
                    .flat_map(|bound| bound.final_outcomes.iter().map(ToString::to_string))
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect(),
            },
            ResolvedRefusalAction::Escalate { emits } => Self::Escalate {
                emits: EventRef::from(emits),
            },
        }
    }

    /// The policy in a few words, for a description.
    pub fn describe(&self) -> String {
        match self {
            Self::Drop => "drop".to_owned(),
            Self::Retry { attempts: None, .. } => "retry".to_owned(),
            Self::Retry {
                attempts: Some(attempts),
                finals,
            } if finals.is_empty() => format!("retry, {attempts} attempts"),
            Self::Retry {
                attempts: Some(attempts),
                finals,
            } => format!(
                "retry, {attempts} attempts, final {}",
                finals
                    .iter()
                    .map(|outcome| format!("`{outcome}`"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Escalate { emits } => format!("escalate, publishing `{emits}`"),
        }
    }
}

/// One declared refusal and its policy.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RefusalRule {
    /// The outcome of the invoked command.
    pub outcome: String,
    /// What answers it.
    #[serde(flatten)]
    pub action: RefusalAction,
}

/// A refusal-selected policy's complete resolved table and fallback.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefusalPolicyContent {
    /// Every declared refusal, in outcome-name order.
    pub refusals: Vec<RefusalRule>,
    /// What answers a failure carrying no declared outcome.
    pub fallback: RefusalAction,
}

impl RefusalPolicyContent {
    /// The content a resolved table carries.
    pub fn of(policy: &ResolvedRefusalPolicy) -> Self {
        let mut refusals: Vec<RefusalRule> = policy
            .refusals
            .iter()
            .map(|rule| RefusalRule {
                outcome: rule.outcome.to_string(),
                action: RefusalAction::of(&rule.action),
            })
            .collect();
        refusals.sort_by(|left, right| left.outcome.cmp(&right.outcome));
        Self {
            refusals,
            fallback: RefusalAction::of(&policy.fallback),
        }
    }

    /// The table in one line, for a description.
    pub fn describe(&self) -> String {
        let mut parts: Vec<String> = self
            .refusals
            .iter()
            .map(|rule| format!("`{}` {}", rule.outcome, rule.action.describe()))
            .collect();
        parts.push(format!("otherwise {}", self.fallback.describe()));
        parts.join("; ")
    }
}
