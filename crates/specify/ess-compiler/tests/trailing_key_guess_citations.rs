//! Where a refusal is cited when the trailing-key guess is wrong and happens to be unique.
//!
//! `needles_from_tokens` in `crates/specify/ess-compiler/src/resolve.rs` offers `<last>:` as the
//! first needle for a document path, and `whole_name_matters` exempts it from the whole-name
//! filter. Two doc comments used to justify that exemption by claiming a wrong guess is harmless —
//! it "matches something else, and something else is not unique, so no line is reported".
//!
//! Uniqueness is a property of the documents being searched, not of the needle. Adversary pass 1's
//! fixture wrote a second field named `refiled`, so `filed:` occurred twice as a raw substring;
//! delete that field and the wrong guess is unique, and `Locator` reports the line it lands on —
//! which belongs to a command in another file that is not refused at all.
//!
//! `story:a-wrong-trailing-key-guess-is-reported-as-a-line` closes it in `Locator::span`: a guess
//! is reported only when its one occurrence lies inside the block of the declaration the path
//! names. The cases below are the adversary's own assertion, unchanged; the same wrong guess with
//! both commands in one file; and a right guess, which must still be cited at its key.

use ess_compiler::resolve::{diagnose_locating, Locator};
use ess_compiler::source::{Location, SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// A command refused for declaring one outcome name twice.
///
/// Adversary pass 1's `DOIT`, with the second event field `refiled` removed. That field is the
/// only reason the needle `filed:` was not unique there.
const DOIT: &str = "\
format: ess/1
system: shop
version: v1
domains: [shop.probe]
domain: shop.probe
events:
  - name: shop.probe.Filed
    fields:
      - name: filed
        type: String
commands:
  - name: shop.probe.Doit
    input:
      - name: note
        type: String
    outcomes:
      - name: filed
        emits:
          - shop.probe.Filed
      - name: filed
        emits:
          - shop.probe.Filed
";

/// A second, entirely valid command in a second file, whose payload block writes the field name.
///
/// Nothing here is refused. The single line `filed: input.note` is the whole of the collision.
const OTHER: &str = "\
domain: shop.probe
commands:
  - name: shop.probe.Other
    input:
      - name: note
        type: String
    outcomes:
      - name: other
        emits:
          - shop.probe.Filed
        payload:
          shop.probe.Filed:
            filed: input.note
";

/// Every bridged refusal, as `(document path, cited file, cited position)`.
fn spans(files: &[(&str, &str)]) -> Vec<(String, String, Option<Location>)> {
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
            let span = diagnostic
                .span
                .as_ref()
                .expect("every bridged refusal spans");
            (span.path.clone(), span.source.clone(), span.located)
        })
        .collect()
}

/// The one span cited for `path`.
fn span_for<'a>(
    spans: &'a [(String, String, Option<Location>)],
    path: &str,
) -> &'a (String, String, Option<Location>) {
    spans
        .iter()
        .find(|(cited, _, _)| cited == path)
        .unwrap_or_else(|| panic!("no refusal at `{path}`; cited: {spans:?}"))
}

/// The 1-based line and column `needle` starts at, which must occur exactly once.
fn position_of(text: &str, needle: &str) -> Location {
    assert_eq!(
        text.match_indices(needle).count(),
        1,
        "`{needle}` is written once in this fixture"
    );
    let index = text.find(needle).expect("just counted one");
    let before = &text[..index];
    let last_line = before.rsplit_once('\n').map_or(before, |(_, rest)| rest);
    Location {
        line: before.matches('\n').count() + 1,
        column: last_line.chars().count() + 1,
    }
}

/// The 1-based line whose trimmed text ends with `needle`.
fn line_ending_with(text: &str, needle: &str) -> usize {
    text.lines()
        .position(|line| line.trim_end().ends_with(needle))
        .unwrap_or_else(|| panic!("`{needle}` ends no line"))
        + 1
}

/// The premise both cases rest on, asserted once: the wrong guess is unique.
///
/// `filed:` occurs exactly **once** across the two documents, and that once is in `b.yaml`, on the
/// payload line of `shop.probe.Other` — a command that is not refused at all. An outcome is written
/// `- name: filed` and never `filed:`, so the needle cannot match its own target in `a.yaml`.
fn assert_the_guess_is_unique_and_cannot_match_its_target() {
    let raw: usize = [DOIT, OTHER]
        .iter()
        .map(|text| text.match_indices("filed:").count())
        .sum();
    assert_eq!(
        raw, 1,
        "the premise: the wrong guess `filed:` occurs exactly once, in b.yaml"
    );
    assert_eq!(
        DOIT.match_indices("filed:").count(),
        0,
        "the premise: the needle cannot match its own target, which is written `- name: filed`"
    );
}

