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
//! without a regression asset that exists, every asset that names a test names one its file
//! declares, and an open record states the message its asset fails with (`failure_pattern`, `none`
//! when the asset is no expected failure).
//!
//! 8. a run that did not pass (T41): a round of one run whose `timing` status is not `passed` or
//!    that names an unexpected test, and a run of a merged round that was cut, or that failed a
//!    test no later run of the round passed.
//!
//! A decision (`--decision <id>`) takes only the worse and lost values it lists in the round's
//! `decision.md`, as backticked list items under the `## ` heading that names the id: a worse
//! value as `<scenario> <metric> <value>`, at that value and no other (T35), a lost value as
//! `<scenario> <metric>`, `<scenario>` or `<lane>: <scenario>`. A value worse than the baseline
//! by no more than its tolerance passes and is named in a note (T36).

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
    /// What the file is, for a reader: written by every update from [`Baseline::sources`] and the
    /// counts, so it never names a round the values no longer come from.
    pub about: String,
    /// The round each lane was last taken from, by lane.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub sources: BTreeMap<String, String>,
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
    /// The round's name (`R-nn`), or the lane's for a run outside a round.
    #[serde(default, rename = "round")]
    pub name: String,
    /// The lane the round ran, `full` when it does not say.
    #[serde(default = "full_lane")]
    pub lane: String,
    /// Scenario, then metric key, then the measured value.
    pub scenarios: BTreeMap<String, BTreeMap<String, f64>>,
    /// Required: a round that does not say what it left unmeasured is not read.
    pub coverage: Coverage,
    /// Required: a round that does not say which engine measured it is not read.
    pub engine: Engine,
    /// Every run merged into the round, each with the engine that measured it.
    #[serde(default)]
    pub runs: Vec<RunEngine>,
    /// How the run ended, for a round of one run (a gate run's `round.json`); a merged round says
    /// it per run.
    #[serde(default)]
    pub timing: Option<RunOutcome>,
    /// The values a later run replaced: scenario, then metric key, then every run's value in run
    /// order. Only a merged round has them.
    #[serde(default)]
    pub replaced: BTreeMap<String, BTreeMap<String, Vec<Replaced>>>,
    /// The engine digest of the committed tree, computed by the ratchet (never read from the
    /// round); `None` skips the digest check, as the canned rounds of the tests do.
    #[serde(skip)]
    pub committed_digest: Option<String>,
}

/// One run of a round and the engine that measured it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct RunEngine {
    /// The run's name.
    pub name: String,
    /// Its engine.
    pub engine: Engine,
    /// The lane it ran: `slow` for a run of a `full` round's measurement; a run of another lane (the
    /// gate runs a round reports for their time) is not judged by its outcome.
    #[serde(default)]
    pub lane: Option<String>,
    /// How it ended.
    #[serde(default)]
    pub timing: Option<RunOutcome>,
}

/// How a run ended, as its `timing` records it (T41 of the R-04 report review).
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct RunOutcome {
    /// Playwright's status of the whole run: `passed`, `failed`, `timedout` or `interrupted`.
    pub status: String,
    /// Each test whose outcome was not the expected one, with its status and why.
    #[serde(default)]
    pub unexpected: Vec<String>,
    /// The same tests as `[<project>] <title>`.
    #[serde(default)]
    pub unexpected_tests: Vec<String>,
    /// Every test that passed, as `[<project>] <title>`; a known failure is not one (T46).
    #[serde(default)]
    pub passed: Vec<String>,
    /// Every test that ended as its open finding states, as `[<project>] <title>`.
    #[serde(default)]
    pub known_failures: Vec<String>,
}

/// One run's value of a scenario and metric that more than one run measured.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Replaced {
    /// The run.
    pub run: String,
    /// Its value.
    pub value: f64,
}

/// A decision: its id and the `<scenario> <metric>` keys it covers.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) struct Decision {
    /// The id, written beside every value it takes.
    pub id: String,
    /// The keys it names.
    pub keys: BTreeSet<String>,
}

/// The engine that measured a round, as `round.json` records it.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Engine {
    /// The digest of the engine files.
    #[serde(default)]
    pub digest: String,
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
    /// A run (or the round) was not measured by the engine the committed tree holds.
    Unattributable { run: String, why: String },
    /// A run (or the round) did not pass, and no later run of the round passed what it failed.
    RunNotPassed { run: String, why: String },
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
            Breach::Unattributable { run, why } => write!(
                f,
                "{run}: not measured by the committed tree's engine: {why}; commit the engine and \
                 measure again"
            ),
            Breach::RunNotPassed { run, why } => write!(
                f,
                "{run}: the run did not pass: {why}; a round is taken only from runs that passed, \
                 or whose failed tests a later run of the round passed"
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

/// Compares a round with the baseline, under no decision.
#[cfg(test)]
pub(crate) fn compare(baseline: &Baseline, round: &Round) -> Vec<Breach> {
    compare_with(baseline, round, None)
}

/// Compares a round with the baseline; `decision` names the keys it decided.
pub(crate) fn compare_with(
    baseline: &Baseline,
    round: &Round,
    decision: Option<&Decision>,
) -> Vec<Breach> {
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
    breaches.extend(unattributable(round));
    breaches.extend(not_passed(round));
    let value_of = |scenario: &str, metric: &str, value: f64| {
        held_value(round, decision, scenario, metric, value)
    };
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
                Some(&value) => breaches.extend(judge(
                    scenario,
                    metric,
                    entry,
                    tolerance,
                    value_of(scenario, metric, value),
                )),
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
                    value: value_of(scenario, metric, value),
                });
            }
        }
    }
    breaches
}

/// The values worse than the baseline by no more than their tolerance (T36): the ratchet lets them
/// pass and names each, so a rise inside the tolerance is seen.
pub(crate) fn within_tolerance(
    baseline: &Baseline,
    round: &Round,
    decision: Option<&Decision>,
) -> Vec<String> {
    let lane = scope(baseline, &round.lane);
    let mut notes = Vec::new();
    for (scenario, entries) in &baseline.scenarios {
        if lane.is_some_and(|set| !set.contains(scenario)) {
            continue;
        }
        let Some(values) = round.scenarios.get(scenario) else {
            continue;
        };
        for (metric, entry) in entries {
            let (Some(&value), Ok(tolerance)) = (
                values.get(metric),
                tolerance_of(baseline, scenario, metric, entry),
            ) else {
                continue;
            };
            let held = held_value(round, decision, scenario, metric, value);
            let allowance = tolerance.allowance(entry.value);
            if held > entry.value + EPSILON && held <= entry.value + allowance + EPSILON {
                // The allowance is a product of floats (6.8500000000000005); four places say it.
                let shown = (allowance * 1e4).round() / 1e4;
                notes.push(format!(
                    "{scenario} {metric}: {held} against {}, worse inside its tolerance of {shown}",
                    entry.value
                ));
            }
        }
    }
    notes
}

