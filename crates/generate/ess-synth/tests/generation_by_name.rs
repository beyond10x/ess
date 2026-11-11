//! A specification the language accepts generates, or is refused by name; it does not panic, and
//! an authored name is not renamed because of one target's keyword list or because two bounded
//! contexts share a last segment (items 28, 29 and 32 of the 2026-11-09 workbench report).
//!
//! The models are the report's minimal examples, under `tests/fixtures/generation-by-name/`:
//!
//! * `28.yaml`: a command whose error outcomes are guarded by `when_related:` and whose payload
//!   reads the subject, beside an outcome that moves it. Both targets used to panic.
//! * `29.yaml`: an outcome named `self`. Rust allocated the invalid variant `Self`.
//! * `32.yaml`: two bounded contexts, each declaring a command `OpenThing` and a view `Things`,
//!   served by one component. Rust allocated each port method twice.

#![allow(dead_code, clippy::too_many_lines)]

mod related_guard_served;
use related_guard_served as served;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_conformance::report::Status;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Synthesis, Target};

const RELATED: &str = include_str!("fixtures/generation-by-name/28.yaml");
const KEYWORD: &str = include_str!("fixtures/generation-by-name/29.yaml");
const VIEWS: &str = include_str!("fixtures/generation-by-name/32.yaml");

/// What separates the documents of one fixture.
const DOCUMENT: &str = "---\n";

/// Every document of a fixture, assembled and resolved as one specification.
fn compile_documents(text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (position, document) in text.split(DOCUMENT).enumerate() {
        let label = format!("gap-{position}.yaml");
        let raw = RawSpecFile::parse(document)
            .unwrap_or_else(|error| panic!("`{label}` is well formed: {error}"));
        sources.insert(label.clone(), document.to_owned());
        labels.push(label.clone());
        parsed.push((Source::new(label), raw));
    }
    let spec =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile_locating(&spec, &sources, &labels).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn synthesis(ir: &EssIr, target: Target) -> Synthesis {
    synthesize_for(ir, target).unwrap_or_else(|error| panic!("{target:?} synthesizes: {error:?}"))
}

fn artifact<'a>(synthesis: &'a Synthesis, suffix: &str) -> &'a str {
    let found: Vec<_> = synthesis
        .artifacts
        .iter()
        .filter(|(path, _)| path.ends_with(suffix))
        .collect();
    assert_eq!(found.len(), 1, "exactly one `{suffix}` artifact");
    &found[0].1.contents
}

fn everything(synthesis: &Synthesis) -> String {
    synthesis
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect()
}

/// `text` as the socket adapter can drive it: it reads a view as no actor, and a view no grant
/// names is open to every caller, so no actor is granted one and the read-grant scenarios drop out
/// of the suite. Nothing the three items are about changes.
fn open_views(text: &str) -> String {
    let open = text
        .replace("  - gap.demo.Items\n", "")
        .replace(", gap.one.Things]", "]")
        .replace(", gap.two.Things]", "]");
    assert_ne!(open, text, "the model grants a view");
    open
}

/// The model served by one network component named `gap-service`, built and driven over a socket.
fn run_generated(text: &str, target: Target, case: &str, outcomes: &[&str]) {
    let ir = served::model(&served::served(&open_views(text), "gap-service"));
    drive(&ir, target, case, outcomes);
}

/// The scenarios of example 28 that await a step the system takes itself (`TakeDefault` is granted
/// to a system actor only), which the socket adapter cannot expose on an instance: it reports
/// `cannot expose awaiting ... a step its system takes itself`, whatever the server does.
const SYSTEM_STEPS: &[&str] = &[
    "gap.demo.Item/transition/finish/by/gap.demo.TakeDefault/defaulted",
    "gap.demo.TakeDefault/outcome/defaulted",
    "gap.demo.TakeDefault/outcome/no-default",
];

