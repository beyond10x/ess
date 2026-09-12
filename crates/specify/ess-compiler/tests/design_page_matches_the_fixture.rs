//! `docs/design/review-typed-diagnostics.md` read as the specification it claims to be.
//!
//! The page's **What the fixtures assert** section describes `repeated_names.yaml` in two halves, a
//! **located** one and an **unlocated** one, and names a file, a line and a column for the first.
//! Nothing in the build compares any of that to the fixture: `grep -rn review-typed-diagnostics
//! crates/edge/ess-xtask/src Taskfile.yml` returns nothing. The page has been wrong about this
//! fixture twice, in opposite directions, and both corrections came from an adversary rather than
//! from a gate — the second time by naming each half's exact opposite after wave 24 added the
//! whole-name filter and a second `shop.repeat.Solo` declaration.
//!
//! This file is the comparison. Each case asserts the page's own sentence is present *before*
//! asserting what it says, so a reader can tell a stale test from a stale document: a failure on
//! the first assertion means the page moved, and a failure on the second means the code did.
//!
//! `story:the-design-page-is-held-to-the-fixture-it-describes` carries making this a gate step
//! rather than a test in one package's suite.

use std::path::{Path, PathBuf};

use ess_compiler::resolve::diagnose_locating;
use ess_compiler::source::{Location, SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const REPEATED: &str = include_str!("fixtures/typed_diagnostics/repeated_names.yaml");

/// The design page this unit's fixture edit is governed by.
fn design_page() -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../docs/design/review-typed-diagnostics.md")
        .canonicalize()
        .expect("the design page exists");
    std::fs::read_to_string(&path).expect("the design page is readable")
}

/// Every bridged refusal from the fixture, as `(document path, cited file, cited position)`.
fn cited() -> Vec<(String, String, Option<Location>)> {
    let files = [("repeated_names.yaml", REPEATED)];
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
        sources.insert(label, text);
        labels.push(label.to_owned());
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

fn span_for(
    spans: &[(String, String, Option<Location>)],
    path: &str,
) -> (String, Option<Location>) {
    let (_, source, located) = spans
        .iter()
        .find(|(cited, _, _)| cited == path)
        .unwrap_or_else(|| panic!("no refusal at `{path}`; cited: {spans:?}"));
    (source.clone(), *located)
}

/// The **located** half. The page names a file, a line and a column; the fixture must carry them.
///
/// The page's sentence is the premise, quoted rather than paraphrased, so that rewriting the page
/// without rerunning the fixture fails here instead of passing vacuously. It reads: `shop.repeat.
/// File` is declared once at line 12 and "all three of `File`'s refusals are cited at
/// `repeated_names.yaml:12:5`".
#[test]
fn the_design_page_s_located_half_is_cited_where_the_page_says_it_is() {
    let page = design_page();
    assert!(
        page.contains("cited at `repeated_names.yaml:12:5`"),
        "the page's own sentence is the premise of this case; it no longer contains it"
    );

    let spans = cited();
    let expected = Some(Location {
        line: 12,
        column: 5,
    });
    for path in [
        "command.shop.repeat.File.input[1]",
        "command.shop.repeat.File.outcomes.filed",
        "command.shop.repeat.File.outcomes",
    ] {
        let (source, located) = span_for(&spans, path);
        assert_eq!(
            (source.as_str(), located),
            ("repeated_names.yaml", expected),
            "`docs/design/review-typed-diagnostics.md` says all three of `shop.repeat.File`'s \
             refusals are cited at `repeated_names.yaml:12:5` — cited: {spans:?}"
        );
    }
}

/// The **unlocated** half. The page says `shop.repeat.Solo` gets no line and no file.
///
/// `Solo` is declared twice, at lines 35 and 56. Two occurrences of a whole name is a real
/// ambiguity that the whole-name filter cannot resolve, and the page says so: both refusals
/// "report `located: None` and `source: <document>`". `<document>` is not a file; it is the whole
/// specification, which is what makes this the half the page must not get backwards.
#[test]
fn the_design_page_s_unlocated_half_reports_no_line_as_the_page_says() {
    let page = design_page();
    assert!(
        page.contains("report `located: None` and `source: <document>`"),
        "the page's own sentence is the premise of this case; it no longer contains it"
    );

    let spans = cited();
    for path in [
        "command.shop.repeat.Solo.outcomes.noted",
        "command.shop.repeat.Solo.outcomes",
    ] {
        let (source, located) = span_for(&spans, path);
        assert_eq!(
            (source.as_str(), located),
            ("<document>", None),
            "`docs/design/review-typed-diagnostics.md` says `shop.repeat.Solo` is declared twice \
             and that both of its refusals report `located: None` and `source: <document>` rather \
             than picking the first — cited: {spans:?}"
        );
    }
}

/// The page's count of `shop.repeat.Solo`'s declarations, read against the fixture.
///
/// The inversion this file exists for was not a wrong line number; it was a wrong claim about how
/// many times a name is written. Both halves above turn on that count, and neither would notice if
/// the fixture lost the second `Solo` declaration while the page kept saying there were two — the
/// refusals would simply become located and the page's *other* half would be the wrong one.
#[test]
fn the_design_page_s_declaration_counts_are_the_fixture_s() {
    let page = design_page();
    assert!(
        page.contains("`shop.repeat.Solo`, declared twice, at lines 35 and 56"),
        "the page's own sentence is the premise of this case; it no longer contains it"
    );
    assert!(
        page.contains("`shop.repeat.File`, declared once at line 12"),
        "the page's own sentence is the premise of this case; it no longer contains it"
    );

    let declarations = |name: &str| {
        REPEATED
            .lines()
            .enumerate()
            .filter(|(_, line)| line.trim() == format!("- name: {name}"))
            .map(|(index, _)| index + 1)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        declarations("shop.repeat.Solo"),
        vec![35, 56],
        "the page names lines 35 and 56"
    );
    assert_eq!(
        declarations("shop.repeat.File"),
        vec![12],
        "the page names line 12"
    );
}
