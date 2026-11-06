//! An `each:` entry admitting repeated identities (`repeated: last_wins`, ess/35,
//! story:an-each-write-may-take-repeated-identities-in-list-order, beyond10x/ess#748 part 3).
//!
//! * The interpreter applies the elements one after another in list order, each to the row as the
//!   elements before it left it, so a row named twice holds what the last element wrote.
//! * Synthesis sends a list naming the held row twice, and its scenario reads the row back holding
//!   the last element's values: a target that keeps the first element for a repeated identity
//!   fails it, and one applying the list in order passes.
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{
    execute, execute_generating, Externals, Generated, GeneratedSlot, Store,
};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const EACH: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-each.yaml");
const DISTINCT: &str = "      - name: duplicated\n        when: {not: {distinct: {in: applied, as: d, by: d.document_id}}}\n        error: demo.feed.DuplicateDocument\n";
const RAN: &str = "demo.feed.RunSource/outcome/ran";

/// The fixture at ess/35 with no `distinct:` refusal and `repeated: last_wins` on its entry.
fn repeated() -> String {
    let edit = |text: String, from: &str, to: &str| {
        assert!(text.contains(from), "the fixture holds:\n{from}");
        text.replacen(from, to, 1)
    };
    let text = edit(EACH.to_owned(), "format: ess/23", "format: ess/35");
    let text = edit(text, DISTINCT, "");
    edit(
        text,
        "each: {in: input.applied, as: doc}",
        "each: {in: input.applied, as: doc, repeated: last_wins}",
    )
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("feed.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn element(id: &str, hash: &str, revision: i64) -> Node {
    Node::Map(BTreeMap::from([
        ("document_id".to_owned(), Node::Text(id.into())),
        ("content_hash".to_owned(), Node::Text(hash.into())),
        ("revision".to_owned(), Node::Number(Number::from(revision))),
    ]))
}

// ---- the interpreter ---------------------------------------------------------------------------

#[test]
fn the_interpreter_applies_a_repeated_identity_in_list_order() {
    let ir = ir_of(&repeated());
    let mut steps = execute_generating(
        &ir,
        &Store::default(),
        &"demo.feed.AddSource".parse().unwrap(),
        &BTreeMap::from([("label".to_owned(), Node::Text("feed".into()))]),
        &Externals::Withheld,
        &Generated::Given(BTreeMap::from([(
            GeneratedSlot::new("demo.feed.SourceAdded".parse().unwrap(), "source_id"),
            Node::Text("s1".into()),
        )])),
    )
    .unwrap_or_else(|error| panic!("{error:?}"));
    let store = steps.remove(0).next;
    let mut steps = execute(
        &ir,
        &store,
        &"demo.feed.RunSource".parse().unwrap(),
        &BTreeMap::from([
            ("source_id".to_owned(), Node::Text("s1".into())),
            ("label".to_owned(), Node::Text("ran".into())),
            (
                "applied".to_owned(),
                Node::Seq(vec![
                    element("d1", "first", 1),
                    element("d2", "other", 2),
                    element("d1", "last", 3),
                ]),
            ),
        ]),
        &Externals::Withheld,
    )
    .unwrap_or_else(|error| panic!("{error:?}"));
    let step = steps.remove(0);
    assert_eq!(
        step.outcome.as_ref().map(ToString::to_string).as_deref(),
        Some("demo.feed.RunSource/ran"),
        "a list naming one row twice reaches the accepting branch"
    );
    let seen = "demo.feed.SeenDocument".parse().unwrap();
    let d1 = step.next.instance(&seen, "d1").expect("d1 is held");
    assert_eq!(
        d1.fields["revision"],
        Node::Number(Number::from(3_i64)),
        "{d1:?}"
    );
    assert_eq!(
        d1.fields["content_hash"],
        Node::Text("last".into()),
        "{d1:?}"
    );
    let d2 = step.next.instance(&seen, "d2").expect("d2 is held");
    assert_eq!(
        d2.fields["revision"],
        Node::Number(Number::from(2_i64)),
        "{d2:?}"
    );
    assert_eq!(
        step.next
            .instances()
            .filter(|(entity, _, _)| **entity == seen)
            .count(),
        2,
        "a repeated identity is one row"
    );
}

// ---- synthesis -----------------------------------------------------------------------------------

/// What a target does with an element naming a row an earlier element of the same list named.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Repeats {
    /// Applied in list order: the last element's values win.
    InOrder,
    /// Skipped: the first element's values stay.
    KeepsFirst,
}

#[derive(Clone)]
struct Seen {
    id: Node,
    source: Node,
    hash: Node,
    revision: Node,
}

struct FeedDesk {
    sources: RefCell<Vec<(String, String)>>,
    seen: RefCell<Vec<Seen>>,
    next: RefCell<usize>,
    repeats: Repeats,
    /// The scenario running, and every scenario that sent a list naming one row twice.
    scenario: RefCell<String>,
    sent_a_repeat: RefCell<BTreeSet<String>>,
}

fn text(input: &BTreeMap<String, Node>, field: &str) -> String {
    input
        .get(field)
        .and_then(Node::as_text)
        .unwrap_or_default()
        .to_owned()
}

fn took(
    command: &CommandRef,
    outcome: &str,
    event: &str,
    payload: Vec<(&str, Node)>,
) -> SemanticCommandResult {
    let mut result =
        SemanticCommandResult::took(OutcomeRef::new(command.clone(), outcome.parse().unwrap()));
    result.consistency = Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
    let mut observed = ObservedEvent::new(event.parse().unwrap());
    for (key, value) in payload {
        observed.payload.insert(key.to_owned(), value);
    }
    result.direct_events.push(observed);
    result
}

