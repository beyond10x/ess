//! The idioms a generated case-record domain uses, held (beyond10x/ess#426, part a).
//!
//! A producer that owns a format — a protocol of claims, evidence and revisions — writes its case
//! record as authored `ess/N` source and gates it with validate, compile and synthesize; ESS has no
//! import adapter for it. `docs/design/generated-case-record-domains.md` states the boundary and
//! three idioms, and each idiom is a case here: quote every scalar, write conjunctions with
//! structured `all`/`any`/`not`, and make the refusal the default branch, which past 64 joint
//! assignments is required. The note and the guide's link to it are held by the last two cases.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use ess_domain::{command::OutcomeCondition, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_primitives::facts::{FactPath, FactStore, FactValue};
use ess_primitives::predicate::Truth;

const NOTE: &str = "docs/design/generated-case-record-domains.md";
const GUIDE: &str = "website/docs/guides/specify/guards-and-predicates.md";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("{relative}: {error}"))
}

/// A case record of `claims` three-valued claims, `a`, `b`, …, decided by `Accept`, whose
/// outcomes are `outcomes`.
fn case_record(claims: usize, outcomes: &str) -> String {
    let names: Vec<char> = ('a'..='z').take(claims).collect();
    let fields = names.iter().fold(String::new(), |mut fields, name| {
        let _ = writeln!(fields, "      - {{name: {name}, type: probe.claims.Truth}}");
        fields
    });
    let unknown: Vec<String> = names
        .iter()
        .map(|name| format!("{name}: Unknown"))
        .collect();
    format!(
        "format: ess/22
system: probe
version: v1
domain: probe.claims
types:
  - {{name: probe.claims.CaseId, kind: newtype, of: Uuid}}
  - name: probe.claims.Truth
    kind: enum
    variants: [\"Holds\", \"Fails\", \"Unknown\"]
  - name: probe.claims.Verdict
    kind: enum
    variants: [\"True\", \"False\", \"Unknown\"]
entities:
  - name: probe.claims.Case
    identity: {{name: case_id, type: probe.claims.CaseId}}
    fields:
      - {{name: verdict, type: probe.claims.Verdict}}
{fields}    lifecycle:
      initial: Open
      states: [Open, Accepted]
      terminal: [Accepted]
      transitions:
        - {{name: accept, from: [Open], to: Accepted}}
errors:
  - {{name: probe.claims.NotReady, summary: A claim does not hold., fields: []}}
events:
  - name: probe.claims.Opened
    fields:
      - {{name: case_id, type: probe.claims.CaseId}}
  - name: probe.claims.Accepted
    fields: []
commands:
  - name: probe.claims.Open
    input: []
    outcomes:
      - name: opened
        creates: probe.claims.Case
        instance: case_id
        emits: [probe.claims.Opened]
        payload:
          probe.claims.Opened: {{case_id: {{generated: true}}}}
        sets: {{verdict: \"Unknown\", {unknown}}}
  - name: probe.claims.Accept
    input:
      - {{name: case_id, type: probe.claims.CaseId}}
    outcomes:
{outcomes}",
        unknown = unknown.join(", ")
    )
}

/// `all: [a == "Holds", …]` over the first `claims` claims.
fn all_hold(claims: usize) -> String {
    let tests: Vec<String> = ('a'..='z')
        .take(claims)
        .map(|name| format!("{name} == \"Holds\""))
        .collect();
    format!("{{all: [{}]}}", tests.join(", "))
}

/// The success, guarded by every claim holding.
fn accepted(guard: &str) -> String {
    format!(
        "      - name: accepted
        when_subject: {{predicate: {guard}}}
        moves: probe.claims.Case.accept
        instance: case_id
        emits: [probe.claims.Accepted]
"
    )
}

/// The refusal, guarded by `guard`, or the default branch where `guard` is `None`.
fn not_ready(guard: Option<&str>) -> String {
    let guard = guard
        .map(|guard| format!("        when_subject: {{predicate: {guard}}}\n"))
        .unwrap_or_default();
    format!("      - name: not-ready\n{guard}        error: probe.claims.NotReady\n")
}

/// A guarded success and its guarded negation over `claims` claims, with no default.
fn guarded_pair(claims: usize) -> String {
    let all = all_hold(claims);
    case_record(
        claims,
        &format!(
            "{}{}",
            not_ready(Some(&format!("{{not: {all}}}"))),
            accepted(&all)
        ),
    )
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("case.yaml"), raw)])
}

fn validates(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{errors}\n---\n{text}"))
}

fn refused(text: &str) -> ValidationErrors {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
}

#[test]
fn quoted_true_false_variants_validate() {
    let text = case_record(
        1,
        &format!("{}{}", not_ready(None), accepted("verdict == \"True\"")),
    );
    let spec = validates(&text);
    let accept = &spec.commands()[&"probe.claims.Accept".parse().unwrap()];
    let success = accept
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "accepted")
        .expect("the success");
    let OutcomeCondition::SubjectPredicate { predicate, .. } = &success.condition else {
        panic!("a subject predicate: {:?}", success.condition)
    };
    // The guard compares with the variant `True`, the text, and selects exactly it.
    let holding = |verdict: &str| {
        let mut facts = FactStore::new();
        facts.set(FactPath::new("verdict").unwrap(), FactValue::text(verdict));
        predicate.evaluate(&facts)
    };
    assert_eq!(holding("True"), Truth::True);
    assert_eq!(holding("False"), Truth::False);
    assert_eq!(holding("Unknown"), Truth::False);
}

