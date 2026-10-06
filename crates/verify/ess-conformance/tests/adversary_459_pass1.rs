//! Adversary pass 1 on W3-1 (beyond10x/ess#459): an `affects:` entry writing one row per element.
//!
//! * The witness picks elements whose read members differ between the held row's two calls; a
//!   `Boolean` member, built at two distinctions of one parity, never does.
//! * "Updated if held": a target that replaces the held row, dropping a field the entry does not
//!   write, is a create-only target the scenario has to fail.
//! * "A refused run writes none": a target that writes the element rows and then refuses.
//! * An element naming the branch's own subject: every other `affects:` entry excepts the subject.
use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{
    execute, execute_generating, Externals, Generated, GeneratedSlot, Store,
};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const EACH: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-each.yaml");
const RAN: &str = "demo.feed.RunSource/outcome/ran";
const DUPLICATED: &str = "demo.feed.RunSource/outcome/duplicated";
const NO_SOURCE: &str = "demo.feed.RunSource/outcome/no-such-source";

fn edited(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds:\n{from}");
    text.replacen(from, to, 1)
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("feed.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
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

// ---- the Boolean member ------------------------------------------------------------------------

/// The fixture with a `Boolean` member on the element and the row, which the entry reads.
fn with_boolean() -> String {
    let model = edited(
        EACH,
        "      - {name: revision, type: Integer}\nentities:",
        "      - {name: revision, type: Integer}\n      - {name: fresh, type: Boolean}\nentities:",
    );
    let model = edited(
        &model,
        "      - {name: revision, type: Integer}\n    lifecycle: {initial: Seen",
        "      - {name: revision, type: Integer}\n      - {name: fresh, type: Boolean}\n    lifecycle: {initial: Seen",
    );
    let model = edited(
        &model,
        "revision: doc.revision}",
        "revision: doc.revision, fresh: doc.fresh}",
    );
    format!("{model}      - {{name: fresh, type: Boolean}}\n")
}

/// An element member of type `Boolean` is as ordinary as `Integer`; the witness must still find a
/// held-row element and a second-call element that differ on it, rather than refuse the branch.
#[test]
fn each_entry_reading_a_boolean_member_is_witnessed() {
    let model = with_boolean();
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&model));
    assert!(
        synthesis.refusals.is_empty(),
        "a Boolean element member leaves `ran` unwitnessed: {:#?}",
        synthesis.refusals
    );
}

// ---- targets -----------------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mutation {
    None,
    /// A held row is replaced by the element's row, so a field the entry does not write is lost.
    Replaces,
    /// The element rows are written, then the repeated list is refused.
    WritesOnDuplicate,
    /// The element rows are written, then the unknown source is refused.
    WritesOnUnknownSource,
    /// An empty list for a held source is refused as a duplicate.
    RefusesEmptyList,
}

#[derive(Clone)]
struct Seen {
    id: Node,
    source: Node,
    hash: Node,
    revision: Node,
    note: Option<Node>,
}

struct FeedDesk {
    sources: RefCell<Vec<(String, String)>>,
    seen: RefCell<Vec<Seen>>,
    next: RefCell<usize>,
    mutation: Mutation,
    notes: bool,
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
    fn new(mutation: Mutation, notes: bool) -> Self {
        Self {
            sources: RefCell::default(),
            seen: RefCell::default(),
            next: RefCell::new(7001),
            mutation,
            notes,
        }
    }

    fn write(&self, source: &str, applied: &[Node], replace: bool) {
        let member = |item: &Node, name: &str| match item {
            Node::Map(members) => members.get(name).cloned().unwrap_or(Node::Null),
            _ => Node::Null,
        };
        let mut seen = self.seen.borrow_mut();
        for item in applied {
            let row = Seen {
                id: member(item, "document_id"),
                source: Node::Text(source.to_owned()),
                hash: member(item, "content_hash"),
                revision: member(item, "revision"),
                note: None,
            };
            match seen.iter().position(|held| held.id == row.id) {
                Some(at) if replace => seen[at] = row,
                Some(at) => {
                    let note = seen[at].note.clone();
                    seen[at] = Seen { note, ..row };
                }
                None => seen.push(row),
            }
        }
    }

