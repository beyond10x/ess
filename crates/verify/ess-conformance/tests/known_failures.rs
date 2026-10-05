//! Known-failure declarations, execution contexts and accounting, against actual reference runs.
//!
//! `docs/design/mutation-scope-and-known-failures.md` (beyond10x/ess#294, beyond10x/ess#296). Every
//! report here is what the Rust runner writes for a real target: the billing reference, or the
//! billing reference with one planted defect (`Fault::ExtraEvent`: cancelling an invoice also
//! announces it paid — the tracked cancellation defect of the design's second adopter). Where a
//! category the runner cannot produce is needed (`skipped`), an actual report is rewritten and
//! read back through `CountReport` before use, as the collector tests do.

use std::collections::BTreeMap;
use std::path::Path;

use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::known_failures::{
    self, account, Accounting, Declaration, ExecutionContext, RefusalKind,
};
use ess_conformance::mutate::{self, Document, MutantClass, BASELINE_DIR, SUITE_FILE};
use ess_conformance::reference::Billing;
use ess_conformance::runner::Runner;
use ess_conformance::target::ConformanceTarget;
use ess_conformance::{AdmittedSuite, CountReport, CountStatus};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::{json, Value};

fn example(name: &str) -> (Vec<Document>, SourceMap) {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name);
    let mut found = Vec::new();
    let mut pending = vec![base];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path.display().to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).expect("well formed"),
        ));
        texts.insert(label, text);
    }
    (parsed, texts)
}

/// Two public builds; only the first is the one the runs here are said to come from.
fn build(digit: char) -> String {
    format!("sha256:{}", digit.to_string().repeat(64))
}

/// The billing suite the mutation audit's baseline runs, as its exact bytes.
fn suite() -> AdmittedSuite {
    let (files, texts) = example("billing");
    let emission = mutate::emit(&files, &texts, &[MutantClass::ErrorSwap]).expect("emits");
    AdmittedSuite::from_json(&emission.files[&format!("{BASELINE_DIR}/{SUITE_FILE}")])
        .expect("admitted")
}

/// The report/2 the Rust runner writes for `target` over `admitted`.
fn run(admitted: &AdmittedSuite, target: &impl ConformanceTarget) -> String {
    let executed = Runner::for_suite(admitted.suite()).run_admitted(admitted, target);
    CountReport::from_run(&executed, admitted)
        .expect("a complete run")
        .to_canonical_json()
        .expect("serializes")
}

fn defective(admitted: &AdmittedSuite) -> String {
    run(admitted, &faulty::billing(Fault::ExtraEvent))
}

fn statuses(report: &str, admitted: &AdmittedSuite) -> BTreeMap<String, &'static str> {
    CountReport::from_json(report, admitted)
        .expect("report/2")
        .statuses()
}

fn ids_with(report: &str, admitted: &AdmittedSuite, status: &str) -> Vec<String> {
    statuses(report, admitted)
        .into_iter()
        .filter(|(_, it)| *it == status)
        .map(|(id, _)| id)
        .collect()
}

fn label(report: &str, admitted: &AdmittedSuite) -> String {
    CountReport::from_json(report, admitted)
        .unwrap()
        .implementation()
        .to_owned()
}

/// A declaration of `scenarios` bound to `admitted`, the report's label and `build`.
fn declaration(
    admitted: &AdmittedSuite,
    implementation: &str,
    build: &str,
    scenarios: &[String],
) -> Value {
    let mut scenarios = scenarios.to_vec();
    scenarios.sort();
    json!({
        "format": "ess-known-failures/1",
        "spec_digest": admitted.suite().provenance.spec_digest.to_string(),
        "suite_digest": admitted.digest(),
        "implementation": implementation,
        "implementation_build": build,
        "failures": scenarios.iter().map(|id| json!({
            "scenario": id,
            "reason": "cancelling also announces the invoice paid",
            "tracking": "ORDERS-412",
        })).collect::<Vec<_>>(),
    })
}

fn text(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap() + "\n"
}

