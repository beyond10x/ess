//! The ratchet of the documentation website's UI iteration loop, and the check of its findings.
//!
//! `website/ux/` measures the site: a Playwright run writes `round.json`, one number per scenario
//! and metric key, every metric lower-is-better, and the round's coverage. A scenario is one page
//! configuration (`<path>@<viewport>/<theme>`) or one task run (`<task>/<viewport>[/keyboard]`).
//! This module decides whether those numbers may land. It fails on:
//!
//! 1. a configuration or task run the round expected and did not measure (`coverage.unmeasured`);
//!    no update takes it, a rerun that measures it is the only way past;
//! 2. a value worse than `website/ux/baseline.json` beyond the metric's tolerance;
//! 3. a value better than the baseline that was not taken into it, within the tolerance or not:
//!    an improvement becomes the baseline in the same change;
//! 4. a value the baseline holds and the round does not: lost coverage, a page set that shrank;
//! 5. a new value: a page, task or metric the baseline does not hold, or a scenario new to the
//!    round's lane. It is reported as new, not as worse, and `--update` takes it without a
//!    decision; the values of a scenario new to the lane are judged in the same pass;
//! 6. a baseline made worse than the one on `main` without a decision id: a worse entry of any
//!    tolerance class, a dropped entry or a scenario a lane dropped;
//! 7. a round measured by an engine with uncommitted changes (`engine.dirty`), or merged from runs
//!    of different engines (`engine.runs_agree` false). No update takes it.
//!
//! A round has a lane. `full` holds every scenario of the baseline to it; any other lane (the
//! every-merge `gate`) holds only the scenarios the baseline lists under `lanes.<lane>`.
//!
//! `--update` writes the baseline only after every comparison and the findings check passed, so a
//! failed run leaves the file as it was.
//!
//! The Playwright specs only measure; the decision is here because a checker in this repository is
//! Rust. The same goes for the findings rule: no closed S1 or S2 record under `website/ux/findings/`
//! without a regression asset that exists, and an asset that names a test names one its file
//! declares.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{self, Write as _};
use std::fs;
use std::path::Path;
use std::process::Command;

/// The baseline, relative to the repository root.
pub(crate) const BASELINE: &str = "website/ux/baseline.json";

/// The finding records, relative to the repository root.
pub(crate) const FINDINGS: &str = "website/ux/findings";

/// The lane that holds every scenario of the baseline.
pub(crate) const FULL: &str = "full";

/// Comparison slack for floating-point values read from JSON.
const EPSILON: f64 = 1e-9;

/// How far a value may move before it counts as worse.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum Tolerance {
    /// A fixed amount in the metric's unit.
    Absolute(f64),
    /// A share of the baseline value, `0.05` for 5 %.
    Relative(f64),
    /// A count that grows with the content of a page (`*_nodes`). Held like a count, with no
    /// allowance: a rise is a worse value and needs a decision id (R-02's rule that took a rise as
    /// new content let a page's serious axe nodes rise unseen, T17 of the R-02 report review).
    Content,
}

impl Tolerance {
    fn allowance(self, base: f64) -> f64 {
        match self {
            Tolerance::Absolute(amount) => amount,
            Tolerance::Relative(share) => base.abs() * share,
            Tolerance::Content => 0.0,
        }
    }
}

/// One baselined value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Entry {
    /// The value, lower is better.
    #[serde(serialize_with = "plain_number")]
    pub value: f64,
    /// The name of a class in [`Baseline::tolerances`].
    pub tolerance: String,
    /// The decision that accepted a worse value, when one did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
}

/// `website/ux/baseline.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Baseline {
    /// What the file is, for a reader.
    pub about: String,
    /// The scenarios each lane other than `full` measures.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub lanes: BTreeMap<String, BTreeSet<String>>,
    /// Scenario, then metric key, then the baselined value.
    pub scenarios: BTreeMap<String, BTreeMap<String, Entry>>,
    /// What a decision dropped: `<scenario>`, `<scenario> <metric>` or `<lane>: <scenario>`, to
    /// the decision id.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub dropped: BTreeMap<String, String>,
    /// Tolerance classes by name.
    pub tolerances: BTreeMap<String, Tolerance>,
}

/// What the round measured of what it expected.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Coverage {
    /// Every page configuration or task run the round expected and did not measure.
    pub unmeasured: Vec<String>,
}

/// The part of `round.json` the ratchet reads; every other field is the report's.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Round {
    /// The lane the round ran, `full` when it does not say.
    #[serde(default = "full_lane")]
    pub lane: String,
    /// Scenario, then metric key, then the measured value.
    pub scenarios: BTreeMap<String, BTreeMap<String, f64>>,
    /// Required: a round that does not say what it left unmeasured is not read.
    pub coverage: Coverage,
    /// Required: a round that does not say which engine measured it is not read.
    pub engine: Engine,
}

/// The engine that measured a round, as `round.json` records it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Engine {
    /// The last commit that changed an engine file.
    #[serde(default)]
    pub commit: String,
    /// The engine's files differ from that commit.
    pub dirty: bool,
    /// Every run merged into the round used the same engine; only a merged round says.
    #[serde(default)]
    pub runs_agree: Option<bool>,
}

fn full_lane() -> String {
    FULL.to_owned()
}

/// One reason the ratchet fails.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Breach {
    /// The round expected this configuration or task run and did not measure it.
    Unmeasured { unit: String },
    /// The engine that measured the round had uncommitted changes.
    DirtyEngine { commit: String },
    /// The round merges runs of different engines.
    RunsDisagree,
    /// The round is worse than the baseline beyond the tolerance.
    Worse {
        scenario: String,
        metric: String,
        baseline: f64,
        value: f64,
    },
    /// The round is better than the baseline and the baseline was not moved.
    ImprovementNotTaken {
        scenario: String,
        metric: String,
        baseline: f64,
        value: f64,
    },
    /// The baseline holds a value the round does not; `*` for a lane scenario with no entry.
    Missing { scenario: String, metric: String },
    /// A value the baseline does not hold.
    New {
        scenario: String,
        metric: String,
        value: f64,
    },
    /// The baseline holds the scenario, and the round's lane measured it for the first time.
    NewInLane { lane: String, scenario: String },
    /// An entry names a tolerance class the baseline does not define.
    UnknownTolerance {
        scenario: String,
        metric: String,
        tolerance: String,
    },
    /// The baseline is worse than `main`'s and the entry carries no new decision id.
    BaselineWorsened {
        scenario: String,
        metric: String,
        main: f64,
        value: f64,
    },
    /// An entry on `main` is gone from the baseline without a new decision id.
    BaselineDropped { scenario: String, metric: String },
    /// A scenario of a lane on `main` is gone from that lane without a new decision id.
    LaneDropped { lane: String, scenario: String },
}

impl fmt::Display for Breach {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Breach::Unmeasured { unit } => write!(
                f,
                "{unit}: unmeasured: the round expected it and measured nothing; only a run that \
                 measures it gets past"
            ),
            Breach::DirtyEngine { commit } => write!(
                f,
                "the round was measured by an engine with uncommitted changes (last engine commit \
                 {commit}); commit the engine and measure again"
            ),
            Breach::RunsDisagree => write!(
                f,
                "the round merges runs of different engines; measure every run with one engine"
            ),
            Breach::Worse {
                scenario,
                metric,
                baseline,
                value,
            } => write!(
                f,
                "{scenario} {metric}: {value} is worse than the baseline {baseline}"
            ),
            Breach::ImprovementNotTaken {
                scenario,
                metric,
                baseline,
                value,
            } => write!(
                f,
                "{scenario} {metric}: {value} improves on the baseline {baseline}; take it into \
                 {BASELINE} in this change (--update)"
            ),
            Breach::Missing { scenario, metric } => write!(
                f,
                "{scenario} {metric}: lost coverage: the baseline holds it and the round measured \
                 nothing"
            ),
            Breach::New {
                scenario,
                metric,
                value,
            } => write!(
                f,
                "{scenario} {metric}: new, {value}: the baseline holds no value; take it into \
                 {BASELINE} (--update)"
            ),
            Breach::NewInLane { lane, scenario } => write!(
                f,
                "{scenario}: new in lane {lane}; take it into the lane (--update)"
            ),
            Breach::UnknownTolerance {
                scenario,
                metric,
                tolerance,
            } => write!(
                f,
                "{scenario} {metric}: tolerance class `{tolerance}` is not defined in `tolerances`"
            ),
            Breach::BaselineWorsened {
                scenario,
                metric,
                main,
                value,
            } => write!(
                f,
                "{scenario} {metric}: the baseline moved from {main} on main to {value} without a new \
                 decision id (--update --decision <id>)"
            ),
            Breach::BaselineDropped { scenario, metric } => write!(
                f,
                "{scenario} {metric}: main's baseline holds it and this baseline dropped it without \
                 a new decision id"
            ),
            Breach::LaneDropped { lane, scenario } => write!(
                f,
                "{scenario}: main's lane {lane} holds it and this baseline's does not, without a \
                 new decision id"
            ),
        }
    }
}

