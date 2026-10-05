//! The two row-set fixtures, the interpreter executing them, and the faulty implementations every
//! runner lane is held to (`docs/design/filtered-related-reads.md`, "Decisive acceptance";
//! `docs/design/expression-family-source22.md`, final decision 13; beyond10x/ess#228, #299).
//!
//! A fault is an implementation that decides one thing about a row set the specification does not:
//! it ignores one conjunct of the selector, copies a row the guard did not select, reads the store
//! after its own effect, miscounts, lets a row whose membership is unknown select, or lets a second
//! creation slip past uniqueness. Each is the interpreter running a model changed in exactly that
//! one place — the target's behaviour, never its report — or, where no model says the wrong thing
//! (an effect applied before the read), the interpreter whose answer is changed at that seam.
#![allow(dead_code)]

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::*;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

/// Attempts: `Retry` copies the delay of the one earlier attempt of the worker and batch.
pub const READS: &str = include_str!("../fixtures/filtered-related-reads.yaml");
/// Identities: `BindIdentity` refuses claims another user of the tenant carries.
pub const UNIQUE: &str = include_str!("../fixtures/unique-within-scope.yaml");

pub fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let specification = Specification::assemble([(Source::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

pub fn target(text: &str) -> Interpreted {
    let target = Interpreted::for_model(model(text));
    begin(&target);
    target
}

pub fn begin(target: &dyn ConformanceTarget) {
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.jobs/authored/row-sets".parse().unwrap(),
            CorrelationId::new("row-sets").unwrap(),
        ))
        .unwrap();
}

pub fn request(command: &str, input: &[(&str, Node)]) -> SemanticCommandRequest {
    SemanticCommandRequest {
        command: command.parse().unwrap(),
        actor: None,
        caller: None,
        input: input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect::<BTreeMap<_, _>>(),
        correlation: CorrelationId::new("row-sets").unwrap(),
    }
}

pub fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

pub fn number(value: i64) -> Node {
    Node::Number(ess_primitives::facts::Number::from(value))
}

pub fn outcome(result: &SemanticCommandResult) -> String {
    result
        .outcome
        .as_ref()
        .map_or_else(|| "<none>".to_owned(), |taken| taken.outcome.to_string())
}

/// Records an attempt and returns its identity.
pub fn record(
    target: &dyn ConformanceTarget,
    worker: &str,
    batch: &str,
    delay: i64,
    note: Option<&str>,
) -> String {
    let mut input = vec![
        ("worker_id", text(worker)),
        ("batch_id", text(batch)),
        ("delay", number(delay)),
    ];
    if let Some(note) = note {
        input.push(("note", text(note)));
    }
    let answer = target
        .execute_command(request("demo.jobs.Record", &input))
        .unwrap_or_else(|error| panic!("an attempt is recorded: {error}"));
    assert_eq!(outcome(&answer), "recorded");
    answer.direct_events[0].payload["attempt_id"]
        .as_text()
        .expect("a text identity")
        .to_owned()
}

pub fn retry(worker: &str, batch: &str) -> SemanticCommandRequest {
    request(
        "demo.jobs.Retry",
        &[("worker_id", text(worker)), ("batch_id", text(batch))],
    )
}

pub fn check_limit(worker: &str, batch: &str, limit: i64) -> SemanticCommandRequest {
    request(
        "demo.jobs.CheckLimit",
        &[
            ("worker_id", text(worker)),
            ("batch_id", text(batch)),
            ("limit", number(limit)),
        ],
    )
}

pub fn close(attempt: &str) -> SemanticCommandRequest {
    request("demo.jobs.Close", &[("attempt_id", text(attempt))])
}

pub fn bind(user: &str, tenant: &str, sub: &str, org: &str) -> SemanticCommandRequest {
    request(
        "demo.binding.BindIdentity",
        &[
            ("user_id", text(user)),
            ("tenant_id", text(tenant)),
            ("sub", text(sub)),
            ("org_id", text(org)),
        ],
    )
}

pub fn import(user: &str, tenant: &str, sub: &str) -> SemanticCommandRequest {
    request(
        "demo.binding.ImportIdentity",
        &[
            ("user_id", text(user)),
            ("tenant_id", text(tenant)),
            ("sub", text(sub)),
        ],
    )
}

/// Every row `view` lists, as its fields.
pub fn rows(target: &dyn ConformanceTarget, view: &str) -> Vec<BTreeMap<String, Node>> {
    target
        .query_view(SemanticViewRequest {
            view: view.parse().unwrap(),
            params: BTreeMap::new(),
            consistency: ess_primitives::consistency::QueryConsistency::Current,
            correlation: CorrelationId::new("row-sets").unwrap(),
            deadline: Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(0)),
        })
        .expect("the view answers")
        .rows
}

pub fn attempts(target: &dyn ConformanceTarget) -> Vec<BTreeMap<String, Node>> {
    rows(target, "demo.jobs.Attempts")
}

pub fn identities(target: &dyn ConformanceTarget) -> Vec<BTreeMap<String, Node>> {
    rows(target, "demo.binding.Identities")
}

// ---- faulty implementations ----------------------------------------------------------------------

