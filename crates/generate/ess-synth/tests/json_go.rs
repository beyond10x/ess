//! The Go target represents `Json` (beyond10x/ess#224).
//!
//! As the generated `primitives.Json`: a wrapper over the document's text, exactly as `Decimal`
//! wraps its wire rendering, at every position the Go target types — a newtype, a struct member
//! (bare, in a list, in a map, optional), a union variant, an entity field, a command input and
//! response, an event payload, an error payload, a view row and a binding's copied field. The
//! served surface reads a `Json` value with its object members in the order they arrived and its
//! numbers in the spelling they arrived in, and writes it back unchanged.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Synthesis, Target};

/// `Json` at every position the Go target types, served over the network so the server package's
/// wire codecs are emitted too, with a binding that copies a `Json` event field into a command.
const EVERY_POSITION: &str = "format: ess/15
system: demo
version: v1
domain: demo.docs
types:
  - {name: demo.docs.Body, kind: newtype, of: Json}
  - name: demo.docs.Envelope
    kind: struct
    fields:
      - {name: body, type: Json}
      - {name: items, type: List<Json>}
      - {name: labels, type: \"Map<String, Json>\"}
      - {name: note, type: Optional<Json>}
  - name: demo.docs.Either
    kind: union
    tag: kind
    variants:
      raw: Json
      text: String
entities:
  - name: demo.docs.Doc
    identity: {name: id, type: String}
    fields:
      - {name: body, type: Json}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
events:
  - name: demo.docs.Stored
    fields:
      - {name: id, type: String}
      - {name: body, type: Json}
      - {name: wrapped, type: demo.docs.Body}
  - name: demo.docs.Noted
    fields:
      - {name: id, type: String}
      - {name: body, type: Json}
commands:
  - name: demo.docs.Store
    input:
      - {name: id, type: String}
      - {name: body, type: Json}
      - {name: wrapped, type: demo.docs.Body}
      - {name: envelope, type: demo.docs.Envelope}
      - {name: either, type: demo.docs.Either}
      - {name: items, type: List<Json>}
      - {name: labels, type: \"Map<String, Json>\"}
    response:
      - {name: echo, type: Json}
    outcomes:
      - name: stored
        emits: [demo.docs.Stored]
        payload:
          demo.docs.Stored: {id: input.id, body: input.body, wrapped: input.wrapped}
      - name: refused
        external: the store refuses the document
        error: demo.docs.Refused
  - name: demo.docs.Note
    input:
      - {name: id, type: String}
      - {name: body, type: Json}
    outcomes:
      - name: noted
        emits: [demo.docs.Noted]
        payload:
          demo.docs.Noted: {id: input.id, body: input.body}
errors:
  - name: demo.docs.Refused
    fields:
      - {name: detail, type: Json}
views:
  - name: demo.docs.Docs
    source: demo.docs.Doc
    consistency: eventual
    fields:
      - {name: id, type: String}
      - {name: body, type: Json}
components:
  - component: docs-service
    owns: {domains: [demo.docs]}
    accepts: {commands: [demo.docs.Store, demo.docs.Note]}
    publishes: {events: [demo.docs.Stored, demo.docs.Noted]}
    reached_by: network
bindings:
  - id: note-on-stored
    when: {event: demo.docs.Stored}
    invoke: {command: demo.docs.Note}
    mapping:
      id: event.id
      body: event.body
    delivery: at_least_once
    on_failure: retry
";

/// The same system with no `Json` anywhere.
const WITHOUT_JSON: &str = "format: ess/15
system: demo
version: v1
domain: demo.msgs
types:
  - {name: demo.msgs.Body, kind: newtype, of: String}
events:
  - name: demo.msgs.Sent
    fields:
      - {name: body, type: demo.msgs.Body}
      - {name: headers, type: \"Map<String, String>\"}
commands:
  - name: demo.msgs.Send
    input:
      - {name: body, type: demo.msgs.Body}
    outcomes:
      - name: sent
        emits: [demo.msgs.Sent]
        payload:
          demo.msgs.Sent: {body: input.body, headers: {generated: true}}
components:
  - component: msgs-service
    owns: {domains: [demo.msgs]}
    accepts: {commands: [demo.msgs.Send]}
    publishes: {events: [demo.msgs.Sent]}
    reached_by: network
";

/// The throwaway test file that exercises the generated server package from outside the generated
/// bytes: it is written beside them into a scratch copy, never emitted.
///
/// The command input carries all six JSON kinds at the top level of a `Json` field, inside a
/// newtype, a struct member, a union variant, a list and a map. Its own top-level members and every
/// `Map<String, Json>` are in key order, because `encoding/json` writes a Go map's keys sorted
/// (TARGET.md records that weakening); a `Json` object's members are deliberately not, and keep
/// the order they arrived in. Numbers are spelled the way no float would write them back.
const HARNESS: &str = r#"package server

import (
	"encoding/json"
	"testing"

	"example.invalid/demo/types/docs"
	"example.invalid/demo/types/primitives"
)

const input = `{"body":{"object":{"k":"v"},"array":[1,"two",[]],"number":-12.50e3,"string":"say \"hi\"","bool":true,"null":null},"either":{"kind":"raw","value":{"z":[true],"a":0.10}},"envelope":{"body":null,"items":[{"b":1,"a":2},[2],3.0,"four",true,null],"labels":{"a":[],"b":false,"n":1E2,"o":{"y":1,"x":2},"s":"x","z":null},"note":{"deep":[{"deeper":null}]}},"id":"d1","items":[null,{"n":"m","m":"n"}],"labels":{"only":[1,{"two":2,"one":1}]},"wrapped":[{},[],0,"",false,null]}`

func marshal(t *testing.T, value any) string {
	t.Helper()
	encoded, err := json.Marshal(value)
	if err != nil {
		t.Fatalf("encoding: %v", err)
	}
	return string(encoded)
}

func TestJsonRoundTrip(t *testing.T) {
	value, refused := readJSON([]byte(input))
	if refused != nil {
		t.Fatalf("the input is refused: %s", refused.body)
	}
	decoded, err := decodeCommandDemoDocsStore(value, "body")
	if err != nil {
		t.Fatalf("the input does not decode: %v", err)
	}
	var body primitives.Json = decoded.Body
	if got, want := body.Value(), `{"object":{"k":"v"},"array":[1,"two",[]],"number":-12.50e3,"string":"say \"hi\"","bool":true,"null":null}`; got != want {
		t.Fatalf("a bare Json member:\n got %s\nwant %s", got, want)
	}

	items := make([]any, 0, len(decoded.Items))
	for _, item := range decoded.Items {
		items = append(items, json.RawMessage(item.Value()))
	}
	labels := map[string]any{}
	for key, label := range decoded.Labels {
		labels[key] = json.RawMessage(label.Value())
	}
	again := marshal(t, map[string]any{
		"body":     json.RawMessage(decoded.Body.Value()),
		"either":   encodeDemoDocsEither(decoded.Either),
		"envelope": encodeDemoDocsEnvelope(decoded.Envelope),
		"id":       decoded.Id,
		"items":    items,
		"labels":   labels,
		"wrapped":  encodeDemoDocsBody(decoded.Wrapped),
	})
	if again != input {
		t.Fatalf("a command input does not round-trip byte for byte:\n got %s\nwant %s", again, input)
	}

	for _, kind := range []string{`{"b":1,"a":[2]}`, `[3,"x",null]`, `-0.500E+01`, `"text"`, `true`, `false`, `null`} {
		note, refused := readJSON([]byte(`{"body":` + kind + `,"id":"n"}`))
		if refused != nil {
			t.Fatalf("%s is refused: %s", kind, refused.body)
		}
		read, err := decodeCommandDemoDocsNote(note, "body")
		if err != nil {
			t.Fatalf("%s does not decode: %v", kind, err)
		}
		if read.Body.Value() != kind {
			t.Fatalf("%s decodes as %s", kind, read.Body.Value())
		}
		if written := marshal(t, json.RawMessage(read.Body.Value())); written != kind {
			t.Fatalf("%s is written as %s", kind, written)
		}
	}

	row := docs.Docs{Id: "d1", Body: primitives.NewJson(`{"z":[1,2],"a":0.5}`)}
	if got, want := rendered(200, encodeViewDemoDocsDocs(row)).body, `{"body":{"z":[1,2],"a":0.5},"id":"d1"}`; got != want {
		t.Fatalf("a view row:\n got %s\nwant %s", got, want)
	}
	refusal := docs.Refused{Detail: primitives.NewJson(`{"why":"no","code":7.0}`)}
	if got, want := marshal(t, encodeErrorDemoDocsRefused(refusal)), `{"detail":{"why":"no","code":7.0}}`; got != want {
		t.Fatalf("an error payload:\n got %s\nwant %s", got, want)
	}
	if got := (primitives.Json{}).Value(); got != "null" {
		t.Fatalf("the zero Json is %q, not null", got)
	}
	t.Log("json round trip: command input, six kinds, view row and error payload unchanged")
}
"#;

fn ir(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("spec.yaml"),
        RawSpecFile::parse(source).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", source);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn go(source: &str) -> Synthesis {
    synthesize_for(&ir(source), Target::Go)
        .unwrap_or_else(|failure| panic!("the Go target represents Json: {failure}"))
}

fn artifact<'a>(synthesis: &'a Synthesis, path: &str) -> &'a str {
    &synthesis
        .artifacts
        .get(path)
        .unwrap_or_else(|| {
            panic!(
                "`{path}` was not emitted; emitted: {:?}",
                synthesis.artifacts.keys().collect::<Vec<_>>()
            )
        })
        .contents
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("json-go-{label}"))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(&destination, &artifact.contents).unwrap();
    }
}

