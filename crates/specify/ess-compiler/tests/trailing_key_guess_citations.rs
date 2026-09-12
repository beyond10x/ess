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
//! Both doc comments now say that instead. The behaviour itself is carried by
//! `story:a-wrong-trailing-key-guess-is-reported-as-a-line` and is **pre-existing**:
//! `whole_name_matters("filed:")` is `false`, so this commit and base `bd722fa9` take the identical
//! path for this needle.
//!
//! The two cases below are one measurement read twice: what the compiler does today, asserted so a
//! change to it is seen; and what the story asks for, `#[ignore]`d until it lands. The second is
//! the adversary's own assertion, unchanged.

use ess_compiler::resolve::diagnose_locating;
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

/// What the compiler does today: it cites a line in a file that declares nothing it refused.
///
/// Green, and pinning a defect rather than a guarantee. `whole_name_matters` deliberately does not
/// narrow this needle, so the filter is not what decides it: `Locator::scan` counts one raw match
/// and reports it. Base `bd722fa9` does the same, so this records a hazard the whole-name change
/// left standing rather than one it created.
///
/// The sibling refusal one path-segment up has no trailing key to guess at, falls through to
/// `name: shop.probe.Doit`, and is cited correctly. Asserting both in one case is what shows the
/// wrong citation is the guess and not the bridge.
///
/// This case goes red when `story:a-wrong-trailing-key-guess-is-reported-as-a-line` lands, at which
/// point the `#[ignore]` below comes off and this one is deleted. That is the intended sequence,
/// not a regression.
#[test]
fn a_wrong_trailing_key_guess_that_is_unique_is_cited_in_the_file_it_lands_in() {
    assert_the_guess_is_unique_and_cannot_match_its_target();

    let spans = spans(&[("a.yaml", DOIT), ("b.yaml", OTHER)]);
    let (_, source, located) = span_for(&spans, "command.shop.probe.Doit.outcomes.filed");
    assert_eq!(
        (source.as_str(), *located),
        ("b.yaml", Some(position_of(OTHER, "filed: input.note"))),
        "today the guess `filed:` is unique in b.yaml and is reported, although b.yaml declares \
         only `shop.probe.Other`, which is not refused — cited: {spans:?}"
    );

    let (_, sibling_source, sibling_located) = span_for(&spans, "command.shop.probe.Doit.outcomes");
    assert_eq!(
        (
            sibling_source.as_str(),
            sibling_located.expect("a line").line
        ),
        ("a.yaml", line_ending_with(DOIT, "name: shop.probe.Doit")),
        "the sibling one path-segment up has no key to guess at and falls through to the \
         declaration needle, which is correct — cited: {spans:?}"
    );
}

/// What the story asks for: the refusal is cited where the refused command is declared.
///
/// The adversary's assertion, unchanged and not relaxed. It is `#[ignore]`d rather than deleted
/// because the fix is a change to what `needles_for` builds — an outcome is written `- name: <x>`
/// and never `<x>:`, so for every `command.*.outcomes.<name>` refusal the first needle tried can
/// never match its own target — and that is larger than the doc correction this file shipped with.
#[test]
#[ignore = "story:a-wrong-trailing-key-guess-is-reported-as-a-line — `needles_for` offers `<x>:` \
            first for `command.*.outcomes.<x>`, which is never how an outcome is written, so any \
            match it finds is wrong by construction"]
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