/// One way of reading a row set wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// The selector ignores `worker_id == input.worker_id`.
    DropFirstConjunct,
    /// The selector ignores `batch_id == input.batch_id`.
    DropSecondConjunct,
    /// The guard selects the right rows, and the value is copied from a row differing in batch.
    CopyWrongRow,
    /// The retried attempt is stored before its delay is read, and the newest matching row — the
    /// one just stored, holding no delay of its own yet, read as zero — is copied.
    ReadAfterEffects,
    /// One matching row is counted as ambiguous: `count: {gt: 1}` read as `{gte: 1}`.
    WrongCardinality,
    /// A row whose stored `org_id` is absent is taken as carrying the claims: the unknown
    /// membership `org_id == input.org_id` leaves selects.
    UnknownSelects,
    /// The uniqueness check is never made: a second creation with taken claims is bound.
    SecondCreateSlips,
}

impl Fault {
    /// Every fault over the attempts fixture.
    pub const READS: [Self; 5] = [
        Self::DropFirstConjunct,
        Self::DropSecondConjunct,
        Self::CopyWrongRow,
        Self::ReadAfterEffects,
        Self::WrongCardinality,
    ];
    /// Every fault over the identities fixture.
    pub const UNIQUE: [Self; 2] = [Self::UnknownSelects, Self::SecondCreateSlips];

    /// The fixture this fault is a model of: the healthy text changed in exactly one place.
    pub fn text(self) -> String {
        const SELECTOR: &str = "{all: [worker_id == input.worker_id, batch_id == input.batch_id]}";
        let changed = match self {
            Self::DropFirstConjunct => {
                READS.replace(SELECTOR, "{all: [batch_id == input.batch_id]}")
            }
            Self::DropSecondConjunct => {
                READS.replace(SELECTOR, "{all: [worker_id == input.worker_id]}")
            }
            Self::CopyWrongRow => {
                // Only the reads of `retried`, never the guards: the guard selects the right rows.
                let (head, tail) = READS
                    .split_once("      - name: retried")
                    .expect("the retried branch");
                let (reads, rest) = tail
                    .split_once("  - name: demo.jobs.CheckLimit")
                    .expect("the next command");
                format!(
                    "{head}      - name: retried{}  - name: demo.jobs.CheckLimit{rest}",
                    reads.replace(
                        SELECTOR,
                        "{all: [worker_id == input.worker_id, batch_id != input.batch_id]}"
                    )
                )
            }
            Self::WrongCardinality => READS.replace("count: {gt: 1}", "count: {gte: 1}"),
            Self::ReadAfterEffects => READS.to_owned(),
            Self::UnknownSelects => UNIQUE.replace(
                "defined(org_id), org_id == input.org_id",
                "{any: [missing(org_id), org_id == input.org_id]}",
            ),
            Self::SecondCreateSlips => {
                let (head, tail) = UNIQUE
                    .split_once("      - name: claims-taken")
                    .expect("the uniqueness branch");
                let (_, rest) = tail
                    .split_once("      - name: bound")
                    .expect("the binding branch");
                format!("{head}      - name: bound{rest}")
            }
        };
        assert!(
            self == Self::ReadAfterEffects || changed != READS && changed != UNIQUE,
            "{self:?} changes its fixture"
        );
        changed
    }
}

/// The interpreter running a faulty model, or — for [`Fault::ReadAfterEffects`] — the healthy one
/// whose retried delay is read after the row it stores.
pub struct Faulty {
    inner: Interpreted,
    fault: Option<Fault>,
}

impl Faulty {
    pub fn new(fault: Option<Fault>) -> Self {
        let text = match fault {
            None => READS.to_owned(),
            Some(fault) => fault.text(),
        };
        Self {
            inner: Interpreted::for_model(model(&text)),
            fault,
        }
    }

    /// The healthy identities fixture, or one of its faults.
    pub fn unique(fault: Option<Fault>) -> Self {
        let text = match fault {
            None => UNIQUE.to_owned(),
            Some(fault) => fault.text(),
        };
        Self {
            inner: Interpreted::for_model(model(&text)),
            fault,
        }
    }
}

impl ConformanceTarget for Faulty {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut answer = self.inner.execute_command(request)?;
        if self.fault == Some(Fault::ReadAfterEffects) && outcome(&answer) == "retried" {
            // The row just stored is the newest match, and holds no delay yet.
            for event in &mut answer.direct_events {
                if event.payload.contains_key("delay") {
                    event.payload.insert("delay".to_owned(), number(0));
                }
            }
        }
        Ok(answer)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

/// The synthesized scenarios each fault fails at least (read off the run, then held).
pub const FAULT_KILLS: &[(Fault, &[&str])] = &[
    (
        Fault::DropFirstConjunct,
        &[
            "demo.jobs.Retry/outcome/retried",
            "demo.jobs.Retry/outcome/started",
        ],
    ),
    (
        Fault::DropSecondConjunct,
        &[
            "demo.jobs.Retry/outcome/retried",
            "demo.jobs.Retry/outcome/started",
        ],
    ),
    (Fault::CopyWrongRow, &["demo.jobs.Retry/outcome/retried"]),
    (
        Fault::ReadAfterEffects,
        &["demo.jobs.Retry/outcome/retried"],
    ),
    (
        Fault::WrongCardinality,
        &["demo.jobs.Retry/outcome/retried"],
    ),
    (
        Fault::UnknownSelects,
        &["demo.binding.BindIdentity/outcome/bound"],
    ),
    (
        Fault::SecondCreateSlips,
        &["demo.binding.BindIdentity/outcome/claims-taken"],
    ),
];
