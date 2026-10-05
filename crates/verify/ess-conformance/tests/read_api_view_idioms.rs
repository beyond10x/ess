//! Read-API view idioms (`docs/design/read-api-view-idioms.md`; beyond10x/ess#441, #442, #443,
//! #444, #446, #447).
//!
//! Each of those requests was declined with an idiom: a construct ESS already has that states the
//! need. The note documents each idiom in its own `## ` section, the directory
//! `docs/design/read-api-view-idioms.example/` holds a validated model of it in its own domain, and
//! the cases here hold the two together: a section that stops stating its idiom fails, an example
//! that stops validating or synthesizing fails, and a refusal the note quotes that changes its
//! message fails. The held-row idiom of #442 is the fixture
//! `tests/fixtures/latest-correlated-record.yaml`, because its suite is decided by an interpreter
//! run rather than read off the example.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::{compile, diagnose};
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Source as AuthoredSource};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioId};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::view::AggregateFunction;
use serde_json::Value;

const NOTE: &str = "docs/design/read-api-view-idioms.md";
const EXAMPLE: &str = "docs/design/read-api-view-idioms.example";
const GUIDE: &str = "website/docs/guides/specify/aggregate-views.md";
const CORRELATED: &str =
    "crates/verify/ess-conformance/tests/fixtures/latest-correlated-record.yaml";

// ---- the note ----------------------------------------------------------------------------------

/// The repository root this crate sits three levels under.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("{relative} is readable: {error}"))
}

/// Runs of whitespace as one space, so a phrase the page wraps across two lines still matches.
fn normalize(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The body of the `## ` section `heading` names, up to the next `## ` heading or the end of the
/// page, split the way `crates/edge/ess-xtask/tests/d2_constraint_home_adversary.rs` splits one.
/// `None` where the page has no line that is exactly that heading.
fn section<'p>(page: &'p str, heading: &str) -> Option<&'p str> {
    let marker = format!("\n{heading}\n");
    let body = page.split(marker.as_str()).nth(1)?;
    body.split("\n## ").next()
}

/// The section `heading` of `path`, failing by name where the heading or any phrase is missing.
fn section_states(path: &str, heading: &str, phrases: &[&str]) -> String {
    let page = read(path);
    let body = section(&page, heading)
        .unwrap_or_else(|| panic!("{path} has no heading `{heading}`"))
        .to_owned();
    let normalized = normalize(&body);
    let missing: Vec<&&str> = phrases
        .iter()
        .filter(|phrase| !normalized.contains(&normalize(phrase)))
        .collect();
    assert!(
        missing.is_empty(),
        "{path} `{heading}` does not state {missing:?}"
    );
    body
}

// ---- the example -------------------------------------------------------------------------------

/// Every `.yaml` file of the example directory, by name, in name order.
fn example_files() -> Vec<(String, String)> {
    let directory = root().join(EXAMPLE);
    let mut files: Vec<(String, String)> = std::fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{EXAMPLE} is readable: {error}"))
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "yaml")
        })
        .map(|path| {
            let name = path
                .file_name()
                .expect("a file name")
                .to_string_lossy()
                .into_owned();
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("{name} is readable: {error}"));
            (name, text)
        })
        .collect();
    files.sort();
    files
}

/// The files as one specification, compiled; or every refusal, with the stable code each compiles
/// to, as one text.
fn assemble(files: &[(String, String)]) -> Result<EssIr, String> {
    let mut parsed = Vec::new();
    for (name, text) in files {
        let raw = RawSpecFile::parse(text).map_err(|error| format!("{name}: {error}"))?;
        parsed.push((Source::new(name.as_str()), raw));
    }
    let specification = Specification::assemble(parsed).map_err(|errors| {
        let codes: Vec<String> = diagnose(&errors, &SourceMap::new())
            .as_slice()
            .iter()
            .map(|diagnostic| format!("{} {diagnostic}", diagnostic.code))
            .collect();
        format!("{errors}\n{}", codes.join("\n"))
    })?;
    compile(&specification, &SourceMap::new()).map_err(|diagnostics| diagnostics.to_string())
}

fn example() -> EssIr {
    assemble(&example_files()).unwrap_or_else(|error| panic!("{EXAMPLE} validates:\n{error}"))
}

