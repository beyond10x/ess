//! D343 U1 adversary pass 1: `ess generate types --http` decodes the published body in Go and
//! TypeScript, which the unit's own CLI case checks only by reading the generated source text.
//!
//! The acceptance asks that `demo.notes.Create.input` decode `{"isPinned":true,"title":"Buy
//! milk"}` "in Rust, Go and TypeScript"; the unit builds and runs the Rust library only. These
//! build the Go package and run `go test` on it, and typecheck the TypeScript declaration with
//! `tsc`, both with the path's `note_id` added as the body that must be refused.

use ess_cli::TemporaryDirectory;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const NOTES: &str =
    include_str!("../../../generate/ess-gen/tests/fixtures/command-view-roots.yaml");

/// `demo.notes.Create` at `PUT /notes/{note_id}`, its `note_id` in the path.
const ROUTES: &str = "routes:
  - command: demo.notes.Create
    method: put
    path: /notes/{note_id}
    parameters: {note_id: input.note_id}
";

struct Fixture(TemporaryDirectory);

impl Fixture {
    /// The notes model with `notes.http.yaml` beside it, carrying the model's own digest.
    fn bound() -> Self {
        let path = TemporaryDirectory::create("ess-adversary-d343-u1").unwrap();
        fs::create_dir_all(path.join("model")).unwrap();
        fs::write(path.join("model/system.yaml"), NOTES).unwrap();
        let fixture = Self(path);
        let output = fixture.run(&[
            "--root",
            "demo.notes.NoteId",
            "--target",
            "typescript",
            "--out",
            "digest",
        ]);
        assert!(output.status.success(), "{output:?}");
        let source: serde_json::Value =
            serde_json::from_str(&fixture.read("digest/source.schema.json")).unwrap();
        let digest = source["x-ess-provenance"]["source_digest"]
            .as_str()
            .unwrap_or_else(|| panic!("no source digest in {source}"))
            .to_owned();
        fs::write(
            fixture.0.join("notes.http.yaml"),
            format!(
                "type: ess-http/1\nspecification:\n  system: demo\n  version: v1\n  source_digest: sha256:{digest}\n{ROUTES}"
            ),
        )
        .unwrap();
        fixture
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.0)
            .args(["generate", "types", "--path", "model"])
            .args(args)
            .output()
            .unwrap()
    }

    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }
}

fn printed(output: &Output) -> String {
    format!(
        "status {:?}\n{}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Decodes the published body, writes it back unchanged, and refuses the same body carrying the
/// path's `note_id`, as the bound `{command}.Input` refuses it.
const GO_WIRE: &str = r#"package demonotes

import (
	"encoding/json"
	"testing"
)

func TestBoundInputDecodesThePublishedBody(t *testing.T) {
	var decoded DemoNotesCreateInput
	if err := json.Unmarshal([]byte(`{"isPinned":true,"title":"Buy milk"}`), &decoded); err != nil {
		t.Fatalf("the published body is refused: %v", err)
	}
	encoded, err := json.Marshal(decoded)
	if err != nil {
		t.Fatalf("the decoded body does not encode: %v", err)
	}
	if string(encoded) != `{"isPinned":true,"title":"Buy milk"}` {
		t.Fatalf("the body does not round-trip: %s", encoded)
	}
	var carrying DemoNotesCreateInput
	if err := json.Unmarshal([]byte(`{"isPinned":true,"title":"Buy milk","note_id":"n-1"}`), &carrying); err == nil {
		t.Fatal("a body carrying the path field note_id is accepted")
	}
}
"#;

/// B1 (acceptance, Go). Green on 7978d1b5d is the expected outcome; red means the Go type does not
/// decode the published body or accepts the path field.
#[test]
fn the_bound_input_decodes_the_published_body_in_go_and_refuses_the_path_field() {
    let fixture = Fixture::bound();
    let output = fixture.run(&[
        "--http",
        "notes.http.yaml",
        "--root",
        "demo.notes.Create.input",
        "--target",
        "go",
        "--package",
        "demonotes",
        "--module",
        "example.invalid/notes",
        "--out",
        "go",
    ]);
    assert!(output.status.success(), "{}", printed(&output));
    let module = fixture.0.join("go");
    assert!(module.join("go.mod").exists(), "no go.mod in {module:?}");
    fs::write(module.join("wire_test.go"), GO_WIRE).unwrap();
    let result = Command::new("go")
        .args(["test", "-count=1", "./..."])
        .current_dir(&module)
        .env("GOTOOLCHAIN", "local")
        .env("GOPROXY", "off")
        .env("GOSUMDB", "off")
        .env("GOWORK", "off")
        .env("GOFLAGS", "")
        .output()
        .expect("`go` is on PATH");
    assert!(
        result.status.success(),
        "{}\n{}",
        fixture.read("go/types.go"),
        printed(&result)
    );
}

/// Assigns the published body to the generated type, and expects `tsc` to refuse the same object
/// literal carrying the path's `note_id` (an excess property): an unused `@ts-expect-error` is
/// itself an error, so a body type that still declares `note_id` fails the check.
const TS_CHECK: &str = r#"import type { DemoNotesCreateInput } from "./types";

export const published: DemoNotesCreateInput = { isPinned: true, title: "Buy milk" };

// @ts-expect-error the path field note_id is not part of the published body
export const carrying: DemoNotesCreateInput = { isPinned: true, title: "Buy milk", note_id: "n-1" };
"#;

/// B2 (acceptance, TypeScript). Green on 7978d1b5d is the expected outcome.
#[test]
fn the_bound_input_typechecks_the_published_body_in_typescript_and_refuses_the_path_field() {
    let fixture = Fixture::bound();
    let output = fixture.run(&[
        "--http",
        "notes.http.yaml",
        "--root",
        "demo.notes.Create.input",
        "--target",
        "typescript",
        "--out",
        "typescript",
    ]);
    assert!(output.status.success(), "{}", printed(&output));
    let package = fixture.0.join("typescript");
    fs::write(package.join("check.ts"), TS_CHECK).unwrap();
    let result = typecheck(&package);
    assert!(
        result.status.success(),
        "{}\n{}",
        fixture.read("typescript/types.ts"),
        printed(&result)
    );
}

fn typecheck(package: &Path) -> Output {
    Command::new("tsc")
        .args([
            "--strict",
            "--noEmit",
            "--target",
            "es2022",
            "--module",
            "esnext",
            "--moduleResolution",
            "bundler",
            "check.ts",
        ])
        .current_dir(package)
        .output()
        .expect("`tsc` is on PATH")
}
