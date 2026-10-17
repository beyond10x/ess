//! Adversary pass 1 on <https://github.com/beyond10x/ess/issues/539>: a `precedence-swap` scored
//! `equivalent` because branches answered before both take every input both select.
//!
//! The verdict claims the two branches' declaration order decides nothing a caller observes. Each
//! case here runs both orders through the reference interpreter and holds the audit to what they
//! answer: where the two orders answer one request differently, the swap is not `equivalent`.
#![allow(clippy::missing_panics_doc)]

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass};
use ess_conformance::target::{ConformanceTarget, ScenarioContext, SemanticCommandRequest};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use serde_json::Value;

/// A row-set command (`refused` reads a row set through the input) whose two accepting branches
/// address different rows: `book-paused` a book through `book`, `shelf-paused` a shelf through
/// `shelf`. `rushed`, an input-guarded refusal the plan reads before both, takes every input
/// `shelf-paused` selects, so it takes their whole overlap. `PAIR` is the two, in either order.
const SHELF: &str = r"format: ess/23
system: demo
version: v1
domain: demo.shelf
entities:
  - name: demo.shelf.Library
    identity: {name: library, type: String}
    fields:
      - {name: curator, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.shelf.Book
    identity: {name: book, type: String}
    fields: []
    lifecycle: {initial: Shelved, states: [Shelved], terminal: [Shelved]}
  - name: demo.shelf.Shelf
    identity: {name: shelf, type: String}
    fields: []
    lifecycle: {initial: Up, states: [Up], terminal: [Up]}
errors:
  - {name: demo.shelf.Refused, summary: A library of the curator is open., fields: []}
  - {name: demo.shelf.Unknown, summary: No row carries the identity., fields: []}
  - {name: demo.shelf.Rushed, summary: A rush is not taken., fields: []}
  - {name: demo.shelf.Idle, summary: Nothing was asked., fields: []}
events:
  - name: demo.shelf.LibraryOpened
    fields: [{name: library, type: String}]
  - name: demo.shelf.ShelfBuilt
    fields: [{name: shelf, type: String}]
  - name: demo.shelf.BookAdded
    fields: [{name: book, type: String}]
  - name: demo.shelf.BookPaused
    fields: [{name: book, type: String}]
  - name: demo.shelf.ShelfPaused
    fields: [{name: shelf, type: String}]
commands:
  - name: demo.shelf.OpenLibrary
    input:
      - {name: library, type: String}
      - {name: curator, type: String}
    outcomes:
      - name: opened
        creates: demo.shelf.Library
        instance: library
        sets: {curator: input.curator}
        emits: [demo.shelf.LibraryOpened]
        payload:
          demo.shelf.LibraryOpened: {library: input.library}
  - name: demo.shelf.BuildShelf
    input:
      - {name: shelf, type: String}
    outcomes:
      - name: built
        creates: demo.shelf.Shelf
        instance: shelf
        emits: [demo.shelf.ShelfBuilt]
        payload:
          demo.shelf.ShelfBuilt: {shelf: input.shelf}
  - name: demo.shelf.AddBook
    input:
      - {name: book, type: String}
    outcomes:
      - name: added
        creates: demo.shelf.Book
        instance: book
        emits: [demo.shelf.BookAdded]
        payload:
          demo.shelf.BookAdded: {book: input.book}
  - name: demo.shelf.Act
    input:
      - {name: book, type: String}
      - {name: shelf, type: String}
      - {name: curator, type: String}
      - {name: pause, type: Boolean}
      - {name: rush, type: Boolean}
    outcomes:
      - {name: unknown, unknown_instance: true, error: demo.shelf.Unknown}
      - name: refused
        when_related:
          entity: demo.shelf.Library
          where: curator == input.curator
          exists: true
        error: demo.shelf.Refused
      - {name: rushed, when: rush == true, error: demo.shelf.Rushed}
PAIR      - {name: idle, error: demo.shelf.Idle}
";

const BOOK_PAUSED: &str = "      - name: book-paused
        when: pause == true
        updates: demo.shelf.Book
        instance: book
        emits: [demo.shelf.BookPaused]
        payload:
          demo.shelf.BookPaused: {book: input.book}
";

const SHELF_PAUSED: &str = "      - name: shelf-paused
        when: rush == true
        updates: demo.shelf.Shelf
        instance: shelf
        emits: [demo.shelf.ShelfPaused]
        payload:
          demo.shelf.ShelfPaused: {shelf: input.shelf}
";

const SHELF_SWAP: &str = "precedence-swap/demo.shelf.Act/book-paused/shelf-paused";

/// `SHELF` with `book-paused` declared first where `book_first`, else `shelf-paused`: the
/// baseline, and the model the swap writes.
fn shelf(book_first: bool) -> String {
    let pair = if book_first {
        format!("{BOOK_PAUSED}{SHELF_PAUSED}")
    } else {
        format!("{SHELF_PAUSED}{BOOK_PAUSED}")
    };
    SHELF.replace("PAIR", &pair)
}

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let label = "model.yaml".to_owned();
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), text.to_owned());
    (vec![(Source::new(label), raw)], texts)
}

