//! Which file and line a refusal is cited against.
//!
//! `Locator` narrows a needle's matches with `whole_name` in
//! `crates/specify/ess-compiler/src/resolve.rs`: a match counts only when the characters beside it
//! are not ones a name is spelt with. Narrowing can only *remove* matches, so every behaviour it
//! can alter is a needle that used to match twice or more and now matches once — which is the one
//! transition `Locator`'s own documentation says must never happen quietly, because a needle that
//! matches once is reported as a line and a line is where the reader edits.
//!
//! The promise this file guards is `Locator`'s own header: "a confidently wrong line is worse than
//! none, because the reader edits there".
//!
//! It is *not* the one `needles_for` used to make — that guessing wrongly is safe because a wrong
//! guess will not be unique. Uniqueness belongs to the documents, not to the needle; adversary
//! pass 2, A3, built the one-match document and got a wrong citation. That claim is gone from
//! `resolve.rs`, `story:a-wrong-trailing-key-guess-is-reported-as-a-line` carries the behaviour,
//! and `tests/trailing_key_guess_citations.rs` measures it.
//!
//! One case here is `#[ignore]`d against a filed story and names it at the attribute. A second was,
//! until `story:a-masked-first-declaration-hides-a-duplicate-name` landed and it began to pass.

use ess_compiler::resolve::{diagnose_locating, Locator};
use ess_compiler::source::{Location, SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationErrors};

// ---- a bad key needle, made unique by the change -------------------------------------------

/// A command refused for declaring one outcome name twice, and an event with two fields.
///
/// `filed` and `refiled` are ordinary field names, and `refiled` is the whole point: the substring
/// `filed:` is spelt inside `refiled:`.
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
      - name: refiled
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

/// A second, entirely valid command in a second file, whose payload block writes both field names.
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
            refiled: input.note
";

/// Every bridged refusal, as `(document path, cited file, cited position)`.
fn spans(files: &[(&str, &str)]) -> Vec<(String, String, Option<Location>)> {
    let (_, spans) = refused(files);
    spans
}