/// The example with the first `from` in `file` replaced by `to`; fails where `from` is not there,
/// so a case cannot pass on a change it never made.
fn changed(file: &str, from: &str, to: &str) -> Vec<(String, String)> {
    let mut files = example_files();
    let (_, text) = files
        .iter_mut()
        .find(|(name, _)| name == file)
        .unwrap_or_else(|| panic!("{EXAMPLE} has no {file}"));
    assert!(text.contains(from), "{file} holds `{from}`");
    *text = text.replacen(from, to, 1);
    files
}

/// The example with `addition` appended to `file`.
fn appended(file: &str, addition: &str) -> Vec<(String, String)> {
    let mut files = example_files();
    let (_, text) = files
        .iter_mut()
        .find(|(name, _)| name == file)
        .unwrap_or_else(|| panic!("{EXAMPLE} has no {file}"));
    text.push_str(addition);
    files
}

fn refused(files: &[(String, String)]) -> String {
    match assemble(files) {
        Ok(_) => panic!("the changed example must be refused"),
        Err(error) => error,
    }
}

fn file(name: &str) -> String {
    example_files()
        .into_iter()
        .find(|(file, _)| file == name)
        .unwrap_or_else(|| panic!("{EXAMPLE} has no {name}"))
        .1
}

fn view_names(ir: &EssIr, domain: &str) -> BTreeSet<String> {
    let prefix = format!("{domain}.");
    ir.views()
        .keys()
        .map(ToString::to_string)
        .filter(|name| name.starts_with(&prefix))
        .collect()
}

fn holds_views(ir: &EssIr, domain: &str, views: &[&str]) {
    assert!(
        ir.domains().keys().any(|name| name.to_string() == domain),
        "{EXAMPLE} declares the domain {domain}"
    );
    let declared = view_names(ir, domain);
    for view in views {
        assert!(
            declared.contains(*view),
            "{EXAMPLE} holds {view}: {declared:?}"
        );
    }
}

fn view<'i>(ir: &'i EssIr, name: &str) -> &'i ess_compiler::ir::ResolvedView {
    ir.views()
        .iter()
        .find(|(declared, _)| declared.to_string() == name)
        .unwrap_or_else(|| panic!("no view {name}"))
        .1
}

// ---- synthesis and runs ------------------------------------------------------------------------

fn scenario_ids(synthesis: &Synthesis) -> BTreeSet<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

/// Every refusal as `<code> <scenario or subject>: <message>`.
fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| {
            let about = refusal
                .scenario
                .as_ref()
                .map_or_else(|| format!("{:?}", refusal.subject), ToString::to_string);
            format!("{} {about}: {}", refusal.code(), refusal.cause)
        })
        .collect()
}

/// `suite` against the interpreter executing `model`, by scenario.
fn run(suite: &ConformanceSuite, model: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn failed(statuses: &BTreeMap<String, Status>) -> BTreeSet<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.clone())
        .collect()
}

/// The steps of one scenario as the suite document writes them.
fn steps(synthesis: &Synthesis, id: &str) -> Vec<Value> {
    let scenario = &synthesis.suite.scenarios[&ScenarioId::parse(id).unwrap()];
    match serde_json::to_value(&scenario.steps).expect("steps serialize") {
        Value::Array(steps) => steps,
        other => panic!("steps are a list: {other}"),
    }
}

// ---- beyond10x/ess#441: derived values over aggregates ------------------------------------------

#[test]
fn derived_values_section_states_the_idiom() {
    section_states(
        NOTE,
        "## Derived values over aggregates are the consumer's",
        &[
            "conditional measures",
            "one row",
            "the consumer derives",
            "rounding",
            "zero denominator",
            "decision 3",
            "stored unit",
            "`param.limit_s * 1000`",
            "`ratio:`",
        ],
    );
}

#[test]
fn aggregate_views_guide_points_at_the_idioms_note() {
    let body = section_states(
        GUIDE,
        "## What an aggregate view does not compute",
        &["ratio", "difference"],
    );
    let targets: Vec<&str> = body
        .split("](")
        .skip(1)
        .filter_map(|rest| rest.split(')').next())
        .collect();
    assert!(
        targets
            .iter()
            .any(|target| target.ends_with("docs/design/read-api-view-idioms.md")),
        "{GUIDE} links the idioms note: {targets:?}"
    );
}

