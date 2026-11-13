//! A struct's `distinct:` invariant, held by the generated decoders
//! (`story:generated-rust-and-go-decoders-check-a-structs-distinct-invariant`).
//!
//! 0.94.0 refused to generate Rust or Go for a workspace in which one struct declared `distinct:`;
//! 0.93.0 generated it and checked nothing. The decoders now compare the keys as the interpreted
//! target and the published schema do (`website/docs/reference/predicates.md`, "From `ess/22`:
//! distinct list members"): numbers exactly, an instant by the instant it names, text exactly, an
//! absent `Optional` key unknown and so never a duplicate.
//!
//! The model has the reporter's shape: six structs and nine `distinct:` invariants, with and
//! without `by:`, over a scalar list, a text member, an enum member and an `Optional` member, and a
//! list nested in a struct of a list.

use super::{cargo, ir, scratch, write};
use ess_compiler::ir::ResolvedBody;
use ess_synth::{synthesize_for, Target};
use serde_json::{json, Value};
use std::fmt::Write as _;

const MODEL: &str = "format: ess/22
system: probe
version: v1
domain: probe.range
types:
  - {name: probe.range.Tag, kind: newtype, of: String}
  - {name: probe.range.Level, kind: enum, variants: [Low, High]}
  - name: probe.range.Labels
    kind: struct
    fields:
      - {name: tags, type: 'List<String>'}
      - {name: stamps, type: 'List<Timestamp>'}
      - {name: amounts, type: 'List<Decimal>'}
    invariants:
      - {distinct: {in: tags, as: tag}}
      - {distinct: {in: stamps, as: stamp}}
      - {distinct: {in: amounts, as: amount}}
  - name: probe.range.Levels
    kind: struct
    fields:
      - {name: levels, type: 'List<probe.range.Level>'}
    invariants:
      - {distinct: {in: levels, as: level}}
  - name: probe.range.Entry
    kind: struct
    fields:
      - {name: name, type: probe.range.Tag}
      - {name: level, type: probe.range.Level}
      - {name: note, type: 'Optional<String>'}
  - name: probe.range.Registry
    kind: struct
    fields:
      - {name: entries, type: 'List<probe.range.Entry>'}
    invariants:
      - {distinct: {in: entries, as: entry, by: entry.name}}
      - {distinct: {in: entries, as: entry, by: entry.level}}
      - {distinct: {in: entries, as: entry, by: entry.note}}
  - name: probe.range.Group
    kind: struct
    fields:
      - {name: name, type: String}
      - {name: items, type: 'List<Integer>'}
  - name: probe.range.Catalog
    kind: struct
    fields:
      - {name: groups, type: 'List<probe.range.Group>'}
    invariants:
      - {distinct: {in: groups, as: group, by: group.name}}
      - forall: {in: groups, as: group, that: {distinct: {in: group.items, as: item}}}
events:
  - name: probe.range.AllSet
    fields: [{name: note, type: String}]
commands:
  - name: probe.range.SetAll
    input:
      - {name: labels, type: probe.range.Labels}
      - {name: levels, type: probe.range.Levels}
      - {name: registry, type: probe.range.Registry}
      - {name: catalog, type: probe.range.Catalog}
      - {name: note, type: String}
    outcomes:
      - name: set
        emits: [probe.range.AllSet]
        payload:
          probe.range.AllSet: {note: input.note}
components:
  - component: ranger
    owns: {domains: [probe.range]}
    accepts: {commands: [probe.range.SetAll]}
    publishes: {events: [probe.range.AllSet]}
    reached_by: network
";

