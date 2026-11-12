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

// Adversary pass 1 (story:collection-quantifiers-witnessed-everywhere).

/// One command whose single guarded branch `blocked` reads `tags`, declared as `{list}`, with
/// `{types}` added to the declared types.
const ONE: &str = "format: ess/32
system: demo
version: v1
domain: demo.files
types:
  - {name: demo.files.ItemId, kind: newtype, of: Uuid}
{types}errors:
  - {name: demo.files.Blocked, summary: A tag blocks the item., fields: []}
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
      - {name: tags, type: '{list}'}
    outcomes:
      - name: blocked
        when: {guard}
        error: demo.files.Blocked
      - name: filed
        creates: demo.files.Item
        instance: item_id
        emits: [demo.files.ItemFiled]
        payload:
          demo.files.ItemFiled: {item_id: {generated: true}}
        sets: {label: input.label}
";

fn one(types: &str, list: &str, guard: &str) -> String {
    ONE.replace("{types}", types)
        .replace("{list}", list)
        .replace("{guard}", guard)
}

/// The `blocked` scenario, run against a target that reads only `tags[0]`, fails.
fn first_only_fails_blocked(guard: &str, first_only: &str) {
    let result = synthesized(&one("", "List<String>", guard));
    let sent_tags = sent(&result, BLOCKED_ID, "tags");
    let run = statuses(&result, &one("", "List<String>", first_only));
    assert_ne!(
        run.get(BLOCKED_ID),
        Some(&Status::Passed),
        "a target reading only tags[0] ({first_only}) fails {BLOCKED_ID}, sent tags = {sent_tags:?}"
    );
}

/// An `exists` that holds, over a body the two elements built at `.1`/`.2` satisfy themselves: the
/// widened list is decided by its first element as much as by its middle one, so a target reading
/// only `tags[0]` passes the scenario the predicate reference says it fails.
#[test]
fn adv_an_exists_its_filler_tags_satisfy_fails_a_target_reading_only_the_first_tag() {
    first_only_fails_blocked(
        "{exists: {in: input.tags, as: t, that: 't != \"\"'}}",
        "'input.tags.0 != \"\"'",
    );
}

/// A `forall` that fails, over a body the two elements built at `.1`/`.2` refute themselves: the
/// widened list is decided by its first element, so a target reading only `tags[0]` passes.
#[test]
fn adv_a_failing_forall_its_filler_tags_refute_fails_a_target_reading_only_the_first_tag() {
    first_only_fails_blocked(
        "{not: {forall: {in: input.tags, as: t, that: 't == \"ok\"'}}}",
        "'input.tags.0 != \"ok\"'",
    );
}

/// A list of a constrained element type: the elements built at `.1`/`.2` are not repaired onto
/// the type, so the widened list is not admitted. The story's outcome is "witnessed with several
/// elements, or says why not": either the deciding element sits between others, or a note says
/// the scenario fell back.
#[test]
fn adv_a_list_of_a_constrained_element_type_is_spread_or_says_why_not() {
    // The type's own invariant scenario is refused apart (`ValueInvariantUnwitnessed`, no
    // scenario named); the branch scenarios are what this case reads.
    let result = synthesize(&ir(&one(
        "  - {name: demo.files.Grade, kind: newtype, of: Integer, invariants: [value >= 10, value <= 20]}\n",
        "List<demo.files.Grade>",
        "{exists: {in: input.tags, as: t, that: t == 15}}",
    )));
    assert!(
        result
            .refusals
            .iter()
            .all(|refusal| refusal.scenario.is_none()),
        "no branch is refused: {:?}",
        result.refusals
    );
    let tags = items(sent(&result, BLOCKED_ID, "tags"));
    let at: Vec<usize> = tags
        .iter()
        .enumerate()
        .filter(|(_, item)| format!("{item:?}").contains("15"))
        .map(|(at, _)| at)
        .collect();
    let spread = tags.len() >= 3 && at.len() == 1 && at[0] != 0 && at[0] != tags.len() - 1;
    let noted = result
        .notes
        .iter()
        .any(|note| note.to_string().contains(BLOCKED_ID));
    assert!(
        spread || noted,
        "{BLOCKED_ID} sends one 15 between other grades, or a note says why not: tags = \
         {tags:?}, notes = {:?}",
        result.notes
    );
}

