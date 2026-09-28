//! The generated Go runtime reads a page of a paged view (`page`, `snapshot_view`, the answer's
//! total; suite/26, beyond10x/ess#174) as the reference runner does (beyond10x/ess#188).
//!
//! The target is `tests/view_paging.rs`'s, with every way it pages wrong, recorded once and
//! replayed to the Go runtime.

mod support_go;

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/view-paging.yaml");

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("view-paging.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn suite() -> ConformanceSuite {
    let suite = ess_conformance::synthesize::synthesize(&ir_of(MODEL)).suite;
    assert!(ess_conformance::view_paging::used_by(&suite));
    suite
}

#[test]
fn go_gives_the_reference_verdict_for_every_paging_mode() {
    let suite = suite();
    let mut caught = 0;
    for mode in [
        Mode::Correct,
        Mode::Shared,
        Mode::IgnoresPaging,
        Mode::IgnoresPage,
        Mode::OneBased,
        Mode::PagesBeforeOrdering,
        Mode::TotalIsPageLength,
        Mode::NoTotal,
    ] {
        let verdicts = support_go::assert_parity(
            &format!("paging-{mode:?}").to_lowercase(),
            &suite,
            Jobs::new(mode),
        );
        let wrong = support_go::not_passed(&verdicts);
        if matches!(mode, Mode::Correct | Mode::Shared) {
            assert!(wrong.is_empty(), "{mode:?}: {verdicts:?}");
        } else if !wrong.is_empty() {
            caught += 1;
        }
    }
    assert_eq!(
        caught, 6,
        "every wrong mode fails a scenario on both runners"
    );
}

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