/// A value every invariant holds for: no two elements of any list share a key.
fn base() -> Value {
    json!({
        "labels": {
            "tags": ["a", "b"],
            "stamps": ["2020-01-01T00:00:00Z", "2020-01-01T01:00:00Z"],
            "amounts": ["1", "2"],
        },
        "levels": {"levels": ["Low", "High"]},
        "registry": {"entries": [
            {"name": "a", "level": "Low", "note": "x"},
            {"name": "b", "level": "High", "note": "y"},
        ]},
        "note": "x",
        "catalog": {"groups": [
            {"name": "g1", "items": [1, 2]},
            {"name": "g2", "items": [1, 2]},
        ]},
    })
}

/// What a decoder answers for one input: the value is admitted, or refused at the struct's
/// position naming the invariant `(struct, index)` declares.
type Verdict = Option<(&'static str, &'static str, usize)>;

/// One case: its label, the input, and the verdict.
type Case = (&'static str, Value, Verdict);

/// `base` changed by `change`.
fn with(change: &dyn Fn(&mut Value)) -> Value {
    let mut input = base();
    change(&mut input);
    input
}

/// Every case.
fn cases() -> Vec<Case> {
    let mut every = scalar_cases();
    every.extend(member_cases());
    every
}

/// Lists of scalars: text, instants, decimals and enum values.
fn scalar_cases() -> Vec<Case> {
    vec![
        ("a value with no repeated key", base(), None),
        (
            "a repeated scalar",
            with(&|v| v["labels"]["tags"] = json!(["a", "a"])),
            Some(("labels", "Labels", 0)),
        ),
        (
            "text keys differing in case are two keys",
            with(&|v| v["labels"]["tags"] = json!(["a", "A"])),
            None,
        ),
        (
            "one instant in two spellings",
            with(&|v| {
                v["labels"]["stamps"] =
                    json!(["2020-01-01T00:00:00Z", "2019-12-31T19:00:00-05:00"]);
            }),
            Some(("labels", "Labels", 1)),
        ),
        (
            "text spelling two instants stays two keys",
            with(&|v| {
                v["labels"]["stamps"] = json!(["2020-01-01T00:00:00Z", "2020-01-01T00:00:01Z"]);
            }),
            None,
        ),
        (
            "one decimal in two spellings",
            with(&|v| v["labels"]["amounts"] = json!(["1", "1.0"])),
            Some(("labels", "Labels", 2)),
        ),
        (
            "two decimals that differ",
            with(&|v| v["labels"]["amounts"] = json!(["1", "10"])),
            None,
        ),
        (
            "a repeated enum value",
            with(&|v| v["levels"]["levels"] = json!(["Low", "Low"])),
            Some(("levels", "Levels", 0)),
        ),
    ]
}

/// Lists of structs keyed by a member, and a list nested in a list of structs.
fn member_cases() -> Vec<Case> {
    vec![
        (
            "a repeated member, a text through a newtype",
            with(&|v| v["registry"]["entries"][1]["name"] = json!("a")),
            Some(("registry", "Registry", 0)),
        ),
        (
            "a repeated member, an enum",
            with(&|v| v["registry"]["entries"][1]["level"] = json!("Low")),
            Some(("registry", "Registry", 1)),
        ),
        (
            "a repeated member, an Optional that is present",
            with(&|v| v["registry"]["entries"][1]["note"] = json!("x")),
            Some(("registry", "Registry", 2)),
        ),
        (
            "an Optional member absent from both is no shared key",
            with(&|v| {
                v["registry"]["entries"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("note");
                v["registry"]["entries"][1]
                    .as_object_mut()
                    .unwrap()
                    .remove("note");
            }),
            None,
        ),
        (
            "an Optional member absent from one",
            with(&|v| {
                v["registry"]["entries"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("note");
            }),
            None,
        ),
        (
            "an absent Optional key beside a known duplicate elsewhere",
            with(&|v| {
                let entries = v["registry"]["entries"].as_array_mut().unwrap();
                entries[0].as_object_mut().unwrap().remove("note");
                entries.push(json!({"name": "c", "level": "Low", "note": "z"}));
            }),
            Some(("registry", "Registry", 1)),
        ),
        (
            "two broken invariants: the first declared answers",
            with(&|v| {
                v["registry"]["entries"][1]["name"] = json!("a");
                v["registry"]["entries"][1]["level"] = json!("Low");
            }),
            Some(("registry", "Registry", 0)),
        ),
        (
            "a repeated member of a list of structs",
            with(&|v| v["catalog"]["groups"][1]["name"] = json!("g1")),
            Some(("catalog", "Catalog", 0)),
        ),
        (
            "a repeated element of a list nested in a list of structs",
            with(&|v| v["catalog"]["groups"][1]["items"] = json!([3, 3])),
            Some(("catalog", "Catalog", 1)),
        ),
        (
            "the same elements in two different nested lists",
            with(&|v| v["catalog"]["groups"][1]["items"] = json!([2, 1])),
            None,
        ),
        (
            "empty and one-element lists hold",
            with(&|v| {
                v["labels"]["tags"] = json!([]);
                v["labels"]["stamps"] = json!(["2020-01-01T00:00:00Z"]);
                v["levels"]["levels"] = json!([]);
                v["registry"]["entries"] = json!([]);
                v["catalog"]["groups"] = json!([{"name": "g", "items": []}]);
            }),
            None,
        ),
    ]
}

/// The words a decoder refuses a value in: the statement of the invariant the model declares,
/// read from the compiled model rather than spelled here.
fn expected_words(structure: &str, index: usize) -> String {
    let model = ir(MODEL);
    let declared = model
        .types()
        .values()
        .find(|declared| declared.name.local() == structure)
        .unwrap_or_else(|| panic!("the model declares `{structure}`"));
    let ResolvedBody::Struct { invariants, .. } = &declared.body else {
        panic!("`{structure}` is a struct");
    };
    let statement = &invariants[index].statement;
    assert!(
        statement.contains("distinct"),
        "`{structure}` invariant {index} is a distinct: {statement}"
    );
    format!("a value satisfying `{statement}`")
}

/// The inputs and the verdict on each, as both harnesses print it.
fn inputs_and_verdicts() -> (Vec<String>, Vec<String>) {
    let mut inputs = Vec::new();
    let mut verdicts = Vec::new();
    for (_, input, verdict) in cases() {
        inputs.push(input.to_string());
        verdicts.push(match verdict {
            None => "ok".to_owned(),
            Some((member, structure, index)) => {
                format!("refused $.{member} | {}", expected_words(structure, index))
            }
        });
    }
    (inputs, verdicts)
}

const RUST_HARNESS: &str = r#"use probe_server::{json, wire};

fn main() {
    for input in std::env::args().skip(1) {
        let document = json::parse(&input).expect("the input parses");
        match wire::decode_command_probe_range_set_all(&document, "$") {
            Ok(_) => println!("ok"),
            Err(refusal) => println!("refused {} | {}", refusal.at, refusal.expected),
        }
    }
}
"#;

const GO_HARNESS: &str = r#"package server

import (
	"errors"
	"fmt"
	"testing"
)

func TestDistinctInvariants(t *testing.T) {
	for _, input := range []string{
{cases}	} {
		document, refused := readJSON([]byte(input))
		if refused != nil {
			t.Fatalf("%s does not parse: %s", input, refused.body)
		}
		_, err := decodeCommandProbeRangeSetAll(document, "$")
		var refusal DecodeError
		switch {
		case err == nil:
			fmt.Println("case\tok")
		case errors.As(err, &refusal):
			fmt.Printf("case\trefused %s | %s\n", refusal.At, refusal.Expected)
		default:
			t.Fatalf("%s: %v", input, err)
		}
	}
}
"#;

#[test]
fn the_reporters_shape_is_six_structs_and_nine_distinct_invariants() {
    let model = ir(MODEL);
    let mut structs = 0;
    let mut distinct = 0;
    for declared in model.types().values() {
        if let ResolvedBody::Struct { invariants, .. } = &declared.body {
            structs += 1;
            distinct += invariants
                .iter()
                .filter(|invariant| invariant.predicate.reads_distinct())
                .count();
        }
    }
    assert_eq!((structs, distinct), (6, 9));
}

/// A workspace 0.93.0 generated generates again, for both targets.
#[test]
fn a_workspace_with_distinct_invariants_in_structs_generates_for_both_targets() {
    for target in [Target::Rust, Target::Go] {
        if let Err(refused) = synthesize_for(&ir(MODEL), target) {
            panic!("{target:?} refuses the workspace:\n{refused}");
        }
    }
}

#[test]
fn model_decoder_refuses_a_struct_whose_list_repeats_a_key() {
    let synthesis = synthesize_for(&ir(MODEL), Target::Rust).expect("the model synthesizes");
    let root = scratch("distinct-invariants");
    let generated = root.join("generated");
    write(&synthesis, &generated);
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    std::fs::write(
        harness.join("Cargo.toml"),
        "[package]\nname = \"distinct-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n\
         [dependencies]\nprobe-server = { path = \"../generated/crates/probe-server\" }\n\n\
         [workspace]\n",
    )
    .unwrap();
    std::fs::write(harness.join("src/main.rs"), RUST_HARNESS).unwrap();
    let (inputs, verdicts) = inputs_and_verdicts();
    let mut arguments = vec![
        "run".to_owned(),
        "--offline".to_owned(),
        "--quiet".to_owned(),
        "--target-dir".to_owned(),
        root.join("target").to_str().unwrap().to_owned(),
        "--".to_owned(),
    ];
    arguments.extend(inputs);
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let run = cargo(&harness, &arguments);
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&root);
    assert!(run.status.success(), "the harness builds and runs");
    for ((label, _, _), (got, want)) in cases()
        .iter()
        .zip(stdout.lines().zip(verdicts.iter().map(String::as_str)))
    {
        assert_eq!(got, want, "{label}");
    }
    assert_eq!(stdout.lines().count(), verdicts.len(), "{stdout}");
}

#[test]
fn go_server_decoder_refuses_a_struct_whose_list_repeats_a_key() {
    if !std::process::Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success())
    {
        eprintln!("no Go toolchain on this machine; the Go distinct check is unchecked here");
        return;
    }
    let synthesis = synthesize_for(&ir(MODEL), Target::Go).expect("the model synthesizes");
    let root = scratch("go-distinct-invariants");
    write(&synthesis, &root);
    let (inputs, verdicts) = inputs_and_verdicts();
    let literals = inputs.iter().fold(String::new(), |mut out, case| {
        let _ = writeln!(out, "\t\t`{case}`,");
        out
    });
    std::fs::write(
        root.join("server/zz_distinct_invariants_test.go"),
        GO_HARNESS.replace("{cases}", &literals),
    )
    .unwrap();
    let unformatted = super::go_tool(&root, "gofmt", &["-l", "."]);
    let vet = super::go_tool(&root, "go", &["vet", "./..."]);
    let run = super::go_tool(
        &root,
        "go",
        &[
            "test",
            "-count=1",
            "-run",
            "TestDistinctInvariants",
            "-v",
            "./server/",
        ],
    );
    let stdout = String::from_utf8_lossy(&run.stdout).into_owned();
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        unformatted.status.success() && unformatted.stdout.is_empty(),
        "the generated Go is gofmt-clean: {}",
        String::from_utf8_lossy(&unformatted.stdout)
    );
    assert!(vet.status.success(), "the generated Go is vet clean");
    assert!(
        run.status.success(),
        "the harness builds and runs: {stdout}"
    );
    let lines: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.strip_prefix("case\t"))
        .collect();
    for ((label, _, _), (got, want)) in cases()
        .iter()
        .zip(lines.iter().zip(verdicts.iter().map(String::as_str)))
    {
        assert_eq!(*got, want, "{label}");
    }
    assert_eq!(lines.len(), verdicts.len(), "{stdout}");
}

