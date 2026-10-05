//! The generated Rust and Go behaviours and invariant checks decide the UTF-8 byte length of a
//! String (`docs/design/expression-family-source22.md`, "String `.utf8_bytes`"): `label.utf8_bytes >
//! 8` by `str::len` in Rust and `len(string)` in Go, never by scalar values, UTF-16 code units or
//! graphemes, and a Go string that is no UTF-8 — the bytes a lone surrogate leaves — is Unknown.
//! Each lane is compiled and run healthy, then once per paired fault patched into the emitted seam,
//! and each fault must fail. A view filter's byte length stays owed by name.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};
use std::path::{Path, PathBuf};
use std::process::Command;

const MODEL: &str = "format: ess/22
system: notes
version: v1
domain: notes.desk
entities:
  - name: notes.desk.Note
    identity: {name: note_id, type: Uuid}
    fields:
      - {name: label, type: String}
      - {name: tags, type: List<String>}
    invariants:
      - label.utf8_bytes <= 8
      - {forall: {in: tags, as: tag, that: tag.utf8_bytes != 4}}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - {name: notes.desk.Long, fields: []}
  - {name: notes.desk.Wide, fields: []}
  - {name: notes.desk.Noted, fields: []}
  - {name: notes.desk.Accepted, fields: []}
  - name: notes.desk.Kept
    fields:
      - {name: note_id, type: Uuid}
commands:
  - name: notes.desk.File
    input:
      - {name: label, type: String}
      - {name: code, type: String}
      - {name: note, type: Optional<String>}
    outcomes:
      - name: too-long
        when: label.utf8_bytes > 8
        emits: [notes.desk.Long]
      - name: wide
        when: code.utf8_bytes == 4
        emits: [notes.desk.Wide]
      - name: noted
        when: note.utf8_bytes >= 2
        emits: [notes.desk.Noted]
      - name: accepted
        emits: [notes.desk.Accepted]
  - name: notes.desk.Keep
    input:
      - {name: label, type: String}
      - {name: tags, type: List<String>}
    outcomes:
      - name: kept
        creates: notes.desk.Note
        instance: note_id
        sets: {label: input.label, tags: input.tags}
        emits: [notes.desk.Kept]
        payload: {notes.desk.Kept: {note_id: {generated: true}}}
components:
  - component: notes-service
    owns: {domains: [notes.desk]}
    accepts: {commands: [notes.desk.File, notes.desk.Keep]}
    publishes: {events: [notes.desk.Long, notes.desk.Wide, notes.desk.Noted, notes.desk.Accepted, notes.desk.Kept]}
";

fn ir_of(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("notes.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn emit(target: Target) -> PathBuf {
    let synthesis = synthesize_for(&ir_of(MODEL), target).unwrap_or_else(|error| {
        panic!("{target:?}: the byte-length model is generated: {error:?}")
    });
    assert!(
        synthesis
            .plan
            .is_generated(CapabilityKind::CommandBehavior, "notes.desk.File"),
        "{target:?}: the byte-length guards are generated, not owed"
    );
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "expression-utf8-bytes-{}-{}",
        target.name(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, artifact) in synthesis.artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    directory
}

/// Runs `program` in `directory`; its output and whether it succeeded.
fn run(directory: &Path, program: &str, arguments: &[&str]) -> (bool, String) {
    let mut command = Command::new(program);
    command.args(arguments).current_dir(directory);
    if program == env!("CARGO") {
        command
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env("RUSTFLAGS", "-D warnings");
    } else {
        command
            .env("GOWORK", "off")
            .env("GOFLAGS", "-mod=mod")
            .env("GOPROXY", "off");
    }
    let output = command.output().unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    (output.status.success(), log)
}

/// Replaces every `before` with `after` in the one emitted file that holds it.
fn patch(directory: &Path, file: &str, before: &str, after: &str) {
    let path = directory.join(file);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(before), "{file} holds `{before}`");
    std::fs::write(&path, text.replace(before, after)).unwrap();
}

