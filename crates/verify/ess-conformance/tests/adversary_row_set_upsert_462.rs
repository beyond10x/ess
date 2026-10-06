//! Adversary pass on beyond10x/ess#462: an absent addressed row a creation branch takes falls
//! through the row sets to the creation (`creation_takes_absent` in `interpret/execute.rs`).
//!
//! Each case derives a variant of `tests/fixtures/row-set-upsert.yaml` and asserts what the design
//! (`docs/design/filtered-related-reads.md`, "Subject borrowing and precedence") requires: a row
//! set that does not fire leaves the command's verdict where the same command without the row set
//! puts it, a creation that cannot be reached for this request does not claim the absent row, and
//! the reference interpreter passes the suite ESS itself synthesizes for the variant.

mod support_go;

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::OutcomeRef;
use ess_conformance::synthesize::synthesize;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/row-set-upsert.yaml");

const REFUSED: &str = "      - name: refused
        when_related:
          entity: demo.shelf.Library
          where: curator == input.curator
          exists: true
        error: demo.shelf.Refused
";

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let specification = Specification::assemble([(Source::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the variant validates:\n{errors}\n{text}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the variant resolves:\n{diagnostics}"))
}

/// The fixture with `Shelve`'s block changed by `change`, every other command untouched.
fn shelve_changed(text: &str, change: impl Fn(&str) -> String) -> String {
    let (head, tail) = text
        .split_once("  - name: demo.shelf.Shelve\n")
        .expect("the Shelve command");
    let (shelve, rest) = tail
        .split_once("  - name: demo.shelf.Restock\n")
        .expect("the next command");
    let changed = change(shelve);
    assert_ne!(changed, shelve, "the variant changes Shelve");
    format!("{head}  - name: demo.shelf.Shelve\n{changed}  - name: demo.shelf.Restock\n{rest}")
}

/// The same variant without `Shelve`'s row-set refusal.
fn unguarded(text: &str) -> String {
    shelve_changed(text, |shelve| {
        assert!(shelve.contains(REFUSED), "the row set is declared");
        shelve.replacen(REFUSED, "", 1)
    })
}

fn replace(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "`{from}` is in the text");
    text.replacen(from, to, 1)
}

fn target(text: &str) -> Interpreted {
    let target = Interpreted::for_model(model(text));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.shelf/authored/adversary-462".parse().unwrap(),
            CorrelationId::new("adversary-462").unwrap(),
        ))
        .unwrap();
    target
}

/// The outcome the target answers, or its error, as text.
fn answer(target: &dyn ConformanceTarget, command: &str, input: &[(&str, &str)]) -> String {
    let request = SemanticCommandRequest {
        command: command.parse().unwrap(),
        actor: None,
        caller: None,
        input: input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), Node::Text((*value).to_owned())))
            .collect(),
        correlation: CorrelationId::new("adversary-462").unwrap(),
    };
    match target.execute_command(request) {
        Ok(result) => result
            .outcome
            .map_or_else(|| "<none>".to_owned(), |taken| taken.outcome.to_string()),
        Err(error) => format!("error: {error}"),
    }
}

fn force(target: &dyn ConformanceTarget, outcome: &str) {
    target
        .configure_external_outcome(ExternalOutcomeControl {
            force: OutcomeRef::new(
                "demo.shelf.Shelve".parse().unwrap(),
                outcome.parse().unwrap(),
            ),
            correlation: CorrelationId::new("adversary-462").unwrap(),
        })
        .unwrap();
}

fn open(target: &dyn ConformanceTarget, library: &str, curator: &str) {
    assert_eq!(
        answer(
            target,
            "demo.shelf.OpenLibrary",
            &[("library", library), ("curator", curator)]
        ),
        "opened"
    );
}

fn run(suite: &ConformanceSuite, target: &Interpreted) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, target)
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> BTreeSet<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

/// `Shelve` with a declared unknown-instance branch, creating only when the input asks for a new
/// book (`mode == "new"`) rather than on an external fact.
fn input_guarded_creation() -> String {
    shelve_changed(MODEL, |shelve| {
        let shelve = replace(
            shelve,
            "      - {name: curator, type: String}\n",
            "      - {name: curator, type: String}\n      - {name: mode, type: String}\n",
        );
        let shelve = replace(
            &shelve,
            "    outcomes:\n",
            "    outcomes:\n      - name: unknown-book\n        unknown_instance: true\n        error: demo.shelf.UnknownBook\n",
        );
        replace(
            &shelve,
            "external: the book is not yet shelved",
            "when: mode == \"new\"",
        )
    })
}

