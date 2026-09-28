//! `story:a-wrong-trailing-key-guess-is-reported-as-a-line`, for the `- name:` lists that were
//! neither `NAMED_LISTS` nor `STRUCTURAL` in `crates/specify/ess-compiler/src/resolve.rs`.
//!
//! Adversary pass 2 found `relations` (its case is in `trailing_key_guess_adversary_pass2.rs`) and
//! named `attributes` (actor, `Vec<Field>`) and `response` (command, `Vec<Field>`) as the same
//! shape. For all three, a path `<kind> <declared>.<list>…` built a declaration needle
//! `name: <declared>.<list>…` that can never match, so the refused construct was never located
//! and the refusal was either cited at a guess that is first-unique anywhere or not at all.

use ess_compiler::resolve::diagnose_locating;
use ess_compiler::source::{Location, SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

type Cited = (String, String, Option<Location>);

fn refusals(files: &[(&str, &str)]) -> Vec<Cited> {
    let errors = Specification::assemble(files.iter().map(|(label, text)| {
        (
            Source::new(*label),
            RawSpecFile::parse(text).expect("the fixture is well formed YAML"),
        )
    }))
    .expect_err("the fixture is refused on purpose");
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    for (label, text) in files {
        sources.insert(*label, *text);
        labels.push((*label).to_owned());
    }
    diagnose_locating(&errors, &sources, &labels)
        .as_slice()
        .iter()
        .map(|diagnostic| {
            let span = diagnostic.span.as_ref().expect("every refusal spans");
            (span.path.clone(), span.source.clone(), span.located)
        })
        .collect()
}

fn cited<'a>(spans: &'a [Cited], path: &str) -> &'a Cited {
    spans
        .iter()
        .find(|(at, _, _)| at == path)
        .unwrap_or_else(|| panic!("no refusal at `{path}`; cited: {spans:?}"))
}

fn line_ending_with(text: &str, needle: &str) -> usize {
    text.lines()
        .position(|line| line.trim_end().ends_with(needle))
        .unwrap_or_else(|| panic!("`{needle}` ends no line"))
        + 1
}

/// A command whose one response field has an undeclared type.
const RESPONDS: &str = "\
format: ess/4
system: shop
version: v1
domains: [shop.probe]
domain: shop.probe
events:
  - name: shop.probe.Filed
    fields:
      - name: note
        type: String
commands:
  - name: shop.probe.Doit
    input:
      - name: note
        type: String
    response:
      - name: receipt
        type: shop.probe.Nope
    outcomes:
      - name: done
        emits:
          - shop.probe.Filed
";

/// A second, valid file whose payload block writes a key spelt like the response field.
const ELSEWHERE: &str = "\
domain: shop.probe
events:
  - name: shop.probe.Receipted
    fields:
      - name: receipt
        type: String
commands:
  - name: shop.probe.Other
    input:
      - name: note
        type: String
    outcomes:
      - name: other
        emits:
          - shop.probe.Receipted
        payload:
          shop.probe.Receipted:
            receipt: input.note
";

/// `command.shop.probe.Doit.response.receipt` is refused in `a.yaml`. The response field is
/// written `- name: receipt`, never `receipt:`, so the guess can only land on something else —
/// here `shop.probe.Other`'s payload key, unique, in `b.yaml`, which holds no refusal.
#[test]
fn a_response_field_refusal_is_not_cited_in_a_file_that_holds_no_refusal() {
    let raw: usize = [RESPONDS, ELSEWHERE]
        .iter()
        .map(|text| text.match_indices("receipt:").count())
        .sum();
    assert_eq!(
        raw, 1,
        "the premise: the guess `receipt:` is unique, in b.yaml"
    );

    let spans = refusals(&[("a.yaml", RESPONDS), ("b.yaml", ELSEWHERE)]);
    let (_, source, located) = cited(&spans, "command.shop.probe.Doit.response.receipt");
    assert_eq!(
        (source.as_str(), located.map(|at| at.line)),
        (
            "a.yaml",
            Some(line_ending_with(RESPONDS, "name: shop.probe.Doit"))
        ),
        "cited at the refused command's declaration — cited: {spans:?}"
    );
}

/// An actor that declares one attribute twice.
const ACTS: &str = "\
format: ess/16
system: demo
version: v1
domains: [demo.notes]
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.notes.Alpha
    attributes:
      - {name: tenant, type: String}
      - {name: tenant, type: String}
    may: [demo.notes.CreateNote]
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
        emits: [demo.notes.NoteCreated]
events:
  - name: demo.notes.NoteCreated
    fields:
      - {name: text, type: String}
views: []
";

/// `actor.demo.notes.Alpha.attributes[1]` is refused. Its declaration needle used to be
/// `name: demo.notes.Alpha.attributes.1`, which no document writes, so the refused actor was never
/// located and the refusal had no line at all. `attributes` is a structural key: the actor is
/// `demo.notes.Alpha`.
#[test]
fn an_attribute_refusal_is_cited_at_the_refused_actor() {
    let spans = refusals(&[("a.yaml", ACTS)]);
    let (_, source, located) = cited(&spans, "actor.demo.notes.Alpha.attributes[1]");
    assert_eq!(
        (source.as_str(), located.map(|at| at.line)),
        (
            "a.yaml",
            Some(line_ending_with(ACTS, "name: demo.notes.Alpha"))
        ),
        "cited at the refused actor's declaration — cited: {spans:?}"
    );
}