#[test]
fn structured_all_conjunction_validates() {
    validates(&case_record(
        2,
        &format!("{}{}", not_ready(None), accepted(&all_hold(2))),
    ));
}

#[test]
fn compact_and_is_refused_naming_structured_form() {
    let errors = refused(&case_record(
        2,
        &format!(
            "{}{}",
            not_ready(None),
            accepted("a == \"Holds\" and b == \"Holds\"")
        ),
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.message.contains("use structured any/all/not")),
        "{errors}"
    );
}

#[test]
fn refusal_default_beside_guarded_success_validates() {
    validates(&case_record(
        4,
        &format!("{}{}", not_ready(None), accepted(&all_hold(4))),
    ));
}

#[test]
fn guarded_pair_within_cap_validates() {
    // 3 × 3 × 3 = 27 joint assignments.
    validates(&guarded_pair(3));
}

#[test]
fn guarded_pair_past_cap_is_refused_non_exhaustive() {
    // 3 × 3 × 3 × 3 = 81 joint assignments: the proof declines, and no default answers instead.
    let errors = refused(&guarded_pair(4));
    assert!(
        errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::NonExhaustiveBranches
                && error.location == "command.probe.claims.Accept.outcomes"
        }),
        "{errors}"
    );
}

/// The section of `page` under the heading `## {heading}`, up to the next one.
///
/// Split the way `crates/edge/ess-xtask/tests/d2_constraint_home_adversary.rs` splits the home
/// page's "Where it is enforced".
fn section<'a>(page: &'a str, heading: &str) -> &'a str {
    page.split(&format!("\n## {heading}\n"))
        .nth(1)
        .unwrap_or_else(|| panic!("the page carries the heading `## {heading}`"))
        .split("\n## ")
        .next()
        .expect("a section ends at the next heading or at the end of the page")
}

#[test]
fn generated_case_record_note_states_the_boundary() {
    let note = read(NOTE);
    for (heading, phrases) in [
        (
            "A producer emits authored source",
            &[
                "`protocol/1`",
                "no ESS import adapter",
                "validate, compile and synthesize",
            ][..],
        ),
        (
            "Three idioms a generator uses",
            &[
                "quote every scalar",
                "`all`/`any`/`not`",
                "default branch",
                "64 joint assignments",
            ][..],
        ),
        (
            "The meaning stays in Canon",
            &["ADR 0067", "ADR 0076", "`canon-case-record/1`"][..],
        ),
        (
            "Names",
            &["case-record domain", "`ess specify protocol`"][..],
        ),
    ] {
        let text = section(&note, heading);
        for phrase in phrases {
            assert!(
                text.contains(phrase),
                "{NOTE}, `## {heading}`, does not state {phrase:?}"
            );
        }
    }
}

#[test]
fn guards_guide_links_the_generated_case_record_note() {
    let guide = read(GUIDE);
    let text = section(&guide, "Guard an outcome by the subject's stored fields");
    let linked = text
        .split("](")
        .skip(1)
        .filter_map(|rest| rest.split(')').next())
        .any(|target| target.ends_with(NOTE));
    assert!(
        linked,
        "{GUIDE}, `## Guard an outcome by the subject's stored fields`, links no target ending in \
         `{NOTE}`"
    );
}
