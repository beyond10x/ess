//! Adversary pass 1 against `story:review-typed-diagnostics`.
//!
//! The unit's own contract document is `docs/design/review-typed-diagnostics.md`. These cases drive
//! the implementation from that document rather than from the suite the same unit wrote:
//!
//! 1. the page says `needles_of_site` "derives the same needles the heuristic derives, from typed
//!    data instead of tokens", and justifies keeping the ASCII-lowercase test by asserting that an
//!    event name "is a `Name` segment but is not a YAML key in the documents this repository has";
//! 2. the same paragraph says the `Key`/`Name` split is what "replaces the `STRUCTURAL` stop-list",
//!    which is only sound if no `Name` a producer writes is ever one of the fifty stop-list words;
//! 3. the page's closing section says `repeated_names.yaml` asserts "the honest `located: None` …
//!    rather than a confidently wrong line".
//!
//! Each case compares a *sited* refusal against the *same* `location` string bridged through the
//! string fallback, which is what the untyped producer emitted before this unit's diff.

use ess_compiler::resolve::diagnose_locating;
use ess_compiler::source::{Location, SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationError, ValidationErrors};

const REPEATED: &str = include_str!("fixtures/typed_diagnostics/repeated_names.yaml");

/// A command that names a payload block for an event it does not emit.
///
/// The rule is `CommandSpec::validate_payload_shape` (`ess-domain/src/command.rs:1233`), migrated by
/// this unit: its site's trailing member is `Segment::Name("shop.tail.Missing")`, a *qualified*
/// event name, whose first character is an ASCII lowercase letter.
const PAYLOAD: &str = "\
format: ess/1
system: shop
version: v1
domains: [shop.tail]
domain: shop.tail
events:
  - name: shop.tail.Noted
    fields:
      - name: note
        type: String
commands:
  - name: shop.tail.Note
    input:
      - name: note
        type: String
    outcomes:
      - name: noted
        emits:
          - shop.tail.Noted
        payload:
          shop.tail.Noted:
            note: input.note
          shop.tail.Missing:
            note: input.note
";

/// A command with an outcome whose author-chosen name is one of the `STRUCTURAL` stop-list words.
///
/// `error` is entry 11 of `STRUCTURAL` (`ess-compiler/src/resolve.rs:729`). The outcome named
/// `error` neither emits nor reports, so `CommandSpec::validate_outcome`
/// (`ess-domain/src/command.rs:1132`, migrated) refuses it at `…outcomes.error`; the *other*
/// outcome writes the only `error:` key in the document.
const STOP_WORD: &str = "\
format: ess/1
system: shop
version: v1
domains: [shop.stop]
domain: shop.stop
errors:
  - name: shop.stop.Refused
    summary: The request was refused.
commands:
  - name: shop.stop.Halt
    input:
      - name: note
        type: String
    outcomes:
      - name: error
      - name: refused
        error: shop.stop.Refused
";

/// Every refusal a document set produces, unbridged.
fn refusals(files: &[(&str, &str)]) -> ValidationErrors {
    Specification::assemble(files.iter().map(|(label, text)| {
        (
            Source::new(*label),
            RawSpecFile::parse(text).expect("the fixture is well formed YAML"),
        )
    }))
    .expect_err("the fixture is refused on purpose")
}

/// The sources, as the compiler reads them.
fn sources_of(files: &[(&str, &str)]) -> (SourceMap, Vec<String>) {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    for (label, text) in files {
        sources.insert(*label, *text);
        labels.push((*label).to_owned());
    }
    (sources, labels)
}

/// Where a refusal is cited, bridged with its typed site.
fn sited_line(errors: &ValidationErrors, files: &[(&str, &str)], suffix: &str) -> Option<Location> {
    let (sources, labels) = sources_of(files);
    let diagnostics = diagnose_locating(errors, &sources, &labels);
    let diagnostic = diagnostics
        .as_slice()
        .iter()
        .find(|diagnostic| {
            diagnostic
                .span
                .as_ref()
                .is_some_and(|span| span.path.ends_with(suffix))
        })
        .unwrap_or_else(|| {
            panic!(
                "no refusal cites a path ending {suffix:?}; the document produced {:?}",
                diagnostics
                    .as_slice()
                    .iter()
                    .map(|diagnostic| diagnostic
                        .span
                        .as_ref()
                        .map(|span| span.path.clone())
                        .unwrap_or_default())
                    .collect::<Vec<_>>()
            )
        });
    diagnostic.span.as_ref().expect("a span").located
}