/// The mutant the brief names beside the three the story's acceptance names: a target reading the
/// first and the last tag but not the middle one fails both deciding scenarios.
#[test]
fn adv_a_target_ignoring_the_middle_tag_fails_the_suite() {
    let result = synthesized(&tags(BLOCKED, UNCHECKED));
    let ((blocked, _), (unchecked, _)) = deciding(&result);
    let mutant = tags(
        &format!(
            "{{any: ['input.tags.0 == \"blocked\"', 'input.tags.{} == \"blocked\"']}}",
            blocked.len() - 1
        ),
        &format!(
            "{{any: ['input.tags.0 == \"\"', 'input.tags.{} == \"\"']}}",
            unchecked.len() - 1
        ),
    );
    assert_killed(&result, &mutant, "ends only");
}

const RELATED_MAP: &str = "format: ess/32
system: demo
version: v1
domain: demo.apps
types:
  - {name: demo.apps.AccountId, kind: newtype, of: Uuid}
  - {name: demo.apps.SessionId, kind: newtype, of: Uuid}
errors:
  - {name: demo.apps.Refused, summary: Refused., fields: []}
  - {name: demo.apps.Unknown, summary: No such account., fields: []}
entities:
  - name: demo.apps.Account
    identity: {name: account_id, type: demo.apps.AccountId}
    fields:
      - {name: redirect_uris, type: 'Map<Boolean, String>'}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.apps.Session
    identity: {name: session_id, type: demo.apps.SessionId}
    fields:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: String}
    relations:
      - {name: account, kind: references, target: demo.apps.Account, cardinality: one, via: account_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.apps.Registered
    fields: [{name: account_id, type: demo.apps.AccountId}]
  - name: demo.apps.Opened
    fields: [{name: session_id, type: demo.apps.SessionId}]
actors:
  - {name: demo.apps.Client, may: [demo.apps.Register, demo.apps.Open]}
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
  - name: demo.apps.Open
    input:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: String}
    outcomes:
      - name: unknown
        when_related: {via: input.account_id, exists: false}
        error: demo.apps.Unknown
      - name: refused
        when_related:
          via: input.account_id
          predicate: {exists: {in: redirect_uris, as: r, that: r == input.application}}
        error: demo.apps.Refused
      - name: opened
        creates: demo.apps.Session
        instance: session_id
        emits: [demo.apps.Opened]
        payload:
          demo.apps.Opened: {session_id: {generated: true}}
        sets:
          account_id: input.account_id
          application: input.application
views:
  - name: demo.apps.Accounts
    source: demo.apps.Account
    consistency: read_your_writes
    fields:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: redirect_uris, type: 'Map<Boolean, String>'}
  - name: demo.apps.Sessions
    source: demo.apps.Session
    consistency: read_your_writes
    fields:
      - {name: session_id, type: demo.apps.SessionId}
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: String}
";

/// The acceptance's `Boolean`-keyed map, read by a related guard instead of a subject guard: the
/// related row holds two entries at most, so it falls back to a one-entry row
/// (`related_guard.rs`, the strict search then the plain one) and says so as the subject row does.
#[test]
fn adv_a_related_quantifier_over_a_boolean_keyed_map_says_it_fell_back_to_a_one_entry_row() {
    let result = synthesize(&ir(RELATED_MAP));
    let id = "demo.apps.Open/outcome/refused";
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id),
        "{id} is witnessed: refusals {:?}",
        result.refusals
    );
    assert!(
        unspread(&result).iter().any(|note| note.contains(id)),
        "a fallback note names {id}: {:?}",
        result.notes
    );
}

// Adversary pass 2 (story:collection-quantifiers-witnessed-everywhere).

const FILED_PREFIX: &str = "demo.files.FileItem/outcome/";
const LINE: &str = "  - name: demo.files.Line\n    kind: struct\n    fields:\n      - {name: sku, type: String}\n      - {name: codes, type: 'List<String>'}\n";
const PRIORITY: &str = "  - {name: demo.files.Priority, kind: enum, variants: [Low, Mid, High]}\n";
const LONGEST: usize = 8;

