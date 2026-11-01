//! Adversary, D357 U1 pass 1: `story:a-refusal-records-the-document-it-was-read-from`.
//!
//! The acceptance: a refusal of a multi-file specification names the file its declaration was read
//! from, including when the same name is declared in two files.
//!
//! `Provenance` answers two questions — which file was being read when a refusal was pushed, and
//! which file the kept copy of a name came from — and its own documentation says neither answers a
//! refusal made *before any file is read*, so the consumer "falls back to whatever it did before".
//! `read_from` in `resolve.rs` does not fall back: when `read_while` is `None` it asks `kept`, and
//! a command refused before the read loop (`set_effects::refuse_*`, run over every file's raw
//! commands before `absorb`) is answered with the file of the *kept* copy of its name. With the
//! name declared in two files and the refused construct written in the copy assembly drops, that
//! is the other file, and the search confined to it then lands on the kept copy's `name:` line.

use ess_compiler::resolve::{diagnose_locating, diagnose_traced};
use ess_compiler::source::{Location, SourceMap};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationErrors};

type Cited = (ValidationCode, String, String, Option<Location>);

fn parsed(files: &[(&str, &str)]) -> Vec<(Source, RawSpecFile)> {
    files
        .iter()
        .map(|(label, text)| {
            (
                Source::new(*label),
                RawSpecFile::parse(text).expect("the fixture is well formed YAML"),
            )
        })
        .collect()
}

fn sources(files: &[(&str, &str)]) -> (SourceMap, Vec<String>) {
    let mut sources = SourceMap::new();
    let mut labels = Vec::new();
    for (label, text) in files {
        sources.insert(*label, *text);
        labels.push((*label).to_owned());
    }
    (sources, labels)
}

fn pair(errors: &ValidationErrors, diagnostics: &ess_compiler::Diagnostics) -> Vec<Cited> {
    assert_eq!(
        errors.len(),
        diagnostics.as_slice().len(),
        "one per refusal"
    );
    errors
        .as_slice()
        .iter()
        .zip(diagnostics.as_slice())
        .map(|(error, diagnostic)| {
            let span = diagnostic
                .span
                .as_ref()
                .expect("every bridged refusal spans");
            (
                error.code,
                error.location.clone(),
                span.source.clone(),
                span.located,
            )
        })
        .collect()
}

/// As `ess specify validate` cites a refused assembly.
fn traced(files: &[(&str, &str)]) -> Vec<Cited> {
    let traced = Specification::assemble_traced(parsed(files))
        .expect_err("the fixture is refused on purpose");
    let (sources, labels) = sources(files);
    pair(&traced.errors, &diagnose_traced(&traced, &sources, &labels))
}

/// As the same command cited a refused assembly before this unit (`diagnose_locating`, unchanged).
fn searched(files: &[(&str, &str)]) -> Vec<Cited> {
    let errors =
        Specification::assemble(parsed(files)).expect_err("the fixture is refused on purpose");
    let (sources, labels) = sources(files);
    pair(&errors, &diagnose_locating(&errors, &sources, &labels))
}

fn one<'a>(cited: &'a [Cited], code: ValidationCode, location: &str) -> &'a Cited {
    let found: Vec<&Cited> = cited
        .iter()
        .filter(|(at_code, at, _, _)| *at_code == code && at == location)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "one {code:?} at `{location}`; cited: {cited:#?}"
    );
    found[0]
}

fn line_of(text: &str, needle: &str) -> usize {
    let lines: Vec<usize> = text
        .lines()
        .enumerate()
        .filter(|(_, line)| line.trim_end().ends_with(needle))
        .map(|(index, _)| index + 1)
        .collect();
    assert_eq!(lines.len(), 1, "`{needle}` ends one line of the fixture");
    lines[0]
}

// ---- below ess/16: `instances:` refused before any file is read ---------------------------

const OLD_FIRST: &str = "\
format: ess/1
system: shop
version: v1
domains: [shop.probe]
domain: shop.probe
events:
  - name: shop.probe.Filed