/// A request for an existing book (`mode: old`) whose book is absent is the declared unknown
/// instance: the creation that shares the identity field is not the branch this input selects.
/// The row set passes over (only a decoy library is open), so it must not change the verdict the
/// unguarded command gives.
#[test]
fn adv_unreachable_creation_does_not_claim_absent_row() {
    let guarded = input_guarded_creation();
    let request = [
        ("book", "absent-book"),
        ("curator", "the-curator"),
        ("mode", "old"),
    ];

    let plain = target(&unguarded(&guarded));
    open(&plain, "decoy", "other-curator");
    let plain = answer(&plain, "demo.shelf.Shelve", &request);

    let with_row_set = target(&guarded);
    open(&with_row_set, "decoy", "other-curator");
    let with_row_set = answer(&with_row_set, "demo.shelf.Shelve", &request);

    assert_eq!(
        (plain.as_str(), with_row_set.as_str()),
        ("unknown-book", "unknown-book"),
        "an absent book the creation does not take is unknown, with or without the row set"
    );
}

/// The suite ESS synthesizes for the input-guarded upsert passes on the reference interpreter.
#[test]
fn adv_input_guarded_upsert_suite_passes_interpreter() {
    let text = input_guarded_creation();
    let synthesis = synthesize(&model(&text));
    let refusals: Vec<String> = synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("demo.shelf.Shelve"))
        .collect();
    let target = Interpreted::for_model(model(&text));
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(&synthesis.suite),
    )
    .run_admitted(&admitted, &target)
    .into_report();
    for scenario in report
        .scenarios
        .iter()
        .filter(|s| s.status != Status::Passed)
    {
        eprintln!("NOT PASSED {}: {:#?}", scenario.scenario, scenario.checks);
        let steps = synthesis
            .suite
            .scenarios
            .iter()
            .find(|(id, _)| **id == scenario.scenario)
            .map(|(_, body)| serde_json::to_string_pretty(&body.steps).unwrap());
        eprintln!("STEPS {}", steps.unwrap_or_default());
    }
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model(&text)));
    let shelve: BTreeSet<&str> = statuses
        .keys()
        .map(String::as_str)
        .filter(|id| id.starts_with("demo.shelf.Shelve/"))
        .collect();
    assert!(
        shelve.contains("demo.shelf.Shelve/outcome/unknown-book"),
        "unknown-book is synthesized: {shelve:?} refusals {refusals:#?}"
    );
    assert_eq!(
        not_passed(&statuses).len(),
        0,
        "every scenario passes: {statuses:#?}"
    );
}

/// `Shelve` whose updating default moves the book, beside a `wrong_state` refusal.
fn wrong_state_beside_upsert() -> String {
    let text = replace(
        MODEL,
        "    lifecycle: {initial: Shelved, states: [Shelved], terminal: [Shelved]}\n",
        "    lifecycle:\n      initial: Shelved\n      states: [Shelved, Withdrawn]\n      terminal: [Withdrawn]\n      transitions:\n        - {name: withdraw, from: [Shelved], to: Withdrawn}\n",
    );
    shelve_changed(&text, |shelve| {
        let shelve = replace(
            shelve,
            "      - name: replaced\n        updates: demo.shelf.Book\n",
            "      - name: replaced\n        moves: demo.shelf.Book.withdraw\n",
        );
        replace(
            &shelve,
            "      - name: added\n",
            "      - name: withdrawn\n        wrong_state: true\n        error: demo.shelf.UnknownBook\n      - name: added\n",
        )
    })
}

/// An absent book beside `wrong_state`: created on a decoy, refused by the row set on a match.
#[test]
fn adv_wrong_state_beside_upsert_absent_book() {
    let text = wrong_state_beside_upsert();
    let request = [("book", "absent-book"), ("curator", "the-curator")];
    let decoy = target(&text);
    open(&decoy, "decoy", "other-curator");
    force(&decoy, "added");
    let matching = target(&text);
    open(&matching, "matching", "the-curator");
    force(&matching, "added");
    assert_eq!(
        (
            answer(&decoy, "demo.shelf.Shelve", &request),
            answer(&matching, "demo.shelf.Shelve", &request)
        ),
        ("added".to_owned(), "refused".to_owned())
    );
    let synthesis = synthesize(&model(&text));
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model(&text)));
    assert_eq!(
        not_passed(&statuses).len(),
        0,
        "every scenario passes: {statuses:#?}"
    );
}