/// The tolerance class a new metric gets. The skill's classes: counts 0, steps 5 %, layout shift
/// 0.01. Two more, decided here: `words` (5 %) for `reading_load`, a word count of the visible
/// text that moves a little with any edit of a page, and `content` for a node count that grows with
/// the page's content (`*_nodes`), held like a count. Every committed value carries the class this
/// function gives its metric (a test holds the committed baseline to it).
pub(crate) fn tolerance_class(metric: &str) -> &'static str {
    if metric.starts_with("layout_shift") {
        "layout_shift"
    } else if metric.starts_with("reading_load") {
        "words"
    } else if metric.ends_with("_nodes") {
        "content"
    } else if metric.starts_with("glance_cost") || metric.contains("time_to_act_steps") {
        "steps"
    } else {
        "count"
    }
}

/// The tolerance classes a new baseline starts with.
pub(crate) fn default_tolerances() -> BTreeMap<String, Tolerance> {
    BTreeMap::from([
        ("content".to_owned(), Tolerance::Content),
        ("count".to_owned(), Tolerance::Absolute(0.0)),
        ("layout_shift".to_owned(), Tolerance::Absolute(0.01)),
        ("steps".to_owned(), Tolerance::Relative(0.05)),
        ("words".to_owned(), Tolerance::Relative(0.05)),
    ])
}

fn tolerance_of(
    baseline: &Baseline,
    scenario: &str,
    metric: &str,
    entry: &Entry,
) -> Result<Tolerance, Breach> {
    baseline
        .tolerances
        .get(&entry.tolerance)
        .copied()
        .ok_or_else(|| Breach::UnknownTolerance {
            scenario: scenario.to_owned(),
            metric: metric.to_owned(),
            tolerance: entry.tolerance.clone(),
        })
}

/// The scenarios a round of `lane` is held to: every one for `full`, the lane's list otherwise.
fn scope<'a>(baseline: &'a Baseline, lane: &str) -> Option<&'a BTreeSet<String>> {
    static EMPTY: BTreeSet<String> = BTreeSet::new();
    (lane != FULL).then(|| baseline.lanes.get(lane).unwrap_or(&EMPTY))
}

fn judge(
    scenario: &str,
    metric: &str,
    entry: &Entry,
    tolerance: Tolerance,
    value: f64,
) -> Option<Breach> {
    let (scenario, metric, base) = (scenario.to_owned(), metric.to_owned(), entry.value);
    if value < base - EPSILON {
        return Some(Breach::ImprovementNotTaken {
            scenario,
            metric,
            baseline: base,
            value,
        });
    }
    (value > base + tolerance.allowance(base) + EPSILON).then_some(Breach::Worse {
        scenario,
        metric,
        baseline: base,
        value,
    })
}

/// Compares a round with the baseline.
pub(crate) fn compare(baseline: &Baseline, round: &Round) -> Vec<Breach> {
    let mut breaches: Vec<Breach> = round
        .coverage
        .unmeasured
        .iter()
        .map(|unit| Breach::Unmeasured { unit: unit.clone() })
        .collect();
    if round.engine.dirty {
        breaches.push(Breach::DirtyEngine {
            commit: round.engine.commit.clone(),
        });
    }
    if round.engine.runs_agree == Some(false) {
        breaches.push(Breach::RunsDisagree);
    }
    let lane = scope(baseline, &round.lane);
    let in_scope = |scenario: &str| lane.is_none_or(|set| set.contains(scenario));
    for (scenario, entries) in &baseline.scenarios {
        // A held scenario the lane measured for the first time is judged too, so a worse value
        // cannot come into the lane with it.
        if !in_scope(scenario) && !round.scenarios.contains_key(scenario) {
            continue;
        }
        let measured = round.scenarios.get(scenario);
        for (metric, entry) in entries {
            let tolerance = match tolerance_of(baseline, scenario, metric, entry) {
                Ok(tolerance) => tolerance,
                Err(breach) => {
                    breaches.push(breach);
                    continue;
                }
            };
            match measured.and_then(|values| values.get(metric)) {
                None => breaches.push(Breach::Missing {
                    scenario: scenario.clone(),
                    metric: metric.clone(),
                }),
                Some(&value) => breaches.extend(judge(scenario, metric, entry, tolerance, value)),
            }
        }
    }
    for scenario in lane.into_iter().flatten() {
        if !baseline.scenarios.contains_key(scenario) && !round.scenarios.contains_key(scenario) {
            breaches.push(Breach::Missing {
                scenario: scenario.clone(),
                metric: "*".to_owned(),
            });
        }
    }
    for (scenario, values) in &round.scenarios {
        let held = baseline.scenarios.get(scenario);
        if held.is_some() && !in_scope(scenario) {
            breaches.push(Breach::NewInLane {
                lane: round.lane.clone(),
                scenario: scenario.clone(),
            });
        }
        for (metric, &value) in values {
            if !held.is_some_and(|entries| entries.contains_key(metric)) {
                breaches.push(Breach::New {
                    scenario: scenario.clone(),
                    metric: metric.clone(),
                    value,
                });
            }
        }
    }
    breaches
}

fn decided_anew(id: Option<&String>, on_main: Option<&String>) -> bool {
    id.is_some_and(|id| !id.trim().is_empty() && on_main != Some(id))
}

/// Compares this baseline with `main`'s: a worse entry, a dropped entry and a scenario a lane
/// dropped each need a decision id `main` does not already carry there, in every tolerance class.
pub(crate) fn compare_with_main(main: &Baseline, current: &Baseline) -> Vec<Breach> {
    let mut breaches = Vec::new();
    for (scenario, entries) in &main.scenarios {
        for (metric, old) in entries {
            let Some(new) = current
                .scenarios
                .get(scenario)
                .and_then(|entries| entries.get(metric))
            else {
                let keys = [scenario.clone(), format!("{scenario} {metric}")];
                if !keys
                    .iter()
                    .any(|key| decided_anew(current.dropped.get(key), main.dropped.get(key)))
                {
                    breaches.push(Breach::BaselineDropped {
                        scenario: scenario.clone(),
                        metric: metric.clone(),
                    });
                }
                continue;
            };
            let tolerance = match tolerance_of(main, scenario, metric, old) {
                Ok(tolerance) => tolerance,
                Err(breach) => {
                    breaches.push(breach);
                    continue;
                }
            };
            let worse = new.value > old.value + tolerance.allowance(old.value) + EPSILON;
            if worse && !decided_anew(new.decision.as_ref(), old.decision.as_ref()) {
                breaches.push(Breach::BaselineWorsened {
                    scenario: scenario.clone(),
                    metric: metric.clone(),
                    main: old.value,
                    value: new.value,
                });
            }
        }
    }
    for (lane, scenarios) in &main.lanes {
        for scenario in scenarios {
            let kept = current
                .lanes
                .get(lane)
                .is_some_and(|set| set.contains(scenario));
            let key = format!("{lane}: {scenario}");
            if !kept && !decided_anew(current.dropped.get(&key), main.dropped.get(&key)) {
                breaches.push(Breach::LaneDropped {
                    lane: lane.clone(),
                    scenario: scenario.clone(),
                });
            }
        }
    }
    breaches
}

