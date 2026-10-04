//! A served branch declaring `returns: true` is answered `200` with the command's response under
//! `response`, from `ess/22` (beyond10x/ess#423, beyond10x/ess#424), by the generated Rust and Go
//! servers alike, and the answer validates against the document the projection publishes.
//!
//! The model is the issues' reproduction. Its command has a typed response, so its behaviour is an
//! obligation; each harness realizes it by returning the response and the event the branch emits,
//! and drives the generated surface's own dispatch.
//!
//! Below `ess/22` the same model keeps the variant, the status and the body it was synthesized
//! with: a realization of an `ess/21` command was never asked for a response, and its clients were
//! told `202`.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::http::{self, Served};
use ess_synth::{synthesize_laid_out, OutputLayout, Synthesis, Target};
use serde_json::{json, Value};

const MODEL: &str = "format: ess/22
system: catalogue
version: v1
domain: catalogue.search
naming:
  wire: search
  display: Search
entities:
  - name: catalogue.search.Title
    identity:
      name: title_id
      type: Uuid
    fields:
      - name: name
        type: String
    lifecycle:
      initial: Listed
      states: [Listed]
      terminal: [Listed]
events:
  - name: catalogue.search.TitlesFound
    fields:
      - name: count
        type: Integer
commands:
  - name: catalogue.search.FindTitles
    naming:
      wire: find-titles
      display: Find titles
    input:
      - name: text
        type: String
    response:
      - name: names
        type: List<String>
    outcomes:
      - name: found
        returns: true
        emits:
          - catalogue.search.TitlesFound
        payload:
          catalogue.search.TitlesFound:
            count: {generated: true}
        summary: The names of every title whose name contains the text.
actors:
  - name: catalogue.search.Reader
    may:
      - catalogue.search.FindTitles
components:
  - component: catalogue-reader
    summary: Answers searches over the catalogue.
    owns:
      domains:
        - catalogue.search
    accepts:
      commands:
        - catalogue.search.FindTitles
    publishes:
      events:
        - catalogue.search.TitlesFound
    reached_by: network
";

/// The obligated behaviour, realized: every name containing the text, and their count.
const RUST_HARNESS: &str = r##"use catalogue_server::catalogue_reader as surface;
use catalogue_server::http;
use catalogue_types::actor::{Actor, Caller};
use catalogue_types::obligation::UnmetObligation;
use catalogue_types::search;
use catalogue_types::search::obligations::FindTitlesBehavior;

struct Shelf;

impl FindTitlesBehavior for Shelf {
    fn find_titles(&mut self, input: search::FindTitles) -> Result<search::FindTitlesOutcome, UnmetObligation> {
        let names: Vec<String> = ["Dune", "Dune Messiah", "Emma"]
            .iter()
            .filter(|name| name.contains(input.text.as_str()))
            .map(|name| (*name).to_owned())
            .collect();
        Ok(search::FindTitlesOutcome::Found {
            titles_found: search::TitlesFound { count: names.len() as i64 },
            response: search::FindTitlesResponse { names },
        })
    }
}

fn main() {
    let path = std::env::args().nth(1).expect("the route");
    let mut system = catalogue_system::System::new(catalogue_reader::CatalogueReader::new(Shelf));
    let caller = Caller { actor: Actor::ALL[0] };
    let request = http::Request {
        method: "POST".to_owned(),
        path,
        query: String::new(),
        headers: Vec::new(),
        body: br#"{"text":"Dune"}"#.to_vec(),
    };
    let answered = surface::dispatch(&mut system, Some(&caller), &request);
    println!("answer\t{}\t{}", answered.status, answered.body.replace('\n', " "));
}
"##;

const GO_HARNESS: &str = r#"package server

import (
	"fmt"
	"net/http/httptest"
	"strings"
	"testing"

	"example.invalid/catalogue/components/cataloguereader"
	"example.invalid/catalogue/system"
	"example.invalid/catalogue/types/obligation"
	"example.invalid/catalogue/types/search"
)

type shelf struct{}

func (s *shelf) FindTitles(input search.FindTitles) (search.FindTitlesOutcome, *obligation.UnmetObligation) {
	names := []string{}
	for _, name := range []string{"Dune", "Dune Messiah", "Emma"} {
		if strings.Contains(name, input.Text) {
			names = append(names, name)
		}
	}
	return search.FindTitlesOutcomeFound{
		Response:    search.FindTitlesResponse{Names: names},
		TitlesFound: search.TitlesFound{Count: int64(len(names))},
	}, nil
}

func TestServedReturns(t *testing.T) {
	s := system.NewSystem(cataloguereader.New(&shelf{}))
	request := httptest.NewRequest("POST", "/search/commands/find-titles", strings.NewReader(`{"text":"Dune"}`))
	answer := dispatchCatalogueReader(s, &Caller{Actor: ActorCatalogueSearchReader}, request)
	fmt.Printf("answer\t%d\t%s\n", answer.status, strings.ReplaceAll(answer.body, "\n", " "))
}
"#;

