//! Complete target failure, independent of capability accounting.

use std::fmt;

use serde::Serialize;

use crate::{SynthesisPlan, Target};

/// A closed rule under which the current target cannot represent valid ESS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetFailureCode {
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
        assert!(!detail.is_empty());
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
        target: Target,
        plan: &SynthesisPlan,
        mut causes: Vec<TargetFailureCause>,
    ) -> Self {
        causes.sort();
        causes.dedup();
        assert!(!causes.is_empty());
        Self {
            format: if matches!(target, Target::Rust | Target::Web) {
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
        Err(TargetFailure::new(target, plan, causes))
    }
}
