//! A paged view is witnessed by two pages of its declared order (`paging:`, ess/16;
//! beyond10x/ess#174, `docs/design/view-paging.md`).
//!
//! The #174 repro, with `paging: {page: page, size: size, total: true}`, arranges the two rows its
//! declared order already needs and reads the view twice more with a page of one row: the first
//! page, snapshotted, and the page after it. Each read requires the page's exact length and a total
//! of at least the rows the scenario made; the second requires that it continues the first — in the
//! declared order, and not the same row again. Every claim holds on a target other users share.
//! One in-memory target pages correctly, and each wrong mode breaks it in one way.

mod support_versions;
use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{ScenarioValue, SuiteFormat, ViewExpectation};
use ess_conformance::target::*;
use ess_conformance::view_paging::{self, Follows};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/view-paging.yaml");

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("view-paging.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn suite_of(text: &str) -> ConformanceSuite {
    ess_conformance::synthesize::synthesize(&ir_of(text)).suite
}

fn number(value: usize) -> Node {
    Node::Number(Number::from(value))
}

/// Every step of every scenario that reads or requires a page, in order, with the scenario's id.
fn paged_steps(suite: &ConformanceSuite) -> Vec<(String, Vec<ScenarioStep>)> {
    suite
        .scenarios
        .iter()
        .filter(|(_, scenario)| {
            scenario.steps.iter().any(|step| {
                matches!(
                    step,
                    ScenarioStep::ExpectView {
                        expectation: ViewExpectation::Page { .. },
                        ..
                    } | ScenarioStep::EventuallyView {
                        expectation: ViewExpectation::Page { .. },
                        ..
                    }
                )
            })
        })
        .map(|(id, scenario)| (id.to_string(), scenario.steps.clone()))
        .collect()
}

fn pages(steps: &[ScenarioStep]) -> Vec<&ViewExpectation> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView { expectation, .. }
            | ScenarioStep::EventuallyView { expectation, .. }
                if matches!(expectation, ViewExpectation::Page { .. }) =>
            {
                Some(expectation)
            }
            _ => None,
        })
        .collect()
}

// ---- the suite ----------------------------------------------------------------------------------

#[test]
fn the_repro_reads_the_first_page_and_the_one_after_it() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(MODEL));
    let suite = &synthesis.suite;
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    assert!(view_paging::used_by(suite));
    let paged = paged_steps(suite);
    assert_eq!(paged.len(), 1, "one scenario creates the rows: {paged:#?}");
    let (_, steps) = &paged[0];
    let follows = || {
        Some(Follows {
            order_by: vec![serde_json::from_value(serde_json::json!("job_id asc")).unwrap()],
            distinct_by: vec!["job_id".to_owned()],
        })
    };
    let page = |page, size, rows, at_least, follows| ViewExpectation::Page {
        page,
        size,
        rows,
        at_least,
        total_at_least: Some(4),
        follows,
    };
    // Four rows are arranged: two pages of one row, two pages of two rows, and one page larger
    // than all four.
    assert_eq!(
        pages(steps),
        [
            &page(0, 1, 1, false, None),
            &page(1, 1, 1, false, follows()),
            &page(0, 2, 2, false, None),
            &page(1, 2, 2, false, follows()),
            &page(0, 5, 4, true, None),
        ]
    );
    // Each page is read with the caller's filter parameter and the two paging parameters, and the
    // first is snapshotted before the second is read.
    let reads: Vec<&BTreeMap<String, ScenarioValue>> = steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::QueryView { params, .. } if params.contains_key("page") => Some(params),
            _ => None,
        })
        .collect();
    assert_eq!(reads.len(), 5, "{steps:#?}");
    for (params, (page, size)) in reads.iter().zip([(0, 1), (1, 1), (0, 2), (1, 2), (0, 5)]) {
        assert_eq!(params["page"], ScenarioValue::literal(number(page)));
        assert_eq!(params["size"], ScenarioValue::literal(number(size)));
        assert!(params.contains_key("type"), "{params:?}");
    }
    let snapshot = steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::SnapshotView { .. }))
        .expect("the first page is snapshotted");
    let second = steps
        .iter()
        .rposition(|step| matches!(step, ScenarioStep::QueryView { params, .. } if params.contains_key("page")))
        .unwrap();
    assert!(snapshot < second, "{steps:#?}");
    // The other reads of the view send no paging parameter, which `paging:` declares to read
    // every row.
    assert!(steps.iter().any(|step| matches!(step,
        ScenarioStep::QueryView { params, .. } if !params.is_empty() && !params.contains_key("page"))));
}

#[test]
fn a_view_that_declares_no_total_asserts_none() {
    let text = MODEL.replace(
        "paging: {page: page, size: size, total: true}",
        "paging: {page: page, size: size}",
    );
    let suite = suite_of(&text);
    let paged = paged_steps(&suite);
    for page in pages(&paged[0].1) {
        let ViewExpectation::Page { total_at_least, .. } = page else {
            unreachable!()
        };
        assert_eq!(*total_at_least, None);
    }
    let statuses = run(&suite, &Jobs::new(Mode::NoTotal));
    assert!(not_passed(&statuses).is_empty(), "{statuses:#?}");
}

