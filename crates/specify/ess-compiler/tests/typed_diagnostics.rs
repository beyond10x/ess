//! Diagnostic identity, held apart from diagnostic wording.
//!
//! `story:review-typed-diagnostics`, from finding F14. The design is
//! `docs/design/review-typed-diagnostics.md`; this is the fixture set that page's closing section
//! describes. Three documents, one per hazard the story names — repeated names, nesting, a
//! cross-file reference — and for each of them the same three questions:
//!
//! 1. what code and what source location does the refusal cite (the pin);
//! 2. does either move when every message and hint is rewritten (the acceptance statement);
//! 3. does the refusal carry a typed site, and does that site render the string it replaced.

use ess_compiler::resolve::diagnose_locating;
use ess_compiler::source::{Location, SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ConstructKind, Segment, ValidationErrors};

const NESTED: &str = include_str!("fixtures/typed_diagnostics/nested.yaml");
const REPEATED: &str = include_str!("fixtures/typed_diagnostics/repeated_names.yaml");
const CROSS_A: &str = include_str!("fixtures/typed_diagnostics/cross_file_a.yaml");
const CROSS_B: &str = include_str!("fixtures/typed_diagnostics/cross_file_b.yaml");

/// What a refusal is expected to cite: the code, the file, the document path, the position.
#[derive(Debug, PartialEq, Eq)]
struct Cited {
    code: String,
    source: String,
    path: String,
    located: Option<Location>,
}

/// Every refusal a fixture produces, as the machine sees it.
fn cited(files: &[(&str, &str)]) -> (ValidationErrors, Vec<Cited>) {
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
    let diagnostics = diagnose_locating(&errors, &sources, &labels);
    let cited = diagnostics
        .as_slice()
        .iter()
        .map(|diagnostic| {
            let span = diagnostic
                .span
                .as_ref()
                .expect("every bridged refusal spans");
            Cited {
                code: diagnostic.code.to_string(),
                source: span.source.clone(),
                path: span.path.clone(),
                located: span.located,
            }
        })
        .collect();
    (errors, cited)
}

/// The same refusals, with every human word in them replaced.
///
/// This is the acceptance statement made mechanical: a wording-only change is exactly this, and
/// nothing a machine reads may move under it.
fn reworded(errors: &ValidationErrors) -> Vec<Cited> {
    let mut rewritten = ValidationErrors::new();
    for (index, error) in errors.as_slice().iter().enumerate() {
        let mut copy = error.clone();
        copy.message = format!("rewritten prose number {index}");
        copy.hint = Some(format!("rewritten remedy number {index}"));
        rewritten.push(copy);
    }
    let mut sources = SourceMap::new();
    for (label, text) in [
        ("nested.yaml", NESTED),
        ("repeated_names.yaml", REPEATED),
        ("cross_file_a.yaml", CROSS_A),
        ("cross_file_b.yaml", CROSS_B),
    ] {
        sources.insert(label, text);
    }
    let labels = [
        "nested.yaml",
        "repeated_names.yaml",
        "cross_file_a.yaml",
        "cross_file_b.yaml",
    ];
    diagnose_locating(&rewritten, &sources, &labels)
        .as_slice()
        .iter()
        .map(|diagnostic| {
            let span = diagnostic
                .span
                .as_ref()
                .expect("every bridged refusal spans");
            Cited {
                code: diagnostic.code.to_string(),
                source: span.source.clone(),
                path: span.path.clone(),
                located: span.located,
            }
        })
        .collect()
}

/// Every error's rendered `location` is its site's render, for every error that has a site.
///
/// The invariant the whole migration rests on: the adopter-facing string is output, never input.
fn every_site_renders_its_own_location(errors: &ValidationErrors) {
    for error in errors.as_slice() {
        if let Some(site) = error.site() {
            assert_eq!(
                site.construct.render(),
                error.location,
                "a site and its rendered location disagree: {error}"
            );
        }
    }
}