/// `{name: {in: <over>, as: <var>, that: <body>}}`.
fn quantifier(name: &str, over: &str, var: &str, body: &str) -> String {
    format!("{{{name}: {{in: {over}, as: {var}, that: {body}}}}}")
}

/// A target reading only the first element of `over`: `exists` is false on `[]`, `forall` true.
fn first_only(exists: bool, over: &str, body: &dyn Fn(&str) -> String) -> String {
    let at = body(&format!("{over}.0"));
    if exists {
        format!("{{all: ['{over}.count > 0', {at}]}}")
    } else {
        format!("{{any: ['{over}.count == 0', {at}]}}")
    }
}

/// A target reading only the last element of `over`, for lists of up to [`LONGEST`] elements.
fn last_only(exists: bool, over: &str, body: &dyn Fn(&str) -> String) -> String {
    let arms: Vec<String> = (1..=LONGEST)
        .map(|n| {
            format!(
                "{{all: ['{over}.count == {n}', {}]}}",
                body(&format!("{over}.{}", n - 1))
            )
        })
        .collect();
    if exists {
        format!("{{any: [{}]}}", arms.join(", "))
    } else {
        format!("{{any: ['{over}.count == 0', {}]}}", arms.join(", "))
    }
}

/// Every literal list each `FileItem` scenario sends at `tags`, by scenario.
fn sent_lists(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .iter()
        .filter(|(id, _)| id.to_string().starts_with(FILED_PREFIX))
        .flat_map(|(id, scenario)| {
            scenario.steps.iter().filter_map(move |step| match step {
                ScenarioStep::ExecuteCommand { input, .. } => match input.get("tags") {
                    Some(ScenarioValue::Literal { value }) => Some(format!("{id}: {value:?}")),
                    _ => None,
                },
                _ => None,
            })
        })
        .collect()
}

/// The mutants of `mutants` that pass every scenario synthesized from the model whose `blocked`
/// guard is `original` over `tags: <list>`, after the model itself passes all of them.
fn survivors(types: &str, list: &str, original: &str, mutants: &[(&str, String)]) -> Vec<String> {
    let model = one(types, list, original);
    let result = synthesized(&model);
    for (id, status) in statuses(&result, &model) {
        assert_eq!(status, Status::Passed, "the model passes its own {id}");
    }
    mutants
        .iter()
        .filter(|(_, mutant)| {
            statuses(&result, &one(types, list, mutant))
                .values()
                .all(|status| *status == Status::Passed)
        })
        .map(|(label, mutant)| format!("{label}: {mutant}"))
        .chain(std::iter::once(String::from("-- sent:")).chain(sent_lists(&result)))
        .collect()
}

/// The three mutants the story names for a flat quantifier `[not] exists|forall t in input.tags:
/// body(t)`, and the survivors among them.
fn flat_survivors(
    types: &str,
    list: &str,
    negated: bool,
    exists: bool,
    body: &dyn Fn(&str) -> String,
) -> Vec<String> {
    let wrap = |guard: String| {
        if negated {
            format!("{{not: {guard}}}")
        } else {
            guard
        }
    };
    let name = |exists: bool| if exists { "exists" } else { "forall" };
    let original = wrap(quantifier(name(exists), "input.tags", "t", &body("t")));
    let mutants = [
        ("first", wrap(first_only(exists, "input.tags", body))),
        ("last", wrap(last_only(exists, "input.tags", body))),
        (
            "swapped",
            wrap(quantifier(name(!exists), "input.tags", "t", &body("t"))),
        ),
    ];
    let found = survivors(types, list, &original, &mutants);
    if found.first().is_some_and(|line| line == "-- sent:") {
        Vec::new()
    } else {
        found
    }
}

fn text_is(literal: &'static str) -> impl Fn(&str) -> String {
    move |at: &str| format!("'{at} == \"{literal}\"'")
}