/// What an update took into the baseline, by kind.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Taken {
    pub improved: usize,
    pub new: usize,
    pub lane: usize,
    pub worse: usize,
    pub dropped: usize,
}

/// Takes a round into the baseline: improvements, new values and scenarios new to the lane always;
/// worse values (a content count that rose among them) and lost values only under a decision id,
/// which is written beside each worse value and under `dropped` for each lost one. An unmeasured
/// unit, a dirty engine and runs of different engines are never taken.
pub(crate) fn update(
    baseline: &Baseline,
    round: &Round,
    decision: Option<&str>,
) -> Result<(Baseline, Taken), Vec<Breach>> {
    let decision = decision.filter(|id| !id.trim().is_empty());
    let mut next = baseline.clone();
    let mut taken = Taken::default();
    let mut refused = Vec::new();
    let lane = (round.lane != FULL).then(|| round.lane.clone());
    for breach in compare(baseline, round) {
        match breach {
            Breach::ImprovementNotTaken {
                scenario,
                metric,
                value,
                ..
            } => {
                set_value(&mut next, &scenario, &metric, value);
                taken.improved += 1;
            }
            Breach::New {
                scenario,
                metric,
                value,
            } => {
                add_new(&mut next, lane.as_deref(), scenario, &metric, value);
                taken.new += 1;
            }
            Breach::NewInLane { lane, scenario } => {
                next.lanes.entry(lane).or_default().insert(scenario);
                taken.lane += 1;
            }
            Breach::Worse {
                scenario,
                metric,
                value,
                baseline: old,
            } => match decision {
                Some(id) => {
                    set_value(&mut next, &scenario, &metric, value);
                    if let Some(entry) = next
                        .scenarios
                        .get_mut(&scenario)
                        .and_then(|entries| entries.get_mut(&metric))
                    {
                        entry.decision = Some(id.to_owned());
                    }
                    taken.worse += 1;
                }
                None => refused.push(Breach::Worse {
                    scenario,
                    metric,
                    baseline: old,
                    value,
                }),
            },
            Breach::Missing { scenario, metric } => match decision {
                Some(id) => {
                    drop_lost(&mut next, round, lane.as_deref(), &scenario, &metric, id);
                    taken.dropped += 1;
                }
                None => refused.push(Breach::Missing { scenario, metric }),
            },
            other => refused.push(other),
        }
    }
    if refused.is_empty() {
        Ok((next, taken))
    } else {
        Err(refused)
    }
}

fn set_value(next: &mut Baseline, scenario: &str, metric: &str, value: f64) {
    if let Some(entry) = next
        .scenarios
        .get_mut(scenario)
        .and_then(|entries| entries.get_mut(metric))
    {
        entry.value = value;
    }
}

/// Adds a value the baseline does not hold, with its tolerance class (defined from the defaults
/// when the baseline lacks it), and to the round's lane.
fn add_new(next: &mut Baseline, lane: Option<&str>, scenario: String, metric: &str, value: f64) {
    if let Some(lane) = lane {
        next.lanes
            .entry(lane.to_owned())
            .or_default()
            .insert(scenario.clone());
    }
    let class = tolerance_class(metric);
    if let Some(tolerance) = default_tolerances().remove(class) {
        next.tolerances.entry(class.to_owned()).or_insert(tolerance);
    }
    next.scenarios.entry(scenario).or_default().insert(
        metric.to_owned(),
        Entry {
            value,
            tolerance: class.to_owned(),
            decision: None,
        },
    );
}

/// Drops what a round no longer measures, under a decision: a whole scenario when the round has
/// none of it, else one metric; for a lane round, the scenario from the lane.
fn drop_lost(
    next: &mut Baseline,
    round: &Round,
    lane: Option<&str>,
    scenario: &str,
    metric: &str,
    id: &str,
) {
    if let Some(lane) = lane {
        if let Some(set) = next.lanes.get_mut(lane) {
            set.remove(scenario);
        }
        next.dropped
            .insert(format!("{lane}: {scenario}"), id.to_owned());
        return;
    }
    if round.scenarios.contains_key(scenario) {
        if let Some(entries) = next.scenarios.get_mut(scenario) {
            entries.remove(metric);
        }
        next.dropped
            .insert(format!("{scenario} {metric}"), id.to_owned());
    } else {
        next.scenarios.remove(scenario);
        for set in next.lanes.values_mut() {
            set.remove(scenario);
        }
        next.dropped.insert(scenario.to_owned(), id.to_owned());
    }
}

/// Writes integral values without a fraction, so a count reads `3`, not `3.0`.
#[allow(clippy::trivially_copy_pass_by_ref)]
fn plain_number<S: serde::Serializer>(value: &f64, serializer: S) -> Result<S::Ok, S::Error> {
    if value.fract() == 0.0 && value.abs() < 9.0e15 {
        #[allow(clippy::cast_possible_truncation)]
        serializer.serialize_i64(*value as i64)
    } else {
        serializer.serialize_f64(*value)
    }
}

fn json(text: &str) -> Result<String> {
    Ok(serde_json::to_string(text)?)
}

/// The baseline as committed: one line per value, so a change reads as one line per value moved.
pub(crate) fn to_text(baseline: &Baseline) -> Result<String> {
    let mut out = String::from("{\n");
    let _ = writeln!(out, "  \"about\": {},", json(&baseline.about)?);
    if !baseline.lanes.is_empty() {
        out.push_str("  \"lanes\": {\n");
        for (i, (lane, scenarios)) in baseline.lanes.iter().enumerate() {
            let _ = writeln!(out, "    {}: [", json(lane)?);
            for (j, scenario) in scenarios.iter().enumerate() {
                let comma = if j + 1 < scenarios.len() { "," } else { "" };
                let _ = writeln!(out, "      {}{comma}", json(scenario)?);
            }
            let comma = if i + 1 < baseline.lanes.len() {
                ","
            } else {
                ""
            };
            let _ = writeln!(out, "    ]{comma}");
        }
        out.push_str("  },\n");
    }
    out.push_str("  \"scenarios\": {\n");
    for (i, (scenario, entries)) in baseline.scenarios.iter().enumerate() {
        let _ = writeln!(out, "    {}: {{", json(scenario)?);
        for (j, (metric, entry)) in entries.iter().enumerate() {
            let comma = if j + 1 < entries.len() { "," } else { "" };
            let entry = serde_json::to_string(entry)?
                .replace("\":", "\": ")
                .replace(",\"", ", \"");
            let _ = writeln!(out, "      {}: {entry}{comma}", json(metric)?);
        }
        let comma = if i + 1 < baseline.scenarios.len() {
            ","
        } else {
            ""
        };
        let _ = writeln!(out, "    }}{comma}");
    }
    out.push_str("  },\n");
    if !baseline.dropped.is_empty() {
        out.push_str("  \"dropped\": {\n");
        for (i, (what, id)) in baseline.dropped.iter().enumerate() {
            let comma = if i + 1 < baseline.dropped.len() {
                ","
            } else {
                ""
            };
            let _ = writeln!(out, "    {}: {}{comma}", json(what)?, json(id)?);
        }
        out.push_str("  },\n");
    }
    out.push_str("  \"tolerances\": {\n");
    for (i, (name, tolerance)) in baseline.tolerances.iter().enumerate() {
        let comma = if i + 1 < baseline.tolerances.len() {
            ","
        } else {
            ""
        };
        let tolerance = serde_json::to_string(tolerance)?.replace("\":", "\": ");
        let _ = writeln!(out, "    {}: {tolerance}{comma}", json(name)?);
    }
    out.push_str("  }\n}\n");
    Ok(out)
}

/// One finding record's header, from the front matter between the first two `---` lines.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Finding {
    /// The file name, `UXF-nnn.md`.
    pub file: String,
    /// `key: value` lines of the front matter.
    pub fields: BTreeMap<String, String>,
}

