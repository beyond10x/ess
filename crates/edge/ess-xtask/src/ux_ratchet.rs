//! The ratchet of the documentation website's UI iteration loop, and the check of its findings.
//!
//! `website/ux/` measures the site: a Playwright run writes `round.json`, one number per scenario
//! and metric key, every metric lower-is-better. This module decides whether those numbers may land.
//! It fails on four things:
//!
//! 1. a value worse than `website/ux/baseline.json` beyond the metric's tolerance;
//! 2. a value better than the baseline beyond the metric's tolerance that was not taken into it: an
//!    improvement becomes the baseline in the same change;
//! 3. a value the baseline holds and the round does not (and a round value the baseline does not
//!    hold);
//! 4. a baseline made worse than the one on `main` without a decision id on the worsened entry.
//!
//! The Playwright specs only measure; the decision is here because a checker in this repository is
//! Rust. The same goes for the findings rule: no closed S1 or S2 record under `website/ux/findings/`
//! without a regression asset that exists.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt::{self, Write as _};
use std::fs;
use std::path::Path;
use std::process::Command;

/// The baseline, relative to the repository root.
pub(crate) const BASELINE: &str = "website/ux/baseline.json";

/// The finding records, relative to the repository root.
pub(crate) const FINDINGS: &str = "website/ux/findings";

/// Comparison slack for floating-point values read from JSON.
const EPSILON: f64 = 1e-9;

/// How far a value may move before it counts as worse or as an improvement.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum Tolerance {
    /// A fixed amount in the metric's unit.
    Absolute(f64),
    /// A share of the baseline value, `0.05` for 5 %.
    Relative(f64),
}

impl Tolerance {
    fn allowance(self, base: f64) -> f64 {
        match self {
            Tolerance::Absolute(amount) => amount,
            Tolerance::Relative(share) => base.abs() * share,
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
    /// Scenario, then metric key, then the baselined value.
    pub scenarios: BTreeMap<String, BTreeMap<String, Entry>>,
    /// Tolerance classes by name.
    pub tolerances: BTreeMap<String, Tolerance>,
}

/// The part of `round.json` the ratchet reads; every other field is the report's.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(crate) struct Round {
    /// Scenario, then metric key, then the measured value.
    pub scenarios: BTreeMap<String, BTreeMap<String, f64>>,
}

/// One reason the ratchet fails.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Breach {
    /// The round is worse than the baseline beyond the tolerance.
    Worse {
        scenario: String,
        metric: String,
        baseline: f64,
        value: f64,
    },
    /// The round is better than the baseline beyond the tolerance and the baseline was not moved.
    ImprovementNotTaken {
        scenario: String,
        metric: String,
        baseline: f64,
        value: f64,
    },
    /// The baseline holds a value the round does not.
    Missing { scenario: String, metric: String },
    /// The round holds a value the baseline does not.
    Unbaselined {
        scenario: String,
        metric: String,
        value: f64,
    },
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
    /// An entry on `main` is gone from the baseline.
    BaselineDropped { scenario: String, metric: String },
}

impl fmt::Display for Breach {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Breach::Worse {
                scenario,
                metric,
                baseline,
                value,
            } => {
                write!(
                    f,
                    "{scenario} {metric}: {value} is worse than the baseline {baseline}"
                )
            }
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
            Breach::Missing { scenario, metric } => {
                write!(
                    f,
                    "{scenario} {metric}: the baseline holds it and the round measured nothing"
                )
            }
            Breach::Unbaselined {
                scenario,
                metric,
                value,
            } => write!(
                f,
                "{scenario} {metric}: the round measured {value} and the baseline holds no value \
                 (--update adds it)"
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
            Breach::BaselineDropped { scenario, metric } => {
                write!(
                    f,
                    "{scenario} {metric}: main's baseline holds it and this baseline does not"
                )
            }
        }
    }
}