/// `forall t in input.tags: t == "ok"` decides `blocked` by the whole list; the scenario a single
/// element decides is the `filed` one beside it (a `forall` that fails), and the three named
/// mutants have to fail there.
#[test]
fn adv2_a_holding_forall_kills_first_last_and_swapped_targets() {
    let found = flat_survivors("", "List<String>", false, false, &text_is("ok"));
    assert!(found.is_empty(), "no mutant survives: {found:#?}");
}

/// `not exists t in input.tags: t == "blocked"`: the scenario one element decides is `filed`.
#[test]
fn adv2_a_negated_exists_kills_first_last_and_swapped_targets() {
    let found = flat_survivors("", "List<String>", true, true, &text_is("blocked"));
    assert!(found.is_empty(), "no mutant survives: {found:#?}");
}

/// `exists t in input.tags: t.sku == "blocked"` over a list of structs.
#[test]
fn adv2_an_exists_over_a_list_of_structs_kills_first_last_and_swapped_targets() {
    let found = flat_survivors(LINE, "List<demo.files.Line>", false, true, &|at: &str| {
        format!("'{at}.sku == \"blocked\"'")
    });
    assert!(found.is_empty(), "no mutant survives: {found:#?}");
}

/// `forall t in input.tags: t.sku == "ok"` over a list of structs.
#[test]
fn adv2_a_forall_over_a_list_of_structs_kills_first_last_and_swapped_targets() {
    let found = flat_survivors(LINE, "List<demo.files.Line>", false, false, &|at: &str| {
        format!("'{at}.sku == \"ok\"'")
    });
    assert!(found.is_empty(), "no mutant survives: {found:#?}");
}

/// `exists t in input.tags: t == High` over a list of an enum: `Low` and `Mid` are admitted
/// beside `High`, so the list is spread and no mutant survives.
#[test]
fn adv2_an_exists_over_a_list_of_an_enum_kills_first_last_and_swapped_targets() {
    let found = flat_survivors(
        PRIORITY,
        "List<demo.files.Priority>",
        false,
        true,
        &|at: &str| format!("'{at} == High'"),
    );
    assert!(found.is_empty(), "no mutant survives: {found:#?}");
}

/// `exists l in input.tags: exists c in l.codes: c == "blocked"`: the mutants on the outer list
/// and on the inner one.
#[test]
fn adv2_a_nested_exists_kills_first_last_and_swapped_targets_on_both_lists() {
    let code = |at: &str| format!("'{at} == \"blocked\"'");
    let inner_exists = |over: &str| quantifier("exists", over, "c", &code("c"));
    let original = quantifier("exists", "input.tags", "l", &inner_exists("l.codes"));
    // The inner `exists` read over a positioned element, written out by position.
    let inner_at = |at: &str| {
        let arms: Vec<String> = (0..LONGEST)
            .map(|j| {
                format!(
                    "{{all: ['{at}.codes.count > {j}', {}]}}",
                    code(&format!("{at}.codes.{j}"))
                )
            })
            .collect();
        format!("{{any: [{}]}}", arms.join(", "))
    };
    let mutants = [
        ("outer first", first_only(true, "input.tags", &inner_at)),
        ("outer last", last_only(true, "input.tags", &inner_at)),
        (
            "outer swapped",
            quantifier("forall", "input.tags", "l", &inner_exists("l.codes")),
        ),
        (
            "inner first",
            quantifier(
                "exists",
                "input.tags",
                "l",
                &first_only(true, "l.codes", &code),
            ),
        ),
        (
            "inner last",
            quantifier(
                "exists",
                "input.tags",
                "l",
                &last_only(true, "l.codes", &code),
            ),
        ),
        (
            "inner swapped",
            quantifier(
                "exists",
                "input.tags",
                "l",
                &quantifier("forall", "l.codes", "c", &code("c")),
            ),
        ),
    ];
    let found = survivors(LINE, "List<demo.files.Line>", &original, &mutants);
    assert!(
        found.first().is_some_and(|line| line == "-- sent:"),
        "no mutant survives: {found:#?}"
    );
}