#[test]
fn derived_value_example_validates() {
    let ir = example();
    assert!(
        file("ratio.yaml")
            .lines()
            .any(|line| line == "domain: idioms.ratio"),
        "ratio.yaml declares the domain idioms.ratio"
    );
    holds_views(&ir, "idioms.ratio", &["idioms.ratio.QueueOutcomes"]);
    let outcomes = view(&ir, "idioms.ratio.QueueOutcomes");
    let aggregation = outcomes
        .aggregation
        .as_ref()
        .expect("QueueOutcomes is an aggregate view");
    assert_eq!(aggregation.group_by, ["queue_id"], "one row per queue");
    let queued = &aggregation.functions["queued"];
    assert_eq!(queued.function, AggregateFunction::Count);
    assert!(queued.r#where.is_none(), "`queued` counts every call");
    let abandoned = &aggregation.functions["abandoned"];
    assert_eq!(abandoned.function, AggregateFunction::Count);
    assert_eq!(
        abandoned
            .r#where
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("abandoned == true"),
        "`abandoned` counts the abandoned calls of the same row"
    );
}

#[test]
fn derived_view_field_stays_refused() {
    const MEASURE: &str = "aggregate: {count: {}, where: abandoned == true}}";
    let divided = refused(&changed(
        "ratio.yaml",
        MEASURE,
        "aggregate: {divide: [abandoned, queued]}}",
    ));
    assert!(
        divided.contains(
            "unknown variant `divide`, expected one of `count`, `count_distinct`, `sum`, `min`, \
             `max`, `avg`"
        ),
        "{divided}"
    );
    let computed = refused(&changed(
        "ratio.yaml",
        MEASURE,
        &format!("{MEASURE}\n      - {{name: rate, type: Decimal}}"),
    ));
    for message in [
        "`idioms.ratio.Call` has no field `rate`, so `idioms.ratio.QueueOutcomes` promises an \
         observation nothing produces",
        "`rate` is neither an aggregate nor a group key, so a row has no single value for it",
    ] {
        assert!(computed.contains(message), "{message}:\n{computed}");
    }
}

#[test]
fn parameter_in_stored_unit_validates() {
    const FILTER: &str = "filter: duration_ms > param.limit_ms + 1000";
    assert!(
        file("ratio.yaml").contains(FILTER),
        "LongCalls reads {FILTER}"
    );
    let ir = example();
    let long = view(&ir, "idioms.ratio.LongCalls");
    let params: Vec<&str> = long
        .params
        .iter()
        .map(|param| param.name.as_str())
        .collect();
    assert_eq!(
        params,
        ["limit_ms"],
        "the limit is declared in milliseconds"
    );
    assert!(long.filter.is_some(), "LongCalls is filtered");
    // A scale on the parameter is read as text, so the comparison does not type.
    let scaled = refused(&changed(
        "ratio.yaml",
        "param.limit_ms + 1000",
        "param.limit_s * 1000",
    ));
    assert!(
        scaled.contains("[type_mismatch]")
            && scaled.contains("Text literal `param.limit_s * 1000`"),
        "{scaled}"
    );
}

// ---- beyond10x/ess#442: the latest earlier row is a held row ------------------------------------

fn correlated(text: &str) -> EssIr {
    assemble(&[("latest-correlated-record.yaml".to_owned(), text.to_owned())])
        .unwrap_or_else(|error| panic!("{CORRELATED} validates:\n{error}"))
}

const CORRELATED_BRANCHES: [&str; 7] = [
    "idioms.correlated.Begin/outcome/opened",
    "idioms.correlated.Begin/outcome/superseded",
    "idioms.correlated.End/outcome/carried",
    "idioms.correlated.End/outcome/ambiguous",
    "idioms.correlated.End/outcome/unmatched",
    "idioms.correlated.End/outcome/clamped",
    "idioms.correlated.End/outcome/derived",
];

#[test]
fn latest_correlated_record_idiom_validates_and_synthesizes() {
    let synthesis = synthesize(&correlated(&read(CORRELATED)));
    assert_eq!(
        refusals(&synthesis),
        Vec::<String>::new(),
        "no branch is refused"
    );
    let ids = scenario_ids(&synthesis);
    for branch in CORRELATED_BRANCHES {
        assert!(ids.contains(branch), "{branch} is synthesized: {ids:#?}");
    }
}

/// The superseding begin, which replaces the held row's instant.
const KEEPS_LATEST: &str = "        updates: idioms.correlated.OpenSpan
        instance: span_key
        sets: {customer: input.customer, session: input.session, began_at: input.at}";

#[test]
fn latest_correlated_record_idiom_runs_interpreted() {
    let text = read(CORRELATED);
    let suite = synthesize(&correlated(&text)).suite;
    let healthy = run(&suite, correlated(&text));
    assert_eq!(healthy.len(), CORRELATED_BRANCHES.len(), "{healthy:?}");
    assert_eq!(failed(&healthy), BTreeSet::new(), "{healthy:?}");
    // A target that keeps the first begin of a key: the update leaves `began_at` as it was.
    assert!(text.contains(KEEPS_LATEST), "the superseding begin");
    let keeps_first = text.replacen(
        KEEPS_LATEST,
        "        updates: idioms.correlated.OpenSpan
        instance: span_key
        sets: {customer: input.customer, session: input.session}",
        1,
    );
    let failures = failed(&run(&suite, correlated(&keeps_first)));
    assert!(
        failures.contains("idioms.correlated.Begin/outcome/superseded"),
        "a target keeping the first begin fails: {failures:?}"
    );
}

#[test]
fn latest_earlier_row_section_states_the_idiom() {
    section_states(
        NOTE,
        "## The latest earlier row is a held row",
        &[
            "one row per correlation key",
            "create-or-update",
            "`count: {gt: 1}`",
            "`count: {eq: 0}`",
            "`forall`",
            "no row order is chosen",
            "`ended_at - began_at`",
            CORRELATED,
        ],
    );
    assert!(
        root().join(CORRELATED).is_file(),
        "the section's fixture {CORRELATED} exists"
    );
}

// ---- beyond10x/ess#443: a total maintained per key is a grouped view -----------------------------

const FOLD_UNFILTERED: [&str; 2] = ["idioms.fold.TotalByLabel", "idioms.fold.TotalBySession"];
const FOLD_FILTERED: [&str; 2] = [
    "idioms.fold.NonZeroTotalByLabel",
    "idioms.fold.NonZeroTotalBySession",
];

#[test]
fn aggregate_fold_idiom_validates() {
    let ir = example();
    holds_views(
        &ir,
        "idioms.fold",
        &[FOLD_UNFILTERED.as_slice(), FOLD_FILTERED.as_slice()].concat(),
    );
    assert!(
        ir.entities()
            .keys()
            .any(|name| name.to_string() == "idioms.fold.SessionEnd"),
        "idioms.fold holds SessionEnd"
    );
    for name in FOLD_UNFILTERED.iter().chain(&FOLD_FILTERED) {
        let declared = view(&ir, name);
        let aggregation = declared.aggregation.as_ref().expect("a grouped view");
        assert_eq!(
            aggregation.functions["duration_sum"].function,
            AggregateFunction::Sum,
            "{name} sums the duration"
        );
        assert_eq!(
            declared.filter.is_some(),
            FOLD_FILTERED.contains(name),
            "{name} is filtered exactly when it is a non-zero total"
        );
    }
}

#[test]
fn aggregate_fold_idiom_synthesizes_group_scenarios() {
    let ir = example();
    let synthesis = synthesize(&ir);
    let ids = scenario_ids(&synthesis);
    let refused = refusals(&synthesis);
    let mut ran = synthesis.suite.clone();
    ran.scenarios.clear();
    for name in FOLD_UNFILTERED {
        let id = format!("{name}/aggregate");
        assert!(ids.contains(&id), "{id} is synthesized: {ids:#?}");
        assert!(
            refused.iter().all(|refusal| !refusal.contains(name)),
            "no refusal names {name}: {refused:#?}"
        );
        let key = ScenarioId::parse(&id).unwrap();
        ran.scenarios
            .insert(key.clone(), synthesis.suite.scenarios[&key].clone());
    }
    assert_eq!(
        failed(&run(&ran, ir)),
        BTreeSet::new(),
        "the interpreter sums"
    );
}

/// Rows of durations 0 and 0 under customer `a`, and 0 and 5 under customer `b`, in one session.
const ZERO_TOTAL: &str = r"
type: ess-scenario/1
domain: idioms.fold
scenario: zero-total-group-is-absent
summary: A group whose durations are all 0 has no row in NonZeroTotalBySession.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: idioms.fold.RecordEnd
    input: {customer: a, session: s1, label: talk, duration: 0}
    outcome: recorded
  - at: 2026-01-05T09:00:01Z
    command: idioms.fold.RecordEnd
    input: {customer: a, session: s1, label: hold, duration: 0}
    outcome: recorded
  - at: 2026-01-05T09:00:02Z
    command: idioms.fold.RecordEnd
    input: {customer: b, session: s1, label: talk, duration: 0}
    outcome: recorded
  - at: 2026-01-05T09:00:03Z
    command: idioms.fold.RecordEnd
    input: {customer: b, session: s1, label: hold, duration: 5}
    outcome: recorded
assert:
  - view: idioms.fold.NonZeroTotalBySession
    contains: {customer: b, session: s1, duration_sum: 5}
  - view: idioms.fold.NonZeroTotalBySession
    excludes: {customer: a, session: s1}
  - view: idioms.fold.NonZeroTotalBySession
    counts: {at_least: 1, at_most: 1}
";

#[test]
fn aggregate_fold_zero_total_group_is_absent() {
    let ir = example();
    let synthesis = synthesize(&ir);
    let refused = refusals(&synthesis);
    for name in FOLD_FILTERED {
        assert!(
            refused
                .iter()
                .any(|refusal| refusal.starts_with("ESS-SYNTH-017") && refusal.contains(name)),
            "synthesis refuses {name} as ESS-SYNTH-017, so an authored scenario holds the claim: \
             {refused:#?}"
        );
    }
    let authoring = compile_authored(&ir, &[AuthoredSource::new("zero-total.yaml", ZERO_TOTAL)]);
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);
    let mut suite = synthesis.suite;
    suite.scenarios = authoring.scenarios;
    assert_eq!(suite.scenarios.len(), 1);
    let healthy = run(&suite, ir);
    assert_eq!(failed(&healthy), BTreeSet::new(), "{healthy:?}");
    // A target that keeps the zero-total group: the session view without its filter.
    let keeps_zero = assemble(&changed(
        "fold.yaml",
        "  - name: idioms.fold.NonZeroTotalBySession
    source: idioms.fold.SessionEnd
    consistency: read_your_writes
    filter: duration > 0
",
        "  - name: idioms.fold.NonZeroTotalBySession
    source: idioms.fold.SessionEnd
    consistency: read_your_writes
",
    ))
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        failed(&run(&suite, keeps_zero)).len(),
        1,
        "a target keeping the zero-total group fails"
    );
}