/// A Go tool inside a generated module, offline: no module download, no toolchain switch.
fn tool(program: &str, directory: &Path, arguments: &[&str]) -> Output {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(directory)
        .env("GOCACHE", scratch("gocache"))
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOTOOLCHAIN", "local")
        .output()
        .unwrap_or_else(|error| panic!("`{program}` runs: {error}"));
    eprintln!(
        "{program} {arguments:?} in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
fn issue_224_go_names_primitives_json_at_every_position() {
    let synthesis = go(EVERY_POSITION);
    let primitives = artifact(&synthesis, "types/primitives/primitives.go");
    assert!(primitives.contains("type Json struct {"), "{primitives}");
    assert!(
        primitives.contains("func NewJson(value string) Json {"),
        "{primitives}"
    );

    let module = artifact(&synthesis, "types/docs/docs.go");
    for position in [
        "\tvalue primitives.Json\n",
        "\tBody primitives.Json\n",
        "\tItems []primitives.Json\n",
        "\tLabels map[string]primitives.Json\n",
        "\tNote *primitives.Json\n",
        "\tValue primitives.Json\n",
        "\tEcho primitives.Json\n",
        "\tDetail primitives.Json\n",
    ] {
        assert!(module.contains(position), "`{position}` missing:\n{module}");
    }
    // The envelope, the entity's data, both command inputs, both events and the view row each
    // carry a `body: Json` — one occurrence per record, so count them rather than find one.
    let bodies = module.matches("\tBody primitives.Json\n").count();
    assert!(
        bodies >= 7,
        "Envelope, Doc, Store, Note, Stored, Noted and Docs each carry `Body`; found \
         {bodies}:\n{module}"
    );

    let wire = artifact(&synthesis, "server/wire.go");
    assert!(wire.contains("jsonAt("), "{wire}");
    assert!(wire.contains("json.RawMessage("), "{wire}");
    assert!(
        synthesis
            .artifacts
            .iter()
            .any(|(path, artifact)| path.starts_with("system/")
                && artifact.contents.contains("note-on-stored")),
        "the binding carrying a Json field is emitted"
    );

    let directory = scratch("every-position");
    write(&synthesis, &directory);
    let unformatted = tool("gofmt", &directory, &["-l", "."]);
    assert!(unformatted.status.success(), "gofmt runs");
    assert_eq!(
        String::from_utf8_lossy(&unformatted.stdout),
        "",
        "the generated Go is not gofmt-clean"
    );
    assert!(
        tool("go", &directory, &["vet", "./..."]).status.success(),
        "`go vet` refuses the generated module"
    );
    assert!(
        tool("go", &directory, &["build", "./..."]).status.success(),
        "`go build` refuses the generated module"
    );
}

#[test]
fn issue_224_json_round_trips_through_the_go_server_package() {
    let synthesis = go(EVERY_POSITION);
    let directory = scratch("round-trip");
    write(&synthesis, &directory);
    std::fs::write(directory.join("server/zz_json_round_trip_test.go"), HARNESS).unwrap();
    let run = tool(
        "go",
        &directory,
        &[
            "test",
            "-count=1",
            "-run",
            "TestJsonRoundTrip",
            "-v",
            "./server/",
        ],
    );
    assert!(
        run.status.success(),
        "the six JSON kinds do not round-trip through the generated server package"
    );
    assert!(
        String::from_utf8_lossy(&run.stdout).contains(
            "json round trip: command input, six kinds, view row and error payload unchanged"
        ),
        "the harness did not reach its end"
    );
}

#[test]
fn issue_224_a_go_model_without_json_carries_no_json_machinery() {
    let synthesis = go(WITHOUT_JSON);
    let primitives = artifact(&synthesis, "types/primitives/primitives.go");
    assert!(!primitives.contains("Json"), "{primitives}");
    let wire = artifact(&synthesis, "server/wire.go");
    for absent in ["jsonAt", "type object struct", "\"bytes\""] {
        assert!(!wire.contains(absent), "`{absent}` in:\n{wire}");
    }
    let server = artifact(&synthesis, "server/server.go");
    assert!(!server.contains("ordered("), "{server}");
}