const RUST_CASES: &str = r#"
use notes_types::{behaviour::Generated, desk::{File, FileOutcome, NoteData, obligations::FileBehavior}, primitives::Uuid};
fn file(label: &str, code: &str, note: Option<&str>) -> &'static str {
    // An accepting branch whose guard is Unknown leaves the outcome undeclared: the behaviour answers
    // with an unmet obligation rather than pick one.
    match Generated::new(())
        .file(File { label: label.to_owned(), code: code.to_owned(), note: note.map(str::to_owned) })
    {
        Ok(FileOutcome::TooLong { .. }) => "too-long",
        Ok(FileOutcome::Wide { .. }) => "wide",
        Ok(FileOutcome::Noted { .. }) => "noted",
        Ok(FileOutcome::Accepted { .. }) => "accepted",
        Err(_) => "unknown",
    }
}
fn note(label: &str, tags: &[&str]) -> Option<&'static str> {
    NoteData {
        note_id: Uuid("00000000-0000-4000-8000-000000000001".to_owned()),
        label: label.to_owned(),
        tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
    }
        .broken_invariant()
}
#[test]
fn guards() {
    let a = Some("a");
    assert_eq!(file("abcdefgh", "c", a), "accepted", "eight ASCII bytes are not over");
    assert_eq!(file("abcdefghi", "c", a), "too-long", "nine are");
    assert_eq!(file("éééé", "c", a), "accepted", "four two-byte scalars are eight bytes");
    assert_eq!(file("éééé!", "c", a), "too-long", "five scalars, nine bytes");
    assert_eq!(file("😀😀a", "c", a), "too-long", "three scalars, five UTF-16 units, nine bytes");
    assert_eq!(file("cafe\u{301}abc", "c", a), "too-long", "a decomposed accent is two scalars");
    assert_eq!(file("ab", "😀", a), "wide", "one scalar, two UTF-16 units, four bytes");
    assert_eq!(file("ab", "é!!", a), "wide", "three scalars, four bytes");
    assert_eq!(file("ab", "abc", a), "accepted", "three bytes");
    assert_eq!(file("ab", "c", Some("é")), "noted", "two bytes");
    assert_eq!(file("ab", "c", Some("")), "accepted", "empty text is zero bytes");
    assert_eq!(file("ab", "c", None), "unknown", "an absent text decides nothing");
}
#[test]
fn invariants() {
    assert_eq!(note("éééé", &[]), None);
    assert!(note("éééé!", &[]).is_some(), "nine bytes break the first invariant");
    assert!(note("ab", &["😀"]).is_some(), "a four-byte tag breaks the second");
    assert_eq!(note("ab", &["abc", "é€"]), None, "three and five bytes");
}
"#;