#[test]
fn per_key_total_section_states_the_idiom() {
    section_states(
        NOTE,
        "## A total maintained per key is a grouped view",
        &[
            "`group_by`",
            "`sum`",
            "`filter: duration > 0`",
            "zero total is absent",
            "`duration >= 0`",
            "materialized",
            "create-or-update",
        ],
    );
}

// ---- beyond10x/ess#444: a unit is a newtype -----------------------------------------------------

#[test]
fn unit_newtype_section_states_the_idiom() {
    section_states(
        NOTE,
        "## A unit is a newtype",
        &[
            "`kind: newtype`",
            "`Millis`",
            "`Seconds`",
            "not assignable",
            "schema title",
            "comparisons across differently named numeric newtypes are admitted",
            "`sum` returns the bare primitive",
        ],
    );
}

#[test]
fn unit_newtype_example_validates() {
    let ir = example();
    holds_views(&ir, "idioms.units", &["idioms.units.CallDurations"]);
    let types: Vec<String> = view(&ir, "idioms.units.CallDurations")
        .fields
        .iter()
        .map(|field| field.type_ref.written().to_string())
        .collect();
    for unit in ["idioms.units.Millis", "idioms.units.Seconds"] {
        assert_eq!(
            types.iter().filter(|written| *written == unit).count(),
            1,
            "CallDurations projects one {unit} field: {types:?}"
        );
    }
}

