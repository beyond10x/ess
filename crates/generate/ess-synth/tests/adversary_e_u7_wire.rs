//! Adversary, pass 1, for unit E-U7: a lone surrogate reaching the generated Go and Rust
//! applications over the wire (`docs/design/expression-family-source22.md`, final review decision
//! 19: "text with a lone surrogate is Unknown in every lane").

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};
use std::path::Path;

const MODEL: &str = "format: ess/22
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

fn ir_of(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("notes.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn emit(target: Target, directory: &Path) {
    let synthesis = synthesize_for(&ir_of(MODEL), target)
        .unwrap_or_else(|error| panic!("{target:?}: generated: {error:?}"));
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in synthesis.artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
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

// The JSON escape of a lone surrogate, as a client sends it: Go's decoder puts U+FFFD — three
// bytes of valid UTF-8 — in its place, so `utf8.ValidString` passes and the byte length is 3.
func TestLoneSurrogateOverTheWire(t *testing.T) {
	for _, body := range []string{`{"code":"\ud800"}`, `{"code":"\udc00"}`} {
		s := system.NewSystem(notesservice.New(behaviour.New(behaviour.Ports{})))
		request := httptest.NewRequest("POST", "/desk/commands/File", strings.NewReader(body))
		answer := dispatchNotesService(s, request)
		fmt.Printf("answer\t%s\t%d\t%s\n", body, answer.status, strings.ReplaceAll(answer.body, "\n", " "))
		if strings.Contains(answer.body, `"wide"`) {
			t.Errorf("%s: a lone surrogate was measured as three UTF-8 bytes and took `wide` (%d)", body, answer.status)
		}
	}
}
"#;

/// Decision 19: a lone surrogate has no UTF-8 byte length in any lane. The generated Go guard's
/// `utf8.ValidString` is the unit's mechanism for it; this sends the surrogate the way a client
/// can, as a JSON escape, and asks that the `code.utf8_bytes == 3` branch not be taken.
#[test]
fn adv_u7_generated_go_does_not_measure_a_lone_surrogate_sent_over_the_wire() {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-e-u7-wire-{}", std::process::id()));
    emit(Target::Go, &directory);
    std::fs::write(directory.join("server/wire_test.go"), GO_WIRE).unwrap();
    let output = std::process::Command::new("go")
        .args([
            "test",
            "-count=1",
            "-v",
            "-run",
            "TestLoneSurrogateOverTheWire",
            "./server",
        ])
        .current_dir(&directory)
        .env("GOWORK", "off")
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(&directory);
    assert!(log.contains("answer\t"), "the Go test ran: {log}");
    assert!(output.status.success(), "{log}");
}

/// A newtype of String and Optionals of both, in guards and in entity invariants: the generated
/// Rust and Go must compile and decide by bytes. The unit's generated-lane model holds only
/// `String`, `Optional<String>` in a guard and `List<String>` in an invariant.
const WRAPPED: &str = "format: ess/22
system: notes
version: v1
domain: notes.desk
types:
  - {name: notes.desk.Title, kind: newtype, of: String}
entities:
  - name: notes.desk.Note
    identity: {name: note_id, type: Uuid}
    fields:
      - {name: title, type: notes.desk.Title}
      - {name: nick, type: Optional<String>}
      - {name: alias, type: Optional<notes.desk.Title>}
    invariants:
      - title.utf8_bytes <= 8
      - nick.utf8_bytes <= 4
      - alias.utf8_bytes != 4
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - {name: notes.desk.Long, fields: []}
  - {name: notes.desk.Wide, fields: []}
  - {name: notes.desk.Fine, fields: []}
  - name: notes.desk.Kept
    fields:
      - {name: note_id, type: Uuid}
commands:
  - name: notes.desk.Check
    input:
      - {name: title, type: notes.desk.Title}
      - {name: alias, type: Optional<notes.desk.Title>}
    outcomes:
      - name: long
        when: title.utf8_bytes > 8
        emits: [notes.desk.Long]
      - name: wide
        when: alias.utf8_bytes == 4
        emits: [notes.desk.Wide]
      - name: fine
        emits: [notes.desk.Fine]
  - name: notes.desk.Keep
    input:
      - {name: title, type: notes.desk.Title}
      - {name: nick, type: Optional<String>}
      - {name: alias, type: Optional<notes.desk.Title>}
    outcomes:
      - name: kept
        creates: notes.desk.Note
        instance: note_id
        sets: {title: input.title, nick: input.nick, alias: input.alias}
        emits: [notes.desk.Kept]
        payload: {notes.desk.Kept: {note_id: {generated: true}}}
components:
  - component: notes-service
    owns: {domains: [notes.desk]}
    accepts: {commands: [notes.desk.Check, notes.desk.Keep]}
    publishes: {events: [notes.desk.Long, notes.desk.Wide, notes.desk.Fine, notes.desk.Kept]}
";

const GO_WRAPPED: &str = r#"package behaviour

import (
	"testing"

	"example.invalid/notes/types/desk"
)

func keep(title string, alias *string) string {
	var wrapped *desk.Title
	if alias != nil {
		value := desk.NewTitle(*alias)
		wrapped = &value
	}
	outcome, err := New(Ports{}).Check(desk.Check{Title: desk.NewTitle(title), Alias: wrapped})
	if err != nil {
		return "unknown"
	}
	switch outcome.(type) {
	case desk.CheckOutcomeLong:
		return "long"
	case desk.CheckOutcomeWide:
		return "wide"
	}
	return "fine"
}

func text(value string) *string { return &value }

func TestWrapped(t *testing.T) {
	cases := []struct{ name, title string; alias *string; want string }{
		{"four two-byte scalars", "éééé", text("ab"), "fine"},
		{"five scalars, nine bytes", "éééé!", text("ab"), "long"},
		{"one scalar, four bytes", "ab", text("😀"), "wide"},
		{"three bytes", "ab", text("abc"), "fine"},
	}
	for _, c := range cases {
		if got := keep(c.title, c.alias); got != c.want {
			t.Errorf("%s: got %s, want %s", c.name, got, c.want)
		}
	}
}
"#;

fn ir_wrapped() -> EssIr {
    ir_of(WRAPPED)
}

#[test]
fn adv_u7_generated_go_decides_wrapped_texts_by_bytes() {
    let synthesis = synthesize_for(&ir_wrapped(), Target::Go)
        .unwrap_or_else(|error| panic!("generated: {error:?}"));
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-e-u7-wrapped-go-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, artifact) in synthesis.artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("types/behaviour/wrapped_test.go"),
        GO_WRAPPED,
    )
    .unwrap();
    let declared: String = std::fs::read_to_string(directory.join("types/desk/desk.go"))
        .unwrap_or_default()
        .lines()
        .filter(|line| line.contains("Title"))
        .take(12)
        .collect::<Vec<_>>()
        .join("\n");
    let output = std::process::Command::new("go")
        .args([
            "test",
            "-count=1",
            "-run",
            "TestWrapped",
            "./types/behaviour",
        ])
        .current_dir(&directory)
        .env("GOWORK", "off")
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(&directory);
    assert!(
        output.status.success() && log.contains("ok"),
        "{log}\n{declared}"
    );
}

