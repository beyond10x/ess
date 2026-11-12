//! A quantifier over a command-input collection is witnessed with several elements, and a stored
//! collection a quantifier cannot be spread over says so
//! (`story:collection-quantifiers-witnessed-everywhere`, beyond10x/ess#240 correction 1).
//!
//! The deciding element sits neither first nor last in a list of at least three, so a target that
//! reads only the first element, only the last, or `forall` for `exists` (and the reverse) answers
//! the scenario wrongly.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Note, Synthesis};
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const BLOCKED: &str = "{exists: {in: input.tags, as: t, that: 't == \"blocked\"'}}";
const UNCHECKED: &str = "{not: {forall: {in: input.tags, as: t, that: 't != \"\"'}}}";

const TAGS: &str = "format: ess/32
system: demo
version: v1
domain: demo.files
types:
  - {name: demo.files.ItemId, kind: newtype, of: Uuid}
errors:
  - {name: demo.files.Blocked, summary: A tag blocks the item., fields: []}
  - {name: demo.files.Unchecked, summary: A tag is empty., fields: []}
entities:
  - name: demo.files.Item
    identity: {name: item_id, type: demo.files.ItemId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Filed, states: [Filed], terminal: [Filed]}
events:
  - name: demo.files.ItemFiled
    fields: [{name: item_id, type: demo.files.ItemId}]
actors:
  - {name: demo.files.Clerk, may: [demo.files.FileItem]}
commands:
  - name: demo.files.FileItem
    input:
      - {name: label, type: String}
      - {name: tags, type: 'List<String>'}
    outcomes:
      - name: blocked
        when: {blocked}
        error: demo.files.Blocked
      - name: unchecked
        when: {unchecked}
        error: demo.files.Unchecked
      - name: filed
        creates: demo.files.Item
        instance: item_id
        emits: [demo.files.ItemFiled]
        payload:
          demo.files.ItemFiled: {item_id: {generated: true}}
        sets: {label: input.label}
";

const LINES: &str = "format: ess/32
system: demo
version: v1
domain: demo.files
types:
  - {name: demo.files.ItemId, kind: newtype, of: Uuid}
  - name: demo.files.Line
    kind: struct
    fields:
      - {name: sku, type: String}
      - {name: codes, type: 'List<String>'}
errors:
  - {name: demo.files.Blocked, summary: A code blocks the item., fields: []}
entities:
  - name: demo.files.Item
    identity: {name: item_id, type: demo.files.ItemId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Filed, states: [Filed], terminal: [Filed]}
events:
  - name: demo.files.ItemFiled
    fields: [{name: item_id, type: demo.files.ItemId}]
actors:
  - {name: demo.files.Clerk, may: [demo.files.FileOrder]}
commands:
  - name: demo.files.FileOrder
    input:
      - {name: label, type: String}
      - {name: lines, type: 'List<demo.files.Line>'}
    outcomes:
      - name: blocked
        when:
          exists:
            in: input.lines
            as: l
            that: {exists: {in: l.codes, as: c, that: 'c == \"blocked\"'}}
        error: demo.files.Blocked
      - name: filed
        creates: demo.files.Item
        instance: item_id
        emits: [demo.files.ItemFiled]
        payload:
          demo.files.ItemFiled: {item_id: {generated: true}}
        sets: {label: input.label}
";

const MAP: &str = "format: ess/18
system: demo
version: v1
domain: demo.apps
types:
  - {name: demo.apps.AccountId, kind: newtype, of: Uuid}
errors:
  - {name: demo.apps.Refused, summary: Refused., fields: []}
entities:
  - name: demo.apps.Account
    identity: {name: account_id, type: demo.apps.AccountId}
    fields:
      - {name: redirect_uris, type: 'Map<Boolean, String>'}
      - {name: last, type: 'Optional<String>'}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
events:
  - name: demo.apps.Registered
    fields: [{name: account_id, type: demo.apps.AccountId}]
  - name: demo.apps.Launched
    fields: [{name: account_id, type: demo.apps.AccountId}]
