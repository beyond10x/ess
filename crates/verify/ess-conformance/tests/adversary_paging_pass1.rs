//! Adversary, pass 1, story `view-paging-and-caller-filters` (`paging:`, ess/16).
//!
//! Wrong targets the synthesized page reads let through, and a read the design says carries no
//! paging parameter that does.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::ViewExpectation;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
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

fn is_page(step: &ScenarioStep) -> bool {
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
}

fn paged_ids(suite: &ConformanceSuite) -> Vec<String> {
    suite
        .scenarios
        .iter()
        .filter(|(_, scenario)| scenario.steps.iter().any(is_page))
        .map(|(id, _)| id.to_string())
        .collect()
}

/// The #174 repro with a `title` the view is ordered by, and a projection without the identity.
fn untitled_identity_model() -> String {
    let (head, views) = MODEL.split_at(MODEL.find("views:").expect("a views block"));
    let views = views.replace(
        "    fields:\n      - {name: job_id, type: demo.jobs.JobId}\n      - {name: type, type: demo.jobs.JobType}\n",
        "    fields:\n      - {name: title, type: String}\n      - {name: type, type: demo.jobs.JobType}\n",
    );
    format!("{head}{views}")
        .replace(
            "    fields:\n      - {name: type, type: demo.jobs.JobType}\n    lifecycle:",
            "    fields:\n      - {name: type, type: demo.jobs.JobType}\n      - {name: title, type: String}\n    lifecycle:",
        )
        .replace(
            "    input:\n      - {name: type, type: demo.jobs.JobType}\n",
            "    input:\n      - {name: type, type: demo.jobs.JobType}\n      - {name: title, type: String}\n",
        )
        .replace(
            "sets: {type: input.type}",
            "sets: {type: input.type, title: input.title}",
        )
        .replace("order_by: [job_id asc]", "order_by: [title asc]")
}

// ---- a target -----------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Filters, orders by `key`, slices `(page - 0) * size`, counts the filtered rows.
    Correct,
    /// Answers the first page whatever page is asked for.
    IgnoresPage,
    /// Computes the page count as `total / size` and answers no row on a page past it, so a
    /// partial last page is never answered.
    DropsPartialLastPage,
}

struct Jobs {
    mode: Mode,
    /// The field the view is ordered by.
    key: &'static str,
    /// The fields the view projects.
    projects: &'static [&'static str],
    rows: RefCell<Vec<BTreeMap<String, Node>>>,
    sequence: RefCell<u64>,
}

impl Jobs {
    fn new(mode: Mode, key: &'static str, projects: &'static [&'static str]) -> Self {
        Self {
            mode,
            key,
            projects,
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
        Ok(ImplementationIdentity::new("adversary-paging", "1"))
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
        let mut sequence = self.sequence.borrow_mut();
        *sequence += 1;
        let id = format!("00000000-0000-4000-8000-{:012}", 900_000 - *sequence);
        let mut row: BTreeMap<String, Node> = request.input.clone().into_iter().collect();
        row.insert("job_id".to_owned(), Node::Text(id.clone()));
        let kind = row.get("type").cloned().unwrap_or(Node::Null);
        self.rows.borrow_mut().push(row);
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            command.clone(),
            "created".parse().unwrap(),
        ));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
        let mut event = ObservedEvent::new("demo.jobs.JobCreated".parse().unwrap());
        event.payload.insert("job_id".to_owned(), Node::Text(id));
        event.payload.insert("type".to_owned(), kind);
        result.direct_events.push(event);
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let wanted = request.params.get("type").and_then(Node::as_text);
        let mut ordered: Vec<BTreeMap<String, Node>> = self
            .rows
            .borrow()
            .iter()
            .filter(|row| {
                wanted.is_none_or(|wanted| row.get("type").and_then(Node::as_text) == Some(wanted))
            })
            .cloned()
            .collect();
        let key = self.key;
        ordered.sort_by(|a, b| {
            let a = a.get(key).and_then(Node::as_text).unwrap_or_default();
            let b = b.get(key).and_then(Node::as_text).unwrap_or_default();
            a.cmp(b)
        });
        let total = ordered.len();
        let slice = match (Self::param(&request, "page"), Self::param(&request, "size")) {
            (Some(page), Some(size)) if size > 0 => {
                let offset = match self.mode {
                    Mode::IgnoresPage => 0,
                    _ => page * size,
                };
                if self.mode == Mode::DropsPartialLastPage && page >= total / size {
                    Vec::new()
                } else {
                    ordered.into_iter().skip(offset).take(size).collect()
                }
            }
            _ => ordered,
        };
        let projects = self.projects;
        let result = SemanticViewResult::of(slice.into_iter().map(|row| {
            row.into_iter()
                .filter(|(field, _)| projects.contains(&field.as_str()))
                .collect()
        }));
        Ok(result.with_total(total as u64))
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "none declared"))
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

