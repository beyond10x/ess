//! A row-set guard whose selector reads only the input, on a command whose default addresses an
//! existing record through the input (beyond10x/ess#462).
//!
//! `tests/fixtures/row-set-upsert.yaml`: `Shelve` is an upsert, refused while a library of the
//! input's curator is open, and otherwise creating the book (`added`, external) or replacing it
//! (`replaced`, the default). `Restock` is the same guard on an update-only command with an
//! unknown-instance branch, and `Unshelve` on a deleting default.
//!
//! Synthesis decides these row sets from `ess/23` (story:feature-request-429, the identity an
//! instance input carries is the literal the selector compares with); below it the three commands
//! keep the 0.53.0 refusals and suite bytes. The reference interpreter takes an absent book on the
//! upsert as the creation's, not as an unknown instance, after the row set answers
//! (`docs/design/filtered-related-reads.md`, "Subject borrowing and precedence").

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::OutcomeRef;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig, ScenarioStep,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use sha2::{Digest, Sha256};

const MODEL: &str = include_str!("fixtures/row-set-upsert.yaml");

/// SHA-256 and byte length of the suite `ess 0.53.0` writes for the fixture at `ess/22`
/// (`ess verify conform synthesize --path <model> --target ir --out <suite>`), which is
/// `ConformanceSuite::to_canonical_json()` after `select_fresh_format_for`.
const SUITE_053_DIGEST: &str = "a9ee6ce89a211ddac0fb15ceab947f056196022f536c1caa33a5dae928923b50";
const SUITE_053_LENGTH: usize = 8_113;

const SHELVE: [&str; 3] = [
    "demo.shelf.Shelve/outcome/added",
    "demo.shelf.Shelve/outcome/refused",
    "demo.shelf.Shelve/outcome/replaced",
];
const RESTOCK_UNSHELVE: [&str; 4] = [
    "demo.shelf.Restock/outcome/refused",
    "demo.shelf.Restock/outcome/restocked",
    "demo.shelf.Unshelve/outcome/refused",
    "demo.shelf.Unshelve/outcome/removed",
];

fn at_ess22(text: &str) -> String {
    let changed = text.replacen("format: ess/23", "format: ess/22", 1);
    assert_ne!(changed, text, "the fixture declares ess/23");
    changed
}

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let specification = Specification::assemble([(Source::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&model(text))
}

fn ids(suite: &ConformanceSuite) -> BTreeSet<String> {
    suite.scenarios.keys().map(ToString::to_string).collect()
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn steps<'s>(suite: &'s ConformanceSuite, id: &str) -> &'s [ScenarioStep] {
    &suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .1
        .steps
}

/// The literal `field` of every `command` the scenario sends, in order.
fn sent(suite: &ConformanceSuite, id: &str, command: &str, field: &str) -> Vec<Node> {
    steps(suite, id)
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(
                input
                    .get(field)
                    .and_then(|value| value.as_literal())
                    .cloned()
                    .unwrap_or_else(|| panic!("{id}: `{command}` sends a literal `{field}`")),
            ),
            _ => None,
        })
        .collect()
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

fn failed(statuses: &BTreeMap<String, Status>) -> BTreeSet<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

fn target(text: &str) -> Interpreted {
    let target = Interpreted::for_model(model(text));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.shelf/authored/row-set-upsert".parse().unwrap(),
            CorrelationId::new("row-set-upsert").unwrap(),
        ))
        .unwrap();
    target
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn request(command: &str, input: &[(&str, &str)]) -> SemanticCommandRequest {
    SemanticCommandRequest {
        command: command.parse().unwrap(),
        actor: None,
        caller: None,
        input: input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), text(value)))
            .collect(),
        correlation: CorrelationId::new("row-set-upsert").unwrap(),
    }
}

fn answer(target: &dyn ConformanceTarget, command: &str, input: &[(&str, &str)]) -> String {
    target
        .execute_command(request(command, input))
        .unwrap_or_else(|error| panic!("{command} answered: {error}"))
        .outcome
        .map_or_else(|| "<none>".to_owned(), |taken| taken.outcome.to_string())
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

fn books(target: &dyn ConformanceTarget) -> Vec<BTreeMap<String, Node>> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.shelf.Books".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: ess_primitives::consistency::QueryConsistency::Current,
            correlation: CorrelationId::new("row-set-upsert").unwrap(),
            deadline: Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(0)),
        })
        .expect("the view answers")
        .rows
}

// ---- synthesis --------------------------------------------------------------------------------