    fn run(&self, command: &CommandRef, input: &BTreeMap<String, Node>) -> SemanticCommandResult {
        let applied = match input.get("applied") {
            Some(Node::Seq(items)) => items.clone(),
            _ => Vec::new(),
        };
        let ids: Vec<Node> = applied
            .iter()
            .map(|item| match item {
                Node::Map(members) => members.get("document_id").cloned().unwrap_or(Node::Null),
                _ => Node::Null,
            })
            .collect();
        let source = text(input, "source_id");
        if ids
            .iter()
            .enumerate()
            .any(|(at, id)| ids[..at].contains(id))
        {
            if self.mutation == Mutation::WritesOnDuplicate {
                self.write(&source, &applied, false);
            }
            return refused(command, "duplicated", "demo.feed.DuplicateDocument");
        }
        let held = self.sources.borrow().iter().any(|(id, _)| *id == source);
        if !held {
            if self.mutation == Mutation::WritesOnUnknownSource {
                self.write(&source, &applied, false);
            }
            return refused(command, "no-such-source", "demo.feed.NoSuchSource");
        }
        if applied.is_empty() && self.mutation == Mutation::RefusesEmptyList {
            return refused(command, "duplicated", "demo.feed.DuplicateDocument");
        }
        for held in self.sources.borrow_mut().iter_mut() {
            if held.0 == source {
                held.1 = text(input, "label");
            }
        }
        self.write(&source, &applied, self.mutation == Mutation::Replaces);
        took(
            command,
            "ran",
            "demo.feed.SourceRan",
            vec![("source_id", Node::Text(source))],
        )
    }

    fn annotate(
        &self,
        command: &CommandRef,
        input: &BTreeMap<String, Node>,
    ) -> SemanticCommandResult {
        let id = input.get("document_id").cloned().unwrap_or(Node::Null);
        let mut seen = self.seen.borrow_mut();
        let Some(row) = seen.iter_mut().find(|row| row.id == id) else {
            return refused(command, "no-such-document", "demo.feed.NoSuchDocument");
        };
        row.note = input.get("note").cloned();
        took(
            command,
            "annotated",
            "demo.feed.DocumentAnnotated",
            vec![("document_id", id)],
        )
    }
}

