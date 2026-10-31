//! Adversary, pass 1: a repeated key in an authored scenario, at the edges of what YAML calls a
//! repeat.
//!
//! A repeat inside a list of mappings, a key written through an alias, quoted and plain spellings
//! of one key, merge keys, keys equal as text but not as YAML values, several documents in one
//! file, and the struct keys whose refusal stays `ESS-AUTHOR-001`.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;

/// One command with a `String`, an `Integer` and a `Map<String, Integer>` input, and one view.
const MODEL: &str = "\
format: ess/25
system: demo
version: v1
domain: demo.board
summary: A board holds a label, a size limit and tagged counts.
types:
  - {name: demo.board.BoardId, kind: newtype, of: Uuid}
entities:
  - name: demo.board.Board
    identity: {name: board_id, type: demo.board.BoardId}
    fields:
      - {name: label, type: String}
      - {name: limit, type: Integer}
      - {name: tags, type: 'Map<String, Integer>'}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
      transitions: []
events:
  - name: demo.board.BoardOpened
    fields:
      - {name: board_id, type: demo.board.BoardId}
      - {name: label, type: String}
actors:
  - name: demo.board.Operator
    may: [demo.board.Open]
commands:
  - name: demo.board.Open
    input:
      - {name: label, type: String}
      - {name: limit, type: Integer}
      - {name: tags, type: 'Map<String, Integer>'}
    outcomes:
      - name: opened
        creates: demo.board.Board
        instance: board_id
        emits: [demo.board.BoardOpened]
        payload: {demo.board.BoardOpened: {board_id: {generated: true}, label: input.label}}
        sets: {label: input.label, limit: input.limit, tags: input.tags}