/// The tolerance class a new metric gets: the skill's counts 0, steps 5 %, layout shift 0.01.
pub(crate) fn tolerance_class(metric: &str) -> &'static str {
    if metric.starts_with("layout_shift") {
        "layout_shift"
    } else if metric.starts_with("glance_cost")
        || metric.starts_with("reading_load")
        || metric.contains("time_to_act_steps")
    {
        "steps"
    } else {
        "count"
    }
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

/// Compares a round with the baseline: worse, improved and not taken, missing, unbaselined.
pub(crate) fn compare(baseline: &Baseline, round: &Round) -> Vec<Breach> {
    let mut breaches = Vec::new();
    for (scenario, entries) in &baseline.scenarios {
        let measured = round.scenarios.get(scenario);
        for (metric, entry) in entries {
            let tolerance = match tolerance_of(baseline, scenario, metric, entry) {
                Ok(tolerance) => tolerance,
                Err(breach) => {
                    breaches.push(breach);
                    continue;
                }
            };
            let Some(&value) = measured.and_then(|values| values.get(metric)) else {
                breaches.push(Breach::Missing {
                    scenario: scenario.clone(),
                    metric: metric.clone(),
                });
                continue;
            };
            let allowance = tolerance.allowance(entry.value);
            if value > entry.value + allowance + EPSILON {
                breaches.push(Breach::Worse {
                    scenario: scenario.clone(),
                    metric: metric.clone(),
                    baseline: entry.value,
                    value,
                });
            } else if value < entry.value - allowance - EPSILON {
                breaches.push(Breach::ImprovementNotTaken {
                    scenario: scenario.clone(),
                    metric: metric.clone(),
                    baseline: entry.value,
                    value,
                });
            }
        }
    }
    for (scenario, values) in &round.scenarios {
        for (metric, &value) in values {
            let held = baseline
                .scenarios
                .get(scenario)
                .is_some_and(|entries| entries.contains_key(metric));
            if !held {
                breaches.push(Breach::Unbaselined {
                    scenario: scenario.clone(),
                    metric: metric.clone(),
                    value,
                });
            }
        }
    }
    breaches
}

