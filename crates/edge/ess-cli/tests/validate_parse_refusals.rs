//! `ess specify validate` reports every predicate in a file that does not parse, each at its
//! declaration, beside the file's other refusals (beyond10x/ess#448).
//!
//! A predicate used to be parsed while the document was deserialized, so the first one that did
//! not parse ended the whole file: one line, no declaration, no line number, no code, and every
//! other refusal in the file hidden behind it. A predicate is now read where it is written, and
//! its declaration's own pass reports it. What still stops a file is a refusal of its structure —
//! a missing required key, an unknown key, a value of the wrong YAML shape — and
//! `website/docs/reference/diagnostics.md` says so beside the `SPEC` family.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/parse-refusals")
            .join(name),
    )
    .unwrap_or_else(|error| panic!("fixture {name}: {error}"))
}

/// What `ess specify validate --format json` printed for `text`, written as one file.
struct Validated {
    code: Option<i32>,
    problems: Vec<String>,
    diagnostics: Vec<Value>,
    output: String,
}

impl Validated {
    /// The problems naming `location`, with the bracketed validation name `name`.
    fn at(&self, name: &str, location: &str) -> Vec<&String> {
        let prefix = format!("[{name}] {location}: ");
        self.problems
            .iter()
            .filter(|problem| problem.starts_with(&prefix))
            .collect()
    }

    /// The compiler diagnostic for the refusal at `location`.
    fn diagnostic(&self, location: &str) -> &Value {
        let found: Vec<&Value> = self
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic["span"]["path"] == location)
            .collect();
        assert_eq!(
            found.len(),
            1,
            "one diagnostic at `{location}`:\n{}",
            self.output
        );
        found[0]
    }

    /// The line a diagnostic was located on.
    fn line(&self, location: &str) -> u64 {
        self.diagnostic(location)["span"]["located"]["line"]
            .as_u64()
            .unwrap_or_else(|| panic!("`{location}` carries a line:\n{}", self.output))
    }

    /// The code a diagnostic carries.
    fn code(&self, location: &str) -> String {
        self.diagnostic(location)["code"]
            .as_str()
            .unwrap_or_default()
            .to_owned()
    }
}

fn validate(name: &str, text: &str) -> Validated {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "validate-parse-refusals-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    let file = directory.join(name);
    std::fs::write(&file, text).expect("the specification is written");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["specify", "validate", "--format", "json", "--path"])
        .arg(&file)
        .output()
        .expect("the `ess` binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let report: Value = serde_json::from_str(&stdout).unwrap_or_else(|error| {
        panic!(
            "a JSON report: {error}\nstdout: {stdout}\nstderr: {}",
            String::from_utf8_lossy(&output.stderr)
        )
    });
    // A valid specification reports neither list.
    let problems = report["problems"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|problem| problem.as_str().expect("a problem is text").to_owned())
        .collect();
    let diagnostics = report["diagnostics"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    Validated {
        code: output.status.code(),
        problems,
        diagnostics,
        output: stdout,
    }
}

/// `text` with `from` replaced by `to`, which must be there.
fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "`{from}` is in the fixture");
    text.replacen(from, to, 1)
}

/// The 1-based line of the first line of `text` at or after `after` that contains `needle`.
fn line_of(text: &str, after: &str, needle: &str) -> u64 {
    let start = text
        .lines()
        .position(|line| line.contains(after))
        .unwrap_or_else(|| panic!("`{after}` is in the fixture"));
    let index = text
        .lines()
        .enumerate()
        .skip(start)
        .find(|(_, line)| line.contains(needle))
        .unwrap_or_else(|| panic!("`{needle}` follows `{after}`"))
        .0;
    u64::try_from(index + 1).expect("a line number")
}

const FIRST: &str = "view.probe.item.FirstView.filter";
const SECOND: &str = "view.probe.item.SecondView.filter";