#[test]
fn row_set_upsert_synthesizes_every_branch() {
    let synthesis = synthesis(MODEL);
    let shelve_refusals: Vec<String> = refusals(&synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains("demo.shelf.Shelve"))
        .collect();
    assert_eq!(
        shelve_refusals.len(),
        0,
        "no Shelve branch is refused: {shelve_refusals:#?}"
    );
    let ids = ids(&synthesis.suite);
    for id in SHELVE {
        assert!(ids.contains(id), "{id} is synthesized: {ids:#?}");
    }

    // `refused`: a library of another curator beside one of the input's curator.
    let suite = &synthesis.suite;
    let refused = "demo.shelf.Shelve/outcome/refused";
    let input = sent(suite, refused, "demo.shelf.Shelve", "curator");
    let input = input.last().expect("the scenario shelves a book");
    let curators = sent(suite, refused, "demo.shelf.OpenLibrary", "curator");
    assert!(
        curators.contains(input),
        "{refused} arranges a library of the input's curator {input:?}: {curators:?}"
    );
    assert!(
        curators.iter().any(|curator| curator != input),
        "{refused} arranges a decoy library of another curator: {curators:?}"
    );

    // `replaced`: the decoy only.
    let replaced = "demo.shelf.Shelve/outcome/replaced";
    let input = sent(suite, replaced, "demo.shelf.Shelve", "curator");
    let input = input.last().expect("the scenario shelves a book");
    let curators = sent(suite, replaced, "demo.shelf.OpenLibrary", "curator");
    assert!(
        !curators.is_empty() && curators.iter().all(|curator| curator != input),
        "{replaced} arranges only decoy libraries beside the input's curator {input:?}: {curators:?}"
    );
}

#[test]
fn row_set_update_and_delete_defaults_synthesize() {
    let synthesis = synthesis(MODEL);
    let refused = refusals(&synthesis);
    assert_eq!(refused.len(), 0, "nothing is refused: {refused:#?}");
    let ids = ids(&synthesis.suite);
    for id in RESTOCK_UNSHELVE {
        assert!(ids.contains(id), "{id} is synthesized: {ids:#?}");
    }
}

#[test]
fn row_set_input_selector_below_ess23_unchanged() {
    let ir = model(&at_ess22(MODEL));
    let mut synthesis = synthesize(&ir);
    let refused = refusals(&synthesis);
    for command in ["Shelve", "Restock", "Unshelve"] {
        let accepting = match command {
            "Shelve" => "replaced",
            "Restock" => "restocked",
            _ => "removed",
        };
        let own = refused
            .iter()
            .find(|refusal| refusal.contains(&format!("`demo.shelf.{command}/outcome/refused`")))
            .unwrap_or_else(|| panic!("{command}/refused stays refused: {refused:#?}"));
        assert!(
            own.contains("ESS-SYNTH-001") && own.contains("its own row set is Unknown"),
            "{own}"
        );
        let default = refused
            .iter()
            .find(|refusal| {
                refusal.contains(&format!("`demo.shelf.{command}/outcome/{accepting}`"))
            })
            .unwrap_or_else(|| panic!("{command}/{accepting} stays refused: {refused:#?}"));
        assert!(
            default.contains("ESS-SYNTH-001") && default.contains("its row set is Unknown"),
            "{default}"
        );
    }
    assert_eq!(refused.len(), 6, "{refused:#?}");

    synthesis.suite.select_fresh_format_for(&ir);
    let json = synthesis
        .suite
        .to_canonical_json()
        .unwrap_or_else(|error| panic!("{error}"));
    let digest = Sha256::digest(json.as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").expect("writing to a String");
            hex
        });
    assert_eq!(
        (digest.as_str(), json.len()),
        (SUITE_053_DIGEST, SUITE_053_LENGTH),
        "the ess/22 suite keeps the bytes ess 0.53.0 wrote"
    );
}

// ---- the reference interpreter ---------------------------------------------------------------

#[test]
fn interpreter_creates_absent_upsert_row_beside_row_set() {
    let suite = synthesis(MODEL).suite;
    let statuses = run(&suite, &Interpreted::for_model(model(MODEL)));
    let upsert: Vec<&str> = statuses
        .keys()
        .map(String::as_str)
        .filter(|id| {
            id.starts_with("demo.shelf.Shelve/") || id.starts_with("demo.shelf.OpenLibrary/")
        })
        .collect();
    assert_eq!(upsert.len(), 4, "{statuses:#?}");
    assert_eq!(
        failed(&statuses).len(),
        0,
        "every ess/23 scenario passes: {statuses:#?}"
    );

    let legacy = at_ess22(MODEL);
    let suite = synthesis(&legacy).suite;
    let statuses = run(&suite, &Interpreted::for_model(model(&legacy)));
    assert_eq!(
        statuses.get("demo.shelf.Shelve/outcome/added"),
        Some(&Status::Passed),
        "{statuses:#?}"
    );
    assert_eq!(
        failed(&statuses).len(),
        0,
        "every ess/22 scenario passes: {statuses:#?}"
    );
}

