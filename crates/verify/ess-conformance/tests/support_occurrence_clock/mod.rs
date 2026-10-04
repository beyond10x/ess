//! A scripted command clock and the two fixture models the occurrence-clock controls run against
//! (beyond10x/ess#244, `docs/design/expression-family-source22.md`, "A3: current time over stored
//! and related rows").
//!
//! The provider answers a different instant on every read and counts its reads, so a target that
//! reads it twice, reuses an earlier reading or substitutes another clock is told apart from one
//! that reads it once per decision.
#![allow(dead_code)]

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::occurrence_clock::{CommandClock, DecisionInstant};
use ess_conformance::target::{
    ConformanceTarget, ScenarioContext, SemanticCommandRequest, SemanticCommandResult, TargetError,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

/// Offers: `AcceptOffer` is accepted while `expires_at >= now`.
pub const OFFERS: &str = include_str!("../fixtures/occurrence-clock-offers.yaml");
/// Claims: `Settle` settles while `due_at >= now`, and a retried settlement replays.
pub const CLAIMS: &str = include_str!("../fixtures/occurrence-clock-claims.yaml");

/// The setup instant, before every deadline the controls send.
pub const T0: &str = "2000-06-01T00:00:00Z";
/// The deadline: strictly after [`T0`] and strictly before [`T1`].
pub const DEADLINE: &str = "2010-01-01T00:00:00Z";
/// The decision instant of the tested command, at full precision.
pub const T1: &str = "2026-10-04T12:00:00.123456789Z";

pub fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let specification = Specification::assemble([(Source::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

pub fn instant(text: &str) -> DecisionInstant {
    DecisionInstant::parse(text).unwrap_or_else(|error| panic!("`{text}`: {error}"))
}

/// A provider answering `script` in order and, after it, one second later on every further read.
/// Every read answers a different instant.
#[derive(Debug)]
pub struct Scripted {
    script: Vec<DecisionInstant>,
    reads: AtomicUsize,
}

impl Scripted {
    pub fn new(script: &[&str]) -> Arc<Self> {
        Arc::new(Self {
            script: script.iter().map(|text| instant(text)).collect(),
            reads: AtomicUsize::new(0),
        })
    }

    /// The setup instant, then the decision instant.
    pub fn setup_then_decision() -> Arc<Self> {
        Self::new(&[T0, T1])
    }

    pub fn reads(&self) -> usize {
        self.reads.load(Ordering::SeqCst)
    }

    /// What the `n`th read (from 0) answers.
    pub fn reading(&self, n: usize) -> DecisionInstant {
        self.script.get(n).copied().unwrap_or_else(|| {
            let last = *self.script.last().expect("a script names an instant");
            let later = i64::try_from(n + 1 - self.script.len()).expect("small");
            DecisionInstant::from_instant(
                last.instant()
                    .plus_seconds(later)
                    .expect("a later instant is spelled"),
            )
        })
    }
}

impl CommandClock for Scripted {
    fn read(&self) -> Option<DecisionInstant> {
        let n = self.reads.fetch_add(1, Ordering::SeqCst);
        Some(self.reading(n))
    }
}

/// The offers model executed by the interpreter with `clock`.
pub fn offers(clock: Option<Arc<Scripted>>) -> Interpreted {
    let target = Interpreted::for_model(model(OFFERS));
    let target = match clock {
        Some(clock) => target.with_command_clock(clock),
        None => target,
    };
    begin(&target);
    target
}

pub fn begin(target: &dyn ConformanceTarget) {
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.offers/authored/occurrence-clock".parse().unwrap(),
            CorrelationId::new("occurrence-clock").unwrap(),
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
        correlation: CorrelationId::new("occurrence-clock").unwrap(),
    }
}

pub fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

pub fn outcome(result: &SemanticCommandResult) -> String {
    result
        .outcome
        .as_ref()
        .map_or_else(|| "<none>".to_owned(), |taken| taken.outcome.to_string())
}

/// Opens an offer expiring at `expires_at` and returns its identity.
pub fn open(target: &dyn ConformanceTarget, expires_at: &str) -> String {
    let opened = target
        .execute_command(request(
            "demo.offers.OpenOffer",
            &[("expires_at", text(expires_at))],
        ))
        .unwrap_or_else(|error| panic!("the offer opens: {error}"));
    assert_eq!(outcome(&opened), "opened");
    opened.direct_events[0].payload["offer_id"]
        .as_text()
        .expect("a text identity")
        .to_owned()
}

pub fn accept(offer: &str, expires_at: &str) -> SemanticCommandRequest {
    request(
        "demo.offers.AcceptOffer",
        &[("offer_id", text(offer)), ("expires_at", text(expires_at))],
    )
}

/// `true` when the answer is the capability refusal an undecidable guard is reported as.
pub fn unsupported(answer: &Result<SemanticCommandResult, TargetError>) -> bool {
    matches!(answer, Err(TargetError::Unsupported { .. }))
}