/// Compares this baseline with `main`'s: a worse entry needs a decision id `main`'s entry lacks.
pub(crate) fn compare_with_main(main: &Baseline, current: &Baseline) -> Vec<Breach> {
    let mut breaches = Vec::new();
    for (scenario, entries) in &main.scenarios {
        for (metric, old) in entries {
            let Some(new) = current
                .scenarios
                .get(scenario)
                .and_then(|entries| entries.get(metric))
            else {
                breaches.push(Breach::BaselineDropped {
                    scenario: scenario.clone(),
                    metric: metric.clone(),
                });
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
            let decided = new
                .decision
                .as_ref()
                .is_some_and(|id| !id.trim().is_empty() && old.decision.as_ref() != Some(id));
            if worse && !decided {
                breaches.push(Breach::BaselineWorsened {
                    scenario: scenario.clone(),
                    metric: metric.clone(),
                    main: old.value,
                    value: new.value,
                });
            }
        }
    }
    breaches
}

/// Takes a round into the baseline: improvements always, new keys always, worse values only under
/// a decision id, which is written beside each of them. A missing value is never taken.
pub(crate) fn update(
    baseline: &Baseline,
    round: &Round,
    decision: Option<&str>,
) -> Result<Baseline, Vec<Breach>> {
    let mut next = baseline.clone();
    let mut refused = Vec::new();
    for breach in compare(baseline, round) {
        match breach {
            Breach::ImprovementNotTaken {
                scenario,
                metric,
                value,
                ..
            } => {
                if let Some(entry) = next
                    .scenarios
                    .get_mut(&scenario)
                    .and_then(|entries| entries.get_mut(&metric))
                {
                    entry.value = value;
                }
            }
            Breach::Unbaselined {
                scenario,
                metric,
                value,
            } => {
                next.scenarios.entry(scenario).or_default().insert(
                    metric.clone(),
                    Entry {
                        value,
                        tolerance: tolerance_class(&metric).to_owned(),
                        decision: None,
                    },
                );
            }
            Breach::Worse {
                scenario,
                metric,
                value,
                baseline: old,
            } => match decision {
                Some(id) if !id.trim().is_empty() => {
                    if let Some(entry) = next
                        .scenarios
                        .get_mut(&scenario)
                        .and_then(|entries| entries.get_mut(&metric))
                    {
                        entry.value = value;
                        entry.decision = Some(id.to_owned());
                    }
                }
                _ => refused.push(Breach::Worse {
                    scenario,
                    metric,
                    baseline: old,
                    value,
                }),
            },
            other => refused.push(other),
        }
    }
    if refused.is_empty() {
        Ok(next)
    } else {
        Err(refused)
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

/// The path part of a regression asset, `website/ux/specs/x.spec.ts › title` to its file.
fn asset_path(asset: &str) -> Option<&str> {
    let first = asset.split_whitespace().next()?;
    let path = first.split("::").next()?.trim_end_matches(',');
    (path.contains('/') && !path.contains("://")).then_some(path)
}

/// Checks one record. `exists` answers whether a repository-relative path exists.
pub(crate) fn check_finding(finding: &Finding, exists: &dyn Fn(&str) -> bool) -> Vec<String> {
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
    let no_asset = asset.is_empty() || asset.eq_ignore_ascii_case("none") || asset == "-";
    if field("status") == "closed" && matches!(field("severity"), "S1" | "S2") && no_asset {
        problems.push(format!(
            "{file}: a closed {} finding needs a regression asset that failed before the fix",
            field("severity")
        ));
    }
    if let Some(path) = asset_path(asset) {
        if !exists(path) {
            problems.push(format!("{file}: regression asset `{path}` does not exist"));
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
    let exists = |path: &str| root.join(path).exists();
    let mut problems = Vec::new();
    for file in &files {
        let text =
            fs::read_to_string(directory.join(file)).with_context(|| format!("reading {file}"))?;
        match parse_finding(file, &text) {
            Ok(finding) => problems.extend(check_finding(&finding, &exists)),
            Err(problem) => problems.push(problem),
        }
    }
    Ok(problems)
}

/// `main`'s baseline through git, or `None` when `main` has none yet.
fn main_baseline(root: &Path) -> Result<Option<Baseline>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["show", &format!("origin/main:{BASELINE}")])
        .output()
        .context("running git show for main's baseline")?;
    if !output.status.success() {
        return Ok(None);
    }
    serde_json::from_slice(&output.stdout)
        .map(Some)
        .context("parsing main's baseline")
}

/// The command: compare `round` with the baseline and with `main`'s, check the findings; with
/// `update`, take the round into the baseline first.
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
    let measured: Round = serde_json::from_str(
        &fs::read_to_string(round).with_context(|| format!("reading {}", round.display()))?,
    )
    .with_context(|| format!("parsing {}", round.display()))?;
    if update_baseline {
        baseline = update(&baseline, &measured, decision).map_err(|refused| {
            anyhow::anyhow!("the baseline was not updated:\n{}", lines(&refused))
        })?;
        let mut text = serde_json::to_string_pretty(&baseline)?;
        text.push('\n');
        fs::write(&baseline_path, text)
            .with_context(|| format!("writing {}", baseline_path.display()))?;
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
        bail!(message.trim_end().to_owned());
    }
    let keys: usize = baseline.scenarios.values().map(BTreeMap::len).sum();
    Ok(format!(
        "ux ratchet: {keys} value(s) in {} scenario(s) hold the baseline\n",
        baseline.scenarios.len()
    ))
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

    fn round(json: &str) -> Round {
        serde_json::from_str(json).expect("canned round parses")
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
    fn a_missing_value_and_an_unbaselined_value_fail() {
        let measured = round(
            r#"{"scenarios": {"install/desktop-1280": {"screen_only.actions": 3, "glance_cost.p95": 2.0},
              "limitations/mobile-390": {"screen_only.actions": 5}}}"#,
        );
        let breaches = compare(&baseline(BASE), &measured);
        assert!(breaches.contains(&Breach::Missing {
            scenario: "install/desktop-1280".into(),
            metric: "layout_shift.max".into()
        }));
        assert!(breaches.contains(&Breach::Unbaselined {
            scenario: "limitations/mobile-390".into(),
            metric: "screen_only.actions".into(),
            value: 5.0
        }));
        assert_eq!(breaches.len(), 2, "{breaches:?}");
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
    fn dropping_an_entry_main_holds_fails() {
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
        let next = update(&baseline(BASE), &measured, Some("D-7"))
            .expect("a decision takes the worse value");
        let entries = &next.scenarios["install/desktop-1280"];
        assert_eq!(entries["screen_only.actions"].value, 2.0);
        assert_eq!(entries["glance_cost.p95"].decision.as_deref(), Some("D-7"));
        assert_eq!(entries["reading_load.max"].tolerance, "steps");
        assert_eq!(compare(&next, &measured), vec![]);
    }

    #[test]
    fn update_refuses_a_missing_value() {
        let measured =
            round(r#"{"scenarios": {"install/desktop-1280": {"screen_only.actions": 3}}}"#);
        assert!(update(&baseline(BASE), &measured, Some("D-1")).is_err());
    }

    #[test]
    fn counts_are_written_without_a_fraction() {
        let text = serde_json::to_string(&baseline(BASE)).unwrap();
        assert!(text.contains(r#""value":3,"#), "{text}");
        assert!(text.contains(r#""value":0.02,"#), "{text}");
    }

    #[test]
    fn new_keys_get_the_skills_tolerance_classes() {
        assert_eq!(tolerance_class("screen_only.actions"), "count");
        assert_eq!(
            tolerance_class("screen_only.time_to_act_steps.max"),
            "steps"
        );
        assert_eq!(tolerance_class("glance_cost.median"), "steps");
        assert_eq!(tolerance_class("layout_shift.max"), "layout_shift");
        assert_eq!(tolerance_class("axe.serious_critical_nodes"), "count");
    }

    fn record(status: &str, severity: &str, asset: &str) -> String {
        format!(
            "---\nid: UXF-001\ntitle: t\nclass: layout\nseverity: {severity}\nmetric: m\nstatus: {status}\n\
             found_by: L2\nevidence_recording: none\nevidence_screenshot: s.png\nevidence_lines: l\n\
             regression_asset: {asset}\n---\n\n# UXF-001\n"
        )
    }

    #[test]
    fn a_closed_s1_or_s2_record_without_an_asset_fails() {
        for severity in ["S1", "S2"] {
            let finding = parse_finding("UXF-001.md", &record("closed", severity, "none")).unwrap();
            let problems = check_finding(&finding, &|_| true);
            assert!(
                problems
                    .iter()
                    .any(|problem| problem.contains("needs a regression asset")),
                "{problems:?}"
            );
        }
        let s3 = parse_finding("UXF-001.md", &record("closed", "S3", "none")).unwrap();
        assert_eq!(check_finding(&s3, &|_| true), Vec::<String>::new());
        let open = parse_finding("UXF-001.md", &record("open", "S1", "none")).unwrap();
        assert_eq!(check_finding(&open, &|_| true), Vec::<String>::new());
    }

    #[test]
    fn an_asset_that_names_a_missing_file_fails() {
        let finding = parse_finding(
            "UXF-001.md",
            &record("closed", "S2", "website/ux/specs/gone.spec.ts › gate 1"),
        )
        .unwrap();
        let problems = check_finding(&finding, &|path| path != "website/ux/specs/gone.spec.ts");
        assert!(
            problems
                .iter()
                .any(|problem| problem.contains("does not exist")),
            "{problems:?}"
        );
        assert_eq!(check_finding(&finding, &|_| true), Vec::<String>::new());
    }

    #[test]
    fn a_record_without_its_header_fields_fails() {
        let finding = parse_finding("UXF-002.md", "---\nid: UXF-003\nclass: vibes\n---\n").unwrap();
        let problems = check_finding(&finding, &|_| true);
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