#[test]
fn unit_newtypes_are_not_assignable() {
    let crossed = refused(&changed(
        "units.yaml",
        "wait_s: input.wait_s}",
        "wait_s: input.talk_ms}",
    ));
    assert!(
        crossed.contains("[type_mismatch]")
            && crossed.contains(
                "`idioms.units.RecordCall.talk_ms` has type `idioms.units.Millis`, and \
                 `idioms.units.Call.wait_s` holds `idioms.units.Seconds`; no conversion is declared"
            ),
        "{crossed}"
    );
    let artifacts = ess_gen::artifact::run(&ess_gen::schema::JsonSchema, &example())
        .expect("the schema projection");
    let entity: Value =
        serde_json::from_str(&artifacts["schema/entities/idioms.units.Call.schema.json"].contents)
            .expect("the entity schema is JSON");
    assert_eq!(
        entity["properties"]["talk_ms"]["$ref"],
        "#/$defs/idioms.units.Millis"
    );
    let millis = &entity["$defs"]["idioms.units.Millis"];
    assert_eq!(millis["title"], "Millis", "{millis}");
    assert_eq!(millis["x-ess-name"], "idioms.units.Millis", "{millis}");
    assert_eq!(millis["x-ess-kind"], "newtype", "{millis}");
}

#[test]
fn unit_newtype_limits_hold_as_documented() {
    // Comparing two differently named numeric newtypes is admitted.
    assemble(&changed(
        "units.yaml",
        "  - name: idioms.units.CallDurations
    source: idioms.units.Call
    consistency: read_your_writes
",
        "  - name: idioms.units.CallDurations
    source: idioms.units.Call
    consistency: read_your_writes
    filter: talk_ms > wait_s
",
    ))
    .unwrap_or_else(|error| panic!("a Millis/Seconds comparison validates:\n{error}"));
    // `sum` returns the bare primitive, so a sum declared in the unit is refused.
    let summed = refused(&appended(
        "units.yaml",
        "
  - name: idioms.units.TalkByQueue
    source: idioms.units.Call
    consistency: read_your_writes
    group_by: [queue_id]
    fields:
      - {name: queue_id, type: String}
      - {name: talk_ms, type: idioms.units.Millis, aggregate: {sum: talk_ms}}
",
    ));
    assert!(
        summed.contains(
            "`idioms.units.TalkByQueue.talk_ms` computes sum(talk_ms), whose result type is \
             Integer, and declares idioms.units.Millis"
        ),
        "{summed}"
    );
}