#[test]
fn a_nested_member_path_is_cited_by_construct_and_line() {
    let (errors, cited) = cited(&[("nested.yaml", NESTED)]);

    assert_eq!(
        cited,
        vec![
            // The entity family is still on the string heuristic — `docs/design/
            // review-typed-diagnostics.md` inventories it — and the honest answer to a path it
            // cannot find is no line at all.
            Cited {
                code: "ESS-ENTITY-005".to_owned(),
                source: "<document>".to_owned(),
                path: "entity shop.orders.Order.transitions[0]".to_owned(),
                located: None,
            },
            Cited {
                code: "ESS-COMMAND-001".to_owned(),
                source: "nested.yaml".to_owned(),
                path: "command.shop.orders.PlaceOrder.outcomes.placed.payload.shop.orders.OrderPlaced.reference".to_owned(),
                located: Some(Location { line: 44, column: 13 }),
            },
        ]
    );
    assert_eq!(reworded(&errors), cited, "wording moved the machine facts");
    every_site_renders_its_own_location(&errors);

    let payload = errors
        .as_slice()
        .iter()
        .find(|error| error.location.ends_with(".reference"))
        .expect("the payload refusal");
    let site = payload.site().expect("a migrated rule carries its site");
    assert_eq!(site.construct.kind(), ConstructKind::Command);
    assert_eq!(site.construct.name(), "shop.orders.PlaceOrder");
    assert_eq!(
        site.construct.members(),
        &[
            Segment::Key("outcomes".to_owned()),
            Segment::Name("placed".to_owned()),
            Segment::Key("payload".to_owned()),
            Segment::Name("shop.orders.OrderPlaced".to_owned()),
            Segment::Name("reference".to_owned()),
        ]
    );
}

#[test]
fn a_name_used_more_than_once_falls_back_to_the_declaration_that_owns_it() {
    let (errors, cited) = cited(&[("repeated_names.yaml", REPEATED)]);

    // Nothing here is located, and that is the point of the fixture. The outcome is written
    // `- name: filed`, so the needle `filed:` occurs zero times in the document; the fallback
    // needle `name: shop.repeat.File` occurs three times as a substring — the command itself, the
    // sibling `shop.repeat.FileTwo`, and the event `shop.repeat.Filed`. A substring search that
    // matches three lines knows nothing, and `Locator` says so instead of picking the first.
    assert_eq!(
        cited,
        vec![
            Cited {
                code: "ESS-COMMAND-006".to_owned(),
                source: "<document>".to_owned(),
                path: "command.shop.repeat.File.input[1]".to_owned(),
                located: None,
            },
            Cited {
                code: "ESS-COMMAND-006".to_owned(),
                source: "<document>".to_owned(),
                path: "command.shop.repeat.File.outcomes.filed".to_owned(),
                located: None,
            },
            Cited {
                code: "ESS-COMMAND-004".to_owned(),
                source: "<document>".to_owned(),
                path: "command.shop.repeat.File.outcomes".to_owned(),
                located: None,
            },
        ]
    );
    assert_eq!(reworded(&errors), cited, "wording moved the machine facts");
    every_site_renders_its_own_location(&errors);

    let duplicate_input = errors
        .as_slice()
        .iter()
        .find(|error| error.location.ends_with("input[1]"))
        .expect("the duplicate input refusal");
    assert_eq!(
        duplicate_input
            .site()
            .expect("a migrated rule carries its site")
            .construct
            .members(),
        &[Segment::Key("input".to_owned()), Segment::Index(1)]
    );

    let all_unconditional = errors
        .as_slice()
        .iter()
        .find(|error| error.location.ends_with(".outcomes"))
        .expect("the unconditional-outcomes refusal");
    assert_eq!(
        all_unconditional
            .site()
            .expect("a migrated rule carries its site")
            .construct
            .members(),
        &[Segment::Key("outcomes".to_owned())]
    );
}

#[test]
fn a_reference_across_files_is_cited_in_the_file_that_wrote_it() {
    let (errors, cited) = cited(&[
        ("cross_file_a.yaml", CROSS_A),
        ("cross_file_b.yaml", CROSS_B),
    ]);

    // The event is declared in `b`; the mistake is written in `a`, and `a` is where the reader has
    // to go.
    assert_eq!(
        cited,
        vec![Cited {
            code: "ESS-COMMAND-001".to_owned(),
            source: "cross_file_a.yaml".to_owned(),
            path: "command.shop.cross.Announce.outcomes.announced.payload.shop.cross.Announced.headline".to_owned(),
            located: Some(Location { line: 17, column: 13 }),
        }]
    );
    assert_eq!(reworded(&errors), cited, "wording moved the machine facts");
    every_site_renders_its_own_location(&errors);

    let site = errors.as_slice()[0]
        .site()
        .expect("a migrated rule carries its site");
    assert_eq!(site.construct.kind(), ConstructKind::Command);
    assert_eq!(site.construct.name(), "shop.cross.Announce");
    assert!(
        site.span.is_none(),
        "no producer supplies a parser position yet; the design page says so"
    );
}

