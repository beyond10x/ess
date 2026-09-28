//! Adversary pass 2 for `story:a-wrong-trailing-key-guess-is-reported-as-a-line`.
//!
//! The correction added `NAMED_LISTS` (`outcomes`, `input`, `fields`, `params`: no `<last>:` guess
//! after them), made a guess first-unique again when no declaration is located, and let
//! `Locator::encloses` reach upwards to the `- ` that opens the declaration's list item. These
//! cases probe each of the three:
//!
//! * `input` is not always a list of `- name:` elements: a system precondition's `input:` is a
//!   `BTreeMap<String, Node>` (`ess-domain` `system.rs` `Precondition`), written `<field>: <value>`.
//!   The base commit cited `system.preconditions[i].input.<field>` at that key; `NAMED_LISTS` now
//!   drops the only needle that could find it.
//! * `relations` is a list of `- name:` elements that `NAMED_LISTS` does not name and `STRUCTURAL`
//!   does not either, so `entity <E>.relations.<r>` has no declaration needle that can match and
//!   its guess `<r>:`, which can never match its own target, is first-unique anywhere.
//! * a list item opened by a bare `-` on its own line is not recognised as the item's first line,
//!   so a right key written above the declaration key in it is coarsened to the declaration line.

use std::path::{Path, PathBuf};

use ess_compiler::resolve::{compile_locating, diagnose_locating, Locator};
use ess_compiler::source::{Location, SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

type Cited = (String, String, Option<Location>);

/// Every `.yaml` file under `root`, as `(label, text)`, sorted by label.
fn read_tree(root: &Path) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                let label = path
                    .strip_prefix(root)
                    .expect("inside the root")
                    .display()
                    .to_string();
                let text = std::fs::read_to_string(&path).expect("readable");
                found.push((label, text));
            }
        }
    }
    found.sort();
    found
}

fn billing() -> Vec<(String, String)> {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
        .canonicalize()
        .expect("the billing example exists");
    read_tree(&root)
}

