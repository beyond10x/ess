//! Adversary, pass 2, story `view-paging-and-caller-filters` (`paging:`, ess/16).
//!
//! Wrong targets the synthesized page reads still let through after pass 1's corrections, and a
//! witness a paged view loses that the same view without `paging:` would never have been refused.

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

/// The #174 repro, ordered by a `title` the view projects in place of the identity.
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

/// The repro with no caller filter: the view's only parameters are the page and the size.
fn unfiltered_model() -> String {
    MODEL
        .replace(
            "      - {name: type, type: Optional<demo.jobs.JobType>}\n      - {name: page",
            "      - {name: page",
        )
        .replace("    filter: type == param.type\n", "")
}

// ---- a target -----------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Filters, orders by `key`, slices `page * size`, counts the filtered rows.
    Correct,
    /// Answers the first page whatever page is asked for.
    IgnoresPage,
    /// Skips `page` rows rather than `page * size`: the page number read as an offset
    /// (`OFFSET :page LIMIT :size`).
    OffsetIsPage,
}

struct Jobs {
    mode: Mode,
    /// The field the view is ordered by, ascending.
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
        Ok(ImplementationIdentity::new("adversary-paging-2", "1"))
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
                    Mode::Correct => page * size,
                    Mode::IgnoresPage => 0,
                    Mode::OffsetIsPage => page,
                };
                ordered.into_iter().skip(offset).take(size).collect()
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

/// Every page read the suite sends has `size: 1` or `page: first_page`, so `page * size` and
/// `page` are the same offset on each of them: a target that reads the page number as a row offset
/// (`OFFSET :page`) answers every read exactly as a correct one does. Telling them apart needs one
/// read past the first page with a size above one.
#[test]
fn adversary_paging_pass2_a_target_that_offsets_by_the_page_number_fails() {
    let suite = suite_of(MODEL);
    assert!(!paged_ids(&suite).is_empty(), "the view is read by page");
    let correct = run(
        &suite,
        &Jobs::new(Mode::Correct, "job_id", &["job_id", "type"]),
    );
    assert!(
        not_passed(&correct).is_empty(),
        "a correct target passes: {correct:#?}"
    );
    let wrong = run(
        &suite,
        &Jobs::new(Mode::OffsetIsPage, "job_id", &["job_id", "type"]),
    );
    assert!(
        !not_passed(&wrong).is_empty(),
        "a target that skips `page` rows instead of `page * size` passes every scenario: {wrong:#?}"
    );
}

/// Pass 1's correction: without the identity, `distinct_by` is every projected field — but only
/// where the scenario knows a literal of *each*. One projected field the command does not set
/// (an optional note) empties it, although `title` alone still tells the scenario's rows apart, and
/// a target answering the first page for every page passes again. The design's stated limit is a
/// view "whose rows the scenario cannot tell apart"; these rows can be.
#[test]
fn adversary_paging_pass2_ignoring_page_fails_when_one_projected_field_is_unset() {
    let text = untitled_identity_model()
        .replace(
            "      - {name: title, type: String}\n    lifecycle:",
            "      - {name: title, type: String}\n      - {name: note, type: Optional<String>}\n    lifecycle:",
        )
        .replace(
            "    fields:\n      - {name: title, type: String}\n      - {name: type, type: demo.jobs.JobType}\n",
            "    fields:\n      - {name: title, type: String}\n      - {name: type, type: demo.jobs.JobType}\n      - {name: note, type: Optional<String>}\n",
        );
    let suite = suite_of(&text);
    assert!(!paged_ids(&suite).is_empty(), "the view is read by page");
    let correct = run(
        &suite,
        &Jobs::new(Mode::Correct, "title", &["title", "type", "note"]),
    );
    assert!(
        not_passed(&correct).is_empty(),
        "a correct target passes: {correct:#?}"
    );
    let wrong = run(
        &suite,
        &Jobs::new(Mode::IgnoresPage, "title", &["title", "type", "note"]),
    );
    assert!(
        !not_passed(&wrong).is_empty(),
        "a target that answers the first page for every page passes every scenario: {wrong:#?}"
    );
}

/// A paged view with no caller filter: page and size are its only parameters. Sanity: a correct
/// target passes and one that ignores `page` fails.
#[test]
fn adversary_paging_pass2_an_unfiltered_paged_view_is_paged_and_checked() {
    let suite = suite_of(&unfiltered_model());
    assert!(!paged_ids(&suite).is_empty(), "the view is read by page");
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

/// `paging:` declares that a read sending neither parameter answers every row, and every other read
/// of the view relies on that. A `deletes:` branch is witnessed on every read-your-writes view read
/// whole — but "whole" is decided by `params.is_empty()`, so declaring `paging:` on an otherwise
/// parameterless view silently drops its deletion witness.
#[test]
fn adversary_paging_pass2_a_deleted_row_is_witnessed_absent_from_an_unfiltered_paged_view() {
    let text = unfiltered_model()
        .replace(
            "may: [demo.jobs.CreateJob]",
            "may: [demo.jobs.CreateJob, demo.jobs.RemoveJob]",
        )
        .replace(
            "events:\n",
            "  - name: demo.jobs.RemoveJob\n    input:\n      - {name: job_id, type: demo.jobs.JobId}\n    outcomes:\n      - {name: removed, deletes: demo.jobs.Job, instance: job_id}\nevents:\n",
        );
    let suite = suite_of(&text);
    let witnessed = suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(step, ScenarioStep::ExpectSubjectAbsent { view, .. }
                if view.to_string() == "demo.jobs.JobList")
        })
    });
    // Control: the same view without `paging:` (and so without its two parameters) is witnessed.
    let unpaged = text
        .replace("    paging: {page: page, size: size, total: true}\n", "")
        .replace(
            "    params:\n      - {name: page, type: Integer}\n      - {name: size, type: Integer}\n",
            "",
        );
    let control = suite_of(&unpaged).scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(step, ScenarioStep::ExpectSubjectAbsent { view, .. }
                if view.to_string() == "demo.jobs.JobList")
        })
    });
    assert!(control, "the control model witnesses the deletion");
    assert!(
        witnessed,
        "declaring `paging:` removed the deletion witness from a view whose unpaged read answers \
         every row"
    );
}