/// The runs that did not pass (T41): the round's own run when its `timing` says it did not pass
/// or names an unexpected test, a run of a merged round whose status is neither `passed` nor
/// `failed`, a passed run that names an unexpected test, and a failed run with a failed test no
/// later run of the round passed. A run without `timing`, and a run of another lane than the
/// round's measurement, is not judged here.
fn not_passed(round: &Round) -> Vec<Breach> {
    let measures = |run: &RunEngine| {
        run.lane
            .as_deref()
            .is_none_or(|lane| lane == round.lane || (lane == "slow" && round.lane == FULL))
    };
    let runs: Vec<&RunEngine> = round.runs.iter().filter(|run| measures(run)).collect();
    let mut out = Vec::new();
    // T47: a round that says neither how its runs ended nor how it ended is not judged as passed.
    if round.runs.is_empty() && round.timing.is_none() {
        out.push(Breach::RunNotPassed {
            run: "the round".to_owned(),
            why: "it names no run and no `timing`, so it does not say how it ended".to_owned(),
        });
    }
    if let Some(timing) = &round.timing {
        let mut why = Vec::new();
        if timing.status != "passed" {
            why.push(format!("its status is `{}`", timing.status));
        }
        if let Some(first) = timing.unexpected.first() {
            why.push(format!(
                "{} unexpected test(s), first {first}",
                timing.unexpected.len()
            ));
        }
        if !why.is_empty() {
            out.push(Breach::RunNotPassed {
                run: "the round".to_owned(),
                why: why.join(", "),
            });
        }
    }
    for (index, run) in runs.iter().enumerate() {
        let Some(timing) = &run.timing else {
            out.push(Breach::RunNotPassed {
                run: run.name.clone(),
                why: "it has no `timing`, so it does not say how it ended".to_owned(),
            });
            continue;
        };
        // A later run rescues a failed test when it passed it or ended it as its open finding
        // states (T46): `passed` holds no known failure.
        let later: BTreeSet<&str> = runs[index + 1..]
            .iter()
            .filter_map(|later| later.timing.as_ref())
            .flat_map(|later| later.passed.iter().chain(&later.known_failures))
            .map(String::as_str)
            .collect();
        let why = match timing.status.as_str() {
            "passed" | "failed" if timing.unexpected.len() != timing.unexpected_tests.len() => {
                Some(format!(
                    "its status is `{}` and it names {} unexpected test(s) by title for {} unexpected outcome(s)",
                    timing.status,
                    timing.unexpected_tests.len(),
                    timing.unexpected.len()
                ))
            }
            "failed" if timing.unexpected_tests.is_empty() => {
                Some("its status is `failed` and it names no failed test".to_owned())
            }
            "passed" | "failed" => {
                let open: Vec<&str> = timing
                    .unexpected_tests
                    .iter()
                    .map(String::as_str)
                    .filter(|test| !later.contains(test))
                    .collect();
                (!open.is_empty()).then(|| {
                    format!(
                        "no later run passed {} of its failed test(s), first {}",
                        open.len(),
                        open[0]
                    )
                })
            }
            status => Some(format!("its status is `{status}`")),
        };
        if let Some(why) = why {
            out.push(Breach::RunNotPassed {
                run: run.name.clone(),
                why,
            });
        }
    }
    out
}

/// Whether the decision takes this worse value: a list item `<scenario> <metric> <value>` that
/// names the scenario, the metric and the value (T35).
fn takes_value(decision: &Decision, scenario: &str, metric: &str, value: f64) -> bool {
    decision.keys.iter().any(|key| {
        let parts: Vec<&str> = key.split_whitespace().collect();
        matches!(parts.as_slice(), [s, m, v] if *s == scenario && *m == metric
            && v.parse::<f64>().is_ok_and(|named| (named - value).abs() <= EPSILON))
    })
}

/// The value the ratchet holds for a scenario and metric: the round's value, or the worst value
/// any of its runs measured when a rerun replaced one (T27), unless the decision names the
/// round's value with its scenario and metric (T35).
fn held_value(
    round: &Round,
    decision: Option<&Decision>,
    scenario: &str,
    metric: &str,
    value: f64,
) -> f64 {
    if decision.is_some_and(|d| takes_value(d, scenario, metric, value)) {
        return value;
    }
    round
        .replaced
        .get(scenario)
        .and_then(|metrics| metrics.get(metric))
        .into_iter()
        .flatten()
        .map(|replaced| replaced.value)
        .fold(value, f64::max)
}