/// A creation that takes its identity from another input field does not claim the absent row the
/// default addresses through `book`.
#[test]
fn adv_creation_from_other_field_keeps_unknown_instance() {
    let text = shelve_changed(MODEL, |shelve| {
        let shelve = replace(
            shelve,
            "      - {name: curator, type: String}\n",
            "      - {name: curator, type: String}\n      - {name: copy, type: String}\n",
        );
        let shelve = replace(
            &shelve,
            "    outcomes:\n",
            "    outcomes:\n      - name: unknown-book\n        unknown_instance: true\n        error: demo.shelf.UnknownBook\n",
        );
        let (head, tail) = shelve.split_once("      - name: added\n").unwrap();
        let tail = replace(
            tail,
            "demo.shelf.BookShelved: {book: input.book}",
            "demo.shelf.BookShelved: {book: input.copy}",
        );
        format!("{head}      - name: added\n{tail}")
    });
    let target = target(&text);
    open(&target, "decoy", "other-curator");
    force(&target, "added");
    assert_eq!(
        answer(
            &target,
            "demo.shelf.Shelve",
            &[
                ("book", "absent-book"),
                ("curator", "the-curator"),
                ("copy", "new-copy")
            ]
        ),
        "unknown-book"
    );
}

/// An `exists: false` row set beside the upsert: refused with no library of the curator, the
/// absent book created once one is open.
#[test]
fn adv_empty_row_set_beside_upsert() {
    let text = shelve_changed(MODEL, |shelve| {
        replace(
            shelve,
            "          exists: true\n",
            "          exists: false\n",
        )
    });
    let request = [("book", "absent-book"), ("curator", "the-curator")];
    let none = target(&text);
    open(&none, "decoy", "other-curator");
    force(&none, "added");
    let matching = target(&text);
    open(&matching, "matching", "the-curator");
    force(&matching, "added");
    assert_eq!(
        (
            answer(&none, "demo.shelf.Shelve", &request),
            answer(&matching, "demo.shelf.Shelve", &request)
        ),
        ("refused".to_owned(), "added".to_owned())
    );
    let synthesis = synthesize(&model(&text));
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model(&text)));
    assert_eq!(
        not_passed(&statuses).len(),
        0,
        "every scenario passes: {statuses:#?}"
    );
}

/// The generated Go runtime gives the reference verdict on the synthesized upsert suite, for the
/// honest target and for the two faulty ones the unit names.
#[test]
fn adv_go_parity_on_upsert_suite_with_faulty_targets() {
    let suite = synthesize(&model(MODEL)).suite;
    let honest = support_go::assert_parity(
        "upsert-honest",
        &suite,
        Interpreted::for_model(model(MODEL)),
    );
    assert_eq!(support_go::not_passed(&honest), Vec::<&str>::new());

    let ignores = unguarded(MODEL);
    let verdicts = support_go::assert_parity(
        "upsert-ignores",
        &suite,
        Interpreted::for_model(model(&ignores)),
    );
    assert!(
        support_go::not_passed(&verdicts).contains(&"demo.shelf.Shelve/outcome/refused"),
        "{verdicts:?}"
    );
    let any = shelve_changed(MODEL, |shelve| {
        shelve.replace(
            "where: curator == input.curator",
            "where: {any: [curator == input.curator, curator != input.curator]}",
        )
    });
    let verdicts =
        support_go::assert_parity("upsert-any", &suite, Interpreted::for_model(model(&any)));
    assert!(
        support_go::not_passed(&verdicts).contains(&"demo.shelf.Shelve/outcome/replaced"),
        "{verdicts:?}"
    );
}

/// The input-guarded upsert without its row set passes its own synthesized suite: the failure of
/// `adv_input_guarded_upsert_suite_passes_interpreter` belongs to the row-set path.
#[test]
fn adv_input_guarded_upsert_suite_passes_interpreter_without_row_set() {
    let text = unguarded(&input_guarded_creation());
    let synthesis = synthesize(&model(&text));
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(&synthesis.suite),
    )
    .run_admitted(&admitted, &Interpreted::for_model(model(&text)))
    .into_report();
    let failing: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{}: {:?}", scenario.scenario, scenario.checks))
        .collect();
    assert_eq!(failing.len(), 0, "{failing:#?}");
}

/// An absent book with no external fact forced, beside a decoy library only: the row set passes
/// over, so the answer is the one the same upsert gives without its row set.
#[test]
fn adv_unforced_absent_book_matches_unguarded() {
    let request = [("book", "absent-book"), ("curator", "the-curator")];
    let mut answers = Vec::new();
    for text in [unguarded(MODEL), MODEL.to_owned()] {
        let target = target(&text);
        open(&target, "decoy", "other-curator");
        answers.push(answer(&target, "demo.shelf.Shelve", &request));
    }
    assert_eq!(
        answers[0], answers[1],
        "unguarded (left) and guarded (right)"
    );
}
