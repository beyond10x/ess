//! Security pass 1 against story:a-no-view-probe-tells-from-states-apart and
//! story:a-no-view-held-state-refusal-probes-the-record: the shortest-sequence search behind the
//! no-view probe, on crafted lifecycles.
//!
//! * `wide(n)`: an item is made in `New`; `n` terminal places `P1..Pn` hang off it. `Note` refuses
//!   a blank note and otherwise updates the item in every state, so its own send tells nothing
//!   apart. `Ask{i}` answers `here` in `P{i}` and `elsewhere` in every other state, so each tells
//!   exactly one place apart from `New`, and the `n` asks together tell every one apart.
//! * `two()`: `Open` and `Closed`, and no send but `Note`'s, which runs from both.
//! * The search is bounded (`MAX_PROBE_SEARCHES` in `existence.rs`) and must be deterministic on
//!   a wide lifecycle: two syntheses give the same bytes.
use std::fmt::Write as _;
use std::time::{Duration, Instant};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::Synthesis;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const BLANK: &str = "demo.wide.Note/outcome/blank";

const HEAD: &str = r"format: ess/18
system: demo
version: v1
domain: demo.wide
summary: An item is made, then entered into one of several places; a blank note is refused.
";

const COMMON: &str = r#"errors:
  - {name: demo.wide.BlankNote, fields: []}
  - {name: demo.wide.UnknownItem, fields: []}
  - {name: demo.wide.Elsewhere, fields: []}
events:
  - name: demo.wide.Made
    fields: [{name: item_id, type: String}]
  - name: demo.wide.Entered
    fields: [{name: item_id, type: String}]
  - name: demo.wide.Noted
    fields: [{name: item_id, type: String}]
commands:
  - name: demo.wide.Make
    input: []
    outcomes:
      - name: made
        creates: demo.wide.Item
        instance: item_id
        emits: [demo.wide.Made]
        payload: {demo.wide.Made: {item_id: {generated: true}}}
  - name: demo.wide.Note
    input:
      - {name: id, type: String}
      - {name: note, type: String}
    outcomes:
      - {name: blank, when: note == "", error: demo.wide.BlankNote}
      - {name: unknown, unknown_instance: true, error: demo.wide.UnknownItem}
      - name: noted
        updates: demo.wide.Item
        instance: id
        emits: [demo.wide.Noted]
        payload: {demo.wide.Noted: {item_id: input.id}}
"#;

/// A command taking `transition`, granted to no actor: the refused `Note` has one, so the probe
/// may not send it (`reads_plainly`'s actor rule), and it only makes the transition taken.
fn ungranted(name: &str, transition: &str) -> String {
    format!(
        "  - name: demo.wide.{name}\n    input:\n      - {{name: id, type: String}}\n    \
         outcomes:\n      - name: entered\n        moves: demo.wide.Item.{transition}\n        \
         instance: id\n        emits: [demo.wide.Entered]\n        payload: {{demo.wide.Entered: \
         {{item_id: input.id}}}}\n"
    )
}

/// `n` places off `New`, each told apart from `New` by its own `Ask{i}` and by nothing else. No
/// view. The places are entered only through ungranted `Enter{i}`, so no send moves the item.
fn wide(n: usize) -> String {
    let places: Vec<String> = (1..=n).map(|i| format!("P{i}")).collect();
    let mut text = String::from(HEAD);
    text.push_str("entities:\n  - name: demo.wide.Item\n");
    text.push_str("    identity: {name: item_id, type: String}\n    fields: []\n");
    text.push_str("    lifecycle:\n      initial: New\n");
    write!(
        text,
        "      states: [New, {}]\n      terminal: [{}]\n      transitions:\n",
        places.join(", "),
        places.join(", ")
    )
    .expect("a String takes every write");
    for i in 1..=n {
        writeln!(text, "        - {{name: enter{i}, from: [New], to: P{i}}}")
            .expect("a String takes every write");
    }
    let asks: Vec<String> = (1..=n).map(|i| format!("demo.wide.Ask{i}")).collect();
    write!(
        text,
        "actors:\n  - name: demo.wide.Clerk\n    may: [demo.wide.Make, demo.wide.Note{}{}]\n",
        if asks.is_empty() { "" } else { ", " },
        asks.join(", ")
    )
    .expect("a String takes every write");
    text.push_str(COMMON);
    for i in 1..=n {
        let elsewhere: Vec<&str> = std::iter::once("New")
            .chain(
                places
                    .iter()
                    .filter(|place| **place != format!("P{i}"))
                    .map(String::as_str),
            )
            .collect();
        write!(text,
            "  - name: demo.wide.Ask{i}\n    input:\n      - {{name: id, type: String}}\n    \
             outcomes:\n      - {{name: unknown, unknown_instance: true, error: \
             demo.wide.UnknownItem}}\n      - name: here\n        when_subject_state: P{i}\n        \
             preserves: demo.wide.Item\n        instance: id\n      - {{name: elsewhere, \
             when_subject_state: [{}], error: demo.wide.Elsewhere}}\n",
            elsewhere.join(", ")
        ).expect("a String takes every write");
        text.push_str(&ungranted(&format!("Enter{i}"), &format!("enter{i}")));
    }
    text
}

