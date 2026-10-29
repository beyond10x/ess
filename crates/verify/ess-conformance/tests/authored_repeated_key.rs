//! An authored scenario that repeats a mapping key is refused, as a model file that does is.
//!
//! The scenario reader holds `input:`, `contains:` and the other free-form mappings as maps, which
//! keep the last of two equal keys without a word: a step's `input:` naming `label` twice sent the
//! second value and dropped the first. The model reader refuses the same repeat. A repeat in any
//! mapping the document reads as a map is now `ESS-AUTHOR-047`, naming the mapping's path, the key
//! and the line the YAML reader gives; a repeated document key keeps its `ESS-AUTHOR-001`.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;

/// One command with an `Integer` and a `String` input, and one view over the record it creates.
const MODEL: &str = "\
format: ess/25
system: demo
version: v1
domain: demo.board
summary: A board holds a label and a size limit.
types:
  - {name: demo.board.BoardId, kind: newtype, of: Uuid}
entities:
  - name: demo.board.Board
    identity: {name: board_id, type: demo.board.BoardId}
    fields:
      - {name: label, type: String}
      - {name: limit, type: Integer}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
      transitions: []
events:
  - name: demo.board.BoardOpened
    fields: [{name: board_id, type: demo.board.BoardId}]
actors:
  - name: demo.board.Operator
    may: [demo.board.Open]
commands:
  - name: demo.board.Open
    input:
      - {name: label, type: String}
      - {name: limit, type: Integer}
    outcomes:
      - name: opened
        creates: demo.board.Board
        instance: board_id
        emits: [demo.board.BoardOpened]
        payload: {demo.board.BoardOpened: {board_id: {generated: true}}}
        sets: {label: input.label, limit: input.limit}