/// A struct that declares no `distinct:` keeps its bytes: neither target carries the key
/// comparison, and the evaluator is the one every model had.
#[test]
fn a_model_without_distinct_carries_no_key_comparison() {
    let model = "format: ess/22
system: probe
version: v1
domain: probe.range
types:
  - name: probe.range.Bag
    kind: struct
    fields:
      - {name: tags, type: 'List<String>'}
    invariants:
      - tags.count >= 0
events:
  - name: probe.range.BagSet
    fields: [{name: note, type: String}]
commands:
  - name: probe.range.SetBag
    input:
      - {name: bag, type: probe.range.Bag}
      - {name: note, type: String}
    outcomes:
      - name: set
        emits: [probe.range.BagSet]
        payload:
          probe.range.BagSet: {note: input.note}
components:
  - component: ranger
    owns: {domains: [probe.range]}
    accepts: {commands: [probe.range.SetBag]}
    publishes: {events: [probe.range.BagSet]}
    reached_by: network
";
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir(model), target).expect("the model synthesizes");
        for (path, artifact) in &synthesis.artifacts {
            assert!(
                ![
                    "iv::distinct",
                    "invariant.Distinct",
                    "fn distinct",
                    "func Distinct"
                ]
                .iter()
                .any(|marker| artifact.contents.contains(marker)),
                "`{path}` carries a key comparison for a model that declares no distinct:"
            );
        }
    }
}