/// The refusals and their spans, for cases that want to read the raw codes as well.
fn refused(files: &[(&str, &str)]) -> (ValidationErrors, Vec<(String, String, Option<Location>)>) {
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
    let spans = diagnose_locating(&errors, &sources, &labels)
        .as_slice()
        .iter()
        .map(|diagnostic| {
            let span = diagnostic
                .span
                .as_ref()
                .expect("every bridged refusal spans");
            (span.path.clone(), span.source.clone(), span.located)
        })
        .collect();
    (errors, spans)
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

/// The 1-based line whose trimmed text ends with `needle`.
fn line_ending_with(text: &str, needle: &str) -> usize {
    text.lines()
        .position(|line| line.trim_end().ends_with(needle))
        .unwrap_or_else(|| panic!("`{needle}` ends no line"))
        + 1
}

/// A refusal about a command in one file is cited against that file, not against another.
///
/// `needles_from_tokens` offers the trailing key first, so the refusal at
/// `command.shop.probe.Doit.outcomes.filed` is searched for as `filed:` before it is searched for
/// as `name: shop.probe.Doit`. An outcome is never *written* `filed:` — it is written
/// `- name: filed` — so that first needle is a guess that cannot match the thing it is guessing at.
///
/// Here the guess is not unique: `filed:` occurs twice as a raw substring, once as
/// `shop.probe.Other`'s payload target and once inside `refiled:` on the line below it. Two matches
/// is no line, so the search falls through to `name: shop.probe.Doit`, which is unique and correct.
/// That is the whole reason this case is green, and it is a property of *this document* — the
/// adversary's `tests/trailing_key_guess_citations.rs` deletes the `refiled` field and gets the
/// wrong file.
///
/// F1 was the same needle narrowed. `whole_name_matters` exempts it now, so `whole_name` is not
/// applied to it and the raw count is what decides; the two raw counts are asserted first so the
/// case says which count it is standing on rather than leaving a reader to assume the filter did
/// it.
#[test]
fn a_refusal_is_cited_against_the_file_the_refused_command_is_declared_in() {
    let files = [("a.yaml", DOIT), ("b.yaml", OTHER)];

    // What the base commit measured: `filed:` is not unique as a raw substring, so it was never a
    // candidate and the fallback needle answered.
    let raw: usize = files
        .iter()
        .map(|(_, text)| text.match_indices("filed:").count())
        .sum();
    assert_eq!(raw, 2, "the premise of this case: `filed:` occurs twice");
    let declarations: usize = files
        .iter()
        .map(|(_, text)| text.match_indices("name: shop.probe.Doit").count())
        .sum();
    assert_eq!(declarations, 1, "the fallback needle is unique");

    let spans = spans(&files);
    let (_, source, located) = span_for(&spans, "command.shop.probe.Doit.outcomes.filed");

    assert_eq!(
        source, "a.yaml",
        "`shop.probe.Doit` is declared in a.yaml and refused there; b.yaml declares \
         `shop.probe.Other`, which is not refused at all — cited: {spans:?}"
    );
    assert_eq!(
        located.expect("a line").line,
        line_ending_with(DOIT, "name: shop.probe.Doit"),
        "the refused command's own declaration, which is what the sibling refusal at \
         `command.shop.probe.Doit.outcomes` is cited at — cited: {spans:?}"
    );
}

// ---- the file the refusal was read from ----------------------------------------------------

/// One entity, declared in two files: the copy-paste the `DuplicateDeclaration` code exists for.
const DUP_A: &str = "\
format: ess/1
system: shop
version: v1
domains: [shop.probe]
domain: shop.probe
types:
  - name: shop.probe.OrderId
    kind: newtype
    of: Uuid
  - name: shop.probe.Channel
    kind: enum
    variants: [Email, Post]
entities:
  - name: shop.probe.Order
    identity:
      name: order_id
      type: shop.probe.OrderId
    fields:
      - name: channel
        type: shop.probe.Channel
    invariants:
      - channel == Fax
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
";

const DUP_B: &str = "\
domain: shop.probe
entities:
  - name: shop.probe.Order
    identity:
      name: order_id
      type: shop.probe.OrderId
    fields:
      - name: channel
        type: shop.probe.Channel
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
";

/// The story's acceptance clause 1, quoted whole: an entity invariant naming an undeclared enum
/// variant is refused "with a stable refusal code, **the file it was read from**, and the variants
/// that are declared".
///
/// The file is not read from the refusal. Nothing in `ValidationError` records which document a
/// refusal came from — it carries a code, a location string, a message and a hint — so the
/// compiler recovers the file by searching for the declaration's name, and reports `<document>`
/// whenever that search is not unique. `<document>` is not a file; it is the whole specification.
///
/// The ambiguity here is the plainest one there is: the same entity declared in two files, which
/// ESS itself refuses with `DuplicateDeclaration` and which is therefore a state the compiler is
/// built to be reached in. `whole_name` does not help — both occurrences are whole names — and
/// this is a case the whole-name filter does not cover, rather than one it broke.
///
/// Ignored, not deleted, and not relaxed: closing it means recording the source document on
/// `ValidationError` (`ess-primitives/src/error.rs`), which is a format-bearing change to a shared
/// envelope and larger than the story this file shipped with.
#[test]
#[ignore = "story:a-refusal-records-the-document-it-was-read-from — ValidationError records no \
            source document, so the file is recovered by a name search that degrades to \
            `<document>` when the name is not unique"]
fn an_undeclared_variant_refusal_names_the_file_it_was_read_from() {
    let files = [("dup_a.yaml", DUP_A), ("dup_b.yaml", DUP_B)];
    let (errors, spans) = refused(&files);

    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::DuplicateDeclaration),
        "the premise: the duplicate is itself refused, so this is a reachable state: {spans:?}"
    );

    let (_, source, _) = span_for(&spans, "entity shop.probe.Order.invariants[0]");
    assert_eq!(
        source, "dup_a.yaml",
        "acceptance clause 1 asks for the file the invariant was read from; the surviving \
         declaration is dup_a.yaml's — cited: {spans:?}"
    );
}

// ---- a name declared twice, and accepted ---------------------------------------------------

/// Two valid declarations of one command name. The control.
const VALID_TWICE: &str = "\
format: ess/1
system: shop
version: v1
domains: [shop.dup]
domain: shop.dup
events:
  - name: shop.dup.Filed
commands:
  - name: shop.dup.Both
    outcomes:
      - name: filed
        emits:
          - shop.dup.Filed
  - name: shop.dup.Both
    outcomes:
      - name: filed
        emits:
          - shop.dup.Filed
";

/// The same two declarations, with one error added to the first of them.
const MASKED: &str = "\
format: ess/1
system: shop
version: v1
domains: [shop.dup]
domain: shop.dup
events:
  - name: shop.dup.Filed
commands:
  - name: shop.dup.Both
    outcomes:
      - name: filed
        emits:
          - shop.dup.Filed
      - name: filed
        emits:
          - shop.dup.Filed
  - name: shop.dup.Both
    outcomes:
      - name: filed
        emits:
          - shop.dup.Filed