// ---- beyond10x/ess#446: an order-dependent value needs a declared order --------------------------

#[test]
fn order_dependent_section_states_the_idiom() {
    section_states(
        NOTE,
        "## An order-dependent value needs a declared order",
        &[
            "`max`",
            "`order_by`",
            "size 1",
            "`group_by: [outer, inner]`",
            "tie rule",
            "windows",
            "`arg_max`",
        ],
    );
}

#[test]
fn order_dependent_idiom_examples_validate() {
    let ir = example();
    holds_views(
        &ir,
        "idioms.latest",
        &[
            "idioms.latest.LatestAt",
            "idioms.latest.LatestEvent",
            "idioms.latest.ByOuterInner",
        ],
    );
    let latest = view(&ir, "idioms.latest.LatestAt")
        .aggregation
        .as_ref()
        .expect("LatestAt is grouped");
    assert_eq!(
        latest.functions["latest_at"].function,
        AggregateFunction::Max
    );
    assert_eq!(
        view(&ir, "idioms.latest.ByOuterInner")
            .aggregation
            .as_ref()
            .expect("ByOuterInner is grouped")
            .group_by,
        ["outer", "inner"]
    );
    let row = view(&ir, "idioms.latest.LatestEvent");
    assert!(row.aggregation.is_none(), "LatestEvent is a row view");
    let params: Vec<&str> = row.params.iter().map(|param| param.name.as_str()).collect();
    assert_eq!(params, ["outer", "page", "size"]);
}

#[test]
fn order_dependent_constructs_stay_refused() {
    let last = refused(&changed(
        "latest.yaml",
        "aggregate: {max: at}}",
        "aggregate: {last: at}}",
    ));
    assert!(
        last.contains(
            "unknown variant `last`, expected one of `count`, `count_distinct`, `sum`, `min`, \
             `max`, `avg`"
        ),
        "{last}"
    );
    let ranked = refused(&changed(
        "latest.yaml",
        "    group_by: [outer, inner]\n",
        "    group_by: [outer, inner]\n    order_by: [outer asc]\n",
    ));
    assert!(
        ranked.contains("ESS-VIEW-009")
            && ranked.contains("ranking aggregate rows is a window, which is not in this cut"),
        "{ranked}"
    );
}