#[test]
fn two_unparsable_view_filters_in_one_file_report_both() {
    let text = fixture("two-unparsable-filters.yaml");
    let validated = validate("two-unparsable-filters.yaml", &text);
    assert_eq!(validated.code, Some(1), "{}", validated.output);
    assert_eq!(
        validated.problems.len(),
        2,
        "one refusal per filter, and nothing else:\n{}",
        validated.output
    );
    for (location, operator, view) in [
        (FIRST, "\"matches\"", "probe.item.FirstView"),
        (SECOND, "\"suffix\"", "probe.item.SecondView"),
    ] {
        let found = validated.at("unparsable_predicate", location);
        assert_eq!(found.len(), 1, "`{location}`:\n{}", validated.output);
        assert!(
            found[0].contains("cannot parse predicate") && found[0].contains(operator),
            "today's sentence, naming {operator}: {}",
            found[0]
        );
        // The line of the declaration, as every refusal located at a view's `filter` is.
        assert_eq!(
            validated.line(location),
            line_of(&text, view, &format!("name: {view}")),
            "`{location}` is located at its view's declaration:\n{}",
            validated.output
        );
    }
}

#[test]
fn unparsable_compact_predicates_report_each_declaration() {
    let validated = validate(
        "two-unparsable-compact.yaml",
        &fixture("two-unparsable-compact.yaml"),
    );
    assert_eq!(validated.code, Some(1), "{}", validated.output);
    assert_eq!(validated.problems.len(), 2, "{}", validated.output);
    for (location, written) in [(FIRST, "label ~="), (SECOND, "label =~")] {
        let found = validated.at("unparsable_predicate", location);
        assert_eq!(found.len(), 1, "`{location}`:\n{}", validated.output);
        assert!(found[0].contains(written), "{}", found[0]);
        validated.line(location);
    }
}

#[test]
fn parse_refusal_does_not_hide_semantic_refusals() {
    let validated = validate(
        "parse-plus-semantic.yaml",
        &fixture("parse-plus-semantic.yaml"),
    );
    assert_eq!(validated.code, Some(1), "{}", validated.output);
    assert_eq!(validated.problems.len(), 2, "{}", validated.output);
    assert_eq!(
        validated.at("unparsable_predicate", FIRST).len(),
        1,
        "the parse refusal:\n{}",
        validated.output
    );
    let semantic = validated.at("unobservable_fact", SECOND);
    assert_eq!(
        semantic.len(),
        1,
        "the semantic refusal in the same file:\n{}",
        validated.output
    );
    assert!(semantic[0].contains("`nosuch`"), "{}", semantic[0]);
}

/// Every predicate position of `every-position.yaml`: what it is written as, an unparsable
/// spelling of it, and the declaration path its refusal names.
const POSITIONS: [(&str, &str, &str); 6] = [
    (
        "    filter: label != \"\"",
        "    filter: label ~= \"\"",
        "view.probe.item.Labelled.filter",
    ),
    (
        "        when: input.label == \"\"",
        "        when: input.label ~= \"\"",
        "command.probe.item.Create.outcomes.empty-label.when",
    ),
    (
        "        when_subject: {predicate: label == \"\"}",
        "        when_subject: {predicate: label ~= \"\"}",
        "command.probe.item.Close.outcomes.unlabelled.when_subject.predicate",
    ),
    (
        "        when_related: {via: input.owner, predicate: label == \"\"}",
        "        when_related: {via: input.owner, predicate: label ~= \"\"}",
        "command.probe.item.Create.outcomes.unlabelled-owner.when_related.predicate",
    ),
    (
        "    invariants: [label != \"\"]",
        "    invariants: [label ~= \"\"]",
        "entity probe.item.Item.invariants[0]",
    ),
    (
        "      where: event.kind == fancy",
        "      where: event.kind ~= fancy",
        "binding.close-fancy.when.where",
    ),
];

#[test]
fn every_predicate_position_collects() {
    let valid = fixture("every-position.yaml");
    let premise = validate("every-position.yaml", &valid);
    assert_eq!(
        premise.code,
        Some(0),
        "the premise: as written, the fixture validates:\n{}",
        premise.output
    );
    let broken = POSITIONS
        .iter()
        .fold(valid, |text, (from, to, _)| replaced(&text, from, to));
    let validated = validate("every-position.yaml", &broken);
    assert_eq!(validated.code, Some(1), "{}", validated.output);
    for (_, _, location) in POSITIONS {
        assert_eq!(
            validated.at("unparsable_predicate", location).len(),
            1,
            "one refusal at `{location}`:\n{}",
            validated.output
        );
        validated.line(location);
    }
    assert_eq!(
        validated.problems.len(),
        POSITIONS.len(),
        "one refusal per position, and nothing else:\n{}",
        validated.output
    );
}

