//! Short names for the body roots `--all-commands` and `--all-views` select (beyond10x/ess#409):
//! a command's `<command>.input` and `<command>.response` and a view's `<view>.row` shorten to the
//! owner's last segment followed by the member, so two commands or two views no longer collide on
//! `Input`, `Response` or `Row`. Declared type and event short names stay the last segment, and a
//! collision that remains is still refused.

use std::collections::BTreeSet;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, spec::Specification, system::Source};
use ess_gen::schema::ModelTypes;
use schema_contract::realize::Plan;

/// Two commands (`Create` with a response, `Pin` without) and two views (`Notes` with its own
/// fields, `Summaries` answering rows of the declared type `Summary`).
const NOTES: &str = include_str!("../../ess-gen/tests/fixtures/command-view-roots.yaml");

fn ir(files: &[(&str, &str)]) -> ess_compiler::ir::EssIr {
    let mut sources = SourceMap::new();
    for (name, text) in files {
        sources.insert(*name, *text);
    }
    let specification = Specification::assemble(files.iter().map(|(name, text)| {
        (
            Source::new(*name),
            RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{name} parses: {error}")),
        )
    }))
    .unwrap_or_else(|errors| panic!("assembles: {errors}"));
    compile(&specification, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// What `ess generate types --all-commands --all-views` selects, plus `extra` roots.
fn every_body(ir: &ess_compiler::ir::EssIr, extra: &[&str]) -> ModelTypes {
    let mut roots: BTreeSet<String> = ModelTypes::command_roots(ir);
    roots.extend(ModelTypes::view_roots(ir));
    roots.extend(extra.iter().map(|root| (*root).to_owned()));
    ModelTypes::select(ir, &roots).unwrap_or_else(|errors| panic!("selects: {errors:?}"))
}

fn short_names(plan: &Plan) -> Vec<&str> {
    let mut names: Vec<&str> = plan.declarations().values().map(String::as_str).collect();
    names.sort_unstable();
    names
}

#[test]
fn all_commands_with_short_names_generates() {
    let ir = ir(&[("notes.yaml", NOTES)]);
    let short = Plan::from_model(&every_body(&ir, &[]))
        .expect("plan")
        .with_short_names()
        .unwrap_or_else(|refused| {
            panic!("--names short --all-commands --all-views is refused: {refused}")
        });
    assert_eq!(
        short_names(&short),
        [
            "CreateInput",
            "CreateResponse",
            "NoteId",
            "NotesRow",
            "PinInput",
            "SummariesRow",
            "Summary",
        ],
    );
    let rust = short.rust("notes").expect("rust").declarations;
    for name in ["CreateInput", "CreateResponse", "PinInput", "NotesRow"] {
        assert!(rust.contains(&format!("pub struct {name} {{")), "{rust}");
    }
    assert!(!rust.contains("DemoNotes"), "{rust}");
    let go = short
        .go("notes", "example.invalid/notes")
        .expect("go")
        .declarations;
    assert!(go.contains("type CreateInput struct"), "{go}");
    assert!(go.contains("type NotesRow struct"), "{go}");
    let typescript = short.typescript().declarations;
    assert!(typescript.contains("CreateResponse"), "{typescript}");
    assert!(typescript.contains("SummariesRow"), "{typescript}");
}

#[test]
fn a_declared_type_and_an_event_keep_their_last_segment_beside_body_roots() {
    let ir = ir(&[("notes.yaml", NOTES)]);
    let short = Plan::from_model(&every_body(
        &ir,
        &["demo.notes.Summary", "demo.notes.Created"],
    ))
    .expect("plan")
    .with_short_names()
    .expect("short names");
    let declarations = short.declarations();
    assert_eq!(declarations["demo.notes.Summary"], "Summary");
    assert_eq!(declarations["demo.notes.NoteId"], "NoteId");
    assert_eq!(declarations["demo.notes.Created"], "Created");
    assert_eq!(declarations["demo.notes.Create.input"], "CreateInput");
    assert_eq!(declarations["demo.notes.Summaries.row"], "SummariesRow");

    // The qualified default is untouched: the domain prefix stays on every name.
    let qualified = Plan::from_model(&every_body(&ir, &[])).expect("plan");
    assert_eq!(
        qualified.declarations()["demo.notes.Create.input"],
        "DemoNotesCreateInput"
    );
}

/// The same command name in two domains: `demo.a.Create.input` and `demo.b.Create.input` both
/// shorten to `CreateInput`, which is still refused and names both.
const DOMAIN_A: &str = r"format: ess/22
system: demo
version: v1
domains: [demo.a, demo.b]
domain: demo.a
events:
  - name: demo.a.Done
    fields:
      - {name: id, type: String}
commands:
  - name: demo.a.Create
    input:
      - {name: id, type: String}
    outcomes:
      - name: done
        emits: [demo.a.Done]
        payload:
          demo.a.Done: {id: input.id}
";

const DOMAIN_B: &str = r"domain: demo.b
events:
  - name: demo.b.Done
    fields:
      - {name: id, type: String}
commands:
  - name: demo.b.Create
    input:
      - {name: id, type: String}
    outcomes:
      - name: done
        emits: [demo.b.Done]
        payload:
          demo.b.Done: {id: input.id}
";

#[test]
fn the_same_command_name_in_two_domains_is_still_a_short_name_collision() {
    let ir = ir(&[("a.yaml", DOMAIN_A), ("b.yaml", DOMAIN_B)]);
    let refused = Plan::from_model(&every_body(&ir, &[]))
        .expect("plan")
        .with_short_names()
        .expect_err("two Create commands");
    let rendered = refused.to_string();
    let collision = refused
        .0
        .iter()
        .map(|finding| format!("{finding:?}"))
        .find(|finding| finding.contains("short_name_collision"))
        .unwrap_or_else(|| panic!("no short_name_collision in {rendered}"));
    assert!(
        collision.contains("demo.a.Create.input")
            && collision.contains("demo.b.Create.input")
            && collision.contains("CreateInput"),
        "{collision}"
    );
}