#[test]
fn interpreter_row_set_refusal_precedes_upsert_creation() {
    let target = target(MODEL);
    open(&target, "decoy", "other-curator");
    open(&target, "matching", "the-curator");
    assert_eq!(
        answer(
            &target,
            "demo.shelf.Shelve",
            &[("book", "absent-book"), ("curator", "the-curator")]
        ),
        "refused"
    );
    assert_eq!(books(&target).len(), 0, "a refusal creates no book");

    // Even where the creation is forced next, the row set answers first.
    target
        .configure_external_outcome(ExternalOutcomeControl {
            force: OutcomeRef::new(
                "demo.shelf.Shelve".parse().unwrap(),
                "added".parse().unwrap(),
            ),
            correlation: CorrelationId::new("row-set-upsert").unwrap(),
        })
        .unwrap();
    assert_eq!(
        answer(
            &target,
            "demo.shelf.Shelve",
            &[("book", "absent-book"), ("curator", "the-curator")]
        ),
        "refused",
        "a matching library refuses a forced creation"
    );
    assert_eq!(books(&target).len(), 0, "a refusal creates no book");

    // With only the decoy open, the forced creation stores the book.
    let target = self::target(MODEL);
    open(&target, "decoy", "other-curator");
    target
        .configure_external_outcome(ExternalOutcomeControl {
            force: OutcomeRef::new(
                "demo.shelf.Shelve".parse().unwrap(),
                "added".parse().unwrap(),
            ),
            correlation: CorrelationId::new("row-set-upsert").unwrap(),
        })
        .unwrap();
    assert_eq!(
        answer(
            &target,
            "demo.shelf.Shelve",
            &[("book", "absent-book"), ("curator", "the-curator")]
        ),
        "added"
    );
    assert_eq!(books(&target).len(), 1, "the creation stores the book");
}

#[test]
fn interpreter_update_without_creation_keeps_unknown_instance() {
    let target = target(MODEL);
    open(&target, "decoy", "other-curator");
    open(&target, "matching", "the-curator");
    for curator in ["the-curator", "other-curator"] {
        assert_eq!(
            answer(
                &target,
                "demo.shelf.Restock",
                &[("book", "absent-book"), ("curator", curator)]
            ),
            "unknown-book",
            "an absent book with {curator} is an unknown instance"
        );
    }
    assert_eq!(books(&target).len(), 0);
}

// ---- faulty targets ---------------------------------------------------------------------------

/// The fixture with `Shelve`'s own outcomes changed by `change`, every other command untouched.
fn shelve_changed(change: impl Fn(&str) -> String) -> String {
    let (head, tail) = MODEL
        .split_once("  - name: demo.shelf.Shelve\n")
        .expect("the Shelve command");
    let (shelve, rest) = tail
        .split_once("  - name: demo.shelf.Restock\n")
        .expect("the next command");
    let changed = change(shelve);
    assert_ne!(changed, shelve, "the fault changes Shelve");
    format!("{head}  - name: demo.shelf.Shelve\n{changed}  - name: demo.shelf.Restock\n{rest}")
}

#[test]
fn upsert_suite_catches_faulty_targets() {
    let suite = synthesis(MODEL).suite;

    // Ignores the selector: no library ever refuses the shelving.
    let ignores = shelve_changed(|shelve| {
        let (head, tail) = shelve.split_once("      - name: refused\n").unwrap();
        let (_, rest) = tail.split_once("      - name: added\n").unwrap();
        format!("{head}      - name: added\n{rest}")
    });
    let statuses = run(&suite, &Interpreted::for_model(model(&ignores)));
    assert!(
        failed(&statuses).contains("demo.shelf.Shelve/outcome/refused"),
        "a target ignoring the selector fails `refused`: {statuses:#?}"
    );

    // Refuses whenever any library is open, whatever its curator.
    let any = shelve_changed(|shelve| {
        shelve.replace(
            "where: curator == input.curator",
            "where: {any: [curator == input.curator, curator != input.curator]}",
        )
    });
    let statuses = run(&suite, &Interpreted::for_model(model(&any)));
    assert!(
        failed(&statuses).contains("demo.shelf.Shelve/outcome/replaced"),
        "a target refusing on any library fails `replaced`: {statuses:#?}"
    );

    // The honest target.
    let statuses = run(&suite, &Interpreted::for_model(model(MODEL)));
    for id in SHELVE {
        assert_eq!(statuses.get(id), Some(&Status::Passed), "{statuses:#?}");
    }
}

// ---- the design note --------------------------------------------------------------------------

#[test]
fn upsert_precedence_sentence_in_design_note() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../docs/design/filtered-related-reads.md"
    );
    let page = std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{path}: {error}"));
    let page = page.split_whitespace().collect::<Vec<_>>().join(" ");
    let (_, section) = page
        .split_once("### Subject borrowing and precedence")
        .expect("the precedence section");
    let section = section.split(" ### ").next().unwrap_or(section);
    assert!(
        section.contains(
            "An absent addressed row that a creation branch of the same entity takes from the \
             same input identity field is that creation's, not an unknown instance"
        ),
        "the precedence section states the upsert rule"
    );
}