fn ir(format: &str) -> EssIr {
    let source = MODEL.replace("ess/22", format);
    let spec = Specification::assemble([(
        Source::new("catalogue.yaml"),
        RawSpecFile::parse(&source).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("{format} validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("catalogue.yaml", source.as_str());
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("{format} compiles: {errors}"))
}

fn route(ir: &EssIr) -> String {
    let component = ir.components().values().next().expect("one component");
    http::routes(ir, component)
        .into_iter()
        .find(|route| matches!(route.serves, Served::Command(_)))
        .expect("the command's route")
        .path
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("served-returns-{label}-{}", std::process::id()))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

fn report(what: &str, output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    eprintln!(
        "{what}\n{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    stdout
}

/// `(status, body)` from the harness's `answer` line.
fn answer(stdout: &str) -> (u16, Value) {
    let line = stdout
        .lines()
        .find_map(|line| line.strip_prefix("answer\t"))
        .unwrap_or_else(|| panic!("the harness answered:\n{stdout}"));
    let (status, body) = line.split_once('\t').expect("status and body");
    (
        status.parse().expect("a status"),
        serde_json::from_str(body).expect("a JSON body"),
    )
}

/// Validates `body` against the schema the published document declares for `status`.
fn validate(ir: &EssIr, status: u16, body: &Value) -> Result<(), String> {
    let component = ir.components().values().next().expect("one component");
    let document: Value =
        serde_json::from_str(&ess_gen::openapi::json(ir, component)).expect("JSON");
    let at = &document["paths"][route(ir)]["post"]["responses"][status.to_string()]["content"]
        ["application/json"]["schema"];
    if !at.is_object() {
        return Err(format!("the document declares no {status}"));
    }
    let mut schema = document.clone();
    for key in ["$ref", "oneOf"] {
        if let Some(value) = at.get(key) {
            schema[key] = value.clone();
        }
    }
    let validator = jsonschema::draft202012::new(&schema).expect("the schema compiles");
    let errors: Vec<String> = validator
        .iter_errors(body)
        .map(|error| format!("{error} at {}", error.instance_path()))
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn expected() -> Value {
    json!({
        "outcome": "found",
        "published": [{"event": "catalogue.search.TitlesFound", "payload": {"count": 2}}],
        "response": {"names": ["Dune", "Dune Messiah"]},
    })
}

#[test]
fn the_rust_server_answers_a_returns_branch_200_with_its_response() {
    let ir = ir("ess/22");
    let synthesis =
        synthesize_laid_out(&ir, Target::Rust, OutputLayout::Workspace).expect("synthesizes");
    let root = scratch("rust");
    write(&synthesis, &root.join("catalogue"));
    let harness = root.join("harness");
    std::fs::create_dir_all(harness.join("src")).expect("mkdir");
    let dependencies = [
        "catalogue-server",
        "catalogue-system",
        "catalogue-reader",
        "catalogue-types",
    ]
    .iter()
    .fold(String::new(), |mut out, name| {
        let _ = writeln!(out, "{name} = {{ path = \"../catalogue/crates/{name}\" }}");
        out
    });
    std::fs::write(
        harness.join("Cargo.toml"),
        format!(
            "[package]\nname = \"served-returns-harness\"\nversion = \"0.0.0\"\nedition = \
             \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
        ),
    )
    .expect("write");
    std::fs::write(harness.join("src/main.rs"), RUST_HARNESS).expect("write");
    let target = root.join("target");
    let ran = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args([
            "run",
            "--offline",
            "--quiet",
            "--target-dir",
            target.to_str().expect("UTF-8"),
            "--",
            &route(&ir),
        ])
        .current_dir(&harness)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("CARGO_INCREMENTAL", "0")
        .env("RUSTFLAGS", "-D warnings")
        .output()
        .expect("cargo runs");
    let stdout = report("rust harness", &ran);
    assert!(ran.status.success(), "the Rust harness builds and runs");
    let _ = std::fs::remove_dir_all(&root);
    let (status, body) = answer(&stdout);
    assert_eq!(status, 200, "{body}");
    assert_eq!(body, expected());
    validate(&ir, status, &body).expect("the published document admits the served answer");
}

#[test]
fn the_go_server_answers_a_returns_branch_200_with_its_response() {
    let ir = ir("ess/22");
    let synthesis =
        synthesize_laid_out(&ir, Target::Go, OutputLayout::Workspace).expect("synthesizes");
    let directory = scratch("go");
    write(&synthesis, &directory);
    std::fs::write(directory.join("server/served_returns_test.go"), GO_HARNESS).expect("write");
    let ran = Command::new("go")
        .args([
            "test",
            "-count=1",
            "-timeout",
            "120s",
            "-v",
            "-run",
            "TestServedReturns",
            "./server/",
        ])
        .current_dir(&directory)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOTOOLCHAIN", "local")
        .output()
        .unwrap_or_else(|error| panic!("`go` runs: {error}"));
    let stdout = report("go harness", &ran);
    assert!(ran.status.success(), "the Go harness builds and runs");
    let _ = std::fs::remove_dir_all(&directory);
    let (status, body) = answer(&stdout);
    assert_eq!(status, 200, "{body}");
    assert_eq!(body, expected());
    validate(&ir, status, &body).expect("the published document admits the served answer");
}

#[test]
fn below_ess_22_a_returns_branch_keeps_its_variant_and_202() {
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_laid_out(&ir("ess/21"), target, OutputLayout::Workspace)
            .expect("synthesizes");
        let text: String = synthesis
            .artifacts
            .values()
            .map(|artifact| artifact.contents.as_str())
            .collect();
        for absent in [
            "encode_response_",
            "encodeResponse",
            "response: FindTitlesResponse",
            "Response FindTitlesResponse",
            "Response search.FindTitlesResponse",
            "\"response\"",
        ] {
            assert!(!text.contains(absent), "{target:?}: `{absent}`");
        }
        let answered = match target {
            Target::Rust => "            202\n        }",
            _ => "\t\treturn rendered(202, body)",
        };
        assert!(text.contains(answered), "{target:?}: the branch keeps 202");
        assert!(!text.contains("200, body)") && !text.contains("            200\n"));
    }
}