/// An invariant a target cannot evaluate is named once for each, not once for each struct: the
/// first of a struct no longer hides the rest from an author fixing them one run at a time.
#[test]
fn every_struct_invariant_a_target_cannot_evaluate_is_named() {
    let model = "format: ess/22
system: probe
version: v1
domain: probe.range
types:
  - name: probe.range.Span
    kind: struct
    fields:
      - {name: low, type: Integer}
      - {name: high, type: Integer}
      - {name: tags, type: 'List<String>'}
    invariants:
      - high <= low + 5
      - low >= 0
      - {distinct: {in: tags, as: tag}}
      - high >= low - 5
      - high <= low + 9
  - name: probe.range.Other
    kind: struct
    fields:
      - {name: low, type: Integer}
      - {name: high, type: Integer}
    invariants:
      - high <= low + 1
      - high >= low - 1
events:
  - name: probe.range.SpanSet
    fields: [{name: note, type: String}]
commands:
  - name: probe.range.SetSpan
    input:
      - {name: span, type: probe.range.Span}
      - {name: other, type: probe.range.Other}
      - {name: note, type: String}
    outcomes:
      - name: set
        emits: [probe.range.SpanSet]
        payload:
          probe.range.SpanSet: {note: input.note}
components:
  - component: ranger
    owns: {domains: [probe.range]}
    accepts: {commands: [probe.range.SetSpan]}
    publishes: {events: [probe.range.SpanSet]}
    reached_by: network
";
    let every = [
        "types.probe.range.Other.invariants.0",
        "types.probe.range.Other.invariants.1",
        "types.probe.range.Span.invariants.0",
        "types.probe.range.Span.invariants.3",
        "types.probe.range.Span.invariants.4",
    ];
    for target in [Target::Rust, Target::Go] {
        let Err(refused) = synthesize_for(&ir(model), target) else {
            panic!("{target:?}: an offset invariant in a struct is refused");
        };
        let named: Vec<&str> = refused
            .causes()
            .iter()
            .flat_map(ess_synth::TargetFailureCause::sources)
            .map(String::as_str)
            .collect();
        assert_eq!(named, every, "{target:?}: {refused}");
        for cause in refused.causes() {
            assert!(
                cause.detail().contains("moves a fact by a constant"),
                "{target:?}: {}",
                cause.detail()
            );
        }
    }
}