/// `Open` and `Closed`, closed only by the ungranted `Close`; `Note` runs from both, and no
/// other command the probe may send reads the item. No view.
fn two() -> String {
    let mut text = String::from(HEAD);
    text.push_str(
        "entities:\n  - name: demo.wide.Item\n    identity: {name: item_id, type: String}\n    \
         fields: []\n    lifecycle:\n      initial: Open\n      states: [Open, Closed]\n      \
         terminal: [Closed]\n      transitions:\n        - {name: close, from: [Open], to: \
         Closed}\nactors:\n  - name: demo.wide.Clerk\n    may: [demo.wide.Make, \
         demo.wide.Note]\n",
    );
    text.push_str(COMMON);
    text.push_str(&ungranted("Close", "close"));
    text
}

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("wide.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|e| panic!("{e:?}\n{text}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&compiled(text))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

fn filed(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
}

/// The control: with six places the search finds the six asks, and the blank-note scenario is
/// filed with its arranged half, so the model shape is not what withdraws it below.
#[test]
fn six_places_each_told_by_its_own_ask_are_probed_and_filed() {
    let result = synthesis(&wide(6));
    assert!(filed(&result, BLANK), "{:#?}", refusals(&result));
}

/// The acceptance: the arranged half sends what tells the arranged state apart, "or the scenario
/// is refused with a cause naming the states it cannot tell apart". With seven places, seven
/// sends (one ask each) tell every place apart from `New`, and each place is told apart by a
/// single send, so no refusal may name a place as one no send tells apart.
#[test]
fn untold_names_no_state_a_single_send_tells_apart() {
    let result = synthesis(&wide(7));
    let lines = refusals(&result);
    let wrong: Vec<&String> = lines
        .iter()
        .filter(|line| {
            line.contains(BLANK)
                && line.contains("no send synthesis can make tells apart")
                && (1..=7).any(|i| line.contains(&format!("P{i}")))
        })
        .collect();
    assert!(
        wrong.is_empty() && filed(&result, BLANK),
        "{BLANK} withdrawn as UNTOLD naming places that `Ask{{i}}` tells apart, one send each; \
         seven sends tell all of them apart, so the cause is false where the search stops at its \
         cap (story:a-no-view-probe-tells-from-states-apart): {lines:#?}"
    );
}

/// The acceptance's other refusal: where the probing branch runs from several states and nothing
/// tells them apart, the cause names those states. Here `Note`'s own send is valid from `Open`
/// and answers alike in `Closed`, so the cause must name `Open` and `Closed`, not claim there is
/// no valid send.
#[test]
fn a_valid_send_that_tells_nothing_apart_is_refused_naming_the_states() {
    let result = synthesis(&two());
    let lines = refusals(&result);
    assert!(!filed(&result, BLANK), "{lines:#?}");
    assert!(
        lines.iter().any(|line| line.contains(BLANK)
            && line.contains("Open, Closed")
            && line.contains("no send synthesis can make tells apart")),
        "{BLANK} withdrawn without naming the states its valid `Note.noted` send cannot tell \
         apart (story:a-no-view-probe-tells-from-states-apart): {lines:#?}"
    );
}

/// The security angle: on a wide crafted lifecycle (41 states, 40 qualifying commands) synthesis
/// ends in bounded time and gives the same suite and the same refusals twice.
#[test]
fn the_probe_search_is_bounded_and_deterministic_on_a_wide_lifecycle() {
    let text = wide(40);
    let bound = Duration::from_secs(120);
    let mut runs = Vec::new();
    for _ in 0..2 {
        let started = Instant::now();
        let result = synthesis(&text);
        let took = started.elapsed();
        eprintln!("wide(40) synthesis took {took:?}");
        assert!(took < bound, "wide(40) synthesis took {took:?}");
        let suite = result
            .suite
            .to_canonical_json()
            .unwrap_or_else(|error| panic!("{error:?}"));
        runs.push((suite, refusals(&result)));
    }
    assert_eq!(runs[0], runs[1], "two syntheses of one model differ");
}