#[test]
fn latest_row_view_idiom_synthesizes() {
    let synthesis = synthesize(&example());
    let refused = refusals(&synthesis);
    assert!(
        refused
            .iter()
            .all(|refusal| !refusal.starts_with("ESS-SYNTH-014")),
        "{refused:#?}"
    );
    let steps = steps(&synthesis, "idioms.latest.RecordEvent/outcome/recorded");
    let size_one = steps.iter().any(|step| {
        step["step"] == "query_view"
            && step["view"] == "idioms.latest.LatestEvent"
            && step["params"]["size"]["value"] == 1.0
    });
    assert!(size_one, "LatestEvent is read a page of size 1: {steps:#?}");
    let ranked = steps
        .iter()
        .any(|step| step["step"] == "expect_view" && step["expectation"]["expect"] == "ranked");
    assert!(ranked, "LatestEvent's order is asserted: {steps:#?}");
}

// ---- beyond10x/ess#447: a view has one source ---------------------------------------------------

#[test]
fn one_source_section_states_the_idiom() {
    section_states(
        NOTE,
        "## A view has one source",
        &[
            "`{related: {via`",
            "value when the row was written",
            "current value",
            "two views",
            "`display:`",
            "`source:` stays one entity",
            "`field: {related:`",
        ],
    );
}

#[test]
fn join_idiom_examples_validate() {
    let ir = example();
    holds_views(
        &ir,
        "idioms.joined",
        &[
            "idioms.joined.CallsByAgentLabel",
            "idioms.joined.Agents",
            "idioms.joined.Calls",
        ],
    );
    assert_eq!(
        view(&ir, "idioms.joined.CallsByAgentLabel")
            .aggregation
            .as_ref()
            .expect("grouped")
            .group_by,
        ["agent_label"],
        "grouped by the copied label"
    );
    assert!(
        file("joined.yaml").contains("agent_label: {related: {via: input.agent_id, field: label}}"),
        "the label is copied at write"
    );
}

#[test]
fn copied_related_field_synthesizes_against_decoys() {
    const RECORDED: &str = "idioms.joined.RecordCall/outcome/recorded";
    let ir = example();
    let synthesis = synthesize(&ir);
    let refused = refusals(&synthesis);
    assert!(
        refused
            .iter()
            .all(|refusal| !refusal.contains("idioms.joined.")),
        "{refused:#?}"
    );
    let steps = steps(&synthesis, RECORDED);
    let agents = steps
        .iter()
        .filter(|step| {
            step["step"] == "execute_command" && step["command"] == "idioms.joined.RegisterAgent"
        })
        .count();
    assert!(
        agents >= 3,
        "the referenced agent sits between decoys: {steps:#?}"
    );
    let asserted = steps.iter().any(|step| {
        step["step"] == "expect_view"
            && step["view"] == "idioms.joined.Calls"
            && step["expectation"]["fields"]["agent_label"].is_object()
    });
    assert!(asserted, "the copied label is read back: {steps:#?}");
    let mut suite = synthesis.suite.clone();
    suite
        .scenarios
        .retain(|id, _| id.to_string().starts_with("idioms.joined."));
    assert_eq!(failed(&run(&suite, ir)), BTreeSet::new());
}

#[test]
fn multi_source_view_fields_stay_refused() {
    let dotted = refused(&changed(
        "joined.yaml",
        "      - {name: agent_label, type: String}\n      - {name: calls",
        "      - {name: agent.label, type: String}\n      - {name: calls",
    ));
    assert!(
        dotted.contains("invalid field name identifier \"agent.label\": contains '.'"),
        "{dotted}"
    );
    let unsourced = refused(&appended(
        "joined.yaml",
        "      - {name: label, type: String}\n",
    ));
    assert!(
        unsourced.contains("ESS-VIEW-001")
            && unsourced.contains(
                "`idioms.joined.Call` has no field `label`, so `idioms.joined.Calls` promises an \
                 observation nothing produces"
            ),
        "{unsourced}"
    );
}