/// A newtype list whose invariant admits other grades than the deciding `15`: the deciding grade
/// is sent between two others, without an `UnspreadInputList` note (the changelog: the note is for
/// an element type that admits no other value).
#[test]
fn adv2_a_constrained_list_that_admits_other_grades_is_spread_without_a_note() {
    let result = synthesize(&ir(&one(
        "  - {name: demo.files.Grade, kind: newtype, of: Integer, invariants: [value >= 10, value <= 20]}\n",
        "List<demo.files.Grade>",
        "{exists: {in: input.tags, as: t, that: t == 15}}",
    )));
    let tags = items(sent(&result, BLOCKED_ID, "tags"));
    let at: Vec<usize> = tags
        .iter()
        .enumerate()
        .filter(|(_, item)| format!("{item:?}").contains("15"))
        .map(|(at, _)| at)
        .collect();
    let notes: Vec<String> = result
        .notes
        .iter()
        .filter(|note| matches!(note, Note::UnspreadInputList { .. }))
        .map(ToString::to_string)
        .collect();
    assert!(
        tags.len() >= 3 && at.len() == 1 && at[0] != 0 && at[0] != tags.len() - 1 && notes.is_empty(),
        "{BLOCKED_ID} sends one 15 between other grades and no note: tags = {tags:?}, notes = {notes:?}"
    );
}

/// An enum of one variant admits no other element: the list cannot be spread, and the
/// `UnspreadInputList` note names the scenario and the quantifier.
#[test]
fn adv2_a_list_whose_element_type_admits_no_other_value_says_so() {
    let result = synthesize(&ir(&one(
        "  - {name: demo.files.Only, kind: enum, variants: [One]}\n",
        "List<demo.files.Only>",
        "{exists: {in: input.tags, as: t, that: t == One}}",
    )));
    let notes: Vec<String> = result
        .notes
        .iter()
        .filter(|note| matches!(note, Note::UnspreadInputList { .. }))
        .map(ToString::to_string)
        .collect();
    assert!(
        notes.len() == 1 && notes[0].contains(BLOCKED_ID) && notes[0].contains("exists"),
        "one UnspreadInputList note names {BLOCKED_ID}: {:?}; refusals {:?}",
        result.notes,
        result.refusals
    );
}

// Adversary pass 3 (story:collection-quantifiers-witnessed-everywhere).

/// `model` synthesized with no branch refused, the model passing every scenario, and the labels of
/// the `mutants` (whole models) that pass every scenario too.
fn adv3_survivors(model: &str, mutants: &[(&str, String)]) -> (Synthesis, Vec<String>) {
    let result = synthesize(&ir(model));
    assert!(
        result
            .refusals
            .iter()
            .all(|refusal| refusal.scenario.is_none()),
        "no branch is refused: {:?}",
        result.refusals
    );
    for (id, status) in statuses(&result, model) {
        assert_eq!(status, Status::Passed, "the model passes its own {id}");
    }
    let found = mutants
        .iter()
        .filter(|(_, mutant)| {
            statuses(&result, mutant)
                .values()
                .all(|status| *status == Status::Passed)
        })
        .map(|(label, _)| (*label).to_owned())
        .collect();
    (result, found)
}

/// Every literal `field` each scenario sends, and every note.
fn adv3_evidence(result: &Synthesis, field: &str) -> Vec<String> {
    result
        .suite
        .scenarios
        .iter()
        .flat_map(|(id, scenario)| {
            scenario.steps.iter().filter_map(move |step| match step {
                ScenarioStep::ExecuteCommand { input, .. } => match input.get(field) {
                    Some(ScenarioValue::Literal { value }) => Some(format!("{id}: {value:?}")),
                    _ => None,
                },
                _ => None,
            })
        })
        .chain(result.notes.iter().map(|note| format!("note: {note}")))
        .collect()
}

fn adv3_unspread_naming(result: &Synthesis, id: &str) -> bool {
    result.notes.iter().any(|note| {
        matches!(
            note,
            Note::UnspreadInputList { .. } | Note::UnspreadCollection { .. }
        ) && note.to_string().contains(id)
    })
}