impl ConformanceTarget for FeedDesk {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("set-each-adversary", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
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
            "demo.feed.Annotate" => self.annotate(command, &request.input),
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
                    let mut out = BTreeMap::from([
                        ("document_id".into(), row.id.clone()),
                        ("source_id".into(), row.source.clone()),
                        ("content_hash".into(), row.hash.clone()),
                        ("revision".into(), row.revision.clone()),
                    ]);
                    if self.notes {
                        out.insert("note".into(), row.note.clone().unwrap_or(Node::Null));
                    }
                    out
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

// ---- a refused run writes none -------------------------------------------------------------------

/// The note: "A refused command changes nothing … so a refused run writes none of the rows." A
/// target that writes the element rows and then answers the refusal has to fail the refusal's
/// scenario; otherwise the suite never holds a target to the sentence.
#[test]
fn a_target_writing_element_rows_on_a_refused_run_fails() {
    let suite = ess_conformance::synthesize::synthesize(&ir_of(EACH)).suite;
    let honest = run(&suite, &FeedDesk::new(Mutation::None, false));
    for id in [RAN, DUPLICATED, NO_SOURCE] {
        assert_eq!(
            honest.get(id),
            Some(&Status::Passed),
            "premise, honest {id}: {honest:#?}"
        );
    }
    let survivors: Vec<String> = [
        (Mutation::WritesOnDuplicate, DUPLICATED),
        (Mutation::WritesOnUnknownSource, NO_SOURCE),
    ]
    .into_iter()
    .filter_map(|(mutation, id)| {
        let statuses = run(&suite, &FeedDesk::new(mutation, false));
        (statuses.get(id) != Some(&Status::Failed))
            .then(|| format!("{mutation:?}: `{id}` is {:?}", statuses.get(id)))
    })
    .collect();
    assert_eq!(
        survivors.len(),
        0,
        "targets writing element rows on a refused run pass the refusal's scenario:\n{}",
        survivors.join("\n")
    );
}

// ---- updated if held, not replaced -----------------------------------------------------------------

/// The fixture with an `Optional<String>` note on `SeenDocument`, published by the view and written
/// by another command, never by the `each:` entry.
fn with_note() -> String {
    let model = edited(
        EACH,
        "      - {name: revision, type: Integer}\n    lifecycle: {initial: Seen",
        "      - {name: revision, type: Integer}\n      - {name: note, type: Optional<String>}\n    lifecycle: {initial: Seen",
    );
    let model = edited(
        &model,
        "  - name: demo.feed.SourceRan\n",
        "  - name: demo.feed.DocumentAnnotated\n    fields: [{name: document_id, type: demo.feed.DocumentId}]\n  - name: demo.feed.SourceRan\n",
    );
    let model = edited(
        &model,
        "errors: [{name: demo.feed.DuplicateDocument}, {name: demo.feed.NoSuchSource}]",
        "errors: [{name: demo.feed.DuplicateDocument}, {name: demo.feed.NoSuchSource}, {name: demo.feed.NoSuchDocument}]",
    );
    let model = edited(
        &model,
        "    may: [demo.feed.AddSource, demo.feed.RunSource]",
        "    may: [demo.feed.AddSource, demo.feed.RunSource, demo.feed.Annotate]",
    );
    let model = edited(
        &model,
        "views:\n",
        "  - name: demo.feed.Annotate\n    input:\n      - {name: document_id, type: demo.feed.DocumentId}\n      - {name: note, type: String}\n    outcomes:\n      - name: annotated\n        updates: demo.feed.SeenDocument\n        instance: document_id\n        emits: [demo.feed.DocumentAnnotated]\n        payload: {demo.feed.DocumentAnnotated: {document_id: input.document_id}}\n        sets: {note: input.note}\n      - {name: no-such-document, unknown_instance: true, error: demo.feed.NoSuchDocument}\nviews:\n",
    );
    format!("{model}      - {{name: note, type: Optional<String>}}\n")
}

/// "Updated if held and created in `initial` if not": an update leaves a field the entry does not
/// write as it was. A target that answers every element by writing a fresh row over the held one —
/// a create-only target whose store overwrites on a repeated key — loses the held row's note, and
/// the `ran` scenario has to see that.
#[test]
fn a_target_replacing_the_held_row_fails_the_each_scenario() {
    let ir = ir_of(&with_note());
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    let honest = run(&suite, &FeedDesk::new(Mutation::None, true));
    assert_eq!(
        honest.get(RAN),
        Some(&Status::Passed),
        "premise, honest: {honest:#?}"
    );
    let replacing = run(&suite, &FeedDesk::new(Mutation::Replaces, true));
    assert_eq!(
        replacing.get(RAN),
        Some(&Status::Failed),
        "a target replacing the held row passes `ran`: {replacing:#?}"
    );
}

// ---- an element naming the subject -------------------------------------------------------------------

const TAG: &str = "format: ess/23
system: demo
version: v1
domain: demo.tag
types:
  - {name: demo.tag.ItemId, kind: newtype, of: String}
  - name: demo.tag.Entry
    kind: struct
    fields:
      - {name: item_id, type: demo.tag.ItemId}
      - {name: label, type: String}
entities:
  - name: demo.tag.Item
    identity: {name: item_id, type: demo.tag.ItemId}
    fields: [{name: label, type: String}]
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
events:
  - name: demo.tag.ItemAdded
    fields: [{name: item_id, type: demo.tag.ItemId}]
  - name: demo.tag.ItemsTagged
    fields: [{name: item_id, type: demo.tag.ItemId}]
errors: [{name: demo.tag.Repeated}, {name: demo.tag.NoSuchItem}]
actors:
  - name: demo.tag.Operator
    may: [demo.tag.AddItem, demo.tag.Tag]
commands:
  - name: demo.tag.AddItem
    input: [{name: label, type: String}]
    outcomes:
      - name: added
        creates: demo.tag.Item
        instance: item_id
        emits: [demo.tag.ItemAdded]
        payload: {demo.tag.ItemAdded: {item_id: {generated: true}}}
        sets: {label: input.label}
  - name: demo.tag.Tag
    input:
      - {name: item_id, type: demo.tag.ItemId}
      - {name: label, type: String}
      - {name: entries, type: List<demo.tag.Entry>}
    outcomes:
      - name: repeated
        when: {not: {distinct: {in: entries, as: e, by: e.item_id}}}
        error: demo.tag.Repeated
      - name: tagged
        updates: demo.tag.Item
        instance: item_id
        emits: [demo.tag.ItemsTagged]
        payload: {demo.tag.ItemsTagged: {item_id: input.item_id}}
        sets: {label: input.label}
        affects:
          - entity: demo.tag.Item
            each: {in: input.entries, as: e}
            instance: e.item_id
            sets: {label: e.label}
      - {name: no-such-item, unknown_instance: true, error: demo.tag.NoSuchItem}
";

/// Every other `affects:` entry excepts the branch's subject ("the subject itself excepted"), so
/// the subject holds what its own branch writes. An `each:` entry over the subject's entity whose
/// element names the subject must either be refused by validate or leave the subject its own
/// `sets:`; the interpreter instead lets the element overwrite it, an order no target is told.
#[test]
fn an_element_naming_the_subject_leaves_the_subject_its_own_writes() {
    let raw = RawSpecFile::parse(TAG).expect("the model parses");
    let Ok(spec) = Specification::assemble([(Source::new("tag.yaml"), raw)]) else {
        return; // Refused by validate: the subject cannot be overwritten.
    };
    let ir = compile(&spec, &SourceMap::new()).expect("the model compiles");
    let mut store = Store::default();
    let mut steps = execute_generating(
        &ir,
        &store,
        &"demo.tag.AddItem".parse().unwrap(),
        &BTreeMap::from([("label".to_owned(), Node::Text("before".into()))]),
        &Externals::Withheld,
        &Generated::Given(BTreeMap::from([(
            GeneratedSlot::new("demo.tag.ItemAdded".parse().unwrap(), "item_id"),
            Node::Text("i1".into()),
        )])),
    )
    .unwrap();
    store = steps.remove(0).next;
    let element = Node::Map(BTreeMap::from([
        ("item_id".to_owned(), Node::Text("i1".into())),
        ("label".to_owned(), Node::Text("element".into())),
    ]));
    let mut steps = execute(
        &ir,
        &store,
        &"demo.tag.Tag".parse().unwrap(),
        &BTreeMap::from([
            ("item_id".to_owned(), Node::Text("i1".into())),
            ("label".to_owned(), Node::Text("own".into())),
            ("entries".to_owned(), Node::Seq(vec![element])),
        ]),
        &Externals::Withheld,
    )
    .unwrap_or_else(|error| panic!("{error:?}"));
    let step = steps.remove(0);
    assert_eq!(
        step.outcome.as_ref().map(ToString::to_string).as_deref(),
        Some("demo.tag.Tag/tagged")
    );
    let subject = step
        .next
        .instance(&"demo.tag.Item".parse().unwrap(), "i1")
        .expect("the subject is held");
    assert_eq!(
        subject.fields["label"],
        Node::Text("own".into()),
        "the element overwrote the subject's own `sets:`: {subject:?}"
    );
}

// ---- the held-distinct rule, executed --------------------------------------------------------------

/// The fixture whose duplicate refusal answers only for a source labelled `locked`; validate admits
/// it (the domain case `adversary_459_pass1` of `ess-domain` holds that it should not).
fn distinct_behind_subject_guard() -> String {
    edited(
        EACH,
        "      - name: duplicated\n        when: {not: {distinct:",
        "      - name: duplicated\n        when_subject: {predicate: label == \"locked\"}\n        when: {not: {distinct:",
    )
}

/// Under that model a repeated `document_id` for an unlocked source reaches `ran`, and the row it
/// names holds whichever element came last: the order-dependent result the rule exists to exclude.
#[test]
fn a_repeated_member_reaches_the_each_branch_when_the_refusal_is_subject_scoped() {
    // Decided after adversary pass 1 (story:feature-request-459): a refusal also guarded by
    // `when_subject:` does not hold the member distinct, so the model this case built is refused
    // where the `each:` entry is declared, and no order-dependent row can be reached.
    let raw = RawSpecFile::parse(&distinct_behind_subject_guard()).expect("the model parses");
    let errors = Specification::assemble([(Source::new("feed.yaml"), raw)])
        .expect_err("a subject-scoped duplicate refusal leaves the each: entry undistinct");
    let text = errors.to_string();
    assert!(
        text.contains("missing_declaration") && text.contains("ran.affects[0].each"),
        "{text}"
    );
}

// ---- an empty list is an accepted answer --------------------------------------------------------------

/// The note: "An empty list writes nothing and is an accepted answer." A target refusing an empty
/// list for a held source has to fail some scenario of `RunSource`.
#[test]
fn a_target_refusing_an_empty_list_fails() {
    let suite = ess_conformance::synthesize::synthesize(&ir_of(EACH)).suite;
    let statuses = run(&suite, &FeedDesk::new(Mutation::RefusesEmptyList, false));
    assert!(
        statuses.values().any(|status| *status == Status::Failed),
        "a target refusing an empty list passes every scenario: {statuses:#?}"
    );
}