fn refused(command: &CommandRef, outcome: &str, error: &str) -> SemanticCommandResult {
    let mut result =
        SemanticCommandResult::took(OutcomeRef::new(command.clone(), outcome.parse().unwrap()));
    result.error = Some(DeclaredErrorValue::new(error.parse().unwrap()));
    result.consistency = Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
    result
}

impl FeedDesk {
    fn new(repeats: Repeats) -> Self {
        Self {
            sources: RefCell::default(),
            seen: RefCell::default(),
            next: RefCell::new(7001),
            repeats,
            scenario: RefCell::default(),
            sent_a_repeat: RefCell::default(),
        }
    }

    fn run(&self, command: &CommandRef, input: &BTreeMap<String, Node>) -> SemanticCommandResult {
        let source = text(input, "source_id");
        if !self.sources.borrow().iter().any(|(id, _)| *id == source) {
            return refused(command, "no-such-source", "demo.feed.NoSuchSource");
        }
        for held in self.sources.borrow_mut().iter_mut() {
            if held.0 == source {
                held.1 = text(input, "label");
            }
        }
        let applied = match input.get("applied") {
            Some(Node::Seq(items)) => items.clone(),
            _ => Vec::new(),
        };
        let member = |item: &Node, name: &str| match item {
            Node::Map(members) => members.get(name).cloned().unwrap_or(Node::Null),
            _ => Node::Null,
        };
        let mut named = Vec::new();
        let mut seen = self.seen.borrow_mut();
        for item in &applied {
            let row = Seen {
                id: member(item, "document_id"),
                source: Node::Text(source.clone()),
                hash: member(item, "content_hash"),
                revision: member(item, "revision"),
            };
            if named.contains(&row.id) {
                self.sent_a_repeat
                    .borrow_mut()
                    .insert(self.scenario.borrow().clone());
                if self.repeats == Repeats::KeepsFirst {
                    continue;
                }
            }
            named.push(row.id.clone());
            match seen.iter().position(|held| held.id == row.id) {
                Some(at) => seen[at] = row,
                None => seen.push(row),
            }
        }
        took(
            command,
            "ran",
            "demo.feed.SourceRan",
            vec![("source_id", Node::Text(source))],
        )
    }
}

impl ConformanceTarget for FeedDesk {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("each-repeated", "1"))
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        *self.scenario.borrow_mut() = context.scenario.to_string();
        self.sources.borrow_mut().clear();
        self.seen.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = &request.command;
        Ok(match command.to_string().as_str() {
            "demo.feed.AddSource" => {
                let mut next = self.next.borrow_mut();
                *next += 11;
                let id = format!("target-source-{next}");
                self.sources
                    .borrow_mut()
                    .push((id.clone(), text(&request.input, "label")));
                took(
                    command,
                    "added",
                    "demo.feed.SourceAdded",
                    vec![("source_id", Node::Text(id))],
                )
            }
            "demo.feed.RunSource" => self.run(command, &request.input),
            other => panic!("unexpected command {other}"),
        })
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows = match request.view.to_string().as_str() {
            "demo.feed.Sources" => self
                .sources
                .borrow()
                .iter()
                .map(|(id, label)| {
                    BTreeMap::from([
                        ("source_id".into(), Node::Text(id.clone())),
                        ("label".into(), Node::Text(label.clone())),
                    ])
                })
                .collect(),
            "demo.feed.SeenDocuments" => self
                .seen
                .borrow()
                .iter()
                .map(|row| {
                    BTreeMap::from([
                        ("document_id".into(), row.id.clone()),
                        ("source_id".into(), row.source.clone()),
                        ("content_hash".into(), row.hash.clone()),
                        ("revision".into(), row.revision.clone()),
                    ])
                })
                .collect(),
            other => panic!("unexpected view {other}"),
        };
        Ok(SemanticViewResult { rows, total: None })
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "unused"))
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

#[test]
fn synthesis_sends_a_repeated_identity_and_fails_a_target_keeping_the_first() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&repeated()));
    assert!(
        synthesis.refusals.is_empty(),
        "the repeated entry is witnessed: {:#?}",
        synthesis.refusals
    );
    let in_order = FeedDesk::new(Repeats::InOrder);
    let honest = run(&synthesis.suite, &in_order);
    assert!(
        honest.values().all(|status| *status == Status::Passed),
        "a target applying the list in order passes every scenario: {honest:#?}"
    );
    assert!(
        in_order.sent_a_repeat.borrow().contains(RAN),
        "`ran` sends a list naming one row twice: {:?}",
        in_order.sent_a_repeat.borrow()
    );
    let keeps_first = run(&synthesis.suite, &FeedDesk::new(Repeats::KeepsFirst));
    assert_eq!(
        keeps_first.get(RAN),
        Some(&Status::Failed),
        "a target keeping the first element of a repeated identity passes `ran`: {keeps_first:#?}"
    );
}

/// Without the key the accepting branch's witness is the one it always was: the list it sends
/// names no row twice (only the `distinct:` refusal's scenario sends a repeat).
#[test]
fn without_the_key_the_accepting_witness_names_no_row_twice() {
    let model = EACH.replacen("format: ess/23", "format: ess/35", 1);
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&model));
    let target = FeedDesk::new(Repeats::InOrder);
    let statuses = run(&synthesis.suite, &target);
    assert_eq!(statuses.get(RAN), Some(&Status::Passed), "{statuses:#?}");
    assert!(
        !target.sent_a_repeat.borrow().contains(RAN),
        "{:?}",
        target.sent_a_repeat.borrow()
    );
}