actors:
  - {name: demo.apps.Client, may: [demo.apps.Register, demo.apps.Launch]}
commands:
  - name: demo.apps.Register
    input:
      - {name: redirect_uris, type: 'Map<Boolean, String>'}
    outcomes:
      - name: registered
        creates: demo.apps.Account
        instance: account_id
        emits: [demo.apps.Registered]
        payload:
          demo.apps.Registered: {account_id: {generated: true}}
        sets:
          redirect_uris: input.redirect_uris
  - name: demo.apps.Launch
    input:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: String}
    outcomes:
      - name: refused
        when_subject: {predicate: {exists: {in: redirect_uris, as: r, that: r == input.application}}}
        error: demo.apps.Refused
      - name: launched
        updates: demo.apps.Account
        instance: account_id
        emits: [demo.apps.Launched]
        payload:
          demo.apps.Launched: {account_id: input.account_id}
        sets:
          last: input.application
views:
  - name: demo.apps.Accounts
    source: demo.apps.Account
    consistency: read_your_writes
    fields:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: state, type: demo.apps.Account.State}
      - {name: redirect_uris, type: 'Map<Boolean, String>'}
      - {name: last, type: 'Optional<String>'}
";

fn tags(blocked: &str, unchecked: &str) -> String {
    TAGS.replace("{blocked}", blocked)
        .replace("{unchecked}", unchecked)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("input-lists.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn synthesized(text: &str) -> Synthesis {
    let result = synthesize(&ir(text));
    assert!(
        result.refusals.is_empty(),
        "no refusal: {:?}",
        result.refusals
    );
    result
}

/// What the suite sends at `field` of the command its scenario `id` runs.
fn sent(result: &Synthesis, id: &str, field: &str) -> Node {
    let scenario = result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| {
            let held: Vec<String> = result
                .suite
                .scenarios
                .keys()
                .map(ToString::to_string)
                .collect();
            panic!("{id} is in the suite: {held:?}")
        })
        .1;
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => match input.get(field) {
                Some(ScenarioValue::Literal { value }) => Some(value.clone()),
                _ => None,
            },
            _ => None,
        })
        .unwrap_or_else(|| panic!("{id} sends a literal {field}"))
}

fn items(node: Node) -> Vec<Node> {
    match node {
        Node::Seq(items) => items,
        other => panic!("not a list: {other:?}"),
    }
}

/// The index of the one element of `list` equal to `deciding`, asserting the list holds at least
/// three elements and that index is neither first nor last.
fn middle(list: &[Node], deciding: &str) -> usize {
    assert!(list.len() >= 3, "at least three elements: {list:?}");
    let at: Vec<usize> = list
        .iter()
        .enumerate()
        .filter(|(_, item)| **item == Node::Text(deciding.to_owned()))
        .map(|(at, _)| at)
        .collect();
    assert_eq!(at.len(), 1, "one {deciding:?} in {list:?}");
    assert!(
        at[0] != 0 && at[0] != list.len() - 1,
        "{deciding:?} is neither first nor last in {list:?}"
    );
    at[0]
}

fn statuses(result: &Synthesis, target: &str) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir(target)))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

const BLOCKED_ID: &str = "demo.files.FileItem/outcome/blocked";
const UNCHECKED_ID: &str = "demo.files.FileItem/outcome/unchecked";

/// The `blocked` and `unchecked` lists the suite sends, and where each one's deciding element is.
fn deciding(result: &Synthesis) -> ((Vec<Node>, usize), (Vec<Node>, usize)) {
    let blocked = items(sent(result, BLOCKED_ID, "tags"));
    let at_blocked = middle(&blocked, "blocked");
    let unchecked = items(sent(result, UNCHECKED_ID, "tags"));
    let at_unchecked = middle(&unchecked, "");
    ((blocked, at_blocked), (unchecked, at_unchecked))
}

#[test]
fn the_blocked_scenario_holds_a_tag_that_is_neither_first_nor_last() {
    let result = synthesized(&tags(BLOCKED, UNCHECKED));
    middle(&items(sent(&result, BLOCKED_ID, "tags")), "blocked");
}