commands:
  - name: shop.probe.Doit
    outcomes:
      - name: filed
        emits:
          - shop.probe.Filed
";

/// The second copy of `shop.probe.Doit`: the one assembly drops, since the first converts.
const OLD_SECOND: &str = "\
domain: shop.probe
commands:
  - name: shop.probe.Doit
    outcomes:
      - name: filed
        instances: {where: always}
        emits:
          - shop.probe.Filed
";

const INSTANCES: &str = "command.shop.probe.Doit.outcomes.filed.instances";

/// The premise, green: before the unit, the name search cited this refusal in `second.yaml`, at
/// the `instances:` key the author wrote — the trailing-key guess is unique across both files.
#[test]
fn before_the_unit_the_below_ess_16_instances_refusal_was_cited_in_the_file_that_wrote_it() {
    let files = [("first.yaml", OLD_FIRST), ("second.yaml", OLD_SECOND)];
    let cited = searched(&files);
    let (_, _, source, located) = one(&cited, ValidationCode::UnsupportedFormatVersion, INSTANCES);
    assert_eq!(
        (source.as_str(), located.map(|at| at.line)),
        (
            "second.yaml",
            Some(line_of(OLD_SECOND, "instances: {where: always}"))
        ),
        "{cited:#?}"
    );
}

/// Red: `diagnose_traced` cites the same refusal in `first.yaml`, the kept copy's file, which
/// never wrote `instances:` — a correct citation the unit turned into a wrong one.
#[test]
fn a_refusal_made_before_any_file_is_read_is_cited_in_the_file_that_wrote_it() {
    let files = [("first.yaml", OLD_FIRST), ("second.yaml", OLD_SECOND)];
    let cited = traced(&files);
    let (_, _, source, located) = one(&cited, ValidationCode::UnsupportedFormatVersion, INSTANCES);
    assert_eq!(
        (source.as_str(), located.map(|at| at.line)),
        (
            "second.yaml",
            Some(line_of(OLD_SECOND, "instances: {where: always}"))
        ),
        "`instances:` is written only in second.yaml; the refusal is pushed before the read \
         loop, so `Provenance::read_while` has no answer and `kept` names the other copy's file: \
         {cited:#?}"
    );
}

// ---- at ess/25: a per-row event without `instances:`, refused before any file is read -----

const NEW_FIRST: &str = "\
format: ess/25
system: shop
version: v1
domains: [shop.probe]
domain: shop.probe
events:
  - name: shop.probe.Filed
commands:
  - name: shop.probe.Doit
    outcomes:
      - name: filed
        emits:
          - shop.probe.Filed
";

const NEW_SECOND: &str = "\
domain: shop.probe
commands:
  - name: shop.probe.Doit
    outcomes:
      - name: filed
        emits:
          - {event: shop.probe.Filed, each: changed}
";

const PER_ROW: &str = "command.shop.probe.Doit.outcomes.filed.emits[0]";

/// Red, at a current format: the per-row entry is written only in `second.yaml`, and the refusal
/// is cited in `first.yaml` at the line of the *other* declaration's `name:` — a file and a line,
/// both wrong, where the name search answered `<document>`.
#[test]
fn a_per_row_event_refused_on_the_dropped_copy_is_not_cited_at_the_kept_copy() {
    let files = [("first.yaml", NEW_FIRST), ("second.yaml", NEW_SECOND)];
    let cited = traced(&files);
    let (code, _, source, located) = cited
        .iter()
        .find(|(_, at, _, _)| at == PER_ROW)
        .unwrap_or_else(|| panic!("the per-row entry is refused at `{PER_ROW}`: {cited:#?}"));
    assert_ne!(*code, ValidationCode::DuplicateDeclaration, "{cited:#?}");
    assert_eq!(
        source.as_str(),
        "second.yaml",
        "the refused entry is written in second.yaml only (cited at {located:?}): {cited:#?}"
    );
}