/// The inner quantifier of `exists l in input.tags: <inner> c in l.codes: body(c)` read as the
/// other quantifier, over its first element only, and over its last only.
fn adv3_inner_mutants(
    model: &dyn Fn(&str) -> String,
    exists: bool,
    body: &dyn Fn(&str) -> String,
) -> Vec<(&'static str, String)> {
    let outer = |inner: String| model(&quantifier("exists", "input.tags", "l", &inner));
    let other = if exists { "forall" } else { "exists" };
    vec![
        (
            "inner swapped",
            outer(quantifier(other, "l.codes", "c", &body("c"))),
        ),
        ("inner first", outer(first_only(exists, "l.codes", body))),
        ("inner last", outer(last_only(exists, "l.codes", body))),
    ]
}

const ADV3_DEEP: &str = "  - name: demo.files.Code\n    kind: struct\n    fields:\n      - {name: name, type: String}\n      - {name: marks, type: 'List<String>'}\n  - name: demo.files.Line\n    kind: struct\n    fields:\n      - {name: sku, type: String}\n      - {name: codes, type: 'List<demo.files.Code>'}\n";

/// Three levels: `exists a in input.tags: exists b in a.codes: exists c in b.marks: c == "x"`. The
/// changelog: a nested quantifier read as the other quantifier, or over its first or last element
/// only, fails a scenario.
#[test]
fn adv3_three_levels_of_nesting_kill_the_inner_mutants() {
    let list = "List<demo.files.Line>";
    let mark = text_is("x");
    let deep = |middle: &str, innermost: String| {
        one(
            ADV3_DEEP,
            list,
            &quantifier(
                "exists",
                "input.tags",
                "a",
                &quantifier(middle, "a.codes", "b", &innermost),
            ),
        )
    };
    let model = deep("exists", quantifier("exists", "b.marks", "c", &mark("c")));
    let mutants = [
        (
            "innermost swapped",
            deep("exists", quantifier("forall", "b.marks", "c", &mark("c"))),
        ),
        (
            "middle swapped",
            deep("forall", quantifier("exists", "b.marks", "c", &mark("c"))),
        ),
        (
            "innermost first",
            deep("exists", first_only(true, "b.marks", &mark)),
        ),
        (
            "innermost last",
            deep("exists", last_only(true, "b.marks", &mark)),
        ),
    ];
    let (result, found) = adv3_survivors(&model, &mutants);
    assert!(
        found.is_empty(),
        "no mutant survives: {found:?}; sent {:#?}",
        adv3_evidence(&result, "tags")
    );
}

/// `exists l in input.tags: forall c in l.codes: c == "ok"`: the nested `forall` holds on the
/// deciding element. An element whose codes are `["ok", "x"]` tells it from `exists` (and from a
/// first-only or last-only read) in the `filed` scenario, as the flat holding `forall` is.
#[test]
#[ignore = "backlog: nested-quantifier-apart (story:a-nested-collection-quantifier-is-witnessed-apart)"]
fn adv3_a_holding_nested_forall_beside_an_outer_exists_kills_the_inner_mutants() {
    let list = "List<demo.files.Line>";
    let model = |guard: &str| one(LINE, list, guard);
    let ok = text_is("ok");
    let original = model(&quantifier(
        "exists",
        "input.tags",
        "l",
        &quantifier("forall", "l.codes", "c", &ok("c")),
    ));
    let (result, found) = adv3_survivors(&original, &adv3_inner_mutants(&model, false, &ok));
    assert!(
        found.is_empty(),
        "no mutant survives: {found:?}; sent {:#?}",
        adv3_evidence(&result, "tags")
    );
}