fn declared(value: &Value) -> Declaration {
    Declaration::from_json(&text(value)).unwrap_or_else(|refusal| panic!("{refusal}"))
}

/// An actual report of `admitted`, with the scenarios of `assigned` moved to the category named,
/// under the generated runners' five-category profile; read back through `CountReport`.
fn rewritten(report: &str, admitted: &AdmittedSuite, assigned: &BTreeMap<String, &str>) -> String {
    let mut value: Value = serde_json::from_str(report).unwrap();
    let mut outcomes: BTreeMap<&str, Vec<String>> =
        ["passed", "failed", "error", "unsupported", "skipped"]
            .into_iter()
            .map(|it| (it, Vec::new()))
            .collect();
    for (id, status) in statuses(report, admitted) {
        let status = assigned.get(&id).copied().unwrap_or(status);
        outcomes.get_mut(status).unwrap().push(id);
    }
    let count = |it: &str| outcomes[it].len();
    let execution = if count("failed") > 0 || count("unsupported") > 0 {
        "failed"
    } else if count("error") > 0 || count("skipped") > 0 {
        "inconclusive"
    } else {
        "passed"
    };
    value["producer_profile"] = "go-scenario-status/2".into();
    value["counts"] = json!({"total": admitted.suite().len(), "passed": count("passed"),
        "failed": count("failed"), "error": count("error"), "unsupported": count("unsupported"),
        "skipped": count("skipped")});
    value["outcomes"] = json!(outcomes);
    value["execution_status"] = execution.into();
    value["conformance_status"] = if execution == "failed" {
        "failed"
    } else {
        "inconclusive"
    }
    .into();
    let written = text(&value);
    CountReport::from_json(&written, admitted).expect("a coherent rewritten report/2");
    written
}

#[test]
fn accounting_partitions_the_original_failures_and_leaves_the_report_failed() {
    let admitted = suite();
    let report = defective(&admitted);
    let failed = ids_with(&report, &admitted, "failed");
    assert!(
        failed.len() >= 2,
        "the planted defect fails more than one scenario: {failed:?}"
    );
    let known = vec![failed[0].clone()];
    let declaration = declared(&declaration(
        &admitted,
        &label(&report, &admitted),
        &build('a'),
        &known,
    ));
    let accounting = account(&report, &admitted, &declaration, &build('a'))
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    let original = CountReport::from_json(&report, &admitted).unwrap();
    assert_eq!(
        &accounting.counts,
        original.counts(),
        "the report's own counts"
    );
    assert_eq!(
        accounting.counts.failed,
        u64::try_from(failed.len()).unwrap(),
        "a known failure is still a failure"
    );
    assert_eq!(
        accounting
            .known_failed
            .iter()
            .map(|it| it.scenario.clone())
            .collect::<Vec<_>>(),
        known
    );
    assert_eq!(accounting.unexpected_failed, failed[1..].to_vec());
    assert_eq!(original.conformance_status(), CountStatus::Failed);
    assert_eq!(accounting.format, known_failures::ACCOUNTING_FORMAT);
    assert_eq!(
        accounting.report_digest,
        known_failures::sha256(report.as_bytes())
    );
    assert_eq!(accounting.suite_digest, admitted.digest());
    assert_eq!(accounting.declaration_digest, declaration.digest());
    let json = accounting.to_canonical_json();
    let value: Value = serde_json::from_str(&json).unwrap();
    let keys: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        [
            "counts",
            "declaration_digest",
            "format",
            "implementation",
            "implementation_build",
            "known_failed",
            "report_digest",
            "spec_digest",
            "suite_digest",
            "unexpected_failed"
        ],
        "closed, and no conformance verdict"
    );
    let read = Accounting::from_json(&json).unwrap();
    assert_eq!(read, accounting);
    read.validate(&report, &admitted, declaration.original())
        .unwrap_or_else(|refusal| panic!("{refusal}"));
}