#[test]
fn utf8_byte_lengths_in_the_generated_rust_behaviour_and_every_fault_fails() {
    let directory = emit(Target::Rust);
    let tests = directory.join("crates/notes-types/tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("bytes.rs"), RUST_CASES).unwrap();
    let cargo = env!("CARGO");
    let arguments = ["test", "--offline", "-p", "notes-types", "--test", "bytes"];
    let (healthy, log) = run(&directory, cargo, &arguments);
    assert!(healthy && log.contains("2 passed; 0 failed"), "{log}");
    let behaviour = "crates/notes-types/src/behaviour.rs";
    let invariant = "crates/notes-types/src/desk.rs";
    let faults = [
        (
            behaviour,
            "scalar count",
            ".map(|value| value.len().to_string())",
            ".map(|value| value.chars().count().to_string())",
        ),
        (
            behaviour,
            "UTF-16 units",
            ".map(|value| value.len().to_string())",
            ".map(|value| value.encode_utf16().count().to_string())",
        ),
        (
            invariant,
            "scalar count in the invariant",
            ".as_str().len()",
            ".chars().count()",
        ),
        (
            invariant,
            "UTF-16 units in the invariant",
            ".as_str().len()",
            ".encode_utf16().count()",
        ),
    ];
    for (file, fault, before, after) in faults {
        patch(&directory, file, before, after);
        let (passed, log) = run(&directory, cargo, &arguments);
        assert!(
            !passed && log.contains("FAILED"),
            "{fault}: the faulty check passed\n{log}"
        );
        patch(&directory, file, after, before);
    }
    let _ = std::fs::remove_dir_all(&directory);
}

const GO_CASES: &str = r#"
package behaviour

import (
	"testing"

	"example.invalid/notes/types/desk"
)

// An accepting branch whose guard is Unknown leaves the outcome undeclared: the behaviour answers
// with an unmet obligation rather than pick one.
func file(label, code string, note *string) string {
	outcome, err := New(Ports{}).File(desk.File{Label: label, Code: code, Note: note})
	if err != nil {
		return "unknown"
	}
	switch outcome.(type) {
	case desk.FileOutcomeTooLong:
		return "too-long"
	case desk.FileOutcomeWide:
		return "wide"
	case desk.FileOutcomeNoted:
		return "noted"
	}
	return "accepted"
}

func text(value string) *string { return &value }

func TestGuards(t *testing.T) {
	// The bytes a lone surrogate leaves in a Go string: no UTF-8, so its length is Unknown.
	lone := string([]byte{0xed, 0xa0, 0x80})
	a := text("a")
	cases := []struct {
		name, label, code string
		note              *string
		want              string
	}{
		{"eight ASCII bytes are not over", "abcdefgh", "c", a, "accepted"},
		{"nine are", "abcdefghi", "c", a, "too-long"},
		{"four two-byte scalars are eight bytes", "éééé", "c", a, "accepted"},
		{"five scalars, nine bytes", "éééé!", "c", a, "too-long"},
		{"three scalars, five UTF-16 units, nine bytes", "😀😀a", "c", a, "too-long"},
		{"a decomposed accent is two scalars", "cafe\u0301abc", "c", a, "too-long"},
		{"one scalar, two UTF-16 units, four bytes", "ab", "😀", a, "wide"},
		{"three scalars, four bytes", "ab", "é!!", a, "wide"},
		{"three bytes", "ab", "abc", a, "accepted"},
		{"two bytes", "ab", "c", text("é"), "noted"},
		{"empty text is zero bytes", "ab", "c", text(""), "accepted"},
		{"an absent text decides nothing", "ab", "c", nil, "unknown"},
		{"a lone surrogate's bytes are no text", "ab", "c", text(lone + "x"), "unknown"},
		{"nor nine bytes of one", "abcdef" + lone, "c", a, "unknown"},
	}
	for _, c := range cases {
		if got := file(c.label, c.code, c.note); got != c.want {
			t.Errorf("%s: got %s, want %s", c.name, got, c.want)
		}
	}
}

func TestInvariants(t *testing.T) {
	note := func(label string, tags ...string) string {
		if tags == nil {
			tags = []string{}
		}
		broken, ok := desk.NoteData{Label: label, Tags: tags}.BrokenInvariant()
		if !ok {
			return "none"
		}
		return broken
	}
	if got := note("éééé"); got != "none" {
		t.Errorf("eight bytes: %s", got)
	}
	if got := note("éééé!"); got == "none" {
		t.Error("nine bytes break the first invariant")
	}
	if got := note("ab", "😀"); got == "none" {
		t.Error("a four-byte tag breaks the second")
	}
	if got := note("ab", "abc", "é€"); got != "none" {
		t.Errorf("three and five bytes: %s", got)
	}
	if got := note("abcdefgh" + string([]byte{0xed, 0xa0, 0x80})); got != "none" {
		t.Errorf("a label that is no UTF-8 decides nothing: %s", got)
	}
}
"#;

#[test]
fn utf8_byte_lengths_in_the_generated_go_behaviour_and_every_fault_fails() {
    let directory = emit(Target::Go);
    std::fs::write(directory.join("types/behaviour/bytes_test.go"), GO_CASES).unwrap();
    let arguments = [
        "test",
        "-count=1",
        "-run",
        "TestGuards|TestInvariants",
        "./types/behaviour",
    ];
    let (healthy, log) = run(&directory, "go", &arguments);
    assert!(healthy && log.contains("ok"), "{log}");
    let behaviour = "types/behaviour/behaviour.go";
    let invariant = "types/desk/desk.go";
    let faults = [
        (
            behaviour,
            "scalar count",
            "strconv.Itoa(len(",
            "strconv.Itoa(utf8.RuneCountInString(",
        ),
        (
            behaviour,
            "invalid text measured",
            "utf8.ValidString(",
            "0 < 1 || utf8.ValidString(",
        ),
        (
            invariant,
            "scalar count in the invariant",
            "Count(len(",
            "Count(utf8.RuneCountInString(",
        ),
        (
            invariant,
            "invalid text measured in the invariant",
            "utf8.ValidString(",
            "0 < 1 || utf8.ValidString(",
        ),
    ];
    for (file, fault, before, after) in faults {
        patch(&directory, file, before, after);
        let (passed, log) = run(&directory, "go", &arguments);
        assert!(
            !passed && log.contains("FAIL"),
            "{fault}: the faulty check passed\n{log}"
        );
        patch(&directory, file, after, before);
    }
    let _ = std::fs::remove_dir_all(&directory);
}

fn disposition(text: &str, kind: CapabilityKind, source: &str) -> String {
    let synthesis =
        synthesize_for(&ir_of(text), Target::Rust).unwrap_or_else(|_| panic!("synthesizes"));
    match synthesis.plan.disposition_of(kind, source) {
        Some(SynthesisDisposition::Obligation(obligation)) => {
            serde_json::to_string(&obligation.reason).unwrap()
        }
        other => format!("{other:?}"),
    }
}

#[test]
fn utf8_a_byte_length_in_a_view_filter_stays_owed_by_name() {
    let text = MODEL.replace(
        "events:\n",
        "views:\n  - name: notes.desk.Short\n    source: notes.desk.Note\n    consistency: read_your_writes\n    filter: label.utf8_bytes <= 4\n    fields:\n      - {name: note_id, type: Uuid}\nevents:\n",
    );
    let reason = disposition(&text, CapabilityKind::ViewQuery, "notes.desk.Short");
    assert!(
        reason.contains("byte length the generated view query does not compare"),
        "{reason}"
    );
}

// ---- a lone surrogate over the wire (decision 19) -------------------------------------------------

/// One served command guarded by a byte length, so a body is read, measured and answered.
const WIRE: &str = "format: ess/22
system: notes
version: v1
domain: notes.desk
events:
  - {name: notes.desk.Wide, fields: []}
  - {name: notes.desk.Accepted, fields: []}
commands:
  - name: notes.desk.File
    input:
      - {name: code, type: String}
    outcomes:
      - name: wide
        when: code.utf8_bytes == 3
        emits: [notes.desk.Wide]
      - name: accepted
        emits: [notes.desk.Accepted]
components:
  - component: notes-service
    owns: {domains: [notes.desk]}
    accepts: {commands: [notes.desk.File]}
    publishes: {events: [notes.desk.Wide, notes.desk.Accepted]}
    reached_by: network
";

/// The bodies each served surface is sent, and the status it must answer: a lone surrogate escape,
/// high or low, is refused as no JSON text; a pair, or the character itself, is read.
const BODIES: [(&str, u16); 5] = [
    (r#"{"code":"\ud800"}"#, 400),
    (r#"{"code":"\udc00"}"#, 400),
    (r#"{"code":"a\ud800b"}"#, 400),
    (r#"{"code":"€"}"#, 202),
    (r#"{"code":"😀"}"#, 202),
];

/// `(status, body)` per `answer` line a harness printed, in order.
fn answers(log: &str) -> Vec<(u16, String)> {
    log.lines()
        .filter_map(|line| line.strip_prefix("answer\t"))
        .map(|line| {
            let (status, body) = line.split_once('\t').expect("a status and a body");
            (status.parse().expect("a status"), body.to_owned())
        })
        .collect()
}

fn assert_wire(lane: &str, log: &str) {
    let answered = answers(log);
    assert_eq!(answered.len(), BODIES.len(), "{lane}: {log}");
    for ((body, status), (got, answer)) in BODIES.iter().zip(&answered) {
        assert_eq!(got, status, "{lane}: {body} answered {answer}");
    }
    assert!(
        answered[3].1.contains("\"wide\""),
        "{lane}: € is three bytes"
    );
    assert!(
        answered[4].1.contains("\"accepted\""),
        "{lane}: U+1F600 is four"
    );
}

const GO_WIRE: &str = r#"package server

import (
	"fmt"
	"net/http/httptest"
	"strings"
	"testing"

	"example.invalid/notes/components/notesservice"
	"example.invalid/notes/system"
	"example.invalid/notes/types/behaviour"
)

func TestWire(t *testing.T) {
	for _, body := range []string{BODIES} {
		s := system.NewSystem(notesservice.New(behaviour.New(behaviour.Ports{})))
		request := httptest.NewRequest("POST", "/desk/commands/File", strings.NewReader(body))
		answer := dispatchNotesService(s, request)
		fmt.Printf("answer\t%d\t%s\n", answer.status, strings.ReplaceAll(answer.body, "\n", " "))
	}
}
"#;

#[test]
fn utf8_a_lone_surrogate_escape_is_refused_by_the_generated_go_server() {
    let synthesis = synthesize_for(&ir_of(WIRE), Target::Go).expect("synthesizes");
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "expression-utf8-bytes-wire-go-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, artifact) in synthesis.artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let bodies: Vec<String> = BODIES.iter().map(|(body, _)| format!("`{body}`")).collect();
    std::fs::write(
        directory.join("server/wire_test.go"),
        GO_WIRE.replace("BODIES", &bodies.join(", ")),
    )
    .unwrap();
    let (passed, log) = run(
        &directory,
        "go",
        &["test", "-count=1", "-v", "-run", "TestWire", "./server"],
    );
    let _ = std::fs::remove_dir_all(&directory);
    assert!(passed, "{log}");
    assert_wire("go", &log);
}

const RUST_WIRE: &str = r#"use notes_server::notes_service as surface;
use notes_server::http;
use notes_types::behaviour::Generated;

fn main() {
    for body in [BODIES] {
        let mut system = notes_system::System::new(notes_service::NotesService::new(Generated::new(())));
        let request = http::Request {
            method: "POST".to_owned(),
            path: "/desk/commands/File".to_owned(),
            query: String::new(),
            headers: Vec::new(),
            body: body.as_bytes().to_vec(),
        };
        let answered = surface::dispatch(&mut system, &request);
        println!("answer\t{}\t{}", answered.status, answered.body.replace('\n', " "));
    }
}
"#;

#[test]
fn utf8_a_lone_surrogate_escape_is_refused_by_the_generated_rust_server() {
    let synthesis = ess_synth::synthesize_laid_out(
        &ir_of(WIRE),
        Target::Rust,
        ess_synth::OutputLayout::Workspace,
    )
    .expect("synthesizes");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "expression-utf8-bytes-wire-rust-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    for (relative, artifact) in synthesis.artifacts {
        let path = root.join("notes").join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).unwrap();
    let dependencies: String = [
        "notes-server",
        "notes-system",
        "notes-service",
        "notes-types",
    ]
    .iter()
    .fold(String::new(), |mut out, name| {
        use std::fmt::Write as _;
        let _ = writeln!(out, "{name} = {{ path = \"../notes/crates/{name}\" }}");
        out
    });
    std::fs::write(
        harness.join("Cargo.toml"),
        format!(
            "[package]\nname = \"wire-harness\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n\
             [dependencies]\n{dependencies}\n[workspace]\n"
        ),
    )
    .unwrap();
    let bodies: Vec<String> = BODIES
        .iter()
        .map(|(body, _)| format!("r#\"{body}\"#"))
        .collect();
    std::fs::write(
        harness.join("src/main.rs"),
        RUST_WIRE.replace("BODIES", &bodies.join(", ")),
    )
    .unwrap();
    let target = root.join("target");
    let (passed, log) = run(
        &harness,
        env!("CARGO"),
        &[
            "run",
            "--offline",
            "--quiet",
            "--target-dir",
            target.to_str().unwrap(),
        ],
    );
    let _ = std::fs::remove_dir_all(&root);
    assert!(passed, "{log}");
    assert_wire("rust", &log);
}