#[test]
fn the_unchecked_scenario_holds_an_empty_tag_that_is_neither_first_nor_last() {
    let result = synthesized(&tags(BLOCKED, UNCHECKED));
    middle(&items(sent(&result, UNCHECKED_ID, "tags")), "");
}

#[test]
fn the_model_it_was_synthesized_from_passes_every_scenario() {
    let result = synthesized(&tags(BLOCKED, UNCHECKED));
    for (id, status) in statuses(&result, &tags(BLOCKED, UNCHECKED)) {
        assert_eq!(status, Status::Passed, "{id}");
    }
}

fn assert_killed(result: &Synthesis, mutant: &str, label: &str) {
    let run = statuses(result, mutant);
    for id in [BLOCKED_ID, UNCHECKED_ID] {
        assert_ne!(
            run.get(id),
            Some(&Status::Passed),
            "{label}: {id} fails against the mutant, got {run:?}"
        );
    }
}

#[test]
fn a_target_reading_only_the_first_tag_fails_the_suite() {
    let result = synthesized(&tags(BLOCKED, UNCHECKED));
    assert_killed(
        &result,
        &tags("'input.tags.0 == \"blocked\"'", "'input.tags.0 == \"\"'"),
        "first",
    );
}

#[test]
fn a_target_reading_only_the_last_tag_fails_the_suite() {
    let result = synthesized(&tags(BLOCKED, UNCHECKED));
    let ((blocked, _), (unchecked, _)) = deciding(&result);
    let mutant = tags(
        &format!("'input.tags.{} == \"blocked\"'", blocked.len() - 1),
        &format!("'input.tags.{} == \"\"'", unchecked.len() - 1),
    );
    assert_killed(&result, &mutant, "last");
}

#[test]
fn a_target_swapping_forall_and_exists_fails_the_suite() {
    let result = synthesized(&tags(BLOCKED, UNCHECKED));
    let swapped = |guard: &str| {
        guard
            .replace("exists", "@")
            .replace("forall", "exists")
            .replace('@', "forall")
    };
    assert_killed(
        &result,
        &tags(&swapped(BLOCKED), &swapped(UNCHECKED)),
        "swapped",
    );
}

#[test]
fn a_quantifier_over_a_member_of_a_bound_element_is_witnessed_with_several_codes() {
    let result = synthesized(LINES);
    let lines = items(sent(
        &result,
        "demo.files.FileOrder/outcome/blocked",
        "lines",
    ));
    let deciding: Vec<&Node> = lines
        .iter()
        .filter(|line| format!("{line:?}").contains("blocked"))
        .collect();
    assert_eq!(deciding.len(), 1, "one line holds the code: {lines:?}");
    let Node::Map(members) = deciding[0] else {
        panic!("a line is a struct: {:?}", deciding[0]);
    };
    middle(&items(members["codes"].clone()), "blocked");
}

fn unspread(result: &Synthesis) -> Vec<String> {
    result
        .notes
        .iter()
        .filter(|note| matches!(note, Note::UnspreadCollection { .. }))
        .map(ToString::to_string)
        .collect()
}

#[test]
fn a_quantifier_over_a_text_keyed_map_a_row_spreads_over_gets_no_fallback_note() {
    let result = synthesize(&ir(
        &MAP.replace("Map<Boolean, String>", "Map<String, String>")
    ));
    assert!(
        unspread(&result).is_empty(),
        "no fallback note: {:?}",
        result.notes
    );
}

#[test]
fn a_quantifier_over_a_boolean_keyed_map_says_it_fell_back_to_a_one_entry_row() {
    let result = synthesize(&ir(MAP));
    let notes: Vec<String> = unspread(&result);
    assert_eq!(notes.len(), 1, "one fallback note: {:?}", result.notes);
    assert!(
        notes[0].contains("demo.apps.Launch/outcome/refused")
            && notes[0].contains("exists")
            && notes[0].contains("one-entry"),
        "the note names the scenario, the guard and the one-entry fallback: {}",
        notes[0]
    );
}
