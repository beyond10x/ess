//! An authored act states the caller it is sent as (`caller:`, `ess-scenario/5`).
//!
//! A command may read an attribute of its caller (`{caller: account_id}`, source format `ess/16`).
//! A synthesized suite fills those values itself; an authored act has to say them, or the command
//! it sends reads an attribute nobody supplied. So an act names the values under `caller:`, each a
//! literal or the identity of an instance arranged with `setup:`, and authoring refuses an attribute
//! the act's actor does not declare and a required attribute the command reads that the act leaves
//! unstated, naming both.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Cause, Source};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ConformanceReport, ConformanceStatus};
use ess_conformance::{AdmittedSuite, ConformanceSuite, ScenarioStep, SuiteProvenance};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;
use ess_primitives::node::Node;

/// The consumer's shape, in neutral names: `OpenNote` records the caller's account on the note it
/// creates and in the event it publishes, and `Member`, the one actor granted it, declares that
/// attribute. `OpenAccount` creates an account whose identity a later act may name.
const MODEL: &str = include_str!("fixtures/authored-caller.yaml");

const ACCOUNT: &str = "3f1d5b7e-0000-4000-8000-000000000001";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(SpecSource::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// One document: `format`, then the arrangement, then one `OpenNote` act whose `actor:` and
/// `caller:` lines are `sender`, then a view assertion that the note carries `ACCOUNT`.
fn document(format: &str, arrange: &str, sender: &str) -> String {
    format!(
        "type: {format}\ndomain: ledger.notes\nscenario: a-note-carries-the-callers-account\n\
         summary: A note opened by a member carries the member's account.\n\
         arrange:\n  - instance: note\n    entity: ledger.notes.Note\n{arrange}\
         timeline:\n  - at: 2026-01-05T09:00:00Z\n    command: ledger.notes.OpenNote\n{sender}\
         \x20   input: {{title: first}}\n    outcome: opened\n    events:\n      - event: \
         ledger.notes.NoteOpened\n        payload: {{account_id: {ACCOUNT}}}\n    capture: \
         {{instance: note, event: ledger.notes.NoteOpened, field: note_id}}\n\
         assert:\n  - view: ledger.notes.Notes\n    contains:\n      {{note_id: {{$instance: \
         note}}, account_id: {ACCOUNT}, title: first}}\n"
    )
}

fn literal_sender() -> String {
    format!("    actor: ledger.notes.Member\n    caller: {{account_id: {ACCOUNT}}}\n")
}

fn authoring(text: &str) -> Authoring {
    compile_authored(&ir(), &[Source::new("note.yaml", text)])
}

fn causes(text: &str) -> Vec<(String, String)> {
    let authoring = authoring(text);
    assert!(
        authoring.scenarios.is_empty(),
        "a refused document produced a scenario"
    );
    authoring
        .refusals
        .iter()
        .map(|refusal| (refusal.code().to_string(), refusal.to_string()))
        .collect()
}

fn suite(authoring: Authoring) -> ConformanceSuite {
    assert!(authoring.is_complete(), "{:#?}", authoring.refusals);
    let ir = ir();
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(&ir));
    for (id, scenario) in authoring.scenarios {
        suite.insert(id, scenario).expect("one id");
    }
    suite.select_fresh_format_for(&ir);
    suite
}

fn run(suite: &ConformanceSuite) -> ConformanceReport {
    let admitted = AdmittedSuite::from_suite(suite).expect("admitted");
    ess_conformance::Runner::for_suite(suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir()))
        .into_report()
}

fn callers(suite: &ConformanceSuite) -> Vec<BTreeMap<String, Node>> {
    suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { caller, .. } => Some(caller.clone()),
            _ => None,
        })
        .collect()
}

fn account() -> Node {
    Node::Text(ACCOUNT.into())
}

#[test]
fn an_act_stating_a_literal_caller_sends_it_and_the_interpreted_target_passes() {
    let suite = suite(authoring(&document(
        "ess-scenario/5",
        "",
        &literal_sender(),
    )));
    assert_eq!(
        callers(&suite),
        [BTreeMap::from([("account_id".to_owned(), account())])]
    );
    let report = run(&suite);
    assert_eq!(report.status, ConformanceStatus::Passed, "{report:#?}");
}