/// The header fields every record carries (the skill's § 5).
pub(crate) const FINDING_FIELDS: &[&str] = &[
    "id",
    "title",
    "class",
    "severity",
    "metric",
    "status",
    "found_by",
    "evidence_recording",
    "evidence_screenshot",
    "evidence_lines",
    "regression_asset",
];

const CLASSES: &[&str] = &[
    "layout",
    "cue",
    "affordance",
    "copy",
    "noise",
    "engine",
    "input",
];
const SEVERITIES: &[&str] = &["S1", "S2", "S3"];
const STATUSES: &[&str] = &["open", "accepted", "closed", "known-loss"];

/// Reads a record's front matter.
pub(crate) fn parse_finding(file: &str, text: &str) -> Result<Finding, String> {
    let mut lines = text.lines();
    if lines.next().map(str::trim) != Some("---") {
        return Err(format!(
            "{file}: the record does not open with a `---` front matter line"
        ));
    }
    let mut fields = BTreeMap::new();
    let mut closed = false;
    for line in lines {
        if line.trim() == "---" {
            closed = true;
            break;
        }
        if line.trim().is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            return Err(format!(
                "{file}: front matter line `{line}` is not `key: value`"
            ));
        };
        fields.insert(
            key.trim().to_owned(),
            value.trim().trim_matches('"').to_owned(),
        );
    }
    if !closed {
        return Err(format!("{file}: the front matter is not closed with `---`"));
    }
    Ok(Finding {
        file: file.to_owned(),
        fields,
    })
}

/// The path part of a regression asset, `website/ux/specs/x.spec.ts :: title` to its file.
fn asset_path(asset: &str) -> Option<&str> {
    let first = asset.split_whitespace().next()?;
    let path = first.split("::").next()?.trim_end_matches(',');
    (path.contains('/') && !path.contains("://")).then_some(path)
}

/// The test title an asset names after `::`, up to `, held by` or the end.
pub(crate) fn asset_title(asset: &str) -> Option<&str> {
    let (_, rest) = asset.split_once("::")?;
    let title = rest.split(", held by").next().unwrap_or(rest).trim();
    (!title.is_empty()).then_some(title)
}

/// The titles of the tests a spec file declares: the first argument of every `test(` call, a
/// quoted or a template string, `${…}` kept as written.
pub(crate) fn declared_titles(text: &str) -> Vec<String> {
    let mut titles = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("test(") {
        let before = rest[..at].chars().next_back();
        rest = &rest[at + "test(".len()..];
        if before.is_some_and(|c| c.is_alphanumeric() || c == '_' || c == '.') {
            continue;
        }
        let trimmed = rest.trim_start();
        let Some(quote) = trimmed
            .chars()
            .next()
            .filter(|c| matches!(c, '\'' | '"' | '`'))
        else {
            continue;
        };
        let body = &trimmed[1..];
        let mut end = None;
        let mut escaped = false;
        for (i, c) in body.char_indices() {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == quote {
                end = Some(i);
                break;
            }
        }
        if let Some(end) = end {
            titles.push(body[..end].replace("\\'", "'"));
        }
    }
    titles
}

/// Whether `title` is an instance of `template`, each `${…}` standing for any text.
pub(crate) fn title_matches(template: &str, title: &str) -> bool {
    let mut parts = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find("${") {
        parts.push(&rest[..open]);
        rest = rest[open..]
            .find('}')
            .map_or("", |close| &rest[open + close + 1..]);
    }
    parts.push(rest);
    if parts.len() == 1 {
        return template == title;
    }
    let (first, last) = (parts[0], parts[parts.len() - 1]);
    if !title.starts_with(first) || !title.ends_with(last) || title.len() < first.len() + last.len()
    {
        return false;
    }
    let mut middle = &title[first.len()..title.len() - last.len()];
    for part in &parts[1..parts.len() - 1] {
        match middle.find(part) {
            Some(at) => middle = &middle[at + part.len()..],
            None => return false,
        }
    }
    true
}

/// Checks one record. `read` returns the text of a repository-relative file, or `None` when it
/// does not exist.
pub(crate) fn check_finding(
    finding: &Finding,
    read: &dyn Fn(&str) -> Option<String>,
) -> Vec<String> {
    let file = &finding.file;
    let field = |key: &str| finding.fields.get(key).map_or("", String::as_str);
    let mut problems = Vec::new();
    for key in FINDING_FIELDS {
        if field(key).is_empty() {
            problems.push(format!("{file}: header field `{key}` is missing or empty"));
        }
    }
    let stem = file.strip_suffix(".md").unwrap_or(file);
    if !field("id").is_empty() && field("id") != stem {
        problems.push(format!(
            "{file}: id `{}` does not match the file name",
            field("id")
        ));
    }
    if !field("class").is_empty() && !CLASSES.contains(&field("class")) {
        problems.push(format!(
            "{file}: class `{}` is not one of {CLASSES:?}",
            field("class")
        ));
    }
    if !field("severity").is_empty() && !SEVERITIES.contains(&field("severity")) {
        problems.push(format!(
            "{file}: severity `{}` is not one of {SEVERITIES:?}",
            field("severity")
        ));
    }
    if !field("status").is_empty() && !STATUSES.contains(&field("status")) {
        problems.push(format!(
            "{file}: status `{}` is not one of {STATUSES:?}",
            field("status")
        ));
    }
    let asset = field("regression_asset");
    let no_asset = asset.is_empty()
        || asset.eq_ignore_ascii_case("none")
        || asset.starts_with("none")
        || asset == "-";
    if field("status") == "closed" && matches!(field("severity"), "S1" | "S2") && no_asset {
        problems.push(format!(
            "{file}: a closed {} finding needs a regression asset that failed before the fix",
            field("severity")
        ));
    }
    if let Some(path) = asset_path(asset) {
        match read(path) {
            None => problems.push(format!("{file}: regression asset `{path}` does not exist")),
            Some(text) => {
                if let Some(title) = asset_title(asset) {
                    if !declared_titles(&text)
                        .iter()
                        .any(|template| title_matches(template, title))
                    {
                        problems.push(format!(
                            "{file}: regression asset `{path}` declares no test `{title}`"
                        ));
                    }
                }
            }
        }
    }
    problems
}

/// Checks every `UXF-*.md` under [`FINDINGS`].
pub(crate) fn check_findings(root: &Path) -> Result<Vec<String>> {
    let directory = root.join(FINDINGS);
    let mut files: Vec<_> = fs::read_dir(&directory)
        .with_context(|| format!("reading {}", directory.display()))?
        .filter_map(|entry| {
            entry
                .ok()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
        })
        .filter(|name| {
            name.starts_with("UXF-")
                && Path::new(name)
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        })
        .collect();
    files.sort();
    let read = |path: &str| fs::read_to_string(root.join(path)).ok();
    let mut problems = Vec::new();
    for file in &files {
        let text =
            fs::read_to_string(directory.join(file)).with_context(|| format!("reading {file}"))?;
        match parse_finding(file, &text) {
            Ok(finding) => problems.extend(check_finding(&finding, &read)),
            Err(problem) => problems.push(problem),
        }
    }
    Ok(problems)
}

/// `main`'s baseline from what git answered: an error when `origin/main` itself cannot be read
/// (a shallow or unfetched checkout must not skip the comparison silently), `None` only when
/// `main` holds no baseline yet.
pub(crate) fn read_main_baseline(
    main_readable: bool,
    file: Option<&[u8]>,
) -> Result<Option<Baseline>> {
    if !main_readable {
        bail!(
            "origin/main cannot be read, so the baseline cannot be compared with main's: fetch \
             origin main first"
        );
    }
    file.map(|bytes| serde_json::from_slice(bytes).context("parsing main's baseline"))
        .transpose()
}

fn git(root: &Path, args: &[&str]) -> Result<std::process::Output> {
    Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .with_context(|| format!("running git {}", args.join(" ")))
}