#[test]
fn pages_numbered_from_one_are_read_from_one() {
    let text = MODEL.replace(
        "paging: {page: page, size: size, total: true}",
        "paging: {page: page, size: size, first_page: 1, total: true}",
    );
    let suite = suite_of(&text);
    let paged = paged_steps(&suite);
    let numbers: Vec<u64> = pages(&paged[0].1)
        .into_iter()
        .map(|page| match page {
            ViewExpectation::Page { page, .. } => *page,
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(numbers, [1, 2, 1, 2, 1]);
    let statuses = run(&suite, &Jobs::new(Mode::OneBased));
    assert!(not_passed(&statuses).is_empty(), "{statuses:#?}");
    let statuses = run(&suite, &Jobs::new(Mode::Correct));
    assert!(
        !not_passed(&statuses).is_empty(),
        "a zero-based target fails a one-based view"
    );
}

#[test]
fn a_view_without_paging_keeps_its_suite() {
    let text = MODEL
        .replace("    paging: {page: page, size: size, total: true}\n", "")
        .replace(
            "      - {name: page, type: Integer}\n      - {name: size, type: Integer}\n",
            "",
        );
    let suite = suite_of(&text);
    assert!(!view_paging::used_by(&suite));
    assert_eq!(suite.provenance.suite_version.major(), 34);
}

// ---- running it ---------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Filters, orders, then slices `(page - 0) * size`, and counts the filtered rows.
    Correct,
    /// Correct, on a target other users share: each creation lands beside two rows of the same
    /// type that another user made, one ranked before every row and one after.
    Shared,
    /// Answers every row whatever the paging parameters say.
    IgnoresPaging,
    /// Answers the first page whatever page is asked for.
    IgnoresPage,
    /// Numbers pages from one: page 0 and page 1 are both the first.
    OneBased,
    /// Slices the rows in the order they were made, and orders only the slice.
    PagesBeforeOrdering,
    /// Answers the page's own length as the total.
    TotalIsPageLength,
    /// Carries no total.
    NoTotal,
}

struct Jobs {
    mode: Mode,
    /// `(job_id, type)`, in the order they were made.
    rows: RefCell<Vec<(String, String)>>,
    sequence: RefCell<u64>,
}

impl Jobs {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            rows: RefCell::new(Vec::new()),
            sequence: RefCell::new(0),
        }
    }

    fn param(request: &SemanticViewRequest, name: &str) -> Option<usize> {
        match request.params.get(name) {
            Some(Node::Number(number)) => number.as_i64().and_then(|n| usize::try_from(n).ok()),
            _ => None,
        }
    }
}

impl ConformanceTarget for Jobs {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("view-paging", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command: CommandRef = request.command.clone();
        assert_eq!(command.to_string(), "demo.jobs.CreateJob");
        let kind = request
            .input
            .get("type")
            .and_then(Node::as_text)
            .unwrap_or_default()
            .to_owned();
        let mut sequence = self.sequence.borrow_mut();
        *sequence += 1;
        // Minted descending, so the order rows are made in is the reverse of `job_id asc`.
        let id = format!("00000000-0000-4000-8000-{:012}", 900_000 - *sequence);
        if self.mode == Mode::Shared {
            let mut rows = self.rows.borrow_mut();
            rows.push((
                format!("00000000-0000-4000-8000-{:012}", *sequence),
                kind.clone(),
            ));
            rows.push((
                format!("00000000-0000-4000-9000-{:012}", *sequence),
                kind.clone(),
            ));
        }
        self.rows.borrow_mut().push((id.clone(), kind.clone()));
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            command.clone(),
            "created".parse().unwrap(),
        ));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
        let mut event = ObservedEvent::new("demo.jobs.JobCreated".parse().unwrap());
        event.payload.insert("job_id".to_owned(), Node::Text(id));
        event.payload.insert("type".to_owned(), Node::Text(kind));
        result.direct_events.push(event);
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let wanted = request.params.get("type").and_then(Node::as_text);
        let admitted: Vec<(String, String)> = self
            .rows
            .borrow()
            .iter()
            .filter(|(_, kind)| wanted.is_none_or(|wanted| wanted == kind))
            .cloned()
            .collect();
        let mut ordered = admitted.clone();
        ordered.sort();
        let (page, size) = (Self::param(&request, "page"), Self::param(&request, "size"));
        let slice = match (page, size) {
            (Some(page), Some(size)) if self.mode != Mode::IgnoresPaging => {
                let offset = match self.mode {
                    Mode::IgnoresPage => 0,
                    Mode::OneBased => page.saturating_sub(1) * size,
                    _ => page * size,
                };
                let source = if self.mode == Mode::PagesBeforeOrdering {
                    &admitted
                } else {
                    &ordered
                };
                let mut slice: Vec<(String, String)> =
                    source.iter().skip(offset).take(size).cloned().collect();
                slice.sort();
                slice
            }
            _ => ordered.clone(),
        };
        let mut result = SemanticViewResult::of(slice.into_iter().map(|(id, kind)| {
            BTreeMap::from([
                ("job_id".to_owned(), Node::Text(id)),
                ("type".to_owned(), Node::Text(kind)),
            ])
        }));
        result.total = match self.mode {
            Mode::NoTotal => None,
            Mode::TotalIsPageLength => Some(result.rows.len() as u64),
            _ => Some(admitted.len() as u64),
        };
        Ok(result)
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "the model declares none",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported("events", "unused"))
    }
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

