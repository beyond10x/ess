//! Complete target failure, independent of capability accounting.

use std::fmt;

use serde::Serialize;

use crate::{SynthesisPlan, Target};

/// A closed rule under which the current target cannot represent valid ESS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetFailureCode {
    /// New bounded-accessor source exceeds its per-mapping or aggregate output budget.
    AccessorResource,
    /// A selected input requires invariant or reading validation the native selector cannot implement.
    SelectionConstraint,
    /// An allocated token cannot name the emitted Rust item.
    InvalidIdentifier,
    /// Two meanings occupy the same emitted namespace.
    SymbolCollision,
    /// Allocated files collide or cannot resolve their module declaration.
    PathCollision,
    /// A generated value contains itself without size-breaking indirection.
    RecursiveLayout,
    /// A generated binding expression does not have the input's Rust type.
    BindingAssignment,
    /// The emitter has no domain location for a declared type.
    MissingTypeOwner,
    /// A generated codec cannot distinguish the declared wire identities.
    WireCollision,
    /// A required reference has no representation in the target.
    MissingRepresentation,
}

/// One source-addressed reason that a complete target cannot be emitted.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct TargetFailureCause {
    code: TargetFailureCode,
    sources: Vec<String>,
    detail: String,
}

impl TargetFailureCause {
    /// The machine-readable target rule.
    pub fn code(&self) -> TargetFailureCode {
        self.code
    }
    /// The nonempty, sorted source identities responsible for this cause.
    pub fn sources(&self) -> &[String] {
        &self.sources
    }
    /// The allocated representation and why it cannot be emitted.
    pub fn detail(&self) -> &str {
        &self.detail
    }

    pub(crate) fn new(code: TargetFailureCode, mut sources: Vec<String>, detail: String) -> Self {
        sources.sort();
        sources.dedup();
        assert!(!sources.is_empty() && sources.iter().all(|source| !source.is_empty()));
        assert_ne!(detail.len(), 0, "a target failure names its detail");
        Self {
            code,
            sources,
            detail,
        }
    }
}

/// A complete emission failure. No artifacts accompany this value.
///
/// Construction is private: consumers can inspect or serialize a failure, but cannot invent an
/// empty failure or a report for a target this envelope does not cover.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TargetFailure {
    format: &'static str,
    target: &'static str,
    plan: Box<SynthesisPlan>,
    causes: Vec<TargetFailureCause>,
}

impl TargetFailure {
    pub(crate) fn new(
        ir: &ess_compiler::EssIr,
        target: Target,
        plan: &SynthesisPlan,
        mut causes: Vec<TargetFailureCause>,
    ) -> Self {
        causes.sort();
        causes.dedup();
        assert_ne!(causes.len(), 0, "a target failure names at least one cause");
        Self {
            format: if causes.iter().any(|cause| {
                matches!(
                    cause.code,
                    TargetFailureCode::AccessorResource | TargetFailureCode::SelectionConstraint
                )
            }) || ir
                .types()
                .values()
                .any(|declared| declared.reading.is_some())
                || ir.bindings().values().any(|binding| {
                    binding.selection.is_some()
                        || binding.cause.periodic().is_some()
                        || binding.mapping.iter().any(|mapping| {
                            matches!(
                                mapping.value,
                                ess_compiler::ir::ResolvedMappingValue::EventAccessor { .. }
                            )
                        })
                }) {
                "ess-target-failure/3"
            } else if matches!(target, Target::Rust | Target::Web) {
                "ess-target-failure/1"
            } else {
                "ess-target-failure/2"
            },
            target: target.name(),
            plan: Box::new(plan.clone()),
            causes,
        }
    }

    /// The failed target.
    pub fn target(&self) -> &str {
        self.target
    }
    /// The unchanged neutral plan, including an empty capability set when appropriate.
    pub fn plan(&self) -> &SynthesisPlan {
        &self.plan
    }
    /// All fatal causes, deterministically ordered and nonempty.
    pub fn causes(&self) -> &[TargetFailureCause] {
        &self.causes
    }
    /// The versioned typed envelope: pretty JSON followed by exactly one newline.
    pub fn to_canonical_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self).expect("a target failure serializes");
        json.push('\n');
        json
    }
}

impl fmt::Display for TargetFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} target cannot emit this workspace", self.target)?;
        for cause in &self.causes {
            write!(f, "\n{}: {}", cause.sources.join(", "), cause.detail)?;
        }
        Ok(())
    }
}

impl std::error::Error for TargetFailure {}

pub(crate) fn binary64(
    ir: &ess_compiler::EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<(), TargetFailure> {
    let causes = ess_compiler::binary64::locations(ir)
        .into_iter()
        .map(|at| {
            TargetFailureCause::new(
                TargetFailureCode::MissingRepresentation,
                vec![at],
                "this target has no qualified finite Binary64 codec".to_owned(),
            )
        })
        .collect::<Vec<_>>();
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, target, plan, causes))
    }
}