/// `main`'s baseline through git.
fn main_baseline(root: &Path) -> Result<Option<Baseline>> {
    let readable = git(
        root,
        &["rev-parse", "--verify", "--quiet", "origin/main^{commit}"],
    )?
    .status
    .success();
    let spec = format!("origin/main:{BASELINE}");
    let file = if readable && git(root, &["cat-file", "-e", &spec])?.status.success() {
        let shown = git(root, &["show", &spec])?;
        if !shown.status.success() {
            bail!("git show {spec} failed");
        }
        Some(shown.stdout)
    } else {
        None
    };
    read_main_baseline(readable, file.as_deref())
}

/// The command: compare `round` with the baseline and with `main`'s, check the findings; with
/// `update`, take the round into the baseline first, in memory, and write the file only once every
/// comparison and the findings check passed against the updated baseline.
pub(crate) fn run(
    root: &Path,
    round: &Path,
    update_baseline: bool,
    decision: Option<&str>,
) -> Result<String> {
    let baseline_path = root.join(BASELINE);
    let mut baseline: Baseline = serde_json::from_str(
        &fs::read_to_string(&baseline_path)
            .with_context(|| format!("reading {}", baseline_path.display()))?,
    )
    .with_context(|| format!("parsing {}", baseline_path.display()))?;
    let round_path = root.join(round);
    let measured: Round = serde_json::from_str(
        &fs::read_to_string(&round_path)
            .with_context(|| format!("reading {}", round_path.display()))?,
    )
    .with_context(|| format!("parsing {}", round_path.display()))?;
    let mut report = String::new();
    if update_baseline {
        let (next, taken) = update(&baseline, &measured, decision).map_err(|refused| {
            anyhow::anyhow!("the baseline was not updated:\n{}", lines(&refused))
        })?;
        baseline = next;
        let _ = writeln!(
            report,
            "ux ratchet: took {} improvement(s), {} new value(s), {} lane scenario(s); {} worse \
             and {} dropped under decision {}",
            taken.improved,
            taken.new,
            taken.lane,
            taken.worse,
            taken.dropped,
            decision.unwrap_or("none")
        );
    }
    let mut breaches = compare(&baseline, &measured);
    if let Some(main) = main_baseline(root)? {
        breaches.extend(compare_with_main(&main, &baseline));
    }
    let findings = check_findings(root)?;
    if !breaches.is_empty() || !findings.is_empty() {
        let mut message = String::new();
        if !breaches.is_empty() {
            let _ = writeln!(
                message,
                "ratchet: {} breach(es)\n{}",
                breaches.len(),
                lines(&breaches)
            );
        }
        if !findings.is_empty() {
            let _ = writeln!(
                message,
                "findings: {} problem(s)\n{}",
                findings.len(),
                findings.join("\n")
            );
        }
        if update_baseline {
            message.push_str("the baseline file was not written\n");
        }
        bail!(message.trim_end().to_owned());
    }
    if update_baseline {
        fs::write(&baseline_path, to_text(&baseline)?)
            .with_context(|| format!("writing {}", baseline_path.display()))?;
    }
    let keys: usize = baseline.scenarios.values().map(BTreeMap::len).sum();
    let _ = writeln!(
        report,
        "ux ratchet: lane {}: {keys} value(s) in {} scenario(s) hold the baseline",
        measured.lane,
        baseline.scenarios.len()
    );
    Ok(report)
}