views:
  - name: demo.board.Boards
    source: demo.board.Board
    consistency: read_your_writes
    fields:
      - {name: board_id, type: demo.board.BoardId}
      - {name: label, type: String}
      - {name: limit, type: Integer}
      - {name: tags, type: 'Map<String, Integer>'}
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(SpecSource::new("system.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

const HEAD: &str = "type: ess-scenario/1
domain: demo.board
scenario: open-board
summary: Opening a board lists it.
arrange:
  - instance: board
    entity: demo.board.Board
  - instance: other
    entity: demo.board.Board
";

/// A document whose two steps send `first` and `second`, with `extra` written into the first
/// step, then one assertion with `contains`.
fn document(first: &str, extra: &str, second: &str, contains: &str) -> String {
    format!(
        "{HEAD}timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.board.Open
    actor: demo.board.Operator
    input: {first}
    outcome: opened
{extra}    capture: {{instance: board, event: demo.board.BoardOpened, field: board_id}}
  - at: 2026-01-05T10:00:00Z
    command: demo.board.Open
    actor: demo.board.Operator
    input: {second}
    outcome: opened
    capture: {{instance: other, event: demo.board.BoardOpened, field: board_id}}
assert:
  - view: demo.board.Boards
    contains: {contains}
"
    )
}

const PLAIN: &str = "{label: morning, limit: 3, tags: {a: 1}}";
const CONTAINS: &str = "{board_id: {$instance: board}, limit: 3}";

fn authoring(text: &str) -> Authoring {
    compile_authored(&ir(), &[Source::new("scenarios/open-board.yaml", text)])
}

/// Every refusal `text` produced, as `(code, rendered)`.
fn refusals(text: &str) -> (usize, Vec<(String, String)>) {
    let authoring = authoring(text);
    let rendered = authoring
        .refusals
        .iter()
        .map(|refusal| (refusal.code().to_string(), refusal.to_string()))
        .collect();
    (authoring.scenarios.len(), rendered)
}

/// The one refusal `text` produced, after checking it compiled nothing.
fn the_refusal(text: &str) -> (String, String) {
    let (scenarios, rendered) = refusals(text);
    assert_eq!(scenarios, 0, "a refused document compiled: {rendered:#?}");
    assert_eq!(rendered.len(), 1, "{rendered:#?}");
    rendered.into_iter().next().expect("one refusal")
}

fn assert_repeat(text: &str, path: &str, key: &str) {
    let (code, message) = the_refusal(text);
    assert_eq!(code, "ESS-AUTHOR-047", "{message}");
    assert!(message.contains(path), "`{path}` missing from:\n{message}");
    assert!(message.contains(key), "`{key}` missing from:\n{message}");
}

#[test]
fn the_unrepeated_document_compiles() {
    let (scenarios, rendered) = refusals(&document(PLAIN, "", PLAIN, CONTAINS));
    assert!(rendered.is_empty(), "{rendered:#?}");
    assert_eq!(scenarios, 1);
}

#[test]
fn a_repeat_in_the_second_item_of_the_timeline_names_that_item() {
    assert_repeat(
        &document(
            PLAIN,
            "",
            "{label: evening, limit: 4, tags: {a: 1}, limit: 5}",
            CONTAINS,
        ),
        "timeline[1].input",
        "\"limit\"",
    );
}

#[test]
fn a_repeat_inside_a_mapping_in_a_list_of_event_claims_is_refused() {
    let events = "    events:
      - {event: demo.board.BoardOpened, payload: {label: morning}}
      - {event: demo.board.BoardOpened, payload: {label: morning, label: evening}}
";
    assert_repeat(
        &document(PLAIN, events, PLAIN, CONTAINS),
        "timeline[0].events[1].payload",
        "\"label\"",
    );
}

#[test]
fn a_repeat_inside_a_nested_literal_map_is_refused() {
    assert_repeat(
        &document(
            "{label: morning, limit: 3, tags: {a: 1, b: 2, a: 3}}",
            "",
            PLAIN,
            CONTAINS,
        ),
        "timeline[0].input.tags",
        "\"a\"",
    );
}

#[test]
fn a_key_written_through_an_alias_repeats_the_key_it_names() {
    assert_repeat(
        &document(
            "{&k label: morning, limit: 3, tags: {a: 1}, *k : evening}",
            "",
            PLAIN,
            CONTAINS,
        ),
        "timeline[0].input",
        "\"label\"",
    );
}

#[test]
fn a_quoted_and_a_plain_spelling_of_one_key_are_a_repeat() {
    for input in [
        "{label: morning, limit: 3, tags: {a: 1}, \"label\": evening}",
        "{label: morning, limit: 3, tags: {a: 1}, 'label': evening}",
        "{\"label\": morning, limit: 3, tags: {a: 1}, 'label': evening}",
    ] {
        assert_repeat(
            &document(input, "", PLAIN, CONTAINS),
            "timeline[0].input",
            "\"label\"",
        );
    }
}

#[test]
fn two_merge_keys_in_one_mapping_are_a_repeat() {
    assert_repeat(
        &document(
            "{<<: {label: morning}, <<: {limit: 3}, label: morning, limit: 3, tags: {a: 1}}",
            "",
            PLAIN,
            CONTAINS,
        ),
        "timeline[0].input",
        "\"<<\"",
    );
}

#[test]
fn a_merge_key_beside_the_key_it_would_override_does_not_compile_silently() {
    // `<<` is an ordinary key in this reader: the document sends a field named `<<`, which the
    // command does not declare. Whatever the refusal, the file must not compile.
    let (scenarios, rendered) = refusals(&document(
        "{<<: {label: morning}, label: evening, limit: 3, tags: {a: 1}}",
        "",
        PLAIN,
        CONTAINS,
    ));
    assert_eq!(scenarios, 0, "{rendered:#?}");
    assert_eq!(rendered.len(), 1, "{rendered:#?}");
}

#[test]
fn an_aliased_mapping_reused_in_two_steps_is_not_a_repeat() {
    let (scenarios, rendered) = refusals(&document(
        "&in {label: morning, limit: 3, tags: {a: 1}}",
        "",
        "*in",
        CONTAINS,
    ));
    assert!(rendered.is_empty(), "{rendered:#?}");
    assert_eq!(scenarios, 1);
}

#[test]
fn equal_keys_in_sibling_mappings_of_one_list_stay_valid() {
    let events = "    events:
      - {event: demo.board.BoardOpened, payload: {label: morning}}
      - {event: demo.board.BoardOpened, payload: {label: morning}}
";
    let (scenarios, rendered) = refusals(&document(PLAIN, events, PLAIN, CONTAINS));
    // Two claims on one event may be refused for their own reason; never as a repeated key.
    assert!(
        rendered.iter().all(|(code, _)| code != "ESS-AUTHOR-047"),
        "{rendered:#?}"
    );
    assert!(scenarios + rendered.len() == 1, "{rendered:#?}");
}
#[test]
fn a_file_of_two_documents_is_refused_as_unreadable() {
    let one = document(PLAIN, "", PLAIN, CONTAINS);
    let text = format!("{one}---\n{one}");
    let (code, message) = the_refusal(&text);
    assert_eq!(code, "ESS-AUTHOR-001", "{message}");
}

#[test]
fn a_second_document_repeating_a_key_is_still_unreadable_not_a_repeat() {
    let one = document(PLAIN, "", PLAIN, CONTAINS);
    let two = document(
        "{label: morning, limit: 3, tags: {a: 1}, label: evening}",
        "",
        PLAIN,
        CONTAINS,
    );
    let (code, message) = the_refusal(&format!("{one}---\n{two}"));
    assert_eq!(code, "ESS-AUTHOR-001", "{message}");
}

#[test]
fn a_repeated_step_key_keeps_its_unreadable_refusal() {
    let text = document(PLAIN, "    outcome: opened\n", PLAIN, CONTAINS);
    let (code, message) = the_refusal(&text);
    assert_eq!(code, "ESS-AUTHOR-001", "{message}");
    assert!(message.contains("duplicate field `outcome`"), "{message}");
}

#[test]
fn a_repeated_type_key_keeps_its_unreadable_refusal() {
    let text = document(PLAIN, "", PLAIN, CONTAINS).replacen(
        "type: ess-scenario/1\n",
        "type: ess-scenario/1\ntype: ess-scenario/1\n",
        1,
    );
    let (code, message) = the_refusal(&text);
    assert_eq!(code, "ESS-AUTHOR-001", "{message}");
    assert!(message.contains("duplicate field `type`"), "{message}");
}

#[test]
fn a_repeated_assertion_key_keeps_its_unreadable_refusal() {
    let text = document(
        PLAIN,
        "",
        PLAIN,
        "{board_id: {$instance: board}}\n    contains: {limit: 3}",
    );
    let (code, message) = the_refusal(&text);
    assert_eq!(code, "ESS-AUTHOR-001", "{message}");
    assert!(message.contains("duplicate field `contains`"), "{message}");
}

#[test]
fn a_repeat_in_a_document_without_a_scenario_type_keeps_the_format_refusal() {
    let text = document(
        "{label: morning, limit: 3, tags: {a: 1}, label: evening}",
        "",
        PLAIN,
        CONTAINS,
    )
    .replacen("type: ess-scenario/1", "type: demo-trace/1", 1);
    let (code, message) = the_refusal(&text);
    assert_ne!(code, "ESS-AUTHOR-047", "{message}");
    assert!(message.contains("demo-trace/1"), "{message}");
}