/// A model that declares an `input_absent:` branch (ess/16, beyond10x/ess#170) is refused by every
/// code target.
///
/// The generated seams decode a request into the command's input before any branch is selected,
/// so a request with no body never reaches a branch the generated code could select, and a seam
/// that treated it as `{}` would answer the other request. Each branch is named rather than
/// emitted with a meaning nobody chose.
pub(crate) fn input_absent(
    ir: &ess_compiler::EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<(), TargetFailure> {
    let causes = ir
        .commands()
        .values()
        .flat_map(|command| {
            command
                .outcomes
                .iter()
                .filter(|outcome| {
                    outcome.condition == ess_compiler::ir::ResolvedCondition::InputAbsent
                })
                .map(move |outcome| {
                    TargetFailureCause::new(
                        TargetFailureCode::MissingRepresentation,
                        vec![format!(
                            "commands.{}.outcomes.{}.input_absent",
                            command.name, outcome.name
                        )],
                        "this target has no seam for a request with no input at all".to_owned(),
                    )
                })
        })
        .collect::<Vec<_>>();
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, target, plan, causes))
    }
}

/// Structural code generation cannot supply atomic, durable one-time issuance.
/// Refuse the named policy before emitting any incomplete implementation artifacts.
pub(crate) fn one_time_response(
    ir: &ess_compiler::EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<(), TargetFailure> {
    let causes = ir.commands().values().flat_map(|command| {
        command.outcomes.iter().filter(|outcome| !outcome.one_time_response.is_empty()).map(move |outcome| {
            TargetFailureCause::new(
                TargetFailureCode::MissingRepresentation,
                vec![format!("commands.{}.outcomes.{}.one_time_response", command.name, outcome.name)],
                "one_time_response requires implementation-owned atomic durable consumption and fresh issuance; this code target cannot implement that policy".to_owned(),
            )
        })
    }).collect::<Vec<_>>();
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, target, plan, causes))
    }
}

/// Every binding construct the generated dispatch cannot represent, refused by name: a policy
/// selected per refusal ([`refusal_policy`]) first, then a bounded retry ([`retry_bound`]). An
/// event-payload condition (ess/22) is represented in every shape, selections and proved members
/// included (`rust::condition`, `go::condition`).
pub(crate) fn binding_policies(
    ir: &ess_compiler::EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<(), TargetFailure> {
    refusal_policy(ir, plan, target)?;
    retry_bound(ir, plan, target)
}

/// A binding whose failure policy is selected per refusal (ess/22, beyond10x/ess#269) is refused
/// by every target that delivers bindings. The command-line target delivers none, so it has
/// nothing to refuse.
///
/// The generated dispatch answers every failure of a binding with one policy and counts no
/// attempts, so emitting it would apply one policy — the fallback's, or none — to every declared
/// refusal, and could not stop a retry at its total bound. Each binding is named, so the
/// representation is owed rather than silently wrong; the refusal is checked before the bounded
/// retry's, because a selected policy's retry bound is one of its own policies.
pub(crate) fn refusal_policy(
    ir: &ess_compiler::EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<(), TargetFailure> {
    if target == Target::Clap {
        return Ok(());
    }
    let causes = ir
        .bindings()
        .values()
        .filter(|binding| binding.refusal_policy.is_some())
        .map(|binding| {
            TargetFailureCause::new(
                TargetFailureCode::MissingRepresentation,
                vec![format!("bindings.{}.on_failure", binding.name)],
                "this target's dispatch answers every failure with one policy and counts no \
                 attempts, so it cannot select the policy per refusal of the invoked command"
                    .to_owned(),
            )
        })
        .collect::<Vec<_>>();
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, target, plan, causes))
    }
}

/// A binding with a bounded retry (ess/16, beyond10x/ess#165) is refused by every target that
/// delivers bindings, as `Json` is. The command-line target delivers none, so it has nothing to
/// refuse.
///
/// The generated retry holds the event for the next pump and counts no attempts, so it cannot stop
/// after `attempts:` or on a `final:` refusal; emitting it would retry forever where the
/// specification says how many times. Each binding is named rather than emitted with a meaning
/// nobody chose.
pub(crate) fn retry_bound(
    ir: &ess_compiler::EssIr,
    plan: &SynthesisPlan,
    target: Target,
) -> Result<(), TargetFailure> {
    if target == Target::Clap {
        return Ok(());
    }
    let causes = ir
        .bindings()
        .values()
        .filter(|binding| binding.retry.is_some())
        .map(|binding| {
            TargetFailureCause::new(
                TargetFailureCode::MissingRepresentation,
                vec![format!("bindings.{}.on_failure.retry", binding.name)],
                "this target's retry holds the event for the next pump and counts no attempts, \
                 so it cannot stop after `attempts:` or on a `final:` refusal"
                    .to_owned(),
            )
        })
        .collect::<Vec<_>>();
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, target, plan, causes))
    }
}
