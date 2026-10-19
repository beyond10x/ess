//! What changed between two suites, grouped by the construct each scenario is about
//! (beyond10x/ess#572).
//!
//! `ess verify diff` compares two specifications and `ess verify impact --suite` lists the earlier
//! suite's scenarios a specification change puts back to owed; neither says what a regenerated
//! suite adds, removes or changes. This does, from the two suites alone: scenario ids added,
//! removed and changed, grouped by construct, and for each changed scenario which of its parts
//! moved and the steps removed from and added to it. Scenarios are compared as read, so two
//! spellings of one scenario (a complete one-time trace and its compact suite/52 form) are the same
//! scenario.
use crate::scenario::{ConformanceScenario, ConformanceSuite, ScenarioId, ScenarioStep};
use std::collections::BTreeMap;

/// The JSON document's format.
pub const FORMAT: &str = "ess-suite-diff/1";

/// What changed between two suites.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SuiteDiff {
    /// [`FORMAT`].
    pub format: &'static str,
    /// The earlier suite.
    pub from: Side,
    /// The later suite.
    pub to: Side,
    /// Scenario counts over the whole comparison.
    pub counts: Counts,
    /// Every construct with an added, removed or changed scenario, by construct.
    pub constructs: BTreeMap<String, Construct>,
}

/// One suite of the comparison.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Side {
    /// Its format.
    pub suite_version: String,
    /// The digest of the specification it was synthesized from.
    pub spec_digest: String,
    /// How many scenarios it holds.
    pub scenarios: usize,
}

/// Scenario counts.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize)]
pub struct Counts {
    /// In the later suite only.
    pub added: usize,
    /// In the earlier suite only.
    pub removed: usize,
    /// In both, and different.
    pub changed: usize,
    /// In both, and the same.
    pub unchanged: usize,
}

/// The scenarios of one construct that moved.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct Construct {
    /// Counts for this construct.
    pub counts: Counts,
    /// Scenario ids in the later suite only.
    pub added: Vec<String>,
    /// Scenario ids in the earlier suite only.
    pub removed: Vec<String>,
    /// Scenarios in both that differ.
    pub changed: Vec<Changed>,
}

/// One scenario in both suites that differs.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Changed {
    /// Its id.
    pub scenario: String,
    /// The parts that differ, among `purpose`, `steps`, `source` and `one_time_response`.
    pub parts: Vec<&'static str>,
    /// Steps of the earlier scenario the later one does not keep, by their earlier position.
    pub removed_steps: Vec<Step>,
    /// Steps of the later scenario the earlier one did not have, by their later position.
    pub added_steps: Vec<Step>,
}

/// One step, at its position.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Step {
    /// Zero-based position in its scenario.
    pub index: usize,
    /// The step as the suite spells it.
    pub step: serde_json::Value,
}

/// The construct a scenario is about: its subject and family (`billing.invoice.CreateInvoice/outcome`,
/// `billing.invoice.Invoice/transition`), and for a disclosure cell also the origin outcome, the
/// marked field and the aspect (`keys.api.IssueKey/disclosure/issued/secret/rotation`), so each
/// cell is listed under the field and aspect it covers, one entry per caller.
pub fn construct(id: &ScenarioId) -> String {
    let id = id.to_string();
    let segments: Vec<_> = id.split('/').collect();
    let keep = if segments.get(1) == Some(&"disclosure") {
        5
    } else {
        2
    };
    segments[..keep.min(segments.len())].join("/")
}

/// Compare two suites.
pub fn diff(from: &ConformanceSuite, to: &ConformanceSuite) -> SuiteDiff {
    let side = |suite: &ConformanceSuite| Side {
        suite_version: suite.provenance.suite_version.to_string(),
        spec_digest: suite.provenance.spec_digest.to_string(),
        scenarios: suite.len(),
    };
    let mut counts = Counts::default();
    let mut constructs: BTreeMap<String, Construct> = BTreeMap::new();
    for (id, before) in &from.scenarios {
        match to.scenarios.get(id) {
            None => {
                counts.removed += 1;
                let entry = constructs.entry(construct(id)).or_default();
                entry.counts.removed += 1;
                entry.removed.push(id.to_string());
            }
            Some(after) if after == before => counts.unchanged += 1,
            Some(after) => {
                counts.changed += 1;
                let entry = constructs.entry(construct(id)).or_default();
                entry.counts.changed += 1;
                entry.changed.push(changed(id, before, after));
            }
        }
    }
    for id in to.scenarios.keys() {
        if !from.scenarios.contains_key(id) {
            counts.added += 1;
            let entry = constructs.entry(construct(id)).or_default();
            entry.counts.added += 1;
            entry.added.push(id.to_string());
        }
    }
    SuiteDiff {
        format: FORMAT,
        from: side(from),
        to: side(to),
        counts,
        constructs,
    }
}