#[test]
fn a_declaration_bound_to_another_suite_specification_implementation_or_build_is_refused() {
    let admitted = suite();
    let report = defective(&admitted);
    let failed = ids_with(&report, &admitted, "failed");
    let implementation = label(&report, &admitted);
    let base = declaration(&admitted, &implementation, &build('a'), &failed);
    for (field, value) in [
        ("suite_digest", json!(build('b'))),
        ("spec_digest", json!("b".repeat(64))),
        (
            "implementation",
            json!(format!("{implementation} (patched)")),
        ),
        ("implementation_build", json!(build('b'))),
    ] {
        let mut changed = base.clone();
        changed[field] = value;
        let refusal =
            account(&report, &admitted, &declared(&changed), &build('a')).expect_err(field);
        assert_eq!(refusal.kind, RefusalKind::Identity, "{field}: {refusal}");
    }
    // The build is the host's, never the declaration's: the same declaration under another build.
    let refusal = account(&report, &admitted, &declared(&base), &build('b')).expect_err("build");
    assert_eq!(refusal.kind, RefusalKind::Identity, "{refusal}");
    assert!(refusal.to_string().contains(&build('b')), "{refusal}");
}

#[test]
fn a_declared_scenario_that_passed_is_stale_and_refused() {
    let admitted = suite();
    let report = defective(&admitted);
    let failed = ids_with(&report, &admitted, "failed");
    let passing = ids_with(&report, &admitted, "passed");
    let implementation = label(&report, &admitted);
    // One declared scenario too many: it passes.
    let mut listed = failed.clone();
    listed.push(passing[0].clone());
    let refusal = account(
        &report,
        &admitted,
        &declared(&declaration(
            &admitted,
            &implementation,
            &build('a'),
            &listed,
        )),
        &build('a'),
    )
    .expect_err("stale");
    assert_eq!(refusal.kind, RefusalKind::Stale, "{refusal}");
    assert!(refusal.to_string().contains(&passing[0]), "{refusal}");
    // The defect repaired: the same declaration against the correct target is stale throughout.
    let repaired = run(&admitted, &Billing::new());
    let refusal = account(
        &repaired,
        &admitted,
        &declared(&declaration(
            &admitted,
            &label(&repaired, &admitted),
            &build('a'),
            &failed,
        )),
        &build('a'),
    )
    .expect_err("stale after repair");
    assert_eq!(refusal.kind, RefusalKind::Stale, "{refusal}");
}

#[test]
fn a_declared_scenario_that_ended_error_unsupported_or_skipped_is_not_a_known_failure() {
    let admitted = suite();
    let report = defective(&admitted);
    let failed = ids_with(&report, &admitted, "failed");
    for category in ["error", "unsupported", "skipped"] {
        let changed = rewritten(
            &report,
            &admitted,
            &BTreeMap::from([(failed[0].clone(), category)]),
        );
        let declaration = declared(&declaration(
            &admitted,
            &label(&changed, &admitted),
            &build('a'),
            &failed,
        ));
        let refusal = account(&changed, &admitted, &declaration, &build('a')).expect_err(category);
        assert_eq!(
            refusal.kind,
            RefusalKind::NotFailed,
            "{category}: {refusal}"
        );
        assert!(refusal.to_string().contains(category), "{refusal}");
    }
}

#[test]
fn skipped_and_unsupported_stay_their_own_categories_in_the_accounted_counts() {
    let admitted = suite();
    let report = defective(&admitted);
    let failed = ids_with(&report, &admitted, "failed");
    let passing = ids_with(&report, &admitted, "passed");
    let mixed = rewritten(
        &report,
        &admitted,
        &BTreeMap::from([
            (passing[0].clone(), "skipped"),
            (passing[1].clone(), "unsupported"),
        ]),
    );
    let declaration = declared(&declaration(
        &admitted,
        &label(&mixed, &admitted),
        &build('a'),
        &failed,
    ));
    let accounting = account(&mixed, &admitted, &declaration, &build('a')).unwrap();
    assert_eq!(accounting.counts.skipped, 1);
    assert_eq!(accounting.counts.unsupported, 1);
    assert_eq!(accounting.unexpected_failed, Vec::<String>::new());
    assert_eq!(
        CountReport::from_json(&mixed, &admitted)
            .unwrap()
            .conformance_status(),
        CountStatus::Failed
    );
}