fn lines(breaches: &[Breach]) -> String {
    breaches
        .iter()
        .map(|breach| format!("  {breach}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn baseline(json: &str) -> Baseline {
        serde_json::from_str(json).expect("canned baseline parses")
    }

    /// A canned round. One without a `coverage` key is a complete round, and one without an
    /// `engine` key was measured by a committed engine: each key is added, so each case states
    /// only what it is about.
    fn round(json: &str) -> Round {
        let mut json = json.to_owned();
        if !json.contains("\"coverage\"") {
            json = json.replacen('{', r#"{"coverage": {"unmeasured": []}, "#, 1);
        }
        if !json.contains("\"engine\"") {
            json = json.replacen(
                '{',
                r#"{"engine": {"digest": "d", "commit": "c", "dirty": false}, "#,
                1,
            );
        }
        serde_json::from_str(&json).expect("canned round parses")
    }

    const BASE: &str = r#"{
      "about": "canned",
      "tolerances": {"count": {"absolute": 0}, "steps": {"relative": 0.05}, "layout_shift": {"absolute": 0.01}},
      "scenarios": {
        "install/desktop-1280": {
          "screen_only.actions": {"value": 3, "tolerance": "count"},
          "glance_cost.p95": {"value": 2.0, "tolerance": "steps"},
          "layout_shift.max": {"value": 0.02, "tolerance": "layout_shift"}
        }
      }
    }"#;

    #[test]
    fn a_round_equal_to_the_baseline_passes() {
        let measured = round(
            r#"{"round": "R-02", "scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        );
        assert_eq!(compare(&baseline(BASE), &measured), vec![]);
    }

    #[test]
    fn a_value_worse_beyond_its_tolerance_fails_and_within_it_passes() {
        let within = round(
            r#"{"scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 2.09, "layout_shift.max": 0.029}}}"#,
        );
        assert_eq!(compare(&baseline(BASE), &within), vec![]);
        let worse = round(
            r#"{"scenarios": {"install/desktop-1280":
              {"screen_only.actions": 4, "glance_cost.p95": 2.2, "layout_shift.max": 0.031}}}"#,
        );
        let breaches = compare(&baseline(BASE), &worse);
        assert_eq!(breaches.len(), 3, "{breaches:?}");
        assert!(
            breaches
                .iter()
                .all(|breach| matches!(breach, Breach::Worse { .. })),
            "{breaches:?}"
        );
    }

    #[test]
    fn an_improvement_not_taken_into_the_baseline_fails() {
        let better = round(
            r#"{"scenarios": {"install/desktop-1280":
              {"screen_only.actions": 2, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        );
        assert_eq!(
            compare(&baseline(BASE), &better),
            vec![Breach::ImprovementNotTaken {
                scenario: "install/desktop-1280".into(),
                metric: "screen_only.actions".into(),
                baseline: 3.0,
                value: 2.0,
            }]
        );
    }

    #[test]
    fn an_improvement_within_its_tolerance_must_still_be_taken() {
        let better = round(
            r#"{"scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 1.95, "layout_shift.max": 0.02}}}"#,
        );
        let breaches = compare(&baseline(BASE), &better);
        assert!(
            matches!(breaches.as_slice(), [Breach::ImprovementNotTaken { metric, .. }] if metric == "glance_cost.p95"),
            "{breaches:?}"
        );
        let (next, taken) =
            update(&baseline(BASE), &better, None).expect("an improvement needs no decision");
        assert_eq!(taken.improved, 1);
        assert_eq!(
            next.scenarios["install/desktop-1280"]["glance_cost.p95"].value,
            1.95
        );
    }

    #[test]
    fn a_missing_value_and_a_new_value_fail() {
        let measured = round(
            r#"{"scenarios": {"install/desktop-1280": {"screen_only.actions": 3, "glance_cost.p95": 2.0},
              "limitations/mobile-390": {"screen_only.actions": 5}}}"#,
        );
        let breaches = compare(&baseline(BASE), &measured);
        assert!(breaches.contains(&Breach::Missing {
            scenario: "install/desktop-1280".into(),
            metric: "layout_shift.max".into()
        }));
        assert!(breaches.contains(&Breach::New {
            scenario: "limitations/mobile-390".into(),
            metric: "screen_only.actions".into(),
            value: 5.0,
        }));
        assert_eq!(breaches.len(), 2, "{breaches:?}");
    }

    #[test]
    fn a_round_without_coverage_or_engine_is_refused() {
        let parsed = serde_json::from_str::<Round>(
            r#"{"engine": {"dirty": false}, "scenarios": {"install/desktop-1280": {"screen_only.actions": 3}}}"#,
        );
        assert!(
            parsed.is_err(),
            "a round that says nothing of its coverage was read"
        );
        let parsed = serde_json::from_str::<Round>(
            r#"{"coverage": {"unmeasured": []}, "scenarios": {"install/desktop-1280": {"screen_only.actions": 3}}}"#,
        );
        assert!(
            parsed.is_err(),
            "a round that says nothing of its engine was read"
        );
    }

    #[test]
    fn an_unmeasured_configuration_is_a_breach_that_update_never_takes() {
        let measured = round(
            r#"{"coverage": {"unmeasured": ["/ess/docs/reference/ess-ui@desktop-1280/light"]},
              "scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        );
        let breaches = compare(&baseline(BASE), &measured);
        assert_eq!(breaches.len(), 1, "{breaches:?}");
        assert!(
            breaches[0].to_string().contains("unmeasured"),
            "{}",
            breaches[0]
        );
        assert!(update(&baseline(BASE), &measured, Some("D-1")).is_err());
    }

    #[test]
    fn a_new_page_is_reported_as_new_and_taken_by_update() {
        let measured = round(
            r#"{"scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 2.0, "layout_shift.max": 0.02},
              "/ess/docs/new@desktop-1280/light": {"axe.serious_critical_nodes": 4}}}"#,
        );
        let breaches = compare(&baseline(BASE), &measured);
        assert_eq!(breaches.len(), 1, "{breaches:?}");
        assert!(breaches[0].to_string().contains(": new"), "{}", breaches[0]);
        let (next, taken) =
            update(&baseline(BASE), &measured, None).expect("a new page needs no decision");
        assert_eq!(taken.new, 1);
        assert_eq!(
            next.scenarios["/ess/docs/new@desktop-1280/light"]["axe.serious_critical_nodes"]
                .tolerance,
            "content"
        );
        assert_eq!(compare(&next, &measured), vec![]);
    }

    const CONTENT: &str = r#"{
      "about": "canned",
      "tolerances": {"count": {"absolute": 0}, "content": "content"},
      "scenarios": {"/ess/docs/a@desktop-1280/light": {
        "axe.serious_critical_rules": {"value": 1, "tolerance": "count"},
        "axe.serious_critical_nodes": {"value": 10, "tolerance": "content"}
      }}
    }"#;

    /// T17 of the R-02 report review: a rise of a `content` value was taken by `--update` with no
    /// decision id. It is a worse value like any other.
    #[test]
    fn a_content_count_that_rises_is_worse_and_needs_a_decision() {
        let one_more_link = round(
            r#"{"scenarios": {"/ess/docs/a@desktop-1280/light":
              {"axe.serious_critical_rules": 1, "axe.serious_critical_nodes": 11}}}"#,
        );
        let breaches = compare(&baseline(CONTENT), &one_more_link);
        assert!(
            matches!(breaches.as_slice(), [Breach::Worse { metric, .. }] if metric == "axe.serious_critical_nodes"),
            "{breaches:?}"
        );
        assert!(
            update(&baseline(CONTENT), &one_more_link, None).is_err(),
            "a content rise was taken without a decision"
        );
        let (next, _) =
            update(&baseline(CONTENT), &one_more_link, Some("D-5")).expect("a decision takes it");
        assert_eq!(
            next.scenarios["/ess/docs/a@desktop-1280/light"]["axe.serious_critical_nodes"]
                .decision
                .as_deref(),
            Some("D-5")
        );
        assert_eq!(compare(&next, &one_more_link), vec![]);
        assert_eq!(compare_with_main(&baseline(CONTENT), &next), vec![]);
        let another_rule = round(
            r#"{"scenarios": {"/ess/docs/a@desktop-1280/light":
              {"axe.serious_critical_rules": 2, "axe.serious_critical_nodes": 10}}}"#,
        );
        assert!(compare(&baseline(CONTENT), &another_rule)
            .iter()
            .any(|breach| matches!(breach, Breach::Worse { .. })));
    }

    /// T17: the comparison with `main`'s baseline exempted the `content` class.
    #[test]
    fn a_content_value_worse_than_mains_needs_a_new_decision_id() {
        let main = baseline(CONTENT);
        let mut risen = main.clone();
        risen
            .scenarios
            .get_mut("/ess/docs/a@desktop-1280/light")
            .unwrap()
            .get_mut("axe.serious_critical_nodes")
            .unwrap()
            .value = 11.0;
        assert!(
            matches!(compare_with_main(&main, &risen).as_slice(), [Breach::BaselineWorsened { metric, .. }] if metric == "axe.serious_critical_nodes"),
            "{:?}",
            compare_with_main(&main, &risen)
        );
    }

    /// T22: a round measured by an engine with uncommitted changes is refused, by `compare` and
    /// by `update` under any decision.
    #[test]
    fn a_round_from_a_dirty_engine_is_refused() {
        let measured = round(
            r#"{"engine": {"digest": "d", "commit": "c", "dirty": true},
              "scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        );
        let breaches = compare(&baseline(BASE), &measured);
        assert!(
            breaches
                .iter()
                .any(|breach| breach.to_string().contains("uncommitted")),
            "{breaches:?}"
        );
        assert!(update(&baseline(BASE), &measured, Some("D-1")).is_err());
    }

    /// T22: a round merged from runs of different engines is refused.
    #[test]
    fn a_round_whose_runs_disagree_is_refused() {
        let measured = round(
            r#"{"engine": {"digest": "d", "commit": "c", "dirty": false, "runs_agree": false},
              "scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        );
        let breaches = compare(&baseline(BASE), &measured);
        assert!(
            breaches
                .iter()
                .any(|breach| breach.to_string().contains("different engines")),
            "{breaches:?}"
        );
        assert!(update(&baseline(BASE), &measured, Some("D-1")).is_err());
    }

    /// T21: `run --update` wrote the baseline before its last comparison; a run that then failed
    /// left the changed file behind. Here the findings check fails after an update that would
    /// take an improvement.
    #[test]
    fn a_run_that_fails_leaves_the_baseline_file_as_it_was() {
        let scratch = crate::scratch::Scratch::new("ess-ux-ratchet-run");
        let root = scratch.path();
        fs::create_dir_all(root.join(FINDINGS)).unwrap();
        fs::write(root.join(BASELINE), BASE).unwrap();
        fs::write(
            root.join(FINDINGS).join("UXF-001.md"),
            "---\nid: UXF-001\n---\n",
        )
        .unwrap();
        fs::write(
            root.join("round.json"),
            r#"{"coverage": {"unmeasured": []},
              "engine": {"digest": "d", "commit": "c", "dirty": false},
              "scenarios": {"install/desktop-1280":
              {"screen_only.actions": 2, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        )
        .unwrap();
        let result = run(root, Path::new("round.json"), true, None);
        assert!(result.is_err(), "{result:?}");
        assert_eq!(
            fs::read_to_string(root.join(BASELINE)).unwrap(),
            BASE,
            "a failed run changed the baseline file"
        );
    }

    /// T24: every value of the committed baseline carries the tolerance class the code gives its
    /// metric, so `reading_load` has one unit in code and baseline.
    #[test]
    fn every_committed_value_carries_the_class_its_metric_has() {
        let root = repository_root();
        let base: Baseline = serde_json::from_str(
            &fs::read_to_string(root.join(BASELINE)).expect("the baseline reads"),
        )
        .expect("the baseline parses");
        let mut wrong = BTreeSet::new();
        for entries in base.scenarios.values() {
            for (metric, entry) in entries {
                if entry.tolerance != tolerance_class(metric) {
                    wrong.insert(format!(
                        "{metric}: `{}` in the baseline, `{}` in the code",
                        entry.tolerance,
                        tolerance_class(metric)
                    ));
                }
            }
        }
        assert_eq!(wrong, BTreeSet::new());
    }

    const LANES: &str = r#"{
      "about": "canned",
      "tolerances": {"count": {"absolute": 0}},
      "lanes": {"gate": ["/a@d/light", "/b@d/light"]},
      "scenarios": {
        "/a@d/light": {"layout.spills": {"value": 0, "tolerance": "count"}},
        "/b@d/light": {"layout.spills": {"value": 0, "tolerance": "count"}},
        "/c@d/light": {"layout.spills": {"value": 0, "tolerance": "count"}}
      }
    }"#;

    #[test]
    fn a_gate_round_is_held_to_the_gate_lane_and_a_page_it_lost_is_a_breach() {
        let gate = round(
            r#"{"lane": "gate", "scenarios": {"/a@d/light": {"layout.spills": 0},
              "/b@d/light": {"layout.spills": 0}}}"#,
        );
        assert_eq!(compare(&baseline(LANES), &gate), vec![]);
        let shrunk =
            round(r#"{"lane": "gate", "scenarios": {"/a@d/light": {"layout.spills": 0}}}"#);
        assert!(compare(&baseline(LANES), &shrunk)
            .iter()
            .any(|breach| breach.to_string().contains("/b@d/light")));
        let full = round(
            r#"{"lane": "full", "scenarios": {"/a@d/light": {"layout.spills": 0},
              "/b@d/light": {"layout.spills": 0}}}"#,
        );
        assert!(compare(&baseline(LANES), &full)
            .iter()
            .any(|breach| breach.to_string().contains("/c@d/light")));
    }

    #[test]
    fn a_scenario_new_to_the_gate_lane_is_new_and_update_adds_it_to_the_lane() {
        let grown = round(
            r#"{"lane": "gate", "scenarios": {"/a@d/light": {"layout.spills": 0},
              "/b@d/light": {"layout.spills": 0}, "/c@d/light": {"layout.spills": 0}}}"#,
        );
        assert_eq!(
            compare(&baseline(LANES), &grown),
            vec![Breach::NewInLane {
                lane: "gate".into(),
                scenario: "/c@d/light".into()
            }]
        );
        let (next, taken) = update(&baseline(LANES), &grown, None).expect("no decision needed");
        assert_eq!(taken.lane, 1);
        assert!(next.lanes["gate"].contains("/c@d/light"));
        assert_eq!(compare(&next, &grown), vec![]);
    }

    /// The gap the R-02 report review confirmed: `--update` on a gate-lane run took a scenario
    /// new to the lane without judging its values, so a worse value came in with it in one pass.
    #[test]
    fn update_judges_the_values_of_a_scenario_new_to_the_lane_in_the_same_pass() {
        let grown_worse = round(
            r#"{"lane": "gate", "scenarios": {"/a@d/light": {"layout.spills": 0},
              "/b@d/light": {"layout.spills": 0}, "/c@d/light": {"layout.spills": 1}}}"#,
        );
        let breaches = compare(&baseline(LANES), &grown_worse);
        assert!(
            breaches.iter().any(|breach| matches!(breach, Breach::Worse { scenario, .. } if scenario == "/c@d/light")),
            "{breaches:?}"
        );
        assert!(
            update(&baseline(LANES), &grown_worse, None).is_err(),
            "a worse value came into the lane without a decision"
        );
    }

    #[test]
    fn a_lane_that_drops_a_scenario_needs_a_decision_against_main() {
        let main = baseline(LANES);
        let shrunk =
            round(r#"{"lane": "gate", "scenarios": {"/a@d/light": {"layout.spills": 0}}}"#);
        assert!(update(&main, &shrunk, None).is_err());
        let (next, _) = update(&main, &shrunk, Some("D-9")).expect("a decision drops it");
        assert_eq!(next.dropped["gate: /b@d/light"], "D-9");
        assert_eq!(compare_with_main(&main, &next), vec![]);
        let mut undecided = next.clone();
        undecided.dropped.clear();
        assert_eq!(
            compare_with_main(&main, &undecided),
            vec![Breach::LaneDropped {
                lane: "gate".into(),
                scenario: "/b@d/light".into()
            }]
        );
    }

    #[test]
    fn update_with_a_decision_replaces_a_baseline_whose_scenarios_are_gone() {
        let measured = round(r#"{"scenarios": {"/ess/@d/light": {"layout.spills": 0}}}"#);
        assert!(update(&baseline(BASE), &measured, None).is_err());
        let (next, taken) = update(&baseline(BASE), &measured, Some("R-02-baseline"))
            .expect("a decision replaces what the round no longer measures");
        assert!(!next.scenarios.contains_key("install/desktop-1280"));
        assert_eq!(next.dropped["install/desktop-1280"], "R-02-baseline");
        assert_eq!(taken.dropped, 3);
        assert_eq!(compare(&next, &measured), vec![]);
        assert_eq!(compare_with_main(&baseline(BASE), &next), vec![]);
    }

    #[test]
    fn an_unknown_tolerance_class_fails() {
        let base = baseline(
            r#"{"about": "", "tolerances": {}, "scenarios": {"s": {"k": {"value": 1, "tolerance": "nope"}}}}"#,
        );
        let breaches = compare(&base, &round(r#"{"scenarios": {"s": {"k": 1}}}"#));
        assert!(
            matches!(breaches.as_slice(), [Breach::UnknownTolerance { .. }]),
            "{breaches:?}"
        );
    }

    #[test]
    fn a_baseline_made_worse_than_main_needs_a_new_decision_id() {
        let main = baseline(BASE);
        let mut worse = main.clone();
        worse
            .scenarios
            .get_mut("install/desktop-1280")
            .unwrap()
            .get_mut("screen_only.actions")
            .unwrap()
            .value = 4.0;
        assert!(matches!(
            compare_with_main(&main, &worse).as_slice(),
            [Breach::BaselineWorsened { .. }]
        ));
        worse
            .scenarios
            .get_mut("install/desktop-1280")
            .unwrap()
            .get_mut("screen_only.actions")
            .unwrap()
            .decision = Some("R-02-decision".into());
        assert_eq!(compare_with_main(&main, &worse), vec![]);
    }

    #[test]
    fn reusing_mains_decision_id_does_not_excuse_a_further_worsening() {
        let mut main = baseline(BASE);
        main.scenarios
            .get_mut("install/desktop-1280")
            .unwrap()
            .get_mut("screen_only.actions")
            .unwrap()
            .decision = Some("R-01-decision".into());
        let mut worse = main.clone();
        worse
            .scenarios
            .get_mut("install/desktop-1280")
            .unwrap()
            .get_mut("screen_only.actions")
            .unwrap()
            .value = 5.0;
        assert!(matches!(
            compare_with_main(&main, &worse).as_slice(),
            [Breach::BaselineWorsened { .. }]
        ));
    }

    #[test]
    fn dropping_an_entry_main_holds_fails_without_a_decision() {
        let main = baseline(BASE);
        let mut dropped = main.clone();
        dropped
            .scenarios
            .get_mut("install/desktop-1280")
            .unwrap()
            .remove("layout_shift.max");
        assert_eq!(
            compare_with_main(&main, &dropped),
            vec![Breach::BaselineDropped {
                scenario: "install/desktop-1280".into(),
                metric: "layout_shift.max".into()
            }]
        );
        dropped
            .dropped
            .insert("install/desktop-1280 layout_shift.max".into(), "D-3".into());
        assert_eq!(compare_with_main(&main, &dropped), vec![]);
    }

    #[test]
    fn update_takes_improvements_and_new_keys_and_refuses_worse_without_a_decision() {
        let measured = round(
            r#"{"scenarios": {"install/desktop-1280":
              {"screen_only.actions": 2, "glance_cost.p95": 2.5, "layout_shift.max": 0.02, "reading_load.max": 120}}}"#,
        );
        let refused = update(&baseline(BASE), &measured, None).unwrap_err();
        assert!(
            matches!(refused.as_slice(), [Breach::Worse { metric, .. }] if metric == "glance_cost.p95"),
            "{refused:?}"
        );
        let (next, _) = update(&baseline(BASE), &measured, Some("D-7"))
            .expect("a decision takes the worse value");
        let entries = &next.scenarios["install/desktop-1280"];
        assert_eq!(entries["screen_only.actions"].value, 2.0);
        assert_eq!(entries["glance_cost.p95"].decision.as_deref(), Some("D-7"));
        assert_eq!(entries["reading_load.max"].tolerance, "words");
    }

    #[test]
    fn update_refuses_a_missing_value_without_a_decision() {
        let measured =
            round(r#"{"scenarios": {"install/desktop-1280": {"screen_only.actions": 3}}}"#);
        assert!(update(&baseline(BASE), &measured, None).is_err());
        let (next, _) = update(&baseline(BASE), &measured, Some("D-1")).expect("dropped under D-1");
        assert_eq!(next.dropped["install/desktop-1280 layout_shift.max"], "D-1");
    }

    #[test]
    fn mains_baseline_unreadable_is_an_error_and_absent_is_none() {
        assert!(read_main_baseline(false, None).is_err());
        assert!(read_main_baseline(true, None)
            .expect("main readable")
            .is_none());
        let found = read_main_baseline(true, Some(BASE.as_bytes())).expect("parses");
        assert_eq!(found, Some(baseline(BASE)));
    }

    #[test]
    fn counts_are_written_without_a_fraction() {
        let text = serde_json::to_string(&baseline(BASE)).unwrap();
        assert!(text.contains(r#""value":3,"#), "{text}");
        assert!(text.contains(r#""value":0.02,"#), "{text}");
    }

    #[test]
    fn the_baseline_text_holds_one_value_per_line_and_reads_back() {
        let mut base = baseline(LANES);
        base.tolerances = default_tolerances();
        base.dropped.insert("old/d".into(), "D-1".into());
        let text = to_text(&base).expect("written");
        assert!(
            text.contains(r#""layout.spills": {"value": 0, "tolerance": "count"}"#),
            "{text}"
        );
        assert!(text.contains(r#""content": "content""#), "{text}");
        let back: Baseline = serde_json::from_str(&text).expect("reads back");
        assert_eq!(back, base);
    }

    #[test]
    fn new_keys_get_the_skills_tolerance_classes_and_two_decided_here() {
        assert_eq!(tolerance_class("screen_only.actions"), "count");
        assert_eq!(
            tolerance_class("screen_only.time_to_act_steps.max"),
            "steps"
        );
        assert_eq!(tolerance_class("glance_cost.median"), "steps");
        assert_eq!(tolerance_class("layout_shift.load"), "layout_shift");
        assert_eq!(tolerance_class("reading_load.max"), "words");
        assert_eq!(tolerance_class("axe.serious_critical_nodes"), "content");
        assert_eq!(tolerance_class("axe.serious_critical_rules"), "count");
        for class in ["content", "count", "layout_shift", "steps", "words"] {
            assert!(default_tolerances().contains_key(class), "{class}");
        }
    }

    fn record(status: &str, severity: &str, asset: &str) -> String {
        format!(
            "---\nid: UXF-001\ntitle: t\nclass: layout\nseverity: {severity}\nmetric: m\nstatus: {status}\n\
             found_by: L2\nevidence_recording: none\nevidence_screenshot: s.png\nevidence_lines: l\n\
             regression_asset: {asset}\n---\n\n# UXF-001\n"
        )
    }

    #[allow(clippy::unnecessary_wraps)]
    fn any_file(_: &str) -> Option<String> {
        Some(String::new())
    }

    #[test]
    fn a_closed_s1_or_s2_record_without_an_asset_fails() {
        for severity in ["S1", "S2"] {
            let finding = parse_finding("UXF-001.md", &record("closed", severity, "none")).unwrap();
            let problems = check_finding(&finding, &any_file);
            assert!(
                problems
                    .iter()
                    .any(|problem| problem.contains("needs a regression asset")),
                "{problems:?}"
            );
        }
        let s3 = parse_finding("UXF-001.md", &record("closed", "S3", "none")).unwrap();
        assert_eq!(check_finding(&s3, &any_file), Vec::<String>::new());
        let open = parse_finding("UXF-001.md", &record("open", "S1", "none")).unwrap();
        assert_eq!(check_finding(&open, &any_file), Vec::<String>::new());
    }

    #[test]
    fn an_asset_that_names_a_missing_file_fails() {
        let finding = parse_finding(
            "UXF-001.md",
            &record("closed", "S2", "website/ux/specs/gone.spec.ts › gate 1"),
        )
        .unwrap();
        let problems = check_finding(&finding, &|path| {
            (path != "website/ux/specs/gone.spec.ts").then(String::new)
        });
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("does not exist")),
            "{problems:?}"
        );
        assert_eq!(check_finding(&finding, &any_file), Vec::<String>::new());
    }

    #[test]
    fn an_asset_must_name_a_test_its_file_declares() {
        let spec = "for (const path of pages) {\n  test(`L2 ${path} ${viewport.name}`, async () => {});\n}\n\
                    test('L2 site search: every template page builds a search control (UXF-001)', () => {});\n\
                    test.describe.configure({ mode: 'parallel' });\n";
        let read = |_: &str| Some(spec.to_owned());
        let named = |asset: &str| {
            check_finding(
                &parse_finding("UXF-001.md", &record("open", "S2", asset)).unwrap(),
                &read,
            )
        };
        assert_eq!(
            named("website/ux/specs/l2-layout.spec.ts :: L2 /ess/docs/start/install desktop-1280"),
            Vec::<String>::new()
        );
        assert_eq!(
            named("website/ux/specs/l2-layout.spec.ts :: L2 site search: every template page builds a search control (UXF-001), held by website/ux/baseline.json"),
            Vec::<String>::new()
        );
        let problems =
            named("website/ux/specs/l2-layout.spec.ts :: L3 install desktop-1280 pointer");
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("declares no test")),
            "{problems:?}"
        );
    }

    #[test]
    fn a_record_without_its_header_fields_fails() {
        let finding = parse_finding("UXF-002.md", "---\nid: UXF-003\nclass: vibes\n---\n").unwrap();
        let problems = check_finding(&finding, &any_file);
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("`severity` is missing")),
            "{problems:?}"
        );
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("does not match the file name")),
            "{problems:?}"
        );
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("class `vibes`")),
            "{problems:?}"
        );
        assert!(parse_finding("UXF-002.md", "# no front matter\n").is_err());
    }

    /// The repository root: the first ancestor of this crate that holds the findings directory.
    fn repository_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .find(|directory| directory.join(FINDINGS).is_dir())
            .expect("an ancestor of this crate holds website/ux/findings")
            .to_path_buf()
    }

    #[test]
    fn the_findings_directory_holds_no_closed_s1_or_s2_record_without_an_asset() {
        let problems = check_findings(&repository_root()).expect("the findings directory reads");
        assert_eq!(problems, Vec::<String>::new());
    }

    #[test]
    fn the_committed_baseline_parses_and_defines_every_tolerance_it_names() {
        let root = repository_root();
        let base: Baseline = serde_json::from_str(
            &fs::read_to_string(root.join(BASELINE)).expect("the baseline reads"),
        )
        .expect("the baseline parses");
        for (scenario, entries) in &base.scenarios {
            for (metric, entry) in entries {
                assert!(
                    tolerance_of(&base, scenario, metric, entry).is_ok(),
                    "{scenario} {metric}"
                );
            }
        }
    }

    #[test]
    fn the_measuring_core_imports_nothing_from_the_adapter_or_the_scenarios() {
        let core = repository_root().join("website/ux/core");
        let mut read = 0;
        for entry in fs::read_dir(&core).expect("website/ux/core reads") {
            let path = entry.expect("a directory entry").path();
            if path.extension().is_some_and(|extension| extension == "ts") {
                read += 1;
                let text = fs::read_to_string(&path).expect("a core file reads");
                for line in text
                    .lines()
                    .filter(|line| line.contains("from '") || line.contains("import("))
                {
                    assert!(
                        !line.contains("adapter")
                            && !line.contains("scenarios")
                            && !line.contains("../specs"),
                        "{}: the core imports project code: {line}",
                        path.display()
                    );
                }
            }
        }
        assert!(read > 0, "no core file was read");
    }
}