fn target(text: &str) -> Interpreted {
    let (files, texts) = documents(text);
    let ir = mutate::compile(files, &texts).unwrap_or_else(|error| panic!("{error:?}\n{text}"));
    let target = Interpreted::for_model(ir);
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.shelf/authored/adversary-l5".parse().unwrap(),
            CorrelationId::new("adversary-l5").unwrap(),
        ))
        .unwrap();
    target
}

/// The outcome `target` answers, or its error, as text.
fn answer(target: &dyn ConformanceTarget, command: &str, input: &[(&str, Node)]) -> String {
    let request = SemanticCommandRequest {
        command: command.parse().unwrap(),
        actor: None,
        caller: None,
        input: input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect(),
        correlation: CorrelationId::new("adversary-l5").unwrap(),
    };
    match target.execute_command(request) {
        Ok(result) => result
            .outcome
            .map_or_else(|| "<none>".to_owned(), |taken| taken.outcome.to_string()),
        Err(error) => format!("error: {error}"),
    }
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

/// What `Act` answers in the model declaring the pair in `book_first` order, on a store holding
/// shelf `s1` and no book, sent `book: b-missing, shelf: s1, pause: false, rush: false`: a
/// request that selects neither swapped branch.
fn act_on_a_held_shelf(book_first: bool) -> String {
    let target = target(&shelf(book_first));
    assert_eq!(
        answer(&target, "demo.shelf.BuildShelf", &[("shelf", text("s1"))]),
        "built"
    );
    answer(
        &target,
        "demo.shelf.Act",
        &[
            ("book", text("b-missing")),
            ("shelf", text("s1")),
            ("curator", text("nobody")),
            ("pause", Node::Bool(false)),
            ("rush", Node::Bool(false)),
        ],
    )
}

fn mutant<'a>(json: &'a Value, id: &str) -> &'a Value {
    json["mutants"]
        .as_array()
        .expect("mutants")
        .iter()
        .find(|it| it["id"] == id)
        .unwrap_or_else(|| panic!("no {id} in {json:#}"))
}

/// The two orders of `book-paused` and `shelf-paused` answer a request that selects neither of
/// them differently: on a row-set command the interpreter's `addressed_row` reads the row the
/// first declared accepting branch addresses, so the swap moves which row's absence refuses.
/// `rushed` taking the pair's whole overlap says nothing about that, so the swap is not
/// `equivalent`.
#[test]
fn a_swap_that_moves_the_addressed_row_is_not_scored_equivalent() {
    let baseline = act_on_a_held_shelf(true);
    let swapped = act_on_a_held_shelf(false);
    assert_ne!(
        baseline, swapped,
        "precondition: the two declaration orders answer the same request differently"
    );
    let (json, rendered) = swaps_audited(&shelf(true));
    let swap = mutant(&json, SHELF_SWAP);
    assert_ne!(
        swap["verdict"], "equivalent",
        "the baseline answers `{baseline}` and the swapped model `{swapped}` to one request, yet \
         the swap is scored equivalent: {swap:#}\n{rendered}"
    );
}

/// The audit of `text`'s precedence swaps against its own interpreter, as JSON, and its text.
fn swaps_audited(text: &str) -> (Value, String) {
    let (files, texts) = documents(text);
    let ir = mutate::compile(files.clone(), &texts).unwrap_or_else(|error| panic!("{error:?}"));
    let report = mutate::audit(&files, &texts, &[MutantClass::PrecedenceSwap], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}\n{text}"));
    let rendered = report.render_text();
    (
        serde_json::from_str(&report.to_canonical_json()).unwrap(),
        rendered,
    )
}