#[test]
fn a_declaration_that_is_not_closed_exact_and_ordered_is_refused() {
    let admitted = suite();
    let report = defective(&admitted);
    let failed = ids_with(&report, &admitted, "failed");
    let base = declaration(&admitted, &label(&report, &admitted), &build('a'), &failed);
    let mut cases: Vec<(&str, String)> = Vec::new();
    let mut unknown = base.clone();
    unknown["waiver"] = json!(true);
    cases.push(("unknown field", text(&unknown)));
    let mut nested = base.clone();
    nested["failures"][0]["severity"] = json!("low");
    cases.push(("unknown failure field", text(&nested)));
    let duplicate = text(&base).replacen(
        "\"format\": \"ess-known-failures/1\",",
        "\"format\": \"ess-known-failures/1\",\n  \"format\": \"ess-known-failures/1\",",
        1,
    );
    cases.push(("duplicate key", duplicate));
    let mut other = base.clone();
    other["format"] = json!("ess-known-failures/2");
    cases.push(("other format", text(&other)));
    let mut empty = base.clone();
    empty["failures"] = json!([]);
    cases.push(("no failures", text(&empty)));
    let mut reason = base.clone();
    reason["failures"][0]["reason"] = json!(" ");
    cases.push(("empty reason", text(&reason)));
    let mut tracking = base.clone();
    tracking["failures"][0]["tracking"] = json!("");
    cases.push(("empty tracking", text(&tracking)));
    let mut digest = base.clone();
    digest["implementation_build"] = json!(build('a').to_uppercase().replace("SHA256", "sha256"));
    cases.push(("uppercase digest", text(&digest)));
    let mut twice = base.clone();
    let first = twice["failures"][0].clone();
    twice["failures"].as_array_mut().unwrap().insert(0, first);
    cases.push(("one scenario twice", text(&twice)));
    if failed.len() > 1 {
        let mut unsorted = base.clone();
        unsorted["failures"].as_array_mut().unwrap().reverse();
        cases.push(("unordered", text(&unsorted)));
    }
    for (case, document) in cases {
        let refusal = Declaration::from_json(&document).expect_err(case);
        assert_eq!(refusal.kind, RefusalKind::Malformed, "{case}: {refusal}");
    }
    // Exact IDs only: a prefix of a failing scenario, or a wildcard, names no scenario.
    let prefix = failed[0].rsplit_once('/').unwrap().0.to_owned();
    for loose in [prefix, format!("{}*", &failed[0][..failed[0].len() - 2])] {
        let mut changed = base.clone();
        changed["failures"] = json!([{"scenario": loose, "reason": "r", "tracking": "t"}]);
        let refusal = declared(&changed).admit(&admitted).expect_err(&loose);
        assert_eq!(refusal.kind, RefusalKind::UnknownScenario, "{refusal}");
    }
}

/// One edit an accounting document is tampered with.
type Tamper = Box<dyn Fn(&mut Accounting)>;

#[test]
fn a_tampered_accounting_is_refused_by_its_validator() {
    let admitted = suite();
    let report = defective(&admitted);
    let failed = ids_with(&report, &admitted, "failed");
    let declaration = declared(&declaration(
        &admitted,
        &label(&report, &admitted),
        &build('a'),
        &failed[..1],
    ));
    let accounting = account(&report, &admitted, &declaration, &build('a')).unwrap();
    let tamper: Vec<(&str, Tamper)> = vec![
        (
            "an unexpected failure dropped",
            Box::new(|it| {
                it.unexpected_failed.pop();
            }),
        ),
        (
            "a known failure counted as a pass",
            Box::new(|it| {
                it.counts.failed -= 1;
                it.counts.passed += 1;
            }),
        ),
        (
            "a known failure removed from the total",
            Box::new(|it| {
                it.counts.failed -= 1;
                it.counts.total -= 1;
            }),
        ),
        (
            "another report",
            Box::new(|it| it.report_digest = build('c')),
        ),
        (
            "another declaration",
            Box::new(|it| it.declaration_digest = build('c')),
        ),
        ("another suite", Box::new(|it| it.suite_digest = build('c'))),
        (
            "another build",
            Box::new(|it| it.implementation_build = build('c')),
        ),
    ];
    for (case, change) in tamper {
        let mut changed = accounting.clone();
        change(&mut changed);
        assert!(
            changed
                .validate(&report, &admitted, declaration.original())
                .is_err(),
            "{case}"
        );
    }
    let mut extra: Value = serde_json::from_str(&accounting.to_canonical_json()).unwrap();
    extra["conformance_status"] = json!("passed");
    assert!(Accounting::from_json(&text(&extra)).is_err(), "closed");
}