// ---- cases --------------------------------------------------------------------------------------

/// A paged view that does not project its entity's identity. `follows` then carries no
/// `distinct_by`, and a tie is "in order", so the second page answering the first page's row again
/// passes: the design's table ("ignores `page` → `follows`: the same row again") holds only where
/// the identity is projected.
#[test]
fn a_target_that_ignores_page_fails_a_view_that_projects_no_identity() {
    let suite = suite_of(&untitled_identity_model());
    let paged = paged_ids(&suite);
    assert!(!paged.is_empty(), "the view is read by page: {suite:#?}");
    let correct = run(
        &suite,
        &Jobs::new(Mode::Correct, "title", &["title", "type"]),
    );
    assert!(
        not_passed(&correct).is_empty(),
        "a correct target passes: {correct:#?}"
    );
    let wrong = run(
        &suite,
        &Jobs::new(Mode::IgnoresPage, "title", &["title", "type"]),
    );
    assert!(
        !not_passed(&wrong).is_empty(),
        "a target that answers the first page for every page passes every scenario: {wrong:#?}"
    );
}

/// An eventual paged view: each page is an `eventually_view`, the snapshot is of the last one.
#[test]
fn an_eventual_paged_view_passes_a_correct_target_and_fails_one_that_ignores_page() {
    let text = MODEL.replace("consistency: read_your_writes", "consistency: eventual");
    let suite = suite_of(&text);
    let paged = paged_ids(&suite);
    assert!(!paged.is_empty(), "the eventual view is read by page");
    let correct = run(
        &suite,
        &Jobs::new(Mode::Correct, "job_id", &["job_id", "type"]),
    );
    assert!(not_passed(&correct).is_empty(), "{correct:#?}");
    let wrong = run(
        &suite,
        &Jobs::new(Mode::IgnoresPage, "job_id", &["job_id", "type"]),
    );
    assert!(!not_passed(&wrong).is_empty(), "{wrong:#?}");
}

/// Every page the suite reads is full (size 1 over two rows), so a target that never answers a
/// partial last page is indistinguishable from a correct one.
#[test]
fn a_target_that_drops_a_partial_last_page_fails() {
    let suite = suite_of(MODEL);
    assert!(!paged_ids(&suite).is_empty());
    let wrong = run(
        &suite,
        &Jobs::new(Mode::DropsPartialLastPage, "job_id", &["job_id", "type"]),
    );
    assert!(
        !not_passed(&wrong).is_empty(),
        "a target that answers no partial last page passes every scenario: {wrong:#?}"
    );
}

/// A creating command with an input named like a paging parameter. `bound()` binds view parameters
/// from the command's settled input by name, so every read the design says "sends no paging
/// parameter" sends `size`, and the view's unpaged reads (`contains`, `counts`, `ranked`) are
/// half-paged.
#[test]
fn an_input_named_like_a_paging_parameter_is_not_sent_on_the_unpaged_reads() {
    let text = MODEL
        .replace(
            "    fields:\n      - {name: type, type: demo.jobs.JobType}\n    lifecycle:",
            "    fields:\n      - {name: type, type: demo.jobs.JobType}\n      - {name: size, type: Integer}\n    lifecycle:",
        )
        .replace(
            "    input:\n      - {name: type, type: demo.jobs.JobType}\n",
            "    input:\n      - {name: type, type: demo.jobs.JobType}\n      - {name: size, type: Integer}\n",
        )
        .replace(
            "sets: {type: input.type}",
            "sets: {type: input.type, size: input.size}",
        );
    let suite = suite_of(&text);
    assert!(!paged_ids(&suite).is_empty());
    let mut half_paged = Vec::new();
    for (id, scenario) in &suite.scenarios {
        let steps = &scenario.steps;
        for (position, step) in steps.iter().enumerate() {
            let (ScenarioStep::QueryView { params, .. }
            | ScenarioStep::EventuallyView { params, .. }) = step
            else {
                continue;
            };
            let is_page_read = params.contains_key("page");
            if !is_page_read && params.contains_key("size") {
                half_paged.push(format!("{id} step {position}: {params:?}"));
            }
        }
    }
    assert!(
        half_paged.is_empty(),
        "reads the design says send no paging parameter send `size`:\n{}",
        half_paged.join("\n")
    );
}