/// `ir` emitted for `target`, built as its entry point and driven by the suite it synthesizes:
/// every scenario passes but the ones no socket can drive, and each of `outcomes` is witnessed by
/// a passing scenario.
fn drive(ir: &EssIr, target: Target, case: &str, outcomes: &[&str]) {
    let (root, _) = served::emit(ir, target, case);
    let binary = served::build(&root, target, "gap", "gap-service", false);
    let statuses = served::run(ir, &binary);
    let failed: Vec<&String> = statuses
        .iter()
        .filter(|(id, status)| **status != Status::Passed && !SYSTEM_STEPS.contains(&id.as_str()))
        .map(|(id, _)| id)
        .collect();
    assert!(
        failed.is_empty(),
        "{target:?} {case}: {} of {} scenarios failed: {failed:#?}",
        failed.len(),
        statuses.len()
    );
    for outcome in outcomes {
        assert!(
            statuses
                .iter()
                .any(|(id, status)| id.contains(outcome) && *status == Status::Passed),
            "{target:?} {case}: a passing scenario witnesses `{outcome}`: {statuses:#?}"
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}

// ---- item 28 -------------------------------------------------------------------------------------

#[test]
fn a_subject_read_in_a_related_guarded_error_payload_generates() {
    let ir = served::model(&served::served(RELATED, "gap-service"));
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesis(&ir, target);
        let behaviour = artifact(
            &synthesis,
            match target {
                Target::Rust => "gap-types/src/behaviour.rs",
                _ => "types/behaviour/behaviour.go",
            },
        );
        // The refusal reads the parent reference from the row the guard was decided on.
        let read = match target {
            Target::Rust => "parent_id: held.data.parent_id.clone()",
            _ => "held.Data.ParentId",
        };
        assert!(
            behaviour.contains(read),
            "{target:?}: the refusal payload reads the held row (`{read}`):\n{behaviour}"
        );
    }
}

/// Example 28 with `TakeDefault` granted to an ordinary actor and its `no-default` branch guarding
/// a field a command sets. The suite then witnesses the `no-parent` and `reserved` refusals, whose
/// payloads read the subject's `parent_id`: with `not defined(dflt)`, the one row the suite can
/// arrange (no command sets `dflt`) answers `no-default` first, and with a system actor no such
/// scenario is synthesized at all.
fn witnessed() -> String {
    let text = RELATED
        .replace("predicate: not defined(dflt)", "predicate: n < 0")
        .replace("  system: true\n", "");
    assert!(text.contains("predicate: n < 0") && !text.contains("system: true"));
    text
}

#[test]
fn the_related_guarded_error_servers_build_and_the_reported_suite_passes() {
    assert!(served::go_available(), "Go is required");
    for (target, label) in [(Target::Rust, "rust"), (Target::Go, "go")] {
        // The example as reported: every scenario a socket can drive passes.
        run_generated(RELATED, target, &format!("related-{label}"), &["unknown"]);
        // The shape that reads the subject in both refusals builds as a server. Its suite is not
        // asserted: it expects the payload `parent_id` to be the input's literal where the server
        // answers the row's stored reference, which is the suite synthesis's own expectation.
        let ir = served::model(&served::served(&open_views(&witnessed()), "gap-service"));
        let (root, _) = served::emit(&ir, target, &format!("witnessed-{label}"));
        served::build(&root, target, "gap", "gap-service", false);
        let _ = std::fs::remove_dir_all(&root);
    }
}

// ---- item 29 -------------------------------------------------------------------------------------

#[test]
fn an_outcome_named_by_a_rust_keyword_generates() {
    let ir = served::model(&served::served(KEYWORD, "gap-service"));
    let rust = synthesis(&ir, Target::Rust);
    let items = everything(&rust);
    assert!(
        items.contains("Self_ {"),
        "the outcome `self` is the variant `Self_`:\n{items}"
    );
    assert_eq!(
        items.matches("Outcome::Self").count(),
        items.matches("Outcome::Self_").count(),
        "no variant is the invalid `Self`:\n{items}"
    );
    let wire = artifact(&rust, "wire.rs");
    assert!(
        wire.contains("\"self\""),
        "the served wire name of the outcome stays `self`:\n{wire}"
    );

    let typed = KEYWORD.replace("  - name: self\n", "  - name: type\n");
    assert_ne!(typed, KEYWORD);
    let ir = served::model(&served::served(&typed, "gap-service"));
    let items = everything(&synthesis(&ir, Target::Rust));
    assert!(
        items.contains("Type {"),
        "the outcome `type` is the variant `Type`:\n{items}"
    );
}

#[test]
fn the_keyword_outcome_is_served_under_its_authored_name() {
    run_generated(KEYWORD, Target::Rust, "keyword-rust", &["self"]);
}

// ---- item 32 -------------------------------------------------------------------------------------

/// `VIEWS` as the report wrote it: the component owns both domains and accepts nothing.
fn views_only() -> String {
    let only = VIEWS
        .replace(
            "    accepts:\n      commands: [gap.one.OpenThing, gap.two.OpenThing]\n    publishes:\n      \
             events: [gap.one.ThingOpened, gap.two.ThingOpened]\n",
            "",
        );
    assert_ne!(only, VIEWS);
    only
}

/// The method a port declares for `declared`, read from the doc line above it.
fn method_of(port: &str, doc: &str) -> String {
    let at = port
        .find(doc)
        .unwrap_or_else(|| panic!("the port documents `{doc}`:\n{port}"));
    let rest = &port[at..];
    let declared = rest
        .find("pub fn ")
        .unwrap_or_else(|| panic!("a method follows `{doc}`"));
    let tail = &rest[declared + "pub fn ".len()..];
    let end = tail.find(['(', '<']).expect("a parameter list");
    tail[..end].to_owned()
}

#[test]
fn two_views_of_one_last_name_in_two_domains_get_distinct_methods() {
    for (label, text) in [
        ("views only", views_only()),
        ("with commands", VIEWS.to_owned()),
    ] {
        let ir = compile_documents(&text);
        let rust = synthesis(&ir, Target::Rust);
        let port = artifact(&rust, "gap-service/src/lib.rs");
        assert_eq!(
            method_of(port, "/// Serves `gap.one.Things` at"),
            "things",
            "{label}:\n{port}"
        );
        assert_eq!(
            method_of(port, "/// Serves `gap.two.Things` at"),
            "things_2",
            "{label}:\n{port}"
        );
        if label == "with commands" {
            assert_eq!(
                method_of(port, "/// Accepts `gap.one.OpenThing`:"),
                "open_thing"
            );
            assert_eq!(
                method_of(port, "/// Accepts `gap.two.OpenThing`:"),
                "open_thing_2"
            );
        }
    }
}

#[test]
fn colliding_port_methods_are_served_each_by_its_own_declaration() {
    let ir = compile_documents(&open_views(VIEWS));
    drive(&ir, Target::Rust, "views-rust", &["OpenThing"]);
}