/// The runs a round cannot be attributed to its committed tree by: a run, or the round itself,
/// whose engine commit is unknown, whose engine was dirty, or whose engine digest is not the
/// committed tree's (T25, T26). `round.engine.dirty` is reported by `DirtyEngine`.
fn unattributable(round: &Round) -> Vec<Breach> {
    let top = RunEngine {
        name: "the round".to_owned(),
        engine: Engine {
            dirty: false,
            ..round.engine.clone()
        },
        lane: None,
        timing: None,
    };
    let mut out = Vec::new();
    for run in std::iter::once(&top).chain(&round.runs) {
        let engine = &run.engine;
        let mut why = Vec::new();
        if matches!(engine.commit.trim(), "" | "unknown" | "none") {
            why.push(format!("its engine commit is `{}`", engine.commit));
        }
        if engine.dirty {
            why.push("its engine had uncommitted changes".to_owned());
        }
        if let Some(committed) = &round.committed_digest {
            if &engine.digest != committed {
                why.push(format!(
                    "its engine digest `{}` is not the committed tree's `{committed}`",
                    engine.digest
                ));
            }
        }
        if !why.is_empty() {
            out.push(Breach::Unattributable {
                run: run.name.clone(),
                why: why.join(", "),
            });
        }
    }
    out
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
/// unit, a dirty engine and runs of different engines are never taken. A decision is its id and
/// the keys it names.
#[cfg(test)]
pub(crate) fn update(
    baseline: &Baseline,
    round: &Round,
    decision: Option<(&str, &[&str])>,
) -> Result<(Baseline, Taken), Vec<Breach>> {
    let decision = decision.map(|(id, keys)| Decision {
        id: id.to_owned(),
        keys: keys.iter().map(|key| (*key).to_owned()).collect(),
    });
    update_with(baseline, round, decision.as_ref())
}

/// The decision `id` of a round's `decision.md`: the section whose `## ` heading names the id in
/// backticks, and the keys its list items name in backticks.
pub(crate) fn decision_of(text: &str, id: &str) -> Result<Decision, String> {
    let named = format!("`{id}`");
    let mut keys = BTreeSet::new();
    let mut found = false;
    let mut inside = false;
    for line in text.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            inside = heading.contains(&named);
            found |= inside;
            continue;
        }
        if !inside {
            continue;
        }
        if let Some(key) = line
            .trim_start()
            .strip_prefix("- `")
            .and_then(|rest| rest.split_once('`'))
            .map(|(key, _)| key.trim())
        {
            if !key.is_empty() {
                keys.insert(key.to_owned());
            }
        }
    }
    if !found {
        return Err(format!(
            "the decision file has no `## ` heading that names {named}"
        ));
    }
    if keys.is_empty() {
        return Err(format!(
            "decision {named} lists no key: list each worse value it takes as a list item \
             `- `<scenario> <metric> <value>``, and each lost value as `- `<scenario> <metric>``, \
             `- `<scenario>`` or `- `<lane>: <scenario>``"
        ));
    }
    Ok(Decision {
        id: id.to_owned(),
        keys,
    })
}

/// Whether the decision lists a lost value: by `<scenario> <metric>`, by its whole `<scenario>`,
/// or, for a scenario a lane dropped, by `<lane>: <scenario>`. A worse value is taken only by
/// [`takes_value`].
fn covers(decision: &Decision, lane: Option<&str>, scenario: &str, metric: &str) -> bool {
    decision.keys.contains(&format!("{scenario} {metric}"))
        || decision.keys.contains(scenario)
        || lane.is_some_and(|lane| decision.keys.contains(&format!("{lane}: {scenario}")))
}