#[test]
fn an_act_names_an_instance_arranged_with_setup_as_a_caller_value() {
    let arrange = format!(
        "  - instance: account\n    entity: ledger.notes.Account\n    setup:\n      identity: \
         {ACCOUNT}\n      fields: {{label: main}}\n      state: Active\n"
    );
    let sender = "    actor: ledger.notes.Member\n    caller: {account_id: {$instance: account}}\n";
    let suite = suite(authoring(&document("ess-scenario/5", &arrange, sender)));
    assert_eq!(
        callers(&suite),
        [BTreeMap::from([("account_id".to_owned(), account())])]
    );
    let report = run(&suite);
    assert_eq!(report.status, ConformanceStatus::Passed, "{report:#?}");
}

#[test]
fn a_caller_value_naming_a_captured_instance_is_refused_naming_the_attribute() {
    // A captured identity is minted by the target at run time, and a caller's values are fixed
    // when the suite is written: the suite has no step that resolves one into a caller.
    let text = document(
        "ess-scenario/5",
        "  - instance: account\n    entity: ledger.notes.Account\n",
        "    actor: ledger.notes.Member\n    caller: {account_id: {$instance: account}}\n",
    )
    .replace(
        "  - at: 2026-01-05T09:00:00Z\n    command: ledger.notes.OpenNote\n",
        "  - at: 2026-01-05T08:00:00Z\n    command: ledger.notes.OpenAccount\n    actor: \
         ledger.notes.Member\n    caller: {account_id: 3f1d5b7e-0000-4000-8000-000000000009}\n    \
         input: {label: main}\n    outcome: opened\n    events:\n      - event: \
         ledger.notes.AccountOpened\n    capture: {instance: account, event: \
         ledger.notes.AccountOpened, field: account_id}\n\
         \x20 - at: 2026-01-05T09:00:00Z\n    command: ledger.notes.OpenNote\n",
    );
    let refused = causes(&text);
    assert_eq!(refused.len(), 1, "{refused:#?}");
    assert_eq!(refused[0].0, "ESS-AUTHOR-015", "{refused:#?}");
    for named in ["`account_id`", "`account`", "ledger.notes.Member", "setup"] {
        assert!(refused[0].1.contains(named), "{named}: {refused:#?}");
    }
}

#[test]
fn an_attribute_the_actor_does_not_declare_is_refused_naming_it_and_the_actor() {
    let sender = format!(
        "    actor: ledger.notes.Member\n    caller: {{account_id: {ACCOUNT}, region: north}}\n"
    );
    let refused = causes(&document("ess-scenario/5", "", &sender));
    assert_eq!(refused.len(), 1, "{refused:#?}");
    assert_eq!(refused[0].0, "ESS-AUTHOR-013", "{refused:#?}");
    for named in [
        "`region`",
        "`ledger.notes.Member`",
        "`ledger.notes.OpenNote`",
    ] {
        assert!(refused[0].1.contains(named), "{named}: {refused:#?}");
    }
}

#[test]
fn a_read_attribute_the_act_does_not_state_is_refused_naming_it_and_the_command() {
    for sender in [
        "    actor: ledger.notes.Member\n",
        "    actor: ledger.notes.Member\n    caller: {}\n",
        "",
    ] {
        let refused = causes(&document("ess-scenario/5", "", sender));
        assert_eq!(refused.len(), 1, "{sender}: {refused:#?}");
        assert_eq!(refused[0].0, "ESS-AUTHOR-014", "{refused:#?}");
        for named in ["`account_id`", "`ledger.notes.OpenNote`"] {
            assert!(refused[0].1.contains(named), "{named}: {refused:#?}");
        }
    }
}