#[test]
fn failed_predicate_does_not_cascade() {
    let valid = fixture("every-position.yaml");
    for (from, to, location) in POSITIONS {
        let validated = validate("every-position.yaml", &replaced(&valid, from, to));
        assert_eq!(
            validated.problems.len(),
            1,
            "only the refusal at `{location}`; nothing that reads the predicate, or the \
             declaration it withholds, is refused a second time:\n{}",
            validated.output
        );
        assert_eq!(
            validated.at("unparsable_predicate", location).len(),
            1,
            "{}",
            validated.output
        );
        assert_eq!(validated.diagnostics.len(), 1, "{}", validated.output);
    }
}

#[test]
fn predicate_refusal_codes_unchanged_after_per_declaration_parse() {
    let valid = fixture("every-position.yaml");
    let nulls = [
        (POSITIONS[0].0, "    filter: label == null", POSITIONS[0].2),
        (
            POSITIONS[1].0,
            "        when: input.label == null",
            POSITIONS[1].2,
        ),
        (
            POSITIONS[4].0,
            "    invariants: [label == null]",
            POSITIONS[4].2,
        ),
    ];
    let broken = nulls.iter().fold(valid.clone(), |text, (from, to, _)| {
        replaced(&text, from, to)
    });
    let validated = validate("every-position.yaml", &broken);
    assert_eq!(
        validated.problems.len(),
        nulls.len(),
        "{}",
        validated.output
    );
    for (_, _, location) in nulls {
        assert_eq!(
            validated.at("null_comparison", location).len(),
            1,
            "{}",
            validated.output
        );
        assert_eq!(
            validated.code(location),
            "ESS-SPEC-017",
            "a null comparison keeps its code at `{location}`:\n{}",
            validated.output
        );
    }

    // An unknown operator: the `SPEC` family it was filed under while it was read, at every
    // position, and never the family of the construct it now names.
    let broken = POSITIONS
        .iter()
        .fold(valid, |text, (from, to, _)| replaced(&text, from, to));
    let validated = validate("every-position.yaml", &broken);
    for (_, _, location) in POSITIONS {
        assert_eq!(
            validated.code(location),
            "ESS-SPEC-012",
            "`{location}`:\n{}",
            validated.output
        );
    }
    for diagnostic in &validated.diagnostics {
        let code = diagnostic["code"].as_str().unwrap_or_default();
        assert!(
            code.starts_with("ESS-SPEC-"),
            "no predicate parse refusal moves to a construct family: {code}\n{}",
            validated.output
        );
    }
}

/// The sentence `diagnostics.md` carries in its `SPEC` row, which a reader of a refusal list needs
/// to know where the list for a file can end early.
const STOP_RULE: &str = "A refusal of a document's structure — a missing required key, an \
                         unknown key, a value of the wrong shape — stops that file at the first \
                         one; a predicate that does not parse does not, and is reported at its \
                         declaration beside the file's other refusals.";

#[test]
fn structural_refusal_still_stops_and_is_documented() {
    // A view with no `source:` beside a view with an unparsable filter: the structural refusal
    // ends the file, so the predicate is never reached.
    let text = replaced(
        &fixture("two-unparsable-filters.yaml"),
        "    source: probe.item.Item\n",
        "",
    );
    let validated = validate("structural.yaml", &text);
    assert_eq!(validated.code, Some(1), "{}", validated.output);
    assert_eq!(
        validated.problems.len(),
        1,
        "one structural refusal, and the file stops there:\n{}",
        validated.output
    );
    assert!(
        validated.problems[0].contains("missing field `source`"),
        "{}",
        validated.output
    );

    let page = std::fs::read_to_string(root().join("website/docs/reference/diagnostics.md"))
        .expect("the diagnostics reference");
    let row = page
        .lines()
        .find(|line| line.starts_with("| `SPEC` |"))
        .expect("the diagnostics reference has a `SPEC` family row");
    assert!(
        row.contains(STOP_RULE),
        "the `SPEC` row states the structural stop rule `{STOP_RULE}`:\n{row}"
    );
}