/// `exists l in input.tags: exists c in l.codes: c == One` over an enum of one variant: the inner
/// list cannot be spread, so the changelog's note ("an input list whose element type admits no
/// other value ... says so with a note: line naming the scenario and the quantifier") is due.
#[test]
#[ignore = "backlog: nested-quantifier-apart (story:a-nested-collection-quantifier-is-witnessed-apart)"]
fn adv3_a_nested_quantifier_over_a_one_variant_enum_is_spread_or_says_why_not() {
    let types = "  - {name: demo.files.Only, kind: enum, variants: [One]}\n  - name: demo.files.Line\n    kind: struct\n    fields:\n      - {name: sku, type: String}\n      - {name: codes, type: 'List<demo.files.Only>'}\n";
    let list = "List<demo.files.Line>";
    let model = |guard: &str| one(types, list, guard);
    let is_one = |at: &str| format!("'{at} == One'");
    let original = model(&quantifier(
        "exists",
        "input.tags",
        "l",
        &quantifier("exists", "l.codes", "c", &is_one("c")),
    ));
    let (result, found) = adv3_survivors(&original, &adv3_inner_mutants(&model, true, &is_one));
    assert!(
        found.is_empty() || adv3_unspread_naming(&result, BLOCKED_ID),
        "the inner mutants {found:?} survive and no note names {BLOCKED_ID}: {:#?}",
        adv3_evidence(&result, "tags")
    );
}

/// `exists l in input.tags: exists c in l.codes: c == 15` where `codes` holds a newtype with an
/// invariant (10..=20): other grades are admitted, so the inner mutants fail, or a note says why.
#[test]
fn adv3_an_inner_list_of_a_constrained_element_kills_the_inner_mutants_or_says_why_not() {
    let types = "  - {name: demo.files.Grade, kind: newtype, of: Integer, invariants: [value >= 10, value <= 20]}\n  - name: demo.files.Line\n    kind: struct\n    fields:\n      - {name: sku, type: String}\n      - {name: codes, type: 'List<demo.files.Grade>'}\n";
    let list = "List<demo.files.Line>";
    let model = |guard: &str| one(types, list, guard);
    let is_15 = |at: &str| format!("'{at} == 15'");
    let original = model(&quantifier(
        "exists",
        "input.tags",
        "l",
        &quantifier("exists", "l.codes", "c", &is_15("c")),
    ));
    let mut mutants = adv3_inner_mutants(&model, true, &is_15);
    mutants.push((
        "outer swapped",
        model(&quantifier(
            "forall",
            "input.tags",
            "l",
            &quantifier("exists", "l.codes", "c", &is_15("c")),
        )),
    ));
    let (result, found) = adv3_survivors(&original, &mutants);
    assert!(
        found.is_empty() || adv3_unspread_naming(&result, BLOCKED_ID),
        "the mutants {found:?} survive and no note names {BLOCKED_ID}: {:#?}",
        adv3_evidence(&result, "tags")
    );
}

const ADV3_RELATED: &str = "format: ess/32
system: demo
version: v1
domain: demo.apps
types:
  - {name: demo.apps.AccountId, kind: newtype, of: Uuid}
  - {name: demo.apps.SessionId, kind: newtype, of: Uuid}
  - name: demo.apps.Line
    kind: struct
    fields:
      - {name: sku, type: String}
      - {name: codes, type: 'List<String>'}
errors:
  - {name: demo.apps.Refused, summary: Refused., fields: []}
  - {name: demo.apps.Unknown, summary: No such account., fields: []}
entities:
  - name: demo.apps.Account
    identity: {name: account_id, type: demo.apps.AccountId}
    fields:
      - {name: lines, type: 'List<demo.apps.Line>'}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.apps.Session
    identity: {name: session_id, type: demo.apps.SessionId}
    fields:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: String}
    relations:
      - {name: account, kind: references, target: demo.apps.Account, cardinality: one, via: account_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.apps.Registered
    fields: [{name: account_id, type: demo.apps.AccountId}]
  - name: demo.apps.Opened
    fields: [{name: session_id, type: demo.apps.SessionId}]
actors:
  - {name: demo.apps.Client, may: [demo.apps.Register, demo.apps.Open]}
commands:
  - name: demo.apps.Register
    input:
      - {name: lines, type: 'List<demo.apps.Line>'}
    outcomes:
      - name: registered
        creates: demo.apps.Account
        instance: account_id
        emits: [demo.apps.Registered]
        payload:
          demo.apps.Registered: {account_id: {generated: true}}
        sets:
          lines: input.lines
  - name: demo.apps.Open
    input:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: String}
    outcomes:
      - name: unknown
        when_related: {via: input.account_id, exists: false}
        error: demo.apps.Unknown
      - name: refused
        when_related:
          via: input.account_id
          predicate: {exists: {in: lines, as: l, that: {inner}}}
        error: demo.apps.Refused
      - name: opened
        creates: demo.apps.Session
        instance: session_id
        emits: [demo.apps.Opened]
        payload:
          demo.apps.Opened: {session_id: {generated: true}}
        sets:
          account_id: input.account_id
          application: input.application
