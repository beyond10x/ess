//! Adversary cases for `story:a-wrong-trailing-key-guess-is-reported-as-a-line`.
//!
//! The fix in `Locator::span` reports a trailing `<last>:` guess only when its one match lies
//! inside the block of the first unique declaration needle. These cases probe what that costs and
//! what it leaves standing:
//!
//! * a refusal whose path has **no declaration needle at all** (`topology.workloads.<component>`)
//!   used to be cited at the workload key, which is the right line, and now falls to `<document>`;
//! * a wrong guess whose one match lies inside the refused command's block, on a sibling outcome's
//!   payload key, is still reported as the refusal's line;
//! * a right guess written above the declaration key within its own list item is no longer cited;
//! * CRLF, comments at column 0 and blank lines inside the block, which must not close it.

use std::path::{Path, PathBuf};

use ess_compiler::resolve::{compile_locating, diagnose_locating, Locator};
use ess_compiler::source::{Location, SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

type Cited = (String, String, Option<Location>);

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
        .canonicalize()
        .expect("the billing example exists")
}

/// Every `.yaml` file of `examples/billing`, as `(label, text)`, sorted by label.
fn billing() -> Vec<(String, String)> {
    let base = example();
    let mut found = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                let label = path
                    .strip_prefix(&base)
                    .expect("inside the example")
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
            .expect_err("the edited example is refused on purpose"),
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
    assert_eq!(
        hits.len(),
        1,
        "the premise: `{needle}` occurs exactly once, so the base commit's `Locator::span` (first \
         unique needle wins) cited it: {hits:?}"
    );
    hits.into_iter().next().expect("one")
}

fn edit(files: &mut [(String, String)], label: &str, from: &str, to: &str) {
    let (_, text) = files
        .iter_mut()
        .find(|(at, _)| at == label)
        .unwrap_or_else(|| panic!("{label} is in the example"));
    assert!(text.contains(from), "`{from}` is in {label}");
    *text = text.replacen(from, to, 1);
}

/// The billing example with one workload made stateful: `topology.workloads.invoice-service` is
/// refused (`ConflictingDeclaration`, `topology.rs` `Workload::check`). Its path has no declaration
/// needle (`workloads` is structural), so the only needle is `invoice-service:` — the workload key,
/// which is exactly the line the author must edit. The base commit cited it; the fix drops it,
/// because no declaration is located, and the refusal falls to `<document>`.
#[test]
fn a_stateful_workload_in_the_billing_example_is_cited_at_its_workload_key() {
    let mut files = billing();
    edit(
        &mut files,
        "topology.yaml",
        "    invoice-service:\n      replicas:\n        min: 2\n      stateless: true",
        "    invoice-service:\n      replicas:\n        min: 2\n      stateless: false",
    );
    let expected = only_occurrence(&files, "invoice-service:");

    let spans = refusals(&files);
    let (_, source, located) = cited(&spans, "topology.workloads.invoice-service");
    assert_eq!(
        (source.clone(), located.map(|at| at.line)),
        (expected.0, Some(expected.1)),
        "the workload key is the refused construct's own line, and the only needle there is"
    );
}

/// The billing example with a workload renamed to a component nobody declares:
/// `topology.workloads.mail-service` is refused by `validate_topology` (`UndeclaredReference`).
/// Same shape, same loss: the only needle is the workload key.
#[test]
fn a_workload_naming_an_undeclared_component_is_cited_at_its_workload_key() {
    let mut files = billing();
    edit(
        &mut files,
        "topology.yaml",
        "    email-service:",
        "    mail-service:",
    );
    let expected = only_occurrence(&files, "mail-service:");

    let spans = refusals(&files);
    let (_, source, located) = cited(&spans, "topology.workloads.mail-service");
    assert_eq!(
        (source.clone(), located.map(|at| at.line)),
        (expected.0, Some(expected.1)),
        "the workload key names the undeclared component and is the line to edit"
    );
}

/// The story's own defect, one level down: the wrong guess `filed:` is unique and lands on a
/// **sibling outcome's** payload key, inside the refused command's block. The acceptance says a
/// needle that cannot match its own target (`- name: filed` is never written `filed:`) is not
/// reported as that refusal's line; the block check lets it through because the block is the
/// command's, not the outcome's.
const SIBLING: &str = "\
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
      - name: other
        emits:
          - shop.probe.Filed
        payload:
          shop.probe.Filed:
            filed: input.note
";

#[test]
fn an_outcome_refusal_is_not_cited_at_a_sibling_outcomes_payload_key() {
    let files = vec![("a.yaml".to_owned(), SIBLING.to_owned())];
    let payload = only_occurrence(&files, "filed:");

    let spans = refusals(&files);
    let (_, source, located) = cited(&spans, "command.shop.probe.Doit.outcomes.filed");
    let line = located.map(|at| at.line);
    assert_ne!(
        (source.as_str(), line),
        (payload.0.as_str(), Some(payload.1)),
        "`filed: input.note` is outcome `other`'s payload key, not outcome `filed`; the guess \
         cannot match its own target and is reported anyway — cited: {spans:?}"
    );
    let declared = SIBLING
        .lines()
        .position(|text| text.ends_with("name: shop.probe.Doit"))
        .expect("declared")
        + 1;
    assert_eq!(
        (source.as_str(), line),
        ("a.yaml", Some(declared)),
        "falls through to the refused command's declaration, as the one-file case does"
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

fn located_line(text: &str, needle: &str) -> usize {
    text.lines()
        .position(|line| line.contains(needle))
        .expect("present")
        + 1
}

/// A right guess written above the declaration key within the same list item. YAML key order is
/// free, so `id:` last is as valid as `id:` first; the key is still the refused construct's line,
/// and the base commit cited it.
#[test]
fn a_right_key_written_above_the_declaration_key_is_still_cited() {
    let text = "\
bindings:
  - mapping:
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
        Some(located_line(text, "recipient:")),
        "cited at the mapping key, not coarsened to the id line: {span}"
    );
}

/// CRLF, a comment at column 0 and a blank line inside the block must not close it; the next item
/// at the declaration's own list indent must.
#[test]
fn crlf_comments_and_blank_lines_keep_the_block_and_the_next_item_closes_it() {
    let text = "bindings:\r\n  - id: notify-on-invoice-created\r\n\r\n# a note\r\n    mapping:\r\n      recipient: customer_email\r\n  - id: other\r\n    mapping:\r\n      address: x\r\n";
    let mut sources = SourceMap::new();
    sources.insert("c.yaml", text);
    let locator = Locator::new(&sources, &["c.yaml"]);

    let inside = locator.span(
        "binding.notify-on-invoice-created.mapping.recipient",
        &needles("recipient", "notify-on-invoice-created"),
    );
    assert_eq!(inside.located.map(|at| at.line), Some(6), "{inside}");

    let outside = locator.span(
        "binding.notify-on-invoice-created.mapping.address",
        &needles("address", "notify-on-invoice-created"),
    );
    assert_eq!(outside.located.map(|at| at.line), Some(2), "{outside}");
}