/// Every refusal of these files, wherever in the pipeline it is raised, as cited spans.
fn refusals(files: &[(String, String)]) -> Vec<Cited> {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    for (label, text) in files {
        sources.insert(label.clone(), text.clone());
        labels.push(label.clone());
    }
    let assembled = Specification::assemble(files.iter().map(|(label, text)| {
        (
            Source::new(label.clone()),
            RawSpecFile::parse(text).expect("well formed YAML"),
        )
    }));
    let diagnostics = match assembled {
        Err(errors) => diagnose_locating(&errors, &sources, &labels),
        Ok(specification) => compile_locating(&specification, &sources, &labels)
            .expect_err("the edited specification is refused on purpose"),
    };
    diagnostics
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

/// The one `(label, line)` where `needle` occurs across `files`, which it must do exactly once.
fn only_occurrence(files: &[(String, String)], needle: &str) -> (String, usize) {
    let hits: Vec<(String, usize)> = files
        .iter()
        .flat_map(|(label, text)| {
            text.match_indices(needle)
                .map(|(index, _)| (label.clone(), text[..index].matches('\n').count() + 1))
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(hits.len(), 1, "premise: `{needle}` occurs once: {hits:?}");
    hits.into_iter().next().expect("one")
}

fn edit(files: &mut [(String, String)], label: &str, from: &str, to: &str) {
    let (_, text) = files
        .iter_mut()
        .find(|(at, _)| at == label)
        .unwrap_or_else(|| panic!("{label} is in the specification"));
    assert!(text.contains(from), "`{from}` is in {label}");
    *text = text.replacen(from, to, 1);
}

const OUTCOME_SHAPES: &str = include_str!("fixtures/outcome-shapes.yaml");

/// `tests/fixtures/outcome-shapes.yaml` with one more key in its precondition's `input:` map:
/// `system.preconditions[0].input.colour` is refused (`UndeclaredReference`, `outcome_shapes.rs`
/// `precondition_input`: "supplies `colour`, which `OpenSession` does not take"). A precondition's
/// input is a map keyed by field name, so `colour:` is the refused key itself, the line the author
/// must edit, and it occurs once. The base commit cited it. `NAMED_LISTS` treats every `input` as
/// a list of `- name:` elements and builds no guess, the path has no declaration needle that can
/// match (`preconditions` is not structural), and the refusal falls to `<document>`.
#[test]
fn a_precondition_input_key_is_cited_where_it_is_written() {
    let mut files = vec![("outcome-shapes.yaml".to_owned(), OUTCOME_SHAPES.to_owned())];
    edit(
        &mut files,
        "outcome-shapes.yaml",
        "    input: {user_id: 00000000-0000-4000-8000-000000000152}",
        "    input: {user_id: 00000000-0000-4000-8000-000000000152, colour: red}",
    );
    let expected = only_occurrence(&files, "colour:");

    let spans = refusals(&files);
    let (_, source, located) = cited(&spans, "system.preconditions[0].input.colour");
    assert_eq!(
        (source.clone(), located.map(|at| at.line)),
        (expected.0, Some(expected.1)),
        "a precondition's `input:` is a map; its key is the refused line — cited: {spans:?}"
    );
}

/// The billing example with the account's relation renamed to `template` and its target made
/// undeclared (two edits in `domains/invoice.yaml`). `entity billing.invoice.Account.relations.template`
/// is refused there. A relation is written `- name: template`, never `template:`, so the guess
/// cannot match its own target; `relations` is in neither `NAMED_LISTS` nor `STRUCTURAL`, so no
/// declaration needle can match either, and the guess is first-unique across every file. It lands
/// on the binding's mapping key `template: invoice-created` in `components.yaml`, a file that holds
/// no refusal — the story's acceptance, first clause.
#[test]
fn a_relation_refusal_is_not_cited_in_a_file_that_holds_no_refusal() {
    let mut files = billing();
    edit(
        &mut files,
        "domains/invoice.yaml",
        "      - name: invoices\n        kind: owns\n        target: billing.invoice.Invoice",
        "      - name: template\n        kind: owns\n        target: billing.invoice.Nope",
    );
    let elsewhere = only_occurrence(&files, "template:");
    assert_eq!(elsewhere.0, "components.yaml", "premise");

    let spans = refusals(&files);
    let path = "entity billing.invoice.Account.relations.template";
    let (_, source, located) = cited(&spans, path);
    assert_ne!(
        source.as_str(),
        "components.yaml",
        "`{path}` is cited at {source}:{:?}, the binding's `template:` mapping key; \
         `components.yaml` holds no refusal — cited: {spans:?}",
        located.map(|at| at.line)
    );
}

fn needles(last: &str, declared: &str) -> Vec<String> {
    vec![
        format!("{last}:"),
        format!("name: {declared}"),
        format!("id: {declared}"),
        format!("component: {declared}"),
    ]
}

/// A list item opened by a bare `-` on its own line, its keys on the lines below. The key the
/// refusal names (`recipient:`) is written above the declaration key (`id:`) in the same item, and
/// is unique. The base commit cited it; `encloses` walks up from `id:` to the first less-indented
/// line, finds `-` without the trailing space its test requires, stops there, and takes the item
/// to start at `id:` — so the right line is coarsened to the declaration line.
#[test]
fn a_right_key_above_the_declaration_in_a_bare_dash_item_is_still_cited() {
    let text = "\
bindings:
  -
    mapping:
      recipient: customer_email
    id: notify-on-invoice-created
";
    let mut sources = SourceMap::new();
    sources.insert("c.yaml", text);
    let locator = Locator::new(&sources, &["c.yaml"]);
    let span = locator.span(
        "binding.notify-on-invoice-created.mapping.recipient",
        &needles("recipient", "notify-on-invoice-created"),
    );
    assert_eq!(
        span.located.map(|at| at.line),
        Some(4),
        "cited at the mapping key in the same item, not coarsened to the id line: {span}"
    );
}
