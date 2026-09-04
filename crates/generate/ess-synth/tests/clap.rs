//! The command-line target, from a specification that declares one.
//!
//! What is checked here, and what is not: these tests read bytes. Whether the emitted crate
//! *compiles*, and whether the completion scripts a shell would source actually parse, are gate
//! steps rather than unit tests — the same division the other three targets use, because a test
//! suite that shelled out to a compiler would make `cargo test` depend on a toolchain it has no
//! other reason to need.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Synthesis, Target};

/// An inline fixture, assembled and compiled from YAML strings.
fn fixture(documents: &[(&str, &str)]) -> EssIr {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (label, text) in documents {
        let raw = RawSpecFile::parse(text)
            .unwrap_or_else(|error| panic!("the fixture `{label}` is well formed: {error}"));
        sources.insert((*label).to_owned(), (*text).to_owned());
        labels.push((*label).to_owned());
        parsed.push((Source::new((*label).to_owned()), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile_locating(&specification, &sources, &labels)
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

const SYSTEM: &str = "\
format: ess/1
system: desk
version: v1
domains:
  - desk.pass
";

const DOMAIN: &str = "\
domain: desk.pass

types:
  - name: desk.pass.Building
    kind: enum
    variants: [North, South, Annex]

entities:
  - name: desk.pass.Visit
    identity:
      name: visit_id
      type: Uuid
    fields:
      - name: building
        type: desk.pass.Building
    lifecycle:
      initial: Expected
      states: [Expected]
      terminal: [Expected]

commands:
  - name: desk.pass.RegisterVisit
    naming:
      wire: register
      summary: Record a visit somebody is expecting
    input:
      - name: building
        type: desk.pass.Building
      - name: note
        type: Optional<String>
    outcomes:
      - name: registered
        creates: desk.pass.Visit
        instance: visit_id
        emits:
          - desk.pass.VisitRegistered
        payload:
          desk.pass.VisitRegistered:
            building: input.building
        summary: The visit is recorded.

events:
  - name: desk.pass.VisitRegistered
    fields:
      - name: visit_id
        type: Uuid
      - name: building
        type: desk.pass.Building

views:
  - name: desk.pass.ExpectedVisits
    source: desk.pass.Visit
    consistency: eventual
    fields:
      - name: visit_id
        type: Uuid
    naming:
      wire: expected
      summary: Who is expected
";

const COMPONENTS: &str = "\
components:
  - component: desk-service
    summary: Visitor passes, from a terminal at the desk.
    owns:
      domains:
        - desk.pass
    accepts:
      commands:
        - desk.pass.RegisterVisit
    reached_by: command_line
    cli:
      binary: desk
      groups:
        - name: visits
          summary: Everything about one visit
          commands:
            - desk.pass.RegisterVisit
          views:
            - desk.pass.ExpectedVisits
";

fn emitted() -> Synthesis {
    let ir = fixture(&[
        ("system.yaml", SYSTEM),
        ("domains/pass.yaml", DOMAIN),
        ("components.yaml", COMPONENTS),
    ]);
    synthesize_for(&ir, Target::Clap)
}

fn source(synthesis: &Synthesis, path: &str) -> String {
    synthesis
        .artifacts
        .get(path)
        .unwrap_or_else(|| {
            panic!(
                "`{path}` is emitted; the tree holds {:?}",
                synthesis.artifacts.keys().collect::<Vec<_>>()
            )
        })
        .contents
        .clone()
}

#[test]
fn the_tree_carries_the_declared_binary_and_its_groups() {
    let synthesis = emitted();
    let tree = source(&synthesis, "crates/desk-cli/src/tree.rs");
    assert!(tree.contains("::clap::Command::new(\"desk\")"), "{tree}");
    assert!(tree.contains("::clap::Command::new(\"visits\")"), "{tree}");
    assert!(
        tree.contains("::clap::Command::new(\"register\")"),
        "{tree}"
    );
}

/// The binary is named by the declaration, not by the crate.
///
/// Without the `[[bin]]` entry Cargo names the executable after the package, and the word an
/// operator types would be `desk-cli` where the specification says `desk` — a surface that
/// disagrees with the document it came from, in the one place a person can see.
#[test]
fn the_manifest_names_the_binary_the_declaration_names() {
    let manifest = source(&emitted(), "crates/desk-cli/Cargo.toml");
    assert!(manifest.contains("[[bin]]\nname = \"desk\""), "{manifest}");
}

/// The reason the tree is in the model at all: a shell completes the *values*, not just the verb.
#[test]
fn an_enum_typed_field_carries_its_whole_closed_set() {
    let tree = source(&emitted(), "crates/desk-cli/src/tree.rs");
    assert!(
        tree.contains("PossibleValuesParser::new([\"North\", \"South\", \"Annex\"])"),
        "{tree}"
    );
}

/// A field the model cannot enumerate completes as free text rather than as a guess.
#[test]
fn a_string_field_offers_no_values() {
    let tree = source(&emitted(), "crates/desk-cli/src/tree.rs");
    let note = tree
        .split(".arg(")
        .find(|block| block.contains("\"note\""))
        .expect("the optional field is emitted");
    assert!(!note.contains("PossibleValuesParser"), "{note}");
    assert!(
        !note.contains(".required(true)"),
        "an optional field is not required: {note}"
    );
}

/// The defect `validate_command_line_views` refuses, checked from the other side.
///
/// A view served with no verb is one nobody can reach. Placing it is refused unless the tree does
/// it, and this is the half that proves the tree then actually carries it.
#[test]
fn a_placed_view_becomes_a_verb() {
    let synthesis = emitted();
    let tree = source(&synthesis, "crates/desk-cli/src/tree.rs");
    assert!(
        tree.contains("::clap::Command::new(\"expected\")"),
        "{tree}"
    );
    let handler = source(&synthesis, "crates/desk-cli/src/handler.rs");
    assert!(
        handler.contains("fn desk_pass_expected_visits"),
        "{handler}"
    );
}

/// Every word the tree places is a method somebody owes, and nothing decides one here.
#[test]
fn every_placed_word_is_an_obligation() {
    let handler = source(&emitted(), "crates/desk-cli/src/handler.rs");
    assert!(handler.contains("fn desk_pass_register_visit"), "{handler}");
    assert!(
        handler.contains("is an obligation nothing has implemented"),
        "{handler}"
    );
}

/// A shell script comes from the same tree, so it cannot describe a different grammar.
#[test]
fn the_binary_generates_its_own_completions() {
    let main = source(&emitted(), "crates/desk-cli/src/main.rs");
    assert!(main.contains("::clap_complete::generate("), "{main}");
    assert!(main.contains("self::tree::command()"), "{main}");
}

/// Same IR, same bytes. The property every target here holds.
#[test]
fn the_emission_is_deterministic() {
    let first = emitted();
    let second = emitted();
    assert_eq!(
        first.artifacts.keys().collect::<Vec<_>>(),
        second.artifacts.keys().collect::<Vec<_>>()
    );
    for (path, artifact) in &first.artifacts {
        assert_eq!(
            artifact.contents, second.artifacts[path].contents,
            "`{path}` differs between two runs of one IR"
        );
    }
}

/// A specification with no command-line component invents no surface.
#[test]
fn a_specification_declaring_no_command_line_emits_no_verbs() {
    let ir = fixture(&[
        ("system.yaml", SYSTEM),
        ("domains/pass.yaml", DOMAIN),
        (
            "components.yaml",
            "\
components:
  - component: desk-service
    owns:
      domains:
        - desk.pass
    accepts:
      commands:
        - desk.pass.RegisterVisit
",
        ),
    ]);
    let synthesis = synthesize_for(&ir, Target::Clap);
    let tree = source(&synthesis, "crates/desk-cli/src/tree.rs");
    assert!(
        !tree.contains("::clap::Command::new(\"register\")"),
        "{tree}"
    );
    let report = synthesis.target.expect("the target reports");
    assert!(
        report
            .refusals
            .iter()
            .any(|refusal| refusal.detail.contains("no component declaring")),
        "{:?}",
        report.refusals
    );
}