/// A swap of branches addressing different rows is killed by its suite and claims nothing about
/// its overlap: `rushed` takes every input both select, but the order still decides which row a
/// request neither of them takes reads, so the mutant carries no `overlap_taken_by` and its text
/// line names no branch taking it. The killed line that does name one (`render_text`'s branch for
/// a verdict other than `equivalent`) is held in `mutation_identical_answer`, on two branches
/// addressing the same row.
#[test]
fn a_killed_swap_of_branches_addressing_different_rows_names_no_overlap_taken() {
    let (json, rendered) = swaps_audited(&shelf(true));
    let swap = mutant(&json, SHELF_SWAP);
    assert_eq!(swap["verdict"], "killed", "{swap:#}\n{rendered}");
    assert!(swap.get("overlap_taken_by").is_none(), "{swap:#}");
    assert_ne!(json["format"], "ess-mutation-report/6", "{rendered}");
    let line = rendered
        .lines()
        .find(|line| line.starts_with(&format!("killed {SHELF_SWAP}")))
        .unwrap_or_else(|| panic!("no killed line for the swap:\n{rendered}"));
    assert!(
        !line.contains("taken first by") && !line.contains("ESS-MUTATE-005"),
        "{line}"
    );
}

/// A command whose branches each create a row, so no row has to be arranged: `paused`
/// (`pause == true`) and `defaulted` (`code` absent or `""`) are the swapped pair, `EARLIER` the
/// branches declared before both, and `plain` the default.
const DESK: &str = r"format: ess/23
system: demo
version: v1
domain: demo.desk
entities:
  - name: demo.desk.Ticket
    identity: {name: ticket, type: Uuid}
    fields: []
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: demo.desk.Early, summary: Refused early., fields: []}
events:
  - name: demo.desk.Early1
    fields: [{name: ticket, type: Uuid}]
  - name: demo.desk.Early2
    fields: [{name: ticket, type: Uuid}]
  - name: demo.desk.Paused
    fields: [{name: ticket, type: Uuid}]
  - name: demo.desk.Defaulted
    fields: [{name: ticket, type: Uuid}]
  - name: demo.desk.Plain
    fields: [{name: ticket, type: Uuid}]
commands:
  - name: demo.desk.File
    input:
      - {name: pause, type: Boolean}
      - {name: code, type: Optional<String>}
    outcomes:
EARLIER      - name: PAIR_ONE
PAIR_ONE_BODY      - name: PAIR_TWO
PAIR_TWO_BODY      - name: plain
        creates: demo.desk.Ticket
        instance: ticket
        emits: [demo.desk.Plain]
        payload:
          demo.desk.Plain: {ticket: {generated: true}}
";

const PAUSED_BODY: &str = "        when: pause == true
        creates: demo.desk.Ticket
        instance: ticket
        emits: [demo.desk.Paused]
        payload:
          demo.desk.Paused: {ticket: {generated: true}}
";

const DEFAULTED_BODY: &str = "        when: {any: [{code: {defined: false}}, 'code == \"\"']}
        creates: demo.desk.Ticket
        instance: ticket
        emits: [demo.desk.Defaulted]
        payload:
          demo.desk.Defaulted: {ticket: {generated: true}}
";

/// An accepting branch `name` declared before the pair, guarded by `when`, emitting `event`.
fn accepting(name: &str, when: &str, event: &str) -> String {
    format!(
        "      - name: {name}\n        when: {when}\n        creates: demo.desk.Ticket\n        \
         instance: ticket\n        emits: [demo.desk.{event}]\n        payload:\n          \
         demo.desk.{event}: {{ticket: {{generated: true}}}}\n"
    )
}

/// An input-guarded refusal `name` declared before the pair, guarded by `when`.
fn refusing(name: &str, when: &str) -> String {
    format!("      - {{name: {name}, when: {when}, error: demo.desk.Early}}\n")
}

/// `DESK` with `earlier` before the pair, `paused` first where `paused_first`.
fn desk(earlier: &str, paused_first: bool) -> String {
    let (one, one_body, two, two_body) = if paused_first {
        ("paused", PAUSED_BODY, "defaulted", DEFAULTED_BODY)
    } else {
        ("defaulted", DEFAULTED_BODY, "paused", PAUSED_BODY)
    };
    DESK.replace("EARLIER", earlier)
        .replace("PAIR_ONE_BODY", one_body)
        .replace("PAIR_TWO_BODY", two_body)
        .replace("PAIR_ONE", one)
        .replace("PAIR_TWO", two)
}

/// What `File` answers in `text` for `pause` and `code` (absent where `None`).
fn filed(text: &str, pause: bool, code: Option<&str>) -> String {
    let target = target(text);
    let mut input = vec![("pause", Node::Bool(pause))];
    if let Some(code) = code {
        input.push(("code", self::text(code)));
    }
    answer(&target, "demo.desk.File", &input)
}