#[test]
fn an_execution_context_binds_the_exact_report_suite_and_label() {
    let admitted = suite();
    let report = defective(&admitted);
    let implementation = label(&report, &admitted);
    let context = ExecutionContext::new(&report, &admitted, &implementation, &build('a')).unwrap();
    let json = context.to_canonical_json();
    let value: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        value.as_object().unwrap().keys().collect::<Vec<_>>(),
        [
            "format",
            "implementation",
            "implementation_build",
            "report_digest",
            "suite_digest"
        ],
        "digests, the label and the build: no values, timestamps or commands"
    );
    let read = ExecutionContext::from_json(&json).unwrap();
    assert_eq!(read, context);
    read.admit(&report, &admitted).unwrap();
    // Other report bytes of the same run: re-encoded without whitespace.
    let compact = serde_json::to_string(&serde_json::from_str::<Value>(&report).unwrap()).unwrap();
    assert_eq!(
        read.admit(&compact, &admitted).unwrap_err().kind,
        RefusalKind::Identity
    );
    let mut relabelled = value.clone();
    relabelled["implementation"] = json!("someone-else 1");
    let relabelled = ExecutionContext::from_json(&text(&relabelled)).unwrap();
    assert_eq!(
        relabelled.admit(&report, &admitted).unwrap_err().kind,
        RefusalKind::Identity
    );
    let mut resuited = value.clone();
    resuited["suite_digest"] = json!(build('d'));
    let resuited = ExecutionContext::from_json(&text(&resuited)).unwrap();
    assert_eq!(
        resuited.admit(&report, &admitted).unwrap_err().kind,
        RefusalKind::Identity
    );
    for (case, document) in [
        ("unknown field", {
            let mut it = value.clone();
            it["host"] = json!("ci-7");
            text(&it)
        }),
        ("invalid build", {
            let mut it = value.clone();
            it["implementation_build"] = json!("1.2.3");
            text(&it)
        }),
        (
            "duplicate key",
            json.replacen(
                "\"format\": \"ess-conformance-execution/1\",",
                "\"format\": \"ess-conformance-execution/1\",\n  \"format\": \"ess-conformance-execution/1\",",
                1,
            ),
        ),
    ] {
        assert_eq!(
            ExecutionContext::from_json(&document).unwrap_err().kind,
            RefusalKind::Malformed,
            "{case}"
        );
    }
    assert!(
        ExecutionContext::new(&report, &admitted, &implementation, "not-a-digest").is_err(),
        "a build is a SHA-256"
    );
}

#[test]
fn every_known_failure_code_and_format_is_named_in_the_formats_reference() {
    let page = include_str!("../../../../website/docs/reference/formats.md");
    for name in [
        known_failures::DECLARATION_FORMAT,
        known_failures::EXECUTION_FORMAT,
        known_failures::ACCOUNTING_FORMAT,
    ] {
        assert!(page.contains(name), "`{name}` is not named in formats.md");
    }
    for kind in [
        RefusalKind::Malformed,
        RefusalKind::Identity,
        RefusalKind::UnknownScenario,
        RefusalKind::Stale,
        RefusalKind::NotFailed,
        RefusalKind::Unbound,
        RefusalKind::Accounting,
    ] {
        assert!(
            page.contains(kind.as_str()),
            "`{}` is not named in formats.md",
            kind.as_str()
        );
    }
}