views:
  - name: demo.apps.Accounts
    source: demo.apps.Account
    consistency: read_your_writes
    fields:
      - {name: account_id, type: demo.apps.AccountId}
      - {name: lines, type: 'List<demo.apps.Line>'}
  - name: demo.apps.Sessions
    source: demo.apps.Session
    consistency: read_your_writes
    fields:
      - {name: session_id, type: demo.apps.SessionId}
      - {name: account_id, type: demo.apps.AccountId}
      - {name: application, type: String}
";

/// A nested quantifier in a `when_related` predicate, over a stored list of structs:
/// `exists l in lines: exists c in l.codes: c == input.application`. The predicate reference: the
/// row is arranged with several entries so `forall` for `exists` fails, or a `note:` says it fell
/// back.
#[test]
#[ignore = "backlog: nested-quantifier-apart (story:a-nested-collection-quantifier-is-witnessed-apart)"]
fn adv3_a_nested_quantifier_in_a_related_predicate_kills_the_inner_swap_or_says_why_not() {
    let id = "demo.apps.Open/outcome/refused";
    let model = |inner: &str| {
        ADV3_RELATED.replace(
            "{inner}",
            &quantifier(inner, "l.codes", "c", "c == input.application"),
        )
    };
    let (result, found) = adv3_survivors(&model("exists"), &[("inner swapped", model("forall"))]);
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id),
        "{id} is witnessed: refusals {:?}",
        result.refusals
    );
    assert!(
        found.is_empty() || adv3_unspread_naming(&result, id),
        "the mutants {found:?} survive and no note names {id}: notes {:?}",
        result.notes
    );
}

/// Two branches over the same nested list with their own literals: `blocked` on `"blocked"`, then
/// `held` on `"held"`. The other candidates' first codes are `"blocked"` and `"held"`, the two
/// values the filler's inner list is drawn from, and each decides one branch itself. `Line`
/// admits other codes, so `held` is spread (its outer mutants fail) and no `UnspreadInputList`
/// note is due.
#[test]
fn adv3_a_second_nested_branch_is_spread_though_the_pooled_codes_each_decide_a_branch() {
    let list = "List<demo.files.Line>";
    let held_id = "demo.files.FileItem/outcome/held";
    let nested = |outer: &str, inner: &str, literal: &'static str| {
        quantifier(
            outer,
            "input.tags",
            "l",
            &quantifier(inner, "l.codes", "c", &text_is(literal)("c")),
        )
    };
    let model = |held: &str| {
        one(LINE, list, &nested("exists", "exists", "blocked"))
            .replace(
                "errors:\n",
                "errors:\n  - {name: demo.files.Held, summary: A code holds the item., fields: []}\n",
            )
            .replace(
                "        error: demo.files.Blocked\n",
                &format!(
                    "        error: demo.files.Blocked\n      - name: held\n        when: {held}\n        error: demo.files.Held\n"
                ),
            )
    };
    let held_code = text_is("held");
    let mutants = [
        (
            "held outer swapped",
            model(&nested("forall", "exists", "held")),
        ),
        (
            "held inner swapped",
            model(&nested("exists", "forall", "held")),
        ),
        (
            "held inner first",
            model(&quantifier(
                "exists",
                "input.tags",
                "l",
                &first_only(true, "l.codes", &held_code),
            )),
        ),
    ];
    let (result, found) = adv3_survivors(&model(&nested("exists", "exists", "held")), &mutants);
    let noted = result.notes.iter().any(|note| {
        matches!(note, Note::UnspreadInputList { .. }) && note.to_string().contains(held_id)
    });
    assert!(
        found.is_empty() && !noted,
        "no mutant survives ({found:?}) and no UnspreadInputList note names {held_id}: {:#?}",
        adv3_evidence(&result, "tags")
    );
}