/// What the story asks for: the refusal is cited where the refused command is declared.
///
/// The adversary's assertion, unchanged and not relaxed. An outcome is written `- name: <x>` and
/// never `<x>:`, so for every `command.*.outcomes.<name>` refusal the guess can never match its own
/// target; its one match here is in `b.yaml`, outside the refused declaration's block, and is not
/// reported.
#[test]
fn a_wrong_trailing_key_guess_is_not_cited_at_all() {
    assert_the_guess_is_unique_and_cannot_match_its_target();

    let spans = spans(&[("a.yaml", DOIT), ("b.yaml", OTHER)]);
    let (_, source, located) = span_for(&spans, "command.shop.probe.Doit.outcomes.filed");

    assert_eq!(
        source, "a.yaml",
        "`shop.probe.Doit` is declared and refused in a.yaml; b.yaml declares `shop.probe.Other`, \
         which is not refused at all — cited: {spans:?}"
    );
    assert_eq!(
        located.expect("a line").line,
        line_ending_with(DOIT, "name: shop.probe.Doit"),
        "the refused command's own declaration, which is where the sibling refusal at \
         `command.shop.probe.Doit.outcomes` is cited — cited: {spans:?}"
    );
}

/// Both commands in one file: `DOIT` with `OTHER`'s command appended to its `commands:` list.
const ONE_FILE: &str = "\
format: ess/1
system: shop
version: v1
domains: [shop.probe]
domain: shop.probe
events:
  - name: shop.probe.Filed
    fields:
      - name: filed
        type: String
commands:
  - name: shop.probe.Doit
    input:
      - name: note
        type: String
    outcomes:
      - name: filed
        emits:
          - shop.probe.Filed
      - name: filed
        emits:
          - shop.probe.Filed
  - name: shop.probe.Other
    input:
      - name: note
        type: String
    outcomes:
      - name: other
        emits:
          - shop.probe.Filed
        payload:
          shop.probe.Filed:
            filed: input.note
";

/// The same wrong guess, when the unrefused command sits in the refused command's own file.
///
/// Being in the right file is not enough: `filed: input.note` is under `shop.probe.Other`, and
/// `shop.probe.Other`'s `- name:` line closes `shop.probe.Doit`'s block before it.
#[test]
fn a_wrong_trailing_key_guess_in_the_same_file_is_not_cited() {
    assert_eq!(
        ONE_FILE.match_indices("filed:").count(),
        1,
        "the premise: the wrong guess is unique, and it is under `shop.probe.Other`"
    );

    let spans = spans(&[("a.yaml", ONE_FILE)]);
    let (_, source, located) = span_for(&spans, "command.shop.probe.Doit.outcomes.filed");
    assert_eq!(
        (source.as_str(), located.expect("a line").line),
        (
            "a.yaml",
            line_ending_with(ONE_FILE, "name: shop.probe.Doit")
        ),
        "cited at the refused command's declaration, not at `shop.probe.Other`'s payload key — \
         cited: {spans:?}"
    );
}

/// A guess that is right is still cited at the key, in whichever file the declaration is.
///
/// The other half of the contract: the guard must not cost the lines the guess exists for. The
/// same needle, `filed:`, against a path naming `shop.probe.Other` — whose block does hold it —
/// is cited at the key; against `shop.probe.Doit`, it falls through to Doit's declaration.
#[test]
fn a_right_trailing_key_guess_inside_the_declaration_is_still_cited() {
    let mut sources = SourceMap::new();
    sources.insert("a.yaml", DOIT);
    sources.insert("b.yaml", OTHER);
    let locator = Locator::new(&sources, &["a.yaml", "b.yaml"]);
    let needles = |declared: &str| {
        vec![
            "filed:".to_owned(),
            format!("name: {declared}"),
            format!("id: {declared}"),
            format!("component: {declared}"),
        ]
    };

    let right = locator.span("probe.right", &needles("shop.probe.Other"));
    assert_eq!(
        (right.source.as_str(), right.located),
        ("b.yaml", Some(position_of(OTHER, "filed: input.note"))),
        "`filed:` lies in `shop.probe.Other`'s block and is its key: {right}"
    );

    let wrong = locator.span("probe.wrong", &needles("shop.probe.Doit"));
    assert_eq!(
        (wrong.source.as_str(), wrong.located.map(|at| at.line)),
        (
            "a.yaml",
            Some(line_ending_with(DOIT, "name: shop.probe.Doit"))
        ),
        "`filed:` lies outside `shop.probe.Doit`'s block, in another file: {wrong}"
    );
}