fn changed(id: &ScenarioId, before: &ConformanceScenario, after: &ConformanceScenario) -> Changed {
    let mut parts = Vec::new();
    if before.purpose != after.purpose {
        parts.push("purpose");
    }
    if before.steps != after.steps {
        parts.push("steps");
    }
    if before.source != after.source {
        parts.push("source");
    }
    if before.one_time_response != after.one_time_response {
        parts.push("one_time_response");
    }
    let (removed_steps, added_steps) = step_changes(&before.steps, &after.steps);
    Changed {
        scenario: id.to_string(),
        parts,
        removed_steps,
        added_steps,
    }
}

/// The steps outside a longest common subsequence of the two step lists.
fn step_changes(before: &[ScenarioStep], after: &[ScenarioStep]) -> (Vec<Step>, Vec<Step>) {
    let (n, m) = (before.len(), after.len());
    let mut common = vec![vec![0_usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            common[i][j] = if before[i] == after[j] {
                common[i + 1][j + 1] + 1
            } else {
                common[i + 1][j].max(common[i][j + 1])
            };
        }
    }
    let step = |index: usize, step: &ScenarioStep| Step {
        index,
        step: serde_json::to_value(step).unwrap_or(serde_json::Value::Null),
    };
    let (mut removed, mut added) = (Vec::new(), Vec::new());
    let (mut i, mut j) = (0, 0);
    while i < n || j < m {
        if i < n && j < m && before[i] == after[j] {
            i += 1;
            j += 1;
        } else if j < m && (i == n || common[i][j + 1] >= common[i + 1][j]) {
            added.push(step(j, &after[j]));
            j += 1;
        } else {
            removed.push(step(i, &before[i]));
            i += 1;
        }
    }
    (removed, added)
}

impl SuiteDiff {
    /// The JSON form, with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self)
            .unwrap_or_else(|error| panic!("a suite diff serialises: {error}"));
        json.push('\n');
        json
    }

    /// The text form: one line for the comparison, one per construct, and under it one line per
    /// added or removed scenario and per changed scenario with its moved steps.
    pub fn to_text(&self) -> String {
        use std::fmt::Write as _;
        let mut out = String::new();
        let counts = |c: &Counts| {
            format!(
                "{} added, {} removed, {} changed",
                c.added, c.removed, c.changed
            )
        };
        let _ = writeln!(
            out,
            "{} ({} scenarios) -> {} ({} scenarios): {}, {} unchanged",
            self.from.suite_version,
            self.from.scenarios,
            self.to.suite_version,
            self.to.scenarios,
            counts(&self.counts),
            self.counts.unchanged
        );
        for (name, construct) in &self.constructs {
            let _ = writeln!(out, "{name}: {}", counts(&construct.counts));
            for id in &construct.added {
                let _ = writeln!(out, "  + {id}");
            }
            for id in &construct.removed {
                let _ = writeln!(out, "  - {id}");
            }
            for changed in &construct.changed {
                let _ = writeln!(
                    out,
                    "  ~ {} ({})",
                    changed.scenario,
                    changed.parts.join(", ")
                );
                for (sign, steps) in [("-", &changed.removed_steps), ("+", &changed.added_steps)] {
                    for step in steps {
                        let _ =
                            writeln!(out, "      {sign} [{}] {}", step.index, brief(&step.step));
                    }
                }
            }
        }
        out
    }
}

/// A step on one line, cut at 160 characters.
fn brief(step: &serde_json::Value) -> String {
    let text = step.to_string();
    if text.chars().count() <= 160 {
        return text;
    }
    let mut cut: String = text.chars().take(159).collect();
    cut.push('…');
    cut
}