/// Where the *same* refusal is cited when it carries only its string, which is what the untyped
/// producer emitted at the base commit.
fn unsited_line(
    errors: &ValidationErrors,
    files: &[(&str, &str)],
    suffix: &str,
) -> Option<Location> {
    let error = errors
        .as_slice()
        .iter()
        .find(|error| error.location.ends_with(suffix))
        .unwrap_or_else(|| panic!("no refusal is located at a path ending {suffix:?}"));
    assert!(
        error.site().is_some(),
        "{suffix} is supposed to be a migrated rule and carries no site"
    );
    let mut restated = ValidationErrors::new();
    restated.push(ValidationError::new(
        error.code,
        error.location.clone(),
        error.message.clone(),
    ));
    let (sources, labels) = sources_of(files);
    let diagnostics = diagnose_locating(&restated, &sources, &labels);
    diagnostics.as_slice()[0]
        .span
        .as_ref()
        .expect("a span")
        .located
}

/// The design page: `needles_of_site` "derives the same needles the heuristic derives".
///
/// A qualified event name starts with an ASCII lowercase letter, so the lowercase test the page
/// keeps does not exclude it, and `shop.tail.Missing:` *is* a YAML key in the documents this
/// repository has — every `payload:` block writes one.
#[test]
fn a_sited_payload_refusal_cites_the_same_line_as_its_own_location_string() {
    let files = [("payload.yaml", PAYLOAD)];
    let errors = refusals(&files);
    let suffix = ".payload.shop.tail.Missing";
    assert_eq!(
        sited_line(&errors, &files, suffix),
        unsited_line(&errors, &files, suffix),
        "the typed needles and the string needles cite different lines for one refusal"
    );
}

/// The design page: the `Key`/`Name` split "replaces the `STRUCTURAL` stop-list".
///
/// It replaces it only for the segments a producer marks `Key`. A `Name` the author chose may be
/// one of the fifty words, and the stop-list existed because searching for it lands on a structural
/// key belonging to something else — here, the sibling outcome's `error:` line.
#[test]
fn a_sited_refusal_whose_name_is_a_stop_list_word_cites_the_same_line_as_its_location_string() {
    let files = [("stop.yaml", STOP_WORD)];
    let errors = refusals(&files);
    let suffix = ".outcomes.error";
    assert_eq!(
        sited_line(&errors, &files, suffix),
        unsited_line(&errors, &files, suffix),
        "the typed needles walked into a structural key the stop-list exists to skip"
    );
}

/// The design page, closing section: `repeated_names.yaml` exercises "a needle that is *not*
/// unique" and asserts "the honest `located: None` … rather than a confidently wrong line".
///
/// No refusal from that fixture is unlocated: the shipped test pins all three to line 12.
#[test]
fn the_repeated_names_fixture_reports_an_unlocated_refusal_as_its_design_page_says() {
    let files = [("repeated_names.yaml", REPEATED)];
    let errors = refusals(&files);
    let (sources, labels) = sources_of(&files);
    let diagnostics = diagnose_locating(&errors, &sources, &labels);
    let unlocated: Vec<&str> = diagnostics
        .as_slice()
        .iter()
        .filter_map(|diagnostic| diagnostic.span.as_ref())
        .filter(|span| span.located.is_none())
        .map(|span| span.path.as_str())
        .collect();
    assert!(
        !unlocated.is_empty(),
        "the design page says this fixture asserts `located: None`; every refusal it produces is \
         located: {:?}",
        diagnostics
            .as_slice()
            .iter()
            .filter_map(|diagnostic| diagnostic.span.as_ref())
            .map(|span| (span.path.clone(), span.located))
            .collect::<Vec<_>>()
    );
}

/// The `clippy::result_large_err` bound the design page rests the `Box` on, pinned as a number.
///
/// Added after the three cases above and green from its first run: nothing in the tree records the
/// size, so un-boxing the site is caught only by a lint step a hurried run can skip. The default
/// `result_large_err` threshold is 128 bytes; the page reports 176 with the field inline.
#[test]
fn a_validation_error_stays_under_the_result_large_err_threshold() {
    let measured = std::mem::size_of::<ValidationError>();
    assert!(
        measured <= 128,
        "ValidationError is {measured} bytes, over the 128-byte `result_large_err` threshold"
    );
}