#[test]
fn every_scenario_passes_against_a_target_that_pages() {
    let suite = suite_of(MODEL);
    for mode in [Mode::Correct, Mode::Shared] {
        let statuses = run(&suite, &Jobs::new(mode));
        assert!(
            not_passed(&statuses).is_empty(),
            "{mode:?}: every scenario passes: {statuses:#?}"
        );
    }
}

#[test]
fn each_way_of_paging_wrong_fails_the_scenario_that_reads_the_pages() {
    let suite = suite_of(MODEL);
    let paged: Vec<String> = paged_steps(&suite).into_iter().map(|(id, _)| id).collect();
    for mode in [
        Mode::IgnoresPaging,
        Mode::IgnoresPage,
        Mode::OneBased,
        Mode::PagesBeforeOrdering,
        Mode::TotalIsPageLength,
        Mode::NoTotal,
    ] {
        let statuses = run(&suite, &Jobs::new(mode));
        assert_eq!(
            not_passed(&statuses),
            paged.iter().map(String::as_str).collect::<Vec<_>>(),
            "{mode:?}: {statuses:#?}"
        );
        assert_eq!(statuses[&paged[0]], Status::Failed, "{mode:?}");
    }
}

// ---- admission ----------------------------------------------------------------------------------

#[test]
fn a_page_under_an_older_suite_label_is_refused_as_vocabulary_it_does_not_have() {
    let mut suite = suite_of(MODEL);
    let original = suite.to_canonical_json().unwrap();
    assert!(original.contains("\"expect\": \"page\""), "{original}");
    let older = support_versions::legacy_json(&original, 24);
    assert_ne!(older, original);
    let error = AdmittedSuite::from_json(&older).expect_err("a page needs suite/26");
    assert_eq!(error.issues[0].reason, "UnsupportedVocabulary", "{error}");
    assert!(error.to_string().contains("suite/26"), "{error}");
    suite.provenance.suite_version = SuiteFormat::parse("ess-conformance/24").unwrap();
    suite.provenance.scenario_initial_state = None;
    let error = ess_conformance::admission::suite(&suite).expect_err("refused");
    assert_eq!(error.issues[0].reason, "UnsupportedVocabulary");
}

#[test]
fn a_page_that_is_no_claim_is_refused_typed_and_in_bytes() {
    let original = suite_of(MODEL).to_canonical_json().unwrap();
    let (id, _) = paged_steps(&suite_of(MODEL)).remove(0);
    for (key, value) in [
        ("size", serde_json::json!(0)),
        ("rows", serde_json::json!(2)),
        ("total_at_least", serde_json::json!(0)),
        ("follows", serde_json::json!({"order_by": []})),
        (
            "follows",
            serde_json::json!({"order_by": ["job_id asc"], "cursor": 1}),
        ),
        ("rows", serde_json::json!(-1)),
        ("cursor", serde_json::json!("next")),
        ("at_least", serde_json::json!("yes")),
    ] {
        let mut document: serde_json::Value = serde_json::from_str(&original).unwrap();
        let steps = document["scenarios"][&id]["steps"].as_array_mut().unwrap();
        let page = steps
            .iter_mut()
            .filter(|step| step["expectation"]["expect"] == "page")
            .nth(1)
            .expect("the scenario asserts a page");
        page["expectation"][key] = value.clone();
        let text = serde_json::to_string_pretty(&document).unwrap();
        assert!(
            AdmittedSuite::from_json(&text).is_err(),
            "{key}: {value} is admitted"
        );
    }
    // Typed: a page longer than its size.
    let mut suite = suite_of(MODEL);
    for scenario in suite.scenarios.values_mut() {
        for step in &mut scenario.steps {
            if let ScenarioStep::ExpectView {
                expectation: ViewExpectation::Page { rows, .. },
                ..
            } = step
            {
                *rows = 3;
            }
        }
    }
    let error = AdmittedSuite::from_suite(&suite).expect_err("refused");
    assert_eq!(error.issues[0].reason, "InvalidSuite", "{error}");
}