const RUST_WRAPPED: &str = r#"
use notes_types::{behaviour::Generated, desk::{Check, CheckOutcome, NoteData, Title, obligations::CheckBehavior}, primitives::Uuid};
fn check(title: &str, alias: Option<&str>) -> &'static str {
    match Generated::new(()).check(Check {
        title: Title(title.to_owned()),
        alias: alias.map(|alias| Title(alias.to_owned())),
    }) {
        Ok(CheckOutcome::Long { .. }) => "long",
        Ok(CheckOutcome::Wide { .. }) => "wide",
        Ok(CheckOutcome::Fine { .. }) => "fine",
        Err(_) => "unknown",
    }
}
#[test]
fn wrapped() {
    assert_eq!(check("éééé", Some("ab")), "fine");
    assert_eq!(check("éééé!", Some("ab")), "long");
    assert_eq!(check("ab", Some("😀")), "wide");
    assert_eq!(check("ab", Some("abc")), "fine");
    let note = |title: &str, nick: Option<&str>, alias: Option<&str>| NoteData {
        note_id: Uuid("00000000-0000-4000-8000-000000000001".to_owned()),
        title: Title(title.to_owned()),
        nick: nick.map(str::to_owned),
        alias: alias.map(|alias| Title(alias.to_owned())),
    }
    .broken_invariant();
    assert_eq!(note("éééé", Some("é!"), Some("abc")), None);
    assert!(note("éééé!", None, None).is_some(), "nine bytes of title");
    assert!(note("ab", Some("éé!"), None).is_some(), "five bytes of nick");
    assert!(note("ab", None, Some("😀")).is_some(), "four bytes of alias");
}
"#;

#[test]
fn adv_u7_generated_rust_decides_wrapped_texts_by_bytes() {
    let synthesis = synthesize_for(&ir_wrapped(), Target::Rust)
        .unwrap_or_else(|error| panic!("generated: {error:?}"));
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-e-u7-wrapped-rust-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, artifact) in synthesis.artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let tests = directory.join("crates/notes-types/tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("wrapped.rs"), RUST_WRAPPED).unwrap();
    let output = std::process::Command::new(env!("CARGO"))
        .args([
            "test",
            "--offline",
            "-p",
            "notes-types",
            "--test",
            "wrapped",
        ])
        .current_dir(&directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .output()
        .unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let _ = std::fs::remove_dir_all(&directory);
    assert!(output.status.success() && log.contains("1 passed"), "{log}");
}
