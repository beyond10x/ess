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
            // OrderId and OrderPlaced are distinct names, so the fallback can locate
            // Order's declaration without pretending to locate its transition leaf.
            Cited {
                code: "ESS-ENTITY-005".to_owned(),
                source: "nested.yaml".to_owned(),
                path: "entity shop.orders.Order.transitions[0]".to_owned(),
                located: Some(Location { line: 11, column: 5 }),
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

    // The outcome is repeated, but its owning command has one exact declaration.
    // FileTwo and Filed must not make the declaration of File appear ambiguous.
    assert_eq!(
        cited,
        vec![
            Cited {
                code: "ESS-COMMAND-006".to_owned(),
                source: "repeated_names.yaml".to_owned(),
                path: "command.shop.repeat.File.input[1]".to_owned(),
                located: Some(Location {
                    line: 12,
                    column: 5
                }),
            },
            Cited {
                code: "ESS-COMMAND-006".to_owned(),
                source: "repeated_names.yaml".to_owned(),
                path: "command.shop.repeat.File.outcomes.filed".to_owned(),
                located: Some(Location {
                    line: 12,
                    column: 5
                }),
            },
            Cited {
                code: "ESS-COMMAND-004".to_owned(),
                source: "repeated_names.yaml".to_owned(),
                path: "command.shop.repeat.File.outcomes".to_owned(),
                located: Some(Location {
                    line: 12,
                    column: 5
                }),
            },
            // …and the same hazard where the search *can* answer. `shop.repeat.Solo` repeats an
            // outcome name too, and no other declaration's name contains its own, so the fallback
            // needle is unique and both refusals are cited at the line the command is declared on.
            // Both commands remain pinned to their own declaration, not to an outcome
            // with the same name elsewhere in the fixture.
            Cited {
                code: "ESS-COMMAND-006".to_owned(),
                source: "repeated_names.yaml".to_owned(),
                path: "command.shop.repeat.Solo.outcomes.noted".to_owned(),
                located: Some(Location {
                    line: 35,
                    column: 5,
                }),
            },
            Cited {
                code: "ESS-COMMAND-004".to_owned(),
                source: "repeated_names.yaml".to_owned(),
                path: "command.shop.repeat.Solo.outcomes".to_owned(),
                located: Some(Location {
                    line: 35,
                    column: 5,
                }),
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
        .find(|error| error.location == "command.shop.repeat.File.outcomes")
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
/// The repository root, from this crate's manifest directory.
fn repository_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn the_inventory_on_the_design_page_is_the_count_in_the_tree() {
    let root = repository_root();
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

/// Production `ess-domain` source, with `#[cfg(test)]` modules removed.
///
/// A `#[cfg(test)]` module in this repository is written at the top level, so it ends at the first
/// `}` in column zero. An assertion inside one pins a location; it does not produce one, and
/// counting it as a producer is how a census stops meaning anything.
fn production(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut skipping = false;
    for line in text.lines() {
        if !skipping && line.trim() == "#[cfg(test)]" {
            skipping = true;
            continue;
        }
        if skipping {
            if line == "}" {
                skipping = false;
            }
            continue;
        }
        kept.push_str(line);
        kept.push('\n');
    }
    kept
}

/// Every `ess-domain` source file, as `(file name, production text)`, in file order.
fn domain_sources() -> Vec<(String, String)> {
    let sources = repository_root().join("crates/specify/ess-domain/src");
    let mut found = Vec::new();
    for entry in std::fs::read_dir(&sources).expect("ess-domain sources are readable") {
        let path = entry.expect("a readable entry").path();
        if path.extension().is_none_or(|kind| kind != "rs") {
            continue;
        }
        found.push((
            path.file_name()
                .expect("a file name")
                .to_string_lossy()
                .into_owned(),
            std::fs::read_to_string(&path).expect("readable"),
        ));
    }
    found.sort();
    found
}

/// The block between two markers in the design page, as whitespace-split rows.
fn page_block(page: &str, marker: &str) -> Vec<Vec<String>> {
    let block = page
        .split_once(&format!("<!-- {marker}:begin -->"))
        .unwrap_or_else(|| panic!("the page carries a `{marker}` block"))
        .1
        .split_once(&format!("<!-- {marker}:end -->"))
        .unwrap_or_else(|| panic!("the `{marker}` block is closed"))
        .0;
    block
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("```"))
        .map(|line| line.split_whitespace().map(str::to_owned).collect())
        .collect()
}

/// A family is the location head a refusal writes, not the file the rule lives in.
///
/// The class adversary pass 2 found (F1, F2): the page's per-file inventory booked four `command.…`
/// refusals under `entity.rs`, and the page then said the command family was migrated. Rewording one
/// of those paths moved `ESS-COMMAND-012` to `ESS-SPEC-012`, because the refusal carried no site.
/// Counting by head instead of by file is what makes that unbookable: a `command.…` literal in any
/// file lands in the `command` rows, and the page has to say why each one is still there.
#[test]
fn the_head_census_on_the_design_page_is_the_count_in_the_tree() {
    // Every head `ess-compiler`'s `family_of` matches, plus the plural spellings the producers
    // write. A location literal opens with one of these followed by the separator its family uses.
    const HEADS: &[&str] = &[
        "actor",
        "actors",
        "binding",
        "bindings",
        "command",
        "commands",
        "component",
        "components",
        "conversion",
        "conversions",
        "domain",
        "domains",
        "entity",
        "entities",
        "error",
        "errors",
        "event",
        "events",
        "spec",
        "topology",
        "type",
        "types",
        "view",
        "views",
    ];

    let mut measured: Vec<(String, String, usize)> = Vec::new();
    for (file, text) in domain_sources() {
        let text = production(&text);
        let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
        let bytes = text.as_bytes();
        for (index, _) in text.match_indices('"') {
            let rest = &text[index + 1..];
            let Some(head) = HEADS
                .iter()
                .filter(|head| rest.starts_with(**head))
                .filter(|head| matches!(rest.as_bytes().get(head.len()), Some(b'.' | b' ')))
                // The longest match, so `types.` is not counted as `type`.
                .max_by_key(|head| head.len())
            else {
                continue;
            };
            // Not an escaped quote inside another literal.
            if index > 0 && bytes[index - 1] == b'\\' {
                continue;
            }
            *counts.entry(head).or_default() += 1;
        }
        for (head, count) in counts {
            measured.push((head.to_owned(), file.clone(), count));
        }
    }
    measured.sort();

    let page =
        std::fs::read_to_string(repository_root().join("docs/design/review-typed-diagnostics.md"))
            .expect("the design page exists");
    let claimed: Vec<(String, String, usize)> = page_block(&page, "heads")
        .into_iter()
        .map(|row| {
            assert_eq!(
                row.len(),
                3,
                "a head row is `<head> <file> <count>`: {row:?}"
            );
            (
                row[0].clone(),
                row[1].clone(),
                row[2].parse().expect("a count"),
            )
        })
        .collect();

    assert_eq!(
        claimed, measured,
        "the head census in `docs/design/review-typed-diagnostics.md` and the tree disagree"
    );

    // The migrated family, stated as the rule rather than left to the reader of a table.
    let command_writers: Vec<&(String, String, usize)> = measured
        .iter()
        .filter(|(head, ..)| head == "command")
        .collect();
    assert_eq!(
        command_writers
            .iter()
            .map(|(_, file, count)| (file.as_str(), *count))
            .collect::<Vec<_>>(),
        vec![("primitive_admission.rs", 1), ("wire.rs", 1)],
        "a `command.…` location is written somewhere the design page does not account for; the \
         command family is the one this wave migrated"
    );
}

/// The design page's type table describes the types it names, attribute for attribute.
///
/// The class adversary pass 2 found (F4): the summary row said `ConstructKind` is
/// `#[non_exhaustive]` while the prose two sections down, the type's doc comment and the declaration
/// all said the opposite — and the row is what a reader adding a kind reads first. The round-1 check
/// compared *identifiers* the page names against the tree; this compares a *claim* about each of
/// them.
#[test]
fn the_type_table_agrees_with_the_declarations_it_describes() {
    let root = repository_root();
    let page = std::fs::read_to_string(root.join("docs/design/review-typed-diagnostics.md"))
        .expect("the design page exists");
    let source = std::fs::read_to_string(root.join("crates/specify/ess-primitives/src/error.rs"))
        .expect("the error module exists");

    let mut checked = 0;
    for name in [
        "ConstructKind",
        "Segment",
        "ConstructRef",
        "SyntaxSpan",
        "Site",
    ] {
        let row = page
            .lines()
            .find(|line| line.starts_with(&format!("| `{name}` |")))
            .unwrap_or_else(|| panic!("the page's type table has a `{name}` row"));
        // The same rule the adversary's own check uses: the attribute is named in the row, or it
        // is not. A row that mentions the attribute in order to deny it is a row a reader skims
        // wrongly, so the token itself is the claim.
        let claimed = row.contains("#[non_exhaustive]");

        let declaration = source
            .split_once(&format!("pub enum {name} {{"))
            .or_else(|| source.split_once(&format!("pub struct {name} {{")))
            .unwrap_or_else(|| panic!("`{name}` is declared in `error.rs`"))
            .0;
        let attributes = declaration
            .rsplit_once("#[derive(")
            .expect("the declaration carries a derive")
            .1;
        let declared = attributes.contains("#[non_exhaustive]");

        assert_eq!(
            claimed, declared,
            "the page's `{name}` row says `#[non_exhaustive]` is {claimed} and `error.rs` \
             declares it {declared}"
        );
        checked += 1;
    }
    assert_eq!(checked, 5, "a type in the table was skipped");
}

/// No production code assigns to a refusal's `location`.
///
/// The class adversary pass 2 found (F6): `location` and the private site are two representations of
/// one fact, and `location` is `pub`. It stays `pub` because the suite's own rewording
/// transformation assigns to it to prove the machine facts do not follow — so the guard cannot be
/// the type system, and is this instead. `ValidationError::rebase` is the one supported re-rooting
/// and it refuses a sited refusal; a `#[cfg(test)]` assignment is the deliberate case and is
/// excluded, as is the accessor's own module.
#[test]
fn no_production_code_assigns_a_refusals_location() {
    let root = repository_root();
    let mut offenders: Vec<String> = Vec::new();
    for directory in [
        "crates/specify/ess-domain/src",
        "crates/specify/ess-compiler/src",
        "crates/specify/ess-primitives/src",
    ] {
        let mut pending = vec![root.join(directory)];
        while let Some(path) = pending.pop() {
            for entry in std::fs::read_dir(&path).expect("the source tree is readable") {
                let path = entry.expect("a readable entry").path();
                if path.is_dir() {
                    pending.push(path);
                    continue;
                }
                if path.extension().is_none_or(|kind| kind != "rs") {
                    continue;
                }
                // `error.rs` is where `location` is defined and where `rebase` writes it.
                if path.ends_with("error.rs") {
                    continue;
                }
                let text = production(&std::fs::read_to_string(&path).expect("readable"));
                for (number, line) in text.lines().enumerate() {
                    let line = line.trim();
                    if line.contains(".location =") || line.contains(".location=") {
                        offenders.push(format!("{}:{}: {line}", path.display(), number + 1));
                    }
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "production code assigns to a refusal's `location`; use `ValidationError::rebase`, which \
         refuses a sited refusal: {offenders:?}"
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