";

/// The fixture the unit edited, read back as the document it now is.
const REPEATED: &str = include_str!("fixtures/typed_diagnostics/repeated_names.yaml");

/// `true` when some refusal is the whole-declaration duplicate `Specification`'s `declare` reports.
///
/// Matched on `location`, not on the code alone: `DuplicateDeclaration` is also the code for a
/// repeated *outcome* name, which every fixture below carries on purpose, so a test that asked
/// only for the code would pass without the name-level duplicate ever being reported.
fn declared_twice(errors: &ValidationErrors, location: &str) -> bool {
    errors.as_slice().iter().any(|error| {
        error.code == ValidationCode::DuplicateDeclaration && error.location == location
    })
}

/// A command name declared twice is refused, including when the first declaration is refused for
/// something else as well.
///
/// This was `#[ignore]`d when it was written, and the story it named —
/// `story:a-masked-first-declaration-hides-a-duplicate-name` — has landed.
/// `Specification::absorb` used to convert each raw member and then insert it, and `insert` was
/// what reported the name-level `DuplicateDeclaration`, so a member whose own conversion failed
/// was never handed to it: the first declaration's outcome-name duplicate consumed the first
/// declaration and the second found the name free. `spec.rs` now asks `declare` — *has this name
/// been written* — before the conversion is attempted, and `record` keeps what the first
/// declaration meant, so all three assertions hold.
///
/// The third is the fixture one. `tests/fixtures/typed_diagnostics/repeated_names.yaml` declares
/// `shop.repeat.Solo` a second time and its comment calls it "the same command declared a second
/// time … two declarations answer to it"; the refusal that says so is now in the list
/// `typed_diagnostics.rs::a_name_used_more_than_once_falls_back_to_the_declaration_that_owns_it`
/// pins by equality.
///
/// The control — the first assertion — was always green: a duplicate of two *valid* declarations
/// has always been refused. It stays, because it is what separates "the refusal exists" from "the
/// refusal exists even when a copy is broken", and the second is the only one that moved.
#[test]
fn a_command_name_declared_twice_is_refused_even_when_the_first_declaration_is_also_refused() {
    let (control, _) = refused(&[("control.yaml", VALID_TWICE)]);
    assert!(
        declared_twice(&control, "command shop.dup.Both"),
        "the control: two valid declarations of one name are refused: {control}"
    );

    let (masked, _) = refused(&[("masked.yaml", MASKED)]);
    assert!(
        declared_twice(&masked, "command shop.dup.Both"),
        "`shop.dup.Both` is declared twice here too, and the only difference is that the first \
         declaration is refused for a second reason: {masked}"
    );

    let (fixture, _) = refused(&[("repeated_names.yaml", REPEATED)]);
    assert!(
        declared_twice(&fixture, "command shop.repeat.Solo"),
        "the unit's own fixture declares `shop.repeat.Solo` twice and is refused for five other \
         reasons, none of them that: {fixture}"
    );
}

// ---- what the change does get right --------------------------------------------------------

/// `spelt_with` names four classes of character, and the suite that shipped with it exercises one.
///
/// Every existing case turns on an ASCII letter or digit — `Invoice`/`InvoiceId`,
/// `File`/`FileTwo`, `File`/`Filed`, `Order`/`OrderId` — so dropping `'_' | '-' | '.'` from
/// `whole_name`'s `spelt_with` changes no shipped assertion. A name segment is
/// `[A-Za-z][A-Za-z0-9_-]*` and segments are joined with `.`, so all three are characters a name is
/// spelt with and all three belong there. This case is green and is written to stay that way: it
/// is the one that goes red if a later edit trims the set.
#[test]
fn a_name_extended_by_a_dot_a_hyphen_or_an_underscore_is_not_an_occurrence_of_it() {
    for extension in [".Extra", "-archive", "_v2"] {
        let text = format!(
            "entities:\n  - name: shop.Order\ntypes:\n  - name: shop.Order{extension}\n    of: \
             Uuid\n"
        );
        let mut sources = SourceMap::new();
        sources.insert("a.yaml", text.clone());
        let locator = Locator::new(&sources, &["a.yaml"]);

        let span = locator.span("entity shop.Order", &["name: shop.Order".to_owned()]);
        assert_eq!(span.source, "a.yaml", "extended by `{extension}`: {span}");
        assert_eq!(
            span.located
                .unwrap_or_else(|| panic!("extended by `{extension}`, the entity is declared once"))
                .line,
            2,
            "`shop.Order{extension}` is a different declaration, not a second occurrence of \
             `shop.Order`: {span}"
        );
    }
}
