//! Adversary pass 2 on story:rust-generation-refuses-by-name-instead-of-panicking-or-misnaming.
//!
//! `pascal` still escapes the whole word `self` to `Self_`, and it is not only the outcome-variant
//! path: actor and event variants that fall back to their pascal-joined full names map every
//! segment through it. A domain segment `self` is legal (`QualifiedName::PATTERN`).

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Synthesis, Target};

const VIEWS: &str = include_str!("fixtures/generation-by-name/32.yaml");

fn compile_documents(text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    let mut parsed = Vec::new();
    for (position, document) in text.split("---\n").enumerate() {
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

fn rust(ir: &EssIr) -> Synthesis {
    synthesize_for(ir, Target::Rust).unwrap_or_else(|error| panic!("Rust synthesizes: {error:?}"))
}

fn everything(synthesis: &Synthesis) -> String {
    synthesis
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect()
}

/// Every `Self_` immediately followed by an upper-case letter: an identifier rustc's
/// `non_camel_case_types` lint rejects, and one no base spelling produced.
fn non_camel(items: &str) -> Vec<String> {
    items
        .lines()
        .filter(|line| {
            line.match_indices("Self_").any(|(at, _)| {
                line[at + "Self_".len()..]
                    .chars()
                    .next()
                    .is_some_and(char::is_uppercase)
            })
        })
        .map(str::to_owned)
        .collect()
}

/// Example 32 with gap.one renamed `gap.self` and gap.two's command and view renamed, so no port
/// method collides: a model the base generated. Its two `Anyone` actors and two `ThingOpened`
/// events collide by last name, so both variant families fall back to the full name
/// pascal-joined, which was `SelfAnyone` / `SelfThingOpened` before the change.
#[test]
fn a_domain_named_self_keeps_its_actor_and_event_variants() {
    let text = VIEWS
        .replace("gap.two.OpenThing", "gap.two.StartThing")
        .replace("gap.two.Things", "gap.two.Stuff")
        .replace("gap.one", "gap.self");
    let ir = compile_documents(&text);
    let items = everything(&rust(&ir));
    let bad = non_camel(&items);
    assert!(
        bad.is_empty(),
        "a model with no keyword outcome changed its variants (was `SelfAnyone`, \
         `SelfThingOpened`):\n{}",
        bad.join("\n")
    );
    assert!(
        items.contains("SelfAnyone"),
        "the actor variant stays `SelfAnyone`"
    );
    assert!(
        items.contains("SelfThingOpened"),
        "the event variant stays `SelfThingOpened`"
    );
}