views:
  - name: demo.board.Boards
    source: demo.board.Board
    consistency: read_your_writes
    fields:
      - {name: board_id, type: demo.board.BoardId}
      - {name: label, type: String}
      - {name: limit, type: Integer}
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(SpecSource::new("system.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// A document opening one board with `input`, then asserting the view `contains` it.
fn document(input: &str, contains: &str) -> String {
    format!(
        "type: ess-scenario/1
domain: demo.board
scenario: open-board
summary: Opening a board lists it with its limit.
arrange:
  - instance: board
    entity: demo.board.Board
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.board.Open
    actor: demo.board.Operator
    input: {input}
    outcome: opened
    capture: {{instance: board, event: demo.board.BoardOpened, field: board_id}}
assert:
  - view: demo.board.Boards
    contains: {contains}
"
    )
}

fn authoring(text: &str) -> Authoring {
    compile_authored(&ir(), &[Source::new("scenarios/open-board.yaml", text)])
}

/// The one refusal `text` produced, as `(code, rendered)`, after checking it compiled nothing.
fn the_refusal(text: &str) -> (String, String) {
    let authoring = authoring(text);
    assert!(
        authoring.scenarios.is_empty(),
        "a refused document produced a scenario"
    );
    let rendered: Vec<(String, String)> = authoring
        .refusals
        .iter()
        .map(|refusal| (refusal.code().to_string(), refusal.to_string()))
        .collect();
    assert_eq!(rendered.len(), 1, "{rendered:#?}");
    rendered.into_iter().next().unwrap()
}

#[test]
fn a_key_repeated_with_another_value_in_a_step_input_is_refused() {
    let (code, message) = the_refusal(&document(
        "{label: morning, limit: 3, label: evening}",
        "{board_id: {$instance: board}, limit: 3}",
    ));
    assert_eq!(code, "ESS-AUTHOR-047", "{message}");
    for part in [
        "scenarios/open-board.yaml",
        "timeline[0].input",
        "\"label\"",
        "at line",
    ] {
        assert!(message.contains(part), "`{part}` missing from:\n{message}");
    }
}

#[test]
fn a_key_repeated_with_the_same_value_is_refused() {
    let (code, message) = the_refusal(&document(
        "{label: morning, limit: 3, label: morning}",
        "{board_id: {$instance: board}, limit: 3}",
    ));
    assert_eq!(code, "ESS-AUTHOR-047", "{message}");
    assert!(message.contains("timeline[0].input"), "{message}");
    assert!(message.contains("\"label\""), "{message}");
}

#[test]
fn a_key_repeated_in_a_nested_assertion_mapping_is_refused() {
    let (code, message) = the_refusal(&document(
        "{label: morning, limit: 3}",
        "{board_id: {$instance: board, $instance: board}, limit: 3}",
    ));
    assert_eq!(code, "ESS-AUTHOR-047", "{message}");
    assert!(message.contains("assert[0].contains.board_id"), "{message}");
    assert!(message.contains("\"$instance\""), "{message}");
    assert!(message.contains("at line"), "{message}");
}

#[test]
fn equal_keys_in_separate_mappings_stay_valid() {
    let text = "type: ess-scenario/1
domain: demo.board
scenario: open-two-boards
summary: Two boards opened one after the other are both listed.
arrange:
  - instance: first
    entity: demo.board.Board
  - instance: second
    entity: demo.board.Board
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.board.Open
    actor: demo.board.Operator
    input: {label: morning, limit: 3}
    outcome: opened
    capture: {instance: first, event: demo.board.BoardOpened, field: board_id}
  - at: 2026-01-05T10:00:00Z
    command: demo.board.Open
    actor: demo.board.Operator
    input: {label: evening, limit: 4}
    outcome: opened
    capture: {instance: second, event: demo.board.BoardOpened, field: board_id}
assert:
  - view: demo.board.Boards
    contains: {board_id: {$instance: first}, label: morning}
  - view: demo.board.Boards
    contains: {board_id: {$instance: second}, label: evening}
";
    let authoring = authoring(text);
    let refusals: Vec<String> = authoring.refusals.iter().map(ToString::to_string).collect();
    assert!(refusals.is_empty(), "{refusals:#?}");
    assert_eq!(authoring.scenarios.len(), 1);
}

#[test]
fn a_repeated_document_key_keeps_its_unreadable_refusal() {
    let text = document(
        "{label: morning, limit: 3}",
        "{board_id: {$instance: board}, limit: 3}",
    )
    .replace(
        "summary: Opening a board lists it with its limit.\n",
        "summary: Opening a board lists it with its limit.\nsummary: Again.\n",
    );
    let (code, message) = the_refusal(&text);
    assert_eq!(code, "ESS-AUTHOR-001", "{message}");
    assert!(message.contains("duplicate field `summary`"), "{message}");
}

/// `1` and `"1"`, or `true` and `"true"`, are two YAML keys but one key once a mapping is read as a
/// map of strings: the map would keep the second value and drop the first.
#[test]
fn keys_that_differ_only_in_yaml_type_are_a_repeat() {
    for (input, key) in [
        (
            "{label: morning, limit: 3, tags: {1: 10, \"1\": 20}}",
            "\"1\"",
        ),
        (
            "{label: morning, limit: 3, tags: {true: 10, \"true\": 20}}",
            "\"true\"",
        ),
        (
            "{label: morning, limit: 3, tags: {\"null\": 10, null: 20}}",
            "\"null\"",
        ),
    ] {
        let (code, message) =
            the_refusal(&document(input, "{board_id: {$instance: board}, limit: 3}"));
        assert_eq!(code, "ESS-AUTHOR-047", "{input}: {message}");
        for part in ["timeline[0].input.tags", key, "at line"] {
            assert!(
                message.contains(part),
                "{input}: `{part}` missing from:\n{message}"
            );
        }
    }
}

/// Keys whose text differs stay two keys, whatever their YAML types.
#[test]
fn keys_with_different_text_are_not_a_repeat() {
    let authoring = authoring(&document(
        "{label: morning, limit: 3, tags: {1: 10, \"01\": 20, \"1.0\": 30}}",
        "{board_id: {$instance: board}, limit: 3}",
    ));
    let codes: Vec<String> = authoring
        .refusals
        .iter()
        .map(|refusal| refusal.code().to_string())
        .collect();
    assert!(
        !codes.iter().any(|code| code == "ESS-AUTHOR-047"),
        "{codes:#?}"
    );
}