/// [`update`] under a decision that names the keys it covers.
pub(crate) fn update_with(
    baseline: &Baseline,
    round: &Round,
    named: Option<&Decision>,
) -> Result<(Baseline, Taken), Vec<Breach>> {
    let lane = (round.lane != FULL).then(|| round.lane.clone());
    // The decision's id when it lists the value, else none: a decision takes only what it lists
    // (T29 of the R-03 report review).
    let decided = |scenario: &str, metric: &str| {
        named
            .filter(|d| !d.id.trim().is_empty() && covers(d, lane.as_deref(), scenario, metric))
            .map(|d| d.id.as_str())
    };
    // A worse value only at the value the decision names (T35).
    let decided_value = |scenario: &str, metric: &str, value: f64| {
        named
            .filter(|d| !d.id.trim().is_empty() && takes_value(d, scenario, metric, value))
            .map(|d| d.id.as_str())
    };
    let mut next = baseline.clone();
    let mut taken = Taken::default();
    let mut refused = Vec::new();
    for breach in compare_with(baseline, round, named) {
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
            } => match decided_value(&scenario, &metric, value) {
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
            Breach::Missing { scenario, metric } => match decided(&scenario, &metric) {
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
        let name = if round.name.trim().is_empty() {
            round.lane.clone()
        } else {
            round.name.clone()
        };
        next.sources.insert(round.lane.clone(), name);
        next.about = about(&next);
        Ok((next, taken))
    } else {
        Err(refused)
    }
}

/// The baseline's `about`: what the file is, the round each lane was taken from, and its size.
pub(crate) fn about(baseline: &Baseline) -> String {
    let lanes = baseline
        .sources
        .iter()
        .map(|(lane, round)| format!("lane {lane} from round {round}"))
        .collect::<Vec<_>>()
        .join(", ");
    let values: usize = baseline.scenarios.values().map(BTreeMap::len).sum();
    format!(
        "UX ratchet baseline of the ESS documentation website, lower is better; written by cargo \
         xtask ux-ratchet --update: {lanes}; {values} values in {} scenario(s)",
        baseline.scenarios.len()
    )
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
    if !baseline.sources.is_empty() {
        let _ = writeln!(
            out,
            "  \"sources\": {},",
            serde_json::to_string(&baseline.sources)?
                .replace("\":", "\": ")
                .replace(",\"", ", \"")
        );
    }
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
    // Every asset of the field, not the first alone (T28 of the R-03 report review).
    for one in assets(asset) {
        let Some(path) = asset_path(one) else {
            continue;
        };
        match read(path) {
            None => problems.push(format!("{file}: regression asset `{path}` does not exist")),
            Some(text) => {
                if let Some(title) = asset_title(one) {
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
    if field("status") == "open" {
        problems.extend(failure_pattern_problems(
            file,
            field("failure_pattern"),
            asset,
        ));
    }
    problems
}

/// The assets a `regression_asset` field names: one, or several separated by `; ` before the next
/// `<path>.spec.ts :: `, as `assetTests` in `website/ux/core/findings.ts` splits them.
fn assets(field: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut rest = field;
    'parts: loop {
        let mut from = 0;
        while let Some(found) = rest[from..].find("; ") {
            let at = from + found;
            let next = &rest[at + 2..];
            let token = next.split(char::is_whitespace).next().unwrap_or("");
            if token.ends_with(".spec.ts")
                && !token.contains(';')
                && next[token.len()..].starts_with(" :: ")
            {
                out.push(&rest[..at]);
                rest = next;
                continue 'parts;
            }
            from = at + 2;
        }
        out.push(rest);
        return out;
    }
}

/// Whether an asset is a test the gate runs as an expected failure (`assetTest` in
/// `website/ux/core/findings.ts`): it names a spec path and a title, and is not held by a file.
fn names_expected_failure(asset: &str) -> bool {
    let Some((spec, rest)) = asset.split_once("::") else {
        return false;
    };
    let spec = spec.trim();
    if !spec.contains('/') || spec.contains(char::is_whitespace) {
        return false;
    }
    let (title, held) = match rest.find(", held by ") {
        Some(at) => (&rest[..at], Some(&rest[at + ", held by ".len()..])),
        None => (rest, None),
    };
    if title.trim().is_empty() {
        return false;
    }
    !held.is_some_and(|by| {
        by.split(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | ':'))
            .next()
            .is_some_and(|token| token.contains('/'))
    })
}

/// An open record states the message its asset fails with (T28 of the R-03 report review): a
/// pattern where the asset is an expected failure, `none` where it is not.
fn failure_pattern_problems(file: &str, pattern: &str, asset: &str) -> Vec<String> {
    let parts = assets(asset);
    let expected = parts
        .first()
        .is_some_and(|first| names_expected_failure(first));
    if pattern.is_empty() {
        return vec![format!(
            "{file}: header field `failure_pattern` is missing or empty: an open record states the \
             message its asset fails with, or `none` when its asset is no expected failure"
        )];
    }
    match (expected, pattern == "none") {
        (true, true) => vec![format!(
            "{file}: its asset is an expected failure, so `failure_pattern` names the message it \
             fails with, not `none`"
        )],
        (false, false) => vec![format!(
            "{file}: its asset is no expected failure (no test, or a test held by a file), so \
             `failure_pattern` is `none`, not `{pattern}`"
        )],
        _ => Vec::new(),
    }
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

/// The loop's root, relative to the repository root.
pub(crate) const LOOP_ROOT: &str = "website/ux";

/// What under [`LOOP_ROOT`] is not engine: dependencies, run output, and what the engine writes
/// or the loop records. The same list as `NOT_ENGINE` in `website/ux/core/engine.ts`; a test
/// holds the two equal.
pub(crate) const NOT_ENGINE: &[&str] = &[
    "node_modules",
    ".out",
    "rounds",
    "findings",
    "baseline.json",
    "trend.md",
];

/// The engine digest of `website/ux/core/engine.ts`: SHA-256 over each engine file's path
/// (relative to the loop's root, sorted) and content, each followed by a zero byte, as 16 hex
/// digits.
pub(crate) fn engine_digest(files: &BTreeMap<String, Vec<u8>>) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for (path, content) in files {
        hasher.update(path.as_bytes());
        hasher.update([0]);
        hasher.update(content);
        hasher.update([0]);
    }
    hasher
        .finalize()
        .iter()
        .take(8)
        .fold(String::new(), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}

/// The engine digest of the tree `HEAD` commits, read from git, so a round is compared with the
/// committed engine and not with the working copy.
fn committed_engine_digest(root: &Path) -> Result<String> {
    let listed = git(root, &["ls-tree", "-r", "-z", "HEAD", "--", LOOP_ROOT])?;
    if !listed.status.success() {
        bail!(
            "git ls-tree HEAD -- {LOOP_ROOT} failed, so the committed engine is unknown: {}",
            String::from_utf8_lossy(&listed.stderr).trim()
        );
    }
    let prefix = format!("{LOOP_ROOT}/");
    let mut files = BTreeMap::new();
    for entry in listed.stdout.split(|byte| *byte == 0) {
        let entry = String::from_utf8_lossy(entry);
        let Some((meta, path)) = entry.split_once('\t') else {
            continue;
        };
        let mut fields = meta.split_whitespace();
        let (Some(_mode), Some("blob"), Some(object)) =
            (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        let Some(relative) = path.strip_prefix(&prefix) else {
            continue;
        };
        let first = relative.split('/').next().unwrap_or(relative);
        if NOT_ENGINE.contains(&first) {
            continue;
        }
        let blob = git(root, &["cat-file", "blob", object])?;
        if !blob.status.success() {
            bail!("git cat-file blob {object} ({path}) failed");
        }
        files.insert(relative.to_owned(), blob.stdout);
    }
    if files.is_empty() {
        bail!("HEAD holds no engine file under {LOOP_ROOT}");
    }
    Ok(engine_digest(&files))
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
    let mut measured = measured;
    measured.committed_digest = Some(committed_engine_digest(root)?);
    // A decision is read from `decision.md` beside the round: its id's section lists what it takes.
    let decision = match decision {
        Some(id) => {
            let file = round_path.with_file_name("decision.md");
            let text = fs::read_to_string(&file).with_context(|| {
                format!(
                    "reading {}, which lists the values decision {id} takes",
                    file.display()
                )
            })?;
            Some(
                decision_of(&text, id)
                    .map_err(|why| anyhow::anyhow!("{}: {why}", file.display()))?,
            )
        }
        None => None,
    };
    let mut report = String::new();
    for note in within_tolerance(&baseline, &measured, decision.as_ref()) {
        let _ = writeln!(report, "ux ratchet: note: {note}");
    }
    if update_baseline {
        let (next, taken) =
            update_with(&baseline, &measured, decision.as_ref()).map_err(|refused| {
                anyhow::anyhow!("{report}the baseline was not updated:\n{}", lines(&refused))
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
            decision.as_ref().map_or("none", |d| d.id.as_str())
        );
    }
    let mut breaches = compare_with(&baseline, &measured, decision.as_ref());
    if let Some(main) = main_baseline(root)? {
        breaches.extend(compare_with_main(&main, &baseline));
    }
    let findings = check_findings(root)?;
    if !breaches.is_empty() || !findings.is_empty() {
        bail!(failure_message(
            &report,
            &breaches,
            &findings,
            update_baseline
        ));
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

/// What the command prints when it fails: the tolerance notes written so far (T48), the breaches,
/// the findings' problems, and whether the baseline file was left unwritten.
fn failure_message(
    report: &str,
    breaches: &[Breach],
    findings: &[String],
    update_baseline: bool,
) -> String {
    let mut message = report.to_owned();
    if !breaches.is_empty() {
        let _ = writeln!(
            message,
            "ratchet: {} breach(es)\n{}",
            breaches.len(),
            lines(breaches)
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
    message.trim_end().to_owned()
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
        // A round says how it ended (T47): one of one run by its `timing`, a merged one by its runs.
        if !json.contains("\"runs\"") && !json.contains("\"timing\"") {
            json = json.replacen('{', r#"{"timing": {"status": "passed"}, "#, 1);
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
        assert!(update(
            &baseline(BASE),
            &measured,
            Some(("D-1", &["install/desktop-1280"]))
        )
        .is_err());
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
        let (next, _) = update(
            &baseline(CONTENT),
            &one_more_link,
            Some((
                "D-5",
                &["/ess/docs/a@desktop-1280/light axe.serious_critical_nodes 11"],
            )),
        )
        .expect("a decision takes it");
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
        assert!(update(
            &baseline(BASE),
            &measured,
            Some(("D-1", &["install/desktop-1280"]))
        )
        .is_err());
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
        assert!(update(
            &baseline(BASE),
            &measured,
            Some(("D-1", &["install/desktop-1280"]))
        )
        .is_err());
    }

    /// A canned round of `BASE`'s values, merged from `runs`, held to a committed tree whose engine
    /// digest is `d`.
    fn merged(runs: &str) -> Round {
        let mut measured = round(&format!(
            r#"{{"engine": {{"digest": "d", "commit": "c", "dirty": false}}, "runs": {runs},
              "scenarios": {{"install/desktop-1280":
              {{"screen_only.actions": 3, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}}}}"#
        ));
        measured.committed_digest = Some("d".into());
        measured
    }

    const CLEAN_RUN: &str = r#"{"name": "slow-l2-1", "engine": {"digest": "d", "commit": "c", "dirty": false},
          "timing": {"status": "passed"}}"#;

    /// T26 of the R-03 report review: R-03 was measured by engine `bc3e06f44faa130b` while the
    /// committed tree's engine was `93f932aacd3f4a09`, and nothing compared the two.
    #[test]
    fn a_run_measured_by_another_engine_than_the_committed_trees_is_refused() {
        assert_eq!(
            compare(&baseline(BASE), &merged(&format!("[{CLEAN_RUN}]"))),
            vec![]
        );
        let older = merged(&format!(
            r#"[{CLEAN_RUN}, {{"name": "rerun-essui", "engine": {{"digest": "e", "commit": "c", "dirty": false}}}}]"#
        ));
        let breaches = compare(&baseline(BASE), &older);
        assert!(
            breaches.iter().any(|breach| {
                let text = breach.to_string();
                text.contains("rerun-essui") && text.contains("committed tree")
            }),
            "{breaches:?}"
        );
        assert!(update(
            &baseline(BASE),
            &older,
            Some(("D-1", &["install/desktop-1280"]))
        )
        .is_err());
    }

    /// T25: two reruns of R-03 ran from an engine copy outside Git (commit "unknown", dirty), and
    /// the round took its engine from its first run only.
    #[test]
    fn a_run_whose_engine_commit_is_unknown_or_dirty_is_refused() {
        for (case, engine) in [
            (
                "unknown",
                r#"{"digest": "d", "commit": "unknown", "dirty": false}"#,
            ),
            ("dirty", r#"{"digest": "d", "commit": "c", "dirty": true}"#),
        ] {
            let outside = merged(&format!(
                r#"[{CLEAN_RUN}, {{"name": "rerun-shift", "engine": {engine}}}]"#
            ));
            let breaches = compare(&baseline(BASE), &outside);
            assert!(
                breaches
                    .iter()
                    .any(|breach| breach.to_string().contains("rerun-shift")),
                "{case}: {breaches:?}"
            );
            assert!(
                update(
                    &baseline(BASE),
                    &outside,
                    Some(("D-1", &["install/desktop-1280"]))
                )
                .is_err(),
                "{case}"
            );
        }
    }

    const SHIFT: &str = r#"{
      "about": "canned",
      "tolerances": {"layout_shift": {"absolute": 0.01}},
      "scenarios": {"/p@m/dark": {"layout_shift.load": {"value": 0, "tolerance": "layout_shift"}}}
    }"#;

    const RERUN: &str = r#"{"scenarios": {"/p@m/dark": {"layout_shift.load": 0}},
      "replaced": {"/p@m/dark": {"layout_shift.load": [
        {"run": "slow-l2-4", "value": 0.0253}, {"run": "rerun-shift", "value": 0}]}}}"#;

    /// T27: a rerun replaced `layout_shift.load` 0.0253 by 0 and the ratchet saw only the 0.
    #[test]
    fn a_value_a_rerun_replaced_is_held_at_the_worse_of_its_runs() {
        let breaches = compare(&baseline(SHIFT), &round(RERUN));
        assert!(
            matches!(breaches.as_slice(), [Breach::Worse { value, .. }] if (*value - 0.0253).abs() < 1e-12),
            "{breaches:?}"
        );
        assert!(update(&baseline(SHIFT), &round(RERUN), None).is_err());
    }

    /// T27: a decision that names the replaced value holds the rerun's value; one that names
    /// another key does not.
    #[test]
    fn a_decision_that_names_a_replaced_value_holds_the_value_that_replaced_it() {
        let named = Decision {
            id: "R-04-shift".into(),
            keys: BTreeSet::from(["/p@m/dark layout_shift.load 0".to_owned()]),
        };
        assert_eq!(
            compare_with(&baseline(SHIFT), &round(RERUN), Some(&named)),
            vec![]
        );
        let other = Decision {
            id: "R-04-other".into(),
            keys: BTreeSet::from(["/q@m/dark layout_shift.load".to_owned()]),
        };
        assert_ne!(
            compare_with(&baseline(SHIFT), &round(RERUN), Some(&other)),
            vec![]
        );
    }

    /// The ratchet computes the committed tree's engine digest as `core/engine.ts` does: paths
    /// sorted as whole strings (`core.ts` before `core/b.ts`); the same value is pinned by the
    /// L1 test "L1 engine: the digest sorts whole paths, as the ratchet does".
    #[test]
    fn the_engine_digest_is_the_one_the_measuring_engine_writes() {
        let files = BTreeMap::from([
            ("core/b.ts".to_owned(), b"two".to_vec()),
            ("core.ts".to_owned(), b"one".to_vec()),
        ]);
        assert_eq!(engine_digest(&files), "804bb99ed3bd75aa");
        let engine = fs::read_to_string(repository_root().join(LOOP_ROOT).join("core/engine.ts"))
            .expect("engine.ts reads");
        let listed = engine
            .lines()
            .find(|line| line.contains("export const NOT_ENGINE"))
            .expect("engine.ts declares NOT_ENGINE");
        let names: Vec<&str> = listed.split('\'').skip(1).step_by(2).collect();
        assert_eq!(names, NOT_ENGINE);
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
        let (next, _) = update(&main, &shrunk, Some(("D-9", &["gate: /b@d/light"])))
            .expect("a decision drops it");
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
        let (next, taken) = update(
            &baseline(BASE),
            &measured,
            Some(("R-02-baseline", &["install/desktop-1280"])),
        )
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
        let (next, _) = update(
            &baseline(BASE),
            &measured,
            Some(("D-7", &["install/desktop-1280 glance_cost.p95 2.5"])),
        )
        .expect("a decision takes the worse value");
        let entries = &next.scenarios["install/desktop-1280"];
        assert_eq!(entries["screen_only.actions"].value, 2.0);
        assert_eq!(entries["glance_cost.p95"].decision.as_deref(), Some("D-7"));
        assert_eq!(entries["reading_load.max"].tolerance, "words");
    }

    /// T23 of the R-02 report review: `about` still named the R-01 round after R-02 replaced every
    /// value. An update writes which round each lane was taken from, and `about` says it.
    #[test]
    fn an_update_names_the_round_each_lane_was_taken_from() {
        let full = round(
            r#"{"round": "R-03", "lane": "full", "scenarios": {"install/desktop-1280":
              {"screen_only.actions": 2, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        );
        let (next, _) = update(&baseline(BASE), &full, None).expect("an improvement");
        assert!(
            next.about.contains("lane full from round R-03")
                && next.about.contains("3 values in 1 scenario"),
            "{}",
            next.about
        );
        let gate = round(
            r#"{"round": "R-04", "lane": "gate", "scenarios": {"install/desktop-1280":
              {"screen_only.actions": 2, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        );
        let (later, _) = update(&next, &gate, None).expect("a lane addition");
        assert!(
            later.about.contains("lane full from round R-03")
                && later.about.contains("lane gate from round R-04"),
            "{}",
            later.about
        );
    }

    #[test]
    fn update_refuses_a_missing_value_without_a_decision() {
        let measured =
            round(r#"{"scenarios": {"install/desktop-1280": {"screen_only.actions": 3}}}"#);
        assert!(update(&baseline(BASE), &measured, None).is_err());
        let (next, _) = update(
            &baseline(BASE),
            &measured,
            Some((
                "D-1",
                &[
                    "install/desktop-1280 glance_cost.p95",
                    "install/desktop-1280 layout_shift.max",
                ],
            )),
        )
        .expect("dropped under D-1");
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
        base.sources.insert("full".into(), "R-03".into());
        base.sources.insert("gate".into(), "R-03".into());
        let text = to_text(&base).expect("written");
        assert!(
            text.contains(r#""sources": {"full": "R-03", "gate": "R-03"},"#),
            "{text}"
        );
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
        let open = record("open", "S1", "none").replacen(
            "regression_asset:",
            "failure_pattern: none\nregression_asset:",
            1,
        );
        let open = parse_finding("UXF-001.md", &open).unwrap();
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
                &parse_finding("UXF-001.md", &record("closed", "S2", asset)).unwrap(),
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

    /// T28 of the R-03 report review: a failure of an open finding's asset was held whatever its
    /// message. An open record states the message its asset fails with, `failure_pattern`, and
    /// says `none` when its asset is no expected failure (no test yet, or held by a file).
    #[test]
    fn an_open_record_states_the_message_its_asset_fails_with() {
        let test_asset = "website/ux/specs/l2-layout.spec.ts :: L2 /ess/ desktop-1280";
        let problems_of = |asset: &str, pattern: Option<&str>| {
            let mut text = record("open", "S2", asset);
            if let Some(pattern) = pattern {
                text = text.replacen(
                    "regression_asset:",
                    &format!("failure_pattern: {pattern}\nregression_asset:"),
                    1,
                );
            }
            check_finding(&parse_finding("UXF-001.md", &text).unwrap(), &|_| {
                Some("test(`L2 ${path} ${viewport.name}`, () => {});".to_owned())
            })
        };
        let missing = problems_of(test_asset, None);
        assert!(
            missing.iter().any(|p| p.contains("`failure_pattern`")),
            "{missing:?}"
        );
        let none = problems_of(test_asset, Some("none"));
        assert!(
            none.iter().any(|p| p.contains("expected failure")),
            "{none:?}"
        );
        assert_eq!(
            problems_of(test_asset, Some("axe label-content-name-mismatch")),
            Vec::<String>::new()
        );
        let proposed = problems_of("none yet; proposed: an L2 probe", Some("gate 5"));
        assert!(
            proposed.iter().any(|p| p.contains("no expected failure")),
            "{proposed:?}"
        );
        assert_eq!(
            problems_of("none yet; proposed: an L2 probe", Some("none")),
            Vec::<String>::new()
        );
        assert_eq!(
            problems_of(
                &format!("{test_asset}, held by website/ux/baseline.json"),
                Some("none")
            ),
            Vec::<String>::new()
        );
        let closed = record("closed", "S2", test_asset);
        assert_eq!(
            check_finding(&parse_finding("UXF-001.md", &closed).unwrap(), &|_| {
                Some("test(`L2 ${path} ${viewport.name}`, () => {});".to_owned())
            }),
            Vec::<String>::new()
        );
    }

    /// T28: only the first asset of a record was checked against its spec file.
    #[test]
    fn every_asset_of_a_record_names_a_test_its_file_declares() {
        let asset = "website/ux/specs/l2-layout.spec.ts :: L2 /ess/ desktop-1280; \
                     website/ux/specs/gone.spec.ts :: L3 a mobile-390 pointer; \
                     website/ux/specs/l2-layout.spec.ts :: L3 b mobile-390 pointer";
        let text = record("open", "S2", asset).replacen(
            "regression_asset:",
            "failure_pattern: gate 5\nregression_asset:",
            1,
        );
        let problems = check_finding(&parse_finding("UXF-001.md", &text).unwrap(), &|path| {
            (path == "website/ux/specs/l2-layout.spec.ts")
                .then(|| "test(`L2 ${path} ${viewport.name}`, () => {});".to_owned())
        });
        assert!(
            problems.iter().any(
                |p| p.contains("website/ux/specs/gone.spec.ts") && p.contains("does not exist")
            ),
            "{problems:?}"
        );
        assert!(
            problems
                .iter()
                .any(|p| p.contains("declares no test `L3 b mobile-390 pointer`")),
            "{problems:?}"
        );
    }

    const TWO_WORSE: &str = r#"{"scenarios": {"install/desktop-1280":
      {"screen_only.actions": 4, "glance_cost.p95": 2.5, "layout_shift.max": 0.02}}}"#;

    /// T29 of the R-03 report review: a decision id took every worse value of the update. It
    /// takes the worse values it lists, each as `<scenario> <metric> <value>` (T35), and no other;
    /// a whole `<scenario>` takes no worse value.
    #[test]
    fn a_decision_takes_only_the_values_it_lists() {
        let measured = round(TWO_WORSE);
        let refused = update(
            &baseline(BASE),
            &measured,
            Some(("D-2", &["install/desktop-1280 glance_cost.p95 2.5"])),
        )
        .unwrap_err();
        assert!(
            matches!(refused.as_slice(), [Breach::Worse { metric, .. }] if metric == "screen_only.actions"),
            "{refused:?}"
        );
        let (next, taken) = update(
            &baseline(BASE),
            &measured,
            Some((
                "D-2",
                &[
                    "install/desktop-1280 glance_cost.p95 2.5",
                    "install/desktop-1280 screen_only.actions 4",
                ],
            )),
        )
        .expect("a decision that lists both takes both");
        assert_eq!(taken.worse, 2);
        assert_eq!(
            next.scenarios["install/desktop-1280"]["screen_only.actions"]
                .decision
                .as_deref(),
            Some("D-2")
        );
        assert!(
            update(
                &baseline(BASE),
                &measured,
                Some(("D-2", &["install/desktop-1280"]))
            )
            .is_err(),
            "a whole scenario took worse values"
        );
        let lost = round(r#"{"scenarios": {"install/desktop-1280": {"screen_only.actions": 3}}}"#);
        assert!(
            update(
                &baseline(BASE),
                &lost,
                Some(("D-2", &["install/desktop-1280 glance_cost.p95"]))
            )
            .is_err(),
            "a decision dropped a value it does not list"
        );
        let shrunk =
            round(r#"{"lane": "gate", "scenarios": {"/a@d/light": {"layout.spills": 0}}}"#);
        assert!(
            update(
                &baseline(LANES),
                &shrunk,
                Some(("D-9", &["gate: /c@d/light"]))
            )
            .is_err(),
            "a decision dropped a lane scenario it does not list"
        );
    }

    /// T29: the ratchet did not read `decision.md`. The keys of decision `<id>` are the backticked
    /// list items of the section whose `## ` heading names the id in backticks.
    #[test]
    fn a_decision_is_read_from_its_section_of_the_decision_file() {
        let text = "# R-04 decision\n\n## Ratchet decision `D-4`: two pages leave the gate\n\n\
                    Prose that names `not/a key`.\n\n\
                    - `gate: /ess/docs/reference/diagnostics@desktop-1280/light`: moved to the slow lane\n\
                    - `/ess/@mobile-390/light axe.serious_critical_nodes`\n\n\
                    ## Ratchet decision `D-5`\n\n- `other/desktop-1280`\n";
        let decision = decision_of(text, "D-4").expect("D-4 is in the file");
        assert_eq!(
            decision.keys,
            BTreeSet::from([
                "gate: /ess/docs/reference/diagnostics@desktop-1280/light".to_owned(),
                "/ess/@mobile-390/light axe.serious_critical_nodes".to_owned(),
            ])
        );
        assert!(
            decision_of(text, "D-6").is_err(),
            "an id the file does not hold"
        );
        assert!(
            decision_of("## Ratchet decision `D-7`\n\nNo list.\n", "D-7").is_err(),
            "a decision that lists no key"
        );
    }

    /// A run of a merged round that ended with `status`, failing `failed` and passing `passed`.
    fn run_with(name: &str, status: &str, failed: &[&str], passed: &[&str]) -> String {
        let list = |items: &[&str]| {
            items
                .iter()
                .map(|item| format!("{item:?}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let unexpected: Vec<String> = failed
            .iter()
            .map(|test| format!("{test} (timedOut, expected passed)"))
            .collect();
        let unexpected: Vec<&str> = unexpected.iter().map(String::as_str).collect();
        format!(
            r#"{{"name": "{name}", "engine": {{"digest": "d", "commit": "c", "dirty": false}},
              "timing": {{"status": "{status}", "unexpected": [{}], "unexpected_tests": [{}],
              "passed": [{}]}}}}"#,
            list(&unexpected),
            list(failed),
            list(passed)
        )
    }

    /// T41 of the R-04 report review: the ratchet read neither a run's status nor its unexpected
    /// failures, so a run that failed or was cut judged as a pass on the values it recorded.
    #[test]
    fn a_run_that_did_not_pass_is_refused_unless_a_later_run_passed_what_it_failed() {
        let failed = run_with(
            "slow-l2-4",
            "failed",
            &["[l2-4] L2 /a m"],
            &["[l2-4] L2 /b m"],
        );
        let cut = run_with("gate-9", "timedout", &[], &[]);
        for (case, runs) in [
            ("failed", format!("[{failed}]")),
            ("timedout", format!("[{CLEAN_RUN}, {cut}]")),
            (
                "rerun earlier",
                format!(
                    "[{}, {failed}]",
                    run_with("rerun-a", "passed", &[], &["[l2-4] L2 /a m"])
                ),
            ),
        ] {
            let breaches = compare(&baseline(BASE), &merged(&runs));
            assert!(
                breaches
                    .iter()
                    .any(|breach| matches!(breach, Breach::RunNotPassed { .. })),
                "{case}: {breaches:?}"
            );
            assert!(
                update(
                    &baseline(BASE),
                    &merged(&runs),
                    Some(("D-1", &["install/desktop-1280"]))
                )
                .is_err(),
                "{case}"
            );
        }
        let rerun = run_with("rerun-a", "passed", &[], &["[l2-4] L2 /a m"]);
        assert_eq!(
            compare(&baseline(BASE), &merged(&format!("[{failed}, {rerun}]"))),
            vec![],
            "a failed test a later run passed"
        );
        let gate = cut.replacen(r#""engine""#, r#""lane": "gate", "engine""#, 1);
        assert_eq!(
            compare(&baseline(BASE), &merged(&format!("[{CLEAN_RUN}, {gate}]"))),
            vec![],
            "a gate run a full round reports for its time is not its measurement"
        );
    }

    /// T41: a gate run's own `round.json` says how the run ended at the top.
    #[test]
    fn a_round_of_one_run_with_an_unexpected_failure_is_refused() {
        let measured = round(
            r#"{"lane": "full", "timing": {"status": "failed",
              "unexpected": ["[l1] L1 x (failed, expected passed)"], "unexpected_tests": ["[l1] L1 x"]},
              "scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        );
        let breaches = compare(&baseline(BASE), &measured);
        assert!(
            matches!(breaches.as_slice(), [Breach::RunNotPassed { run, .. }] if run == "the round"),
            "{breaches:?}"
        );
        let passed = round(
            r#"{"timing": {"status": "passed"}, "scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        );
        assert_eq!(compare(&baseline(BASE), &passed), vec![]);
    }

    /// T35 of the R-04 report review: a decision listed keys, so it took any later worse value of
    /// a listed key. A list item `<scenario> <metric> <value>` takes that value and no other; a
    /// key without a value, or a whole scenario, takes no worse value.
    #[test]
    fn a_decision_takes_a_worse_value_only_at_the_value_it_names() {
        let measured = round(TWO_WORSE);
        let (_, taken) = update(
            &baseline(BASE),
            &measured,
            Some((
                "D-3",
                &[
                    "install/desktop-1280 glance_cost.p95 2.5",
                    "install/desktop-1280 screen_only.actions 4",
                ],
            )),
        )
        .expect("a decision that names both values takes both");
        assert_eq!(taken.worse, 2);
        for keys in [
            &[
                "install/desktop-1280 glance_cost.p95 2.4",
                "install/desktop-1280 screen_only.actions 4",
            ][..],
            &[
                "install/desktop-1280 glance_cost.p95",
                "install/desktop-1280 screen_only.actions",
            ][..],
            &["install/desktop-1280"][..],
        ] {
            assert!(
                update(&baseline(BASE), &measured, Some(("D-3", keys))).is_err(),
                "{keys:?} took a value it does not name"
            );
        }
    }

    /// T35 with T27: a decision takes a rerun's value only when it names scenario, metric and
    /// that value; else the worse of the runs is held.
    #[test]
    fn a_rerun_is_held_at_its_worse_value_unless_the_decision_names_the_value_it_takes() {
        let keyed = |key: &str| Decision {
            id: "R-05-shift".into(),
            keys: BTreeSet::from([key.to_owned()]),
        };
        assert_eq!(
            compare_with(
                &baseline(SHIFT),
                &round(RERUN),
                Some(&keyed("/p@m/dark layout_shift.load 0"))
            ),
            vec![]
        );
        for key in [
            "/p@m/dark layout_shift.load",
            "/p@m/dark",
            "/p@m/dark layout_shift.load 0.01",
        ] {
            let breaches = compare_with(&baseline(SHIFT), &round(RERUN), Some(&keyed(key)));
            assert!(
                matches!(breaches.as_slice(), [Breach::Worse { value, .. }] if (*value - 0.0253).abs() < 1e-12),
                "{key}: {breaches:?}"
            );
        }
    }

    /// T36 of the R-04 report review: four values rose against R-03 inside their tolerance and
    /// no line named them.
    #[test]
    fn a_worsening_inside_its_tolerance_is_named() {
        let within = round(
            r#"{"scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 2.09, "layout_shift.max": 0.02}}}"#,
        );
        assert_eq!(compare(&baseline(BASE), &within), vec![]);
        let notes = within_tolerance(&baseline(BASE), &within, None);
        assert_eq!(notes.len(), 1, "{notes:?}");
        assert!(
            notes[0].contains("install/desktop-1280 glance_cost.p95")
                && notes[0].contains("2.09")
                && notes[0].contains("inside its tolerance"),
            "{notes:?}"
        );
        let equal = round(
            r#"{"scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        );
        assert_eq!(
            within_tolerance(&baseline(BASE), &equal, None),
            Vec::<String>::new()
        );
    }

    /// T47 (and T37) of the R-05 report review: a round that names no run and no `timing`, or a run
    /// without its `timing`, was not judged, so a `round.json` written without them was taken.
    #[test]
    fn a_round_or_run_that_does_not_say_how_it_ended_is_refused() {
        let silent: Round = serde_json::from_str(
            r#"{"coverage": {"unmeasured": []},
              "engine": {"digest": "d", "commit": "c", "dirty": false},
              "scenarios": {"install/desktop-1280":
              {"screen_only.actions": 3, "glance_cost.p95": 2.0, "layout_shift.max": 0.02}}}"#,
        )
        .expect("canned round parses");
        let breaches = compare(&baseline(BASE), &silent);
        assert!(
            matches!(breaches.as_slice(), [Breach::RunNotPassed { run, .. }] if run == "the round"),
            "no runs, no timing: {breaches:?}"
        );
        let untimed = merged(
            r#"[{"name": "slow-l2-2", "engine": {"digest": "d", "commit": "c", "dirty": false}}]"#,
        );
        let breaches = compare(&baseline(BASE), &untimed);
        assert!(
            matches!(breaches.as_slice(), [Breach::RunNotPassed { run, .. }] if run == "slow-l2-2"),
            "a run without timing: {breaches:?}"
        );
    }

    /// T46 of the R-05 report review: `passed` held the known failures too, and the rescue of an
    /// earlier failure read it. The reporter's `passed` now holds only tests that passed, so a test
    /// a later run ended as its open finding states (`known_failures`) still rescues it.
    #[test]
    fn a_later_run_that_ended_a_test_as_its_finding_states_rescues_it() {
        let failed = run_with("slow-l2-1", "failed", &["[l2-1] L2 /ess/ m"], &[]);
        let known = r#"{"name": "rerun-a", "engine": {"digest": "d", "commit": "c", "dirty": false},
              "timing": {"status": "passed", "known_failures": ["[l2-1] L2 /ess/ m"]}}"#;
        assert_eq!(
            compare(&baseline(BASE), &merged(&format!("[{failed}, {known}]"))),
            vec![]
        );
    }

    /// T48 of the R-05 report review: the notes of a worsening inside its tolerance were printed
    /// only when the ratchet passed, so a failing run named none of them.
    #[test]
    fn a_failing_run_prints_its_tolerance_notes() {
        let notes = "ux ratchet: note: install/desktop-1280 glance_cost.p95 2.09 is worse \
                     than the baseline 2 inside its tolerance\n";
        let message = failure_message(
            notes,
            &[Breach::Missing {
                scenario: "a/b".into(),
                metric: "m".into(),
            }],
            &[],
            false,
        );
        assert!(
            message.contains("note: install/desktop-1280 glance_cost.p95 2.09"),
            "{message}"
        );
        assert!(message.contains("a/b m: lost coverage"), "{message}");
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