#[test]
fn a_caller_with_no_actor_is_refused() {
    let sender = format!("    caller: {{account_id: {ACCOUNT}}}\n");
    let refused = causes(&document("ess-scenario/5", "", &sender));
    assert!(
        refused
            .iter()
            .any(|(code, text)| code == "ESS-AUTHOR-013" && text.contains("`account_id`")),
        "{refused:#?}"
    );
}

/// The format consequence: `caller:` is new in `ess-scenario/5`. Every earlier version refuses it
/// by name rather than reading a document an older `ess` could not have read; an older reader
/// refuses `ess-scenario/5` by its header.
#[test]
fn caller_requires_scenario_5_and_every_earlier_version_refuses_it() {
    for format in [
        "ess-scenario/1",
        "ess-scenario/2",
        "ess-scenario/3",
        "ess-scenario/4",
    ] {
        let refused = causes(&document(format, "", &literal_sender()));
        assert_eq!(refused.len(), 1, "{format}: {refused:#?}");
        assert_eq!(refused[0].0, "ESS-AUTHOR-001", "{refused:#?}");
        assert!(
            refused[0].1.contains("type: ess-scenario/5"),
            "{refused:#?}"
        );
    }
}

/// A suite authored from a document with no `caller:` key, against a command that reads no caller,
/// keeps its bytes: `ess-scenario/5` and every older header compile it to the same suite.
#[test]
fn a_document_with_no_caller_key_keeps_its_bytes_under_every_version() {
    let text = |format: &str| {
        format!(
            "type: {format}\ndomain: ledger.notes\nscenario: an-account-opens\nsummary: An \
             account opens.\narrange:\n  - instance: account\n    entity: ledger.notes.Account\n\
             timeline:\n  - at: 2026-01-05T09:00:00Z\n    command: ledger.notes.OpenAccount\n    \
             input: {{label: main}}\n    outcome: opened\n    events:\n      - event: \
             ledger.notes.AccountOpened\n    capture: {{instance: account, event: \
             ledger.notes.AccountOpened, field: account_id}}\n"
        )
    };
    let bytes = |format: &str| {
        let suite = suite(authoring(&text(format)));
        assert_eq!(callers(&suite), [BTreeMap::new()]);
        suite.to_canonical_json().expect("serializes")
    };
    let newest = bytes("ess-scenario/5");
    assert!(!newest.contains("\"caller\""), "{newest}");
    for format in [
        "ess-scenario/1",
        "ess-scenario/2",
        "ess-scenario/3",
        "ess-scenario/4",
    ] {
        assert_eq!(bytes(format), newest, "{format}");
    }
}

/// A scenario that reaches a caller attribute nobody supplied — a suite whose command step carries
/// no caller — is `unsupported`, and the reason names the attribute and where to state it.
#[test]
fn a_command_reaching_an_unsupplied_caller_attribute_is_unsupported_naming_it() {
    let mut suite = suite(authoring(&document(
        "ess-scenario/5",
        "",
        &literal_sender(),
    )));
    for scenario in suite.scenarios.values_mut() {
        for step in &mut scenario.steps {
            if let ScenarioStep::ExecuteCommand { caller, .. } = step {
                caller.clear();
            }
        }
    }
    let report = run(&suite);
    assert_eq!(report.status, ConformanceStatus::Failed, "{report:#?}");
    let json = serde_json::to_string(&report).expect("serializes");
    for named in ["caller attribute `account_id`", "no caller", "`caller:`"] {
        assert!(json.contains(named), "{named}: {json}");
    }
    assert!(!json.contains("mint"), "{json}");
}

#[test]
fn the_typescript_runner_accepts_the_authored_caller_suite() {
    let suite = suite(authoring(&document(
        "ess-scenario/5",
        "",
        &literal_sender(),
    )));
    ess_conformance::ts::emit(&suite).expect("the TypeScript runner sends authored callers");
}

#[test]
fn every_new_refusal_is_one_the_catalogue_already_names() {
    // Reusing `ESS-AUTHOR-013`, `-014` and `-015` adds no diagnostic code.
    for code in [13, 14, 15] {
        assert!(Cause::CATALOGUE.iter().any(|entry| entry.key == code));
    }
}
