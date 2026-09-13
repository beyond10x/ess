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
//! A quoted sentence is matched with whitespace collapsed, because the page is hard-wrapped and a
//! premise a reflow can silently switch off is not a premise. And each half's list of refusals is
//! matched against every refusal the fixture actually makes about that name, because the halves
//! state *counts*: the page went from two refusals to three and the unlocated half's list stayed
//! at two, so the file that exists solely to hold the page to the fixture was guarding two thirds
//! of it and said nothing. A hand-kept list is the defect there; the missing entry is the symptom.
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

/// The page, with every run of whitespace collapsed, so a sentence can be quoted across its wrap.
///
/// The page is hard-wrapped, so `contains` on a whole sentence is a check that a reflow silently
/// turns off. Every count this file reads off the page is in a sentence, and a count is the exact
/// thing that must not be quotable only in halves.
fn page_says(sentence: &str) -> bool {
    let flatten = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
    flatten(&design_page()).contains(&flatten(sentence))
}

/// Every cited refusal whose path names `name` — and not a longer name spelt around it.
///
/// `shop.repeat.FileTwo` contains `shop.repeat.File`, and a collision between exactly those two is
/// where this fixture's unlocated half wrongly came from once. A count taken by plain substring
/// would repeat that mistake inside the guard against it.
fn refusals_naming(spans: &[(String, String, Option<Location>)], name: &str) -> Vec<String> {
    spans
        .iter()
        .map(|(path, _, _)| path)
        .filter(|path| {
            path.match_indices(name).any(|(at, _)| {
                path[at + name.len()..]
                    .chars()
                    .next()
                    .is_none_or(|next| !next.is_alphanumeric())
            })
        })
        .cloned()
        .collect()
}

/// Every refusal the fixture makes about `name` is one of `checked`, and none is left over.
///
/// The halves below each assert a *list* of paths, and a list is the kind of guard that goes stale
/// without saying so: the page moved from two refusals to three and the unlocated half's loop kept
/// checking two, so a third of the claim it guards went unmeasured. Pairing the list against what
/// the fixture actually produces is what makes the page's count checkable rather than copied — a
/// refusal this file does not name fails here, whichever direction the drift came from.
fn every_refusal_is_checked(
    spans: &[(String, String, Option<Location>)],
    name: &str,
    checked: &[&str],
) {
    let mut produced = refusals_naming(spans, name);
    produced.sort();
    let mut named: Vec<String> = checked.iter().map(|path| (*path).to_owned()).collect();
    named.sort();
    assert_eq!(
        produced, named,
        "`{name}` is refused at {produced:?} and this case checks {named:?}. The page states a \
         count of these refusals and this file is what holds the page to it, so a refusal it does \
         not name is a part of the page nothing measures."
    );
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
    assert!(
        page_says("all three of `File`'s refusals are cited at `repeated_names.yaml:12:5`"),
        "the page's own sentence is the premise of this case; it no longer contains it"
    );

    let spans = cited();
    let expected = Some(Location {
        line: 12,
        column: 5,
    });
    let located_paths = [
        "command.shop.repeat.File.input[1]",
        "command.shop.repeat.File.outcomes.filed",
        "command.shop.repeat.File.outcomes",
    ];
    every_refusal_is_checked(&spans, "shop.repeat.File", &located_paths);
    for path in located_paths {
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
/// ambiguity that the whole-name filter cannot resolve, and the page says so: **all three** of its
/// refusals "report `located: None` and `source: <document>`". `<document>` is not a file; it is
/// the whole specification, which is what makes this the half the page must not get backwards.
///
/// Three, and the page said two until
/// `story:a-masked-first-declaration-hides-a-duplicate-name` added the third — the name-level
/// duplicate the second declaration earns, which nothing reported while `Solo`'s first declaration
/// failed its own conversion and never reached the command registry. The page moved and this loop
/// did not, so the guard covered two thirds of the claim it exists to guard. The count is read off
/// the page as a *sentence* now, whitespace collapsed so a rewrap cannot hide it: a page that says
/// `three` while this loop names two fails here instead of passing quietly.
#[test]
fn the_design_page_s_unlocated_half_reports_no_line_as_the_page_says() {
    assert!(
        page_says(
            "so all three of its refusals report `located: None` and `source: <document>` rather \
             than picking the first."
        ),
        "the page's own sentence is the premise of this case; it no longer contains it"
    );

    let spans = cited();
    let unlocated = [
        "command.shop.repeat.Solo.outcomes.noted",
        "command.shop.repeat.Solo.outcomes",
        "command shop.repeat.Solo",
    ];
    every_refusal_is_checked(&spans, "shop.repeat.Solo", &unlocated);
    for path in unlocated {
        let (source, located) = span_for(&spans, path);
        assert_eq!(
            (source.as_str(), located),
            ("<document>", None),
            "`docs/design/review-typed-diagnostics.md` says `shop.repeat.Solo` is declared twice \
             and that all three of its refusals report `located: None` and `source: <document>` \
             rather than picking the first — cited: {spans:?}"
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
    assert!(
        page_says("`shop.repeat.Solo`, declared twice, at lines 35 and 56"),
        "the page's own sentence is the premise of this case; it no longer contains it"
    );
    assert!(
        page_says("`shop.repeat.File`, declared once at line 12"),
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