/// Property: wherever the audit names branches taking a swap's overlap or scores it
/// `equivalent`, the two declaration orders answer every input alike. `pause` takes both values
/// and `code` is absent, `""` or another text — every class of input any guard here tells apart.
#[test]
fn a_swap_scored_equivalent_answers_every_input_alike_in_both_orders() {
    let earlier = [
        accepting(
            "early",
            "{all: ['pause == true', {any: [{code: {defined: false}}, 'code == \"\"']}]}",
            "Early1",
        ),
        accepting("early", "pause == true", "Early1"),
        accepting("early", "pause != false", "Early1"),
        format!(
            "{}{}",
            accepting(
                "early-absent",
                "{all: ['pause == true', {code: {defined: false}}]}",
                "Early1"
            ),
            accepting(
                "early-empty",
                "{all: ['pause == true', 'code == \"\"']}",
                "Early2"
            )
        ),
        accepting(
            "early",
            "{all: ['pause == true', 'code != \"x\"']}",
            "Early1",
        ),
        accepting(
            "early",
            "{all: ['pause == true', {code: {defined: false}}]}",
            "Early1",
        ),
        accepting("early", "{not: {all: ['pause == false']}}", "Early1"),
        refusing(
            "early",
            "{all: ['pause == true', {any: [{code: {defined: false}}, 'code == \"\"']}]}",
        ),
        format!(
            "{}{}",
            refusing("early-empty", "{all: ['pause == true', 'code == \"\"']}"),
            accepting(
                "early-absent",
                "{all: ['pause == true', {code: {defined: false}}]}",
                "Early1"
            )
        ),
    ];
    let swap = "precedence-swap/demo.desk.File/paused/defaulted";
    let mut claimed = 0;
    for earlier in &earlier {
        let baseline = desk(earlier, true);
        let swapped = desk(earlier, false);
        let (json, rendered) = swaps_audited(&baseline);
        let entry = mutant(&json, swap);
        if entry.get("overlap_taken_by").is_none() && entry["verdict"] != "equivalent" {
            continue;
        }
        claimed += 1;
        for pause in [false, true] {
            for code in [None, Some(""), Some("x")] {
                assert_eq!(
                    filed(&baseline, pause, code),
                    filed(&swapped, pause, code),
                    "pause {pause}, code {code:?}: the orders answer differently, yet {entry:#}\n\
                     {rendered}\n{baseline}"
                );
            }
        }
    }
    assert!(
        claimed >= 3,
        "the property was asked of {claimed} models only"
    );
}

/// A `precedence-swap` the manifest lists as stillborn — no suite, a `stillborn` cause — while it
/// carries `overlap_taken_by` is refused as incoherent: a stillborn mutant was never decided, and
/// `to_entry` would copy the member into a `stillborn` report entry. The same entry without it is
/// read, so the refusal is the member's.
#[test]
fn a_stillborn_swap_carrying_overlap_taken_by_is_refused() {
    let (files, texts) = documents(&desk(&accepting("early", "pause == true", "Early1"), true));
    let emission = mutate::emit_for(&files, &texts, &[MutantClass::PrecedenceSwap], None)
        .unwrap_or_else(|refusal| panic!("{refusal}"));
    assert_eq!(emission.manifest.format, "ess-mutation-manifest/6");
    let mut manifest: Value =
        serde_json::from_str(&emission.files[mutate::MANIFEST_FILE]).expect("the manifest");
    let swap = manifest["mutants"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|it| it["id"] == "precedence-swap/demo.desk.File/paused/defaulted")
        .expect("the swap");
    assert_eq!(
        swap["overlap_taken_by"],
        serde_json::json!(["early"]),
        "{swap:#}"
    );
    let object = swap.as_object_mut().unwrap();
    for key in [
        "dir",
        "refusals",
        "refused",
        "scenarios",
        "spec_digest",
        "suite_digest",
    ] {
        object.remove(key);
    }
    object.insert(
        "stillborn".to_owned(),
        serde_json::json!({"cause": "ESS-COMMAND-005", "code": "ESS-MUTATE-002", "message": "refused"}),
    );
    let refusal = mutate::Manifest::from_json(&manifest.to_string())
        .expect_err("a stillborn mutant carries no `overlap_taken_by`");
    assert!(refusal.contains("neither a stillborn mutant"), "{refusal}");
    let swap = manifest["mutants"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|it| it["id"] == "precedence-swap/demo.desk.File/paused/defaulted")
        .unwrap();
    swap.as_object_mut().unwrap().remove("overlap_taken_by");
    mutate::Manifest::from_json(&manifest.to_string())
        .unwrap_or_else(|refusal| panic!("the same stillborn entry without it is read: {refusal}"));
}