/// The design page's inventory is the count in the tree, in both directions.
///
/// The class adversary pass 1 found (F5): the page's inventory was written by hand and was wrong
/// twice in one table — a row said seven sites where the tree had eight, and the per-function
/// totals did not add up to the grep. A hand-maintained census of a migration in progress is a
/// defect, not a document, because the only thing that keeps it true is somebody re-counting. So
/// the page carries a machine-readable block and this test is the check: a file with sites that the
/// block omits, a count that has drifted, and a file the block lists that has no sites, are each a
/// failure. Migrating another family cannot go green without moving the page.
#[test]
fn the_inventory_on_the_design_page_is_the_count_in_the_tree() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let page = std::fs::read_to_string(root.join("docs/design/review-typed-diagnostics.md"))
        .expect("the design page exists");
    let block = page
        .split_once("<!-- inventory:begin -->")
        .expect("the page carries a machine-readable inventory")
        .1
        .split_once("<!-- inventory:end -->")
        .expect("the inventory block is closed")
        .0;

    let mut claimed: Vec<(String, usize, usize)> = Vec::new();
    for line in block.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("```") {
            continue;
        }
        let mut parts = line.split_whitespace();
        let file = parts.next().expect("a file name").to_owned();
        let untyped = parts
            .next()
            .and_then(|count| count.parse().ok())
            .unwrap_or_else(|| panic!("`{line}` does not carry a `ValidationError::new` count"));
        let typed = parts
            .next()
            .and_then(|count| count.parse().ok())
            .unwrap_or_else(|| panic!("`{line}` does not carry a `ValidationError::at` count"));
        assert!(
            parts.next().is_none(),
            "`{line}` has more than three fields"
        );
        claimed.push((file, untyped, typed));
    }
    assert!(
        claimed.windows(2).all(|pair| pair[0].0 < pair[1].0),
        "the inventory block is not in file order, so a reader cannot find a row"
    );

    let sources = root.join("crates/specify/ess-domain/src");
    let mut measured: Vec<(String, usize, usize)> = Vec::new();
    for entry in std::fs::read_dir(&sources).expect("ess-domain sources are readable") {
        let path = entry.expect("a readable entry").path();
        if path.extension().is_none_or(|kind| kind != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("readable");
        let untyped = text.matches("ValidationError::new").count();
        let typed = text.matches("ValidationError::at").count();
        if untyped == 0 && typed == 0 {
            continue;
        }
        measured.push((
            path.file_name()
                .expect("a file name")
                .to_string_lossy()
                .into_owned(),
            untyped,
            typed,
        ));
    }
    measured.sort();

    assert_eq!(
        claimed, measured,
        "the inventory in `docs/design/review-typed-diagnostics.md` and the tree disagree"
    );
}

/// The adopter-facing sample in `website/docs/guides/write-a-specification.md:134-137`.
///
/// It prints two `location` strings and one code. The guide is a published page; if the migration
/// moved either, the page would be wrong and nothing else in the gate would notice.
#[test]
fn the_guide_sample_still_prints_the_locations_it_documents() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/billing");
    let mut files = Vec::new();
    let mut pending = vec![root.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the billing example is readable") {
            let path = entry.expect("a readable entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|kind| kind == "yaml") {
                let label = path
                    .strip_prefix(&root)
                    .expect("under the example")
                    .to_string_lossy()
                    .into_owned();
                let mut text = std::fs::read_to_string(&path).expect("readable");
                // Exactly the edit the guide tells the reader to make: rename the event in the
                // `accepted` outcome's `emits:` list and its `payload:` key, and nowhere else.
                if label.ends_with("invoice.yaml") {
                    text = text
                        .replace(
                            "          - billing.invoice.InvoiceCreated\n",
                            "          - billing.invoice.InvoiceRaised\n",
                        )
                        .replace(
                            "          billing.invoice.InvoiceCreated:\n",
                            "          billing.invoice.InvoiceRaised:\n",
                        );
                }
                files.push((label, text));
            }
        }
    }
    files.sort();

    let errors = Specification::assemble(
        files
            .iter()
            .map(|(label, text)| {
                (
                    Source::new(label.clone()),
                    RawSpecFile::parse(text).expect("the example is well formed"),
                )
            })
            .collect::<Vec<_>>(),
    )
    .expect_err("the renamed event is undeclared");

    let rendered = errors.to_string();
    for printed in [
        "[undeclared_reference] command.billing.invoice.CreateInvoice.outcomes.accepted.emits: \
         `billing.invoice.InvoiceRaised` is not a declared event",
        "[undeclared_reference] command.billing.invoice.CreateInvoice.outcomes.accepted.instance:",
    ] {
        assert!(
            rendered.contains(printed),
            "the guide prints {printed:?} and the run does not: {rendered}"
        );
    }
    every_site_renders_its_own_location(&errors);
}
