//! Declaring, dropping or changing a view's `paging:` is a semantic change (ess/16,
//! beyond10x/ess#174, `docs/design/view-paging.md`).
//!
//! `paging-changed` carries the paging contract on each side — the parameters that carry the page
//! and its size, the first page's number and whether a total is answered — and needs `ess-diff/9`,
//! so an older reader refuses the delta rather than reading a paged view as unchanged. A delta that
//! moves nothing about paging keeps its earlier format.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, DeltaFormat, EssDelta, RawEssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const JOBS: &str = include_str!("../../ess-conformance/tests/fixtures/view-paging.yaml");

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("jobs.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn paging(block: &str) -> String {
    JOBS.replace("paging: {page: page, size: size, total: true}", block)
}

#[test]
fn a_changed_paging_block_is_paging_changed_at_diff_9() {
    let before = ir(JOBS);
    let after = ir(&paging("paging: {page: page, size: size, first_page: 1}"));
    let delta = diff(&before, &after).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(delta.format.to_string(), "ess-diff/9", "{json}");
    assert!(json.contains(r#""kind": "paging-changed""#), "{json}");
    assert!(json.contains(r#""total": true"#), "{json}");
    assert!(json.contains(r#""first_page": 1"#), "{json}");
    assert_eq!(delta.changes().len(), 1, "{json}");
    assert!(
        delta.changes()[0]
            .describe()
            .contains("paged by `page` and `size`"),
        "{}",
        delta.changes()[0].describe()
    );
    // The same delta cannot be written in a vocabulary that lacks the kind.
    assert!(delta
        .to_canonical_json_for(DeltaFormat::parse("ess-diff/8").unwrap())
        .is_err());
}

#[test]
fn declaring_paging_is_a_change_from_nothing() {
    let unpaged = JOBS
        .replace("    paging: {page: page, size: size, total: true}\n", "")
        .replace(
            "      - {name: page, type: Integer}\n      - {name: size, type: Integer}\n",
            "",
        );
    let delta = diff(&ir(&unpaged), &ir(JOBS)).unwrap();
    let json = delta.to_canonical_json();
    assert!(json.contains(r#""kind": "paging-changed""#), "{json}");
    assert!(
        json.contains(r#""before": null"#) || !json.contains(r#""before": {"#),
        "{json}"
    );
    assert!(
        delta
            .changes()
            .iter()
            .any(|change| change.describe().contains("not paged")),
        "{json}"
    );
}

#[test]
fn a_delta_that_moves_nothing_about_paging_keeps_its_format() {
    let delta = diff(
        &ir(JOBS),
        &ir(&JOBS.replace("order_by: [job_id asc]", "order_by: [job_id desc]")),
    )
    .unwrap();
    assert!(delta.format.major() < 9, "{}", delta.to_canonical_json());
}

#[test]
fn a_delta_carrying_paging_changed_under_diff_8_is_refused_on_read() {
    let delta = diff(&ir(JOBS), &ir(&paging("paging: {page: page, size: size}"))).unwrap();
    let json = delta.to_canonical_json();
    let older = json.replace("\"ess-diff/9\"", "\"ess-diff/8\"");
    assert_ne!(older, json);
    let raw: RawEssDelta = serde_json::from_str(&older).unwrap();
    assert!(EssDelta::try_from(raw).is_err());
    let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
    assert_eq!(EssDelta::try_from(raw).unwrap(), delta);
}
