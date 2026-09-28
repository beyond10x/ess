//! Adversarial cases against declared fault injection, second pass (story
//! `declared-fault-injection`).
//!
//! The checker groups one request's operations by `retry_of` and orders a `replays:` answer after an
//! operation of its request that answered the origin branch or never answered. An operation that
//! never answered is still free, in the search, to take no branch — or the origin branch — whatever
//! its request's other operations answered. Two histories no correct target writes follow from
//! that, each with a witness the model can see:
//!
//! * a replay after an original that never answered, then a new request creating the very identity
//!   the replay says was already created;
//! * an original that never answered but whose effect a read shows, and a retry that answered the
//!   origin branch again: one request applied twice.
//!
//! Both histories are the ones the Rust recorder writes when the command declares `replays:` and
//! another `external:` branch (so the original may be injected `Delayed` or `Unanswered`); the retry
//! fixture here gains that branch through a textual rewrite.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::{self, Completion, Verdict};
use ess_conformance::linearize;
use ess_conformance::record::{Atomic, Call, Subject};
use ess_conformance::reference::Retained;
use ess_conformance::scenario::SuiteProvenance;
use ess_conformance::sessions::{self, Act};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const RETRY: &str = "crates/verify/ess-conformance/tests/fixtures/explore-retry";

fn model_rewritten(path: &str, rewrite: impl Fn(&str) -> String) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(path)
        .canonicalize()
        .unwrap_or_else(|error| panic!("`{path}` exists: {error}"));
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let entry = entry.expect("an entry").path();
            if entry.is_dir() {
                pending.push(entry);
            } else if entry.extension().is_some_and(|it| it == "yaml") {
                found.push(entry);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for file in found {
        let label = file
            .strip_prefix(&base)
            .expect("inside")
            .display()
            .to_string();
        let text = rewrite(&std::fs::read_to_string(&file).expect("readable"));
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("`{path}` validates:\n{errors}"));
    compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("`{path}` resolves:\n{diagnostics}"))
}

/// The retry fixture with one more `external:` branch on `Seed`, so an injecting recorder may delay
/// or drop its answer.
fn with_external(text: &str) -> String {
    let replayed = "        replays: seeded\n";
    let commands = "commands:\n";
    assert!(
        text.contains(replayed),
        "the fixture declares `replays: seeded`"
    );
    assert!(text.contains(commands));
    text.replacen(
        commands,
        "errors:\n  - name: retry.core.Unavailable\n    summary: The store was unavailable.\ncommands:\n",
        1,
    )
    .replace(
        replayed,
        "        replays: seeded\n      - name: refused\n        external: the store is unavailable\n        error: retry.core.Unavailable\n",
    )
}

/// [`with_external`], and the record's identity supplied by the caller rather than generated, so a
/// call that never answered still names its subject.
fn with_external_and_supplied_identity(text: &str) -> String {
    let input = "    input: [{name: document, type: String}]\n";
    let generated = "            record_id: {generated: true}\n";
    assert!(text.contains(input) && text.contains(generated));
    with_external(
        &text
            .replace(
                input,
                "    input: [{name: record_id, type: Uuid}, {name: document, type: String}]\n",
            )
            .replace(generated, "            record_id: input.record_id\n"),
    )
}

fn id(n: u8) -> String {
    format!("00000000-0000-4000-8000-0000000000{n:02}")
}

const X: &str = "00000000-0000-4000-8000-0000000000a1";
const Y: &str = "00000000-0000-4000-8000-0000000000b2";

fn returned(
    n: u8,
    client: u64,
    subject: &str,
    at: (u64, u64),
    outcome: &str,
    retry_of: Option<u8>,
) -> String {
    let retry = retry_of.map_or_else(String::new, |of| format!(r#","retry_of":"{}""#, id(of)));
    format!(
        r#"{{"operation_id":"{}","client":{client},"command":"retry.core.Seed","subject_key":"{subject}","invoked_at":{},"returned_at":{},"completion":"Returned","outcome":"{outcome}"{retry}}}"#,
        id(n),
        at.0,
        at.1
    )
}

fn unanswered(n: u8, client: u64, subject: &str, invoked: u64) -> String {
    format!(
        r#"{{"operation_id":"{}","client":{client},"command":"retry.core.Seed","subject_key":"{subject}","invoked_at":{invoked},"completion":"Indeterminate"}}"#,
        id(n)
    )
}

fn read(n: u8, client: u64, at: (u64, u64), rows: &[&str]) -> String {
    let rows: Vec<String> = rows.iter().map(|row| format!(r#""{row}""#)).collect();
    format!(
        r#"{{"operation_id":"{}","client":{client},"command":"retry.core.Records","subject_key":"","invoked_at":{},"returned_at":{},"completion":"Returned","outcome":"read","rows":[{}]}}"#,
        id(n),
        at.0,
        at.1,
        rows.join(",")
    )
}

fn check(model: &EssIr, operations: &[String]) -> linearize::Checked {
    let digest = SuiteProvenance::of(model).spec_digest;
    let bytes = format!(
        r#"{{"format":"ess-history/1","history_id":"00000000-0000-4000-8000-000000000001","spec_digest":"{digest}","seed":1,"clients":2,"operations":[{}]}}"#,
        operations.join(",")
    );
    let history = history::read(bytes.as_bytes(), &digest).expect("admitted");
    linearize::check(model, &history, linearize::DEFAULT_BUDGET).expect("checked")
}

#[test]
fn a_replay_after_an_unanswered_original_does_not_let_a_new_request_create_what_it_replayed() {
    // The recorder's `Unanswered` injection: client 0's `Seed` of X is dropped, its retry is
    // answered `replayed`, and then client 1, a new request, creates X. A correct target answers
    // `replayed` only for a request it applied, so X was held when client 1 asked and `seeded`
    // is impossible for it. A target that retains a request when it arrives and applies it only at
    // the answer — the answer never came — writes exactly this: a lost write acknowledged as a
    // replay. The checker reads the unanswered original as never having happened *and* as having
    // retained what its retry replays.
    let model = model_rewritten(RETRY, with_external_and_supplied_identity);
    // Control: without the retry the original may simply never have happened.
    let control = check(
        &model,
        &[
            unanswered(1, 0, X, 1),
            returned(3, 1, X, (4, 5), "seeded", None),
        ],
    );
    assert_eq!(control.verdict, Verdict::Linearizable, "{control:?}");
    let checked = check(
        &model,
        &[
            unanswered(1, 0, X, 1),
            returned(2, 0, X, (2, 3), "replayed", Some(1)),
            returned(3, 1, X, (4, 5), "seeded", None),
        ],
    );
    assert_eq!(
        checked.verdict,
        Verdict::Violation,
        "a replay proves its request was applied, yet a later request created the same identity: \
         {checked:?}"
    );
}

#[test]
fn an_original_that_never_answered_but_took_effect_and_a_retry_answering_the_origin_branch_is_applied_twice(
) {
    // The recorder's `Delayed` injection: client 0's `Seed` executed and created X, its answer
    // came after the client stopped waiting (written `Indeterminate`, subject X from the late
    // answer). Its retry arrived unretained and created Y — `RetryCreatesSecondEntity` with the
    // original delayed rather than merely overlapping. A later read shows both. One request, two
    // records.
    let model = model_rewritten(RETRY, with_external);
    // Control: two requests, not one, create two records — nothing forbids that.
    let control = check(
        &model,
        &[
            unanswered(1, 0, X, 1),
            returned(2, 0, Y, (2, 3), "seeded", None),
            read(3, 1, (6, 7), &[X, Y]),
        ],
    );
    assert_eq!(control.verdict, Verdict::Linearizable, "{control:?}");
    let checked = check(
        &model,
        &[
            unanswered(1, 0, X, 1),
            returned(2, 0, Y, (2, 3), "seeded", Some(1)),
            read(3, 1, (6, 7), &[X, Y]),
        ],
    );
    assert_eq!(
        checked.verdict,
        Verdict::Violation,
        "the read shows the unanswered original took `seeded`, and its retry answered `seeded` \
         too: {checked:?}"
    );
}

#[test]
fn a_retry_of_a_retry_is_one_request() {
    // Retry chains: the explorers never write one, a hand-built history may.
    let model = model_rewritten(RETRY, ToOwned::to_owned);
    let replayed = check(
        &model,
        &[
            returned(1, 0, X, (1, 2), "seeded", None),
            returned(2, 0, "", (3, 4), "replayed", Some(1)),
            returned(3, 0, "", (5, 6), "replayed", Some(2)),
        ],
    );
    assert_eq!(replayed.verdict, Verdict::Linearizable, "{replayed:?}");
    let twice = check(
        &model,
        &[
            returned(1, 0, X, (1, 2), "seeded", None),
            returned(2, 0, "", (3, 4), "replayed", Some(1)),
            returned(3, 0, Y, (5, 6), "seeded", Some(2)),
        ],
    );
    assert_eq!(twice.verdict, Verdict::Violation, "{twice:?}");
    // A replay whose chain root never answered the origin branch.
    let nothing = check(
        &model,
        &[
            returned(1, 0, "", (1, 2), "replayed", None),
            returned(2, 0, "", (3, 4), "replayed", Some(1)),
        ],
    );
    assert_eq!(nothing.verdict, Verdict::Violation, "{nothing:?}");
}

#[test]
fn a_replay_of_a_generated_identity_is_searched_beside_the_record_it_replays() {
    // The retry of a creation with a generated identity is written with no subject: it is placed
    // in its original's partition. A replay whose original created X, answered while a second
    // request of another client is creating Y, stays linearizable, and one before its original
    // answered anything is not.
    let model = model_rewritten(RETRY, ToOwned::to_owned);
    let fine = check(
        &model,
        &[
            returned(1, 0, X, (1, 3), "seeded", None),
            returned(2, 1, Y, (2, 5), "seeded", None),
            returned(3, 0, "", (4, 6), "replayed", Some(1)),
        ],
    );
    assert_eq!(fine.verdict, Verdict::Linearizable, "{fine:?}");
    let early = check(
        &model,
        &[
            returned(1, 0, X, (3, 6), "seeded", None),
            returned(2, 1, Y, (1, 2), "seeded", None),
            returned(3, 0, Y, (4, 5), "replayed", Some(1)),
        ],
    );
    // Replay 3 is written with Y's subject but belongs to request 1; it returned before 1 could
    // have been ordered only if 1 linearized in [3, 5] — which is allowed. So this is linearizable.
    assert_eq!(early.verdict, Verdict::Linearizable, "{early:?}");
}

#[test]
fn retry_creates_second_entity_is_caught_when_the_command_also_declares_another_external_branch() {
    // The unit's own recorder and faulty target, on the retry fixture with one more declared
    // `external:` branch — the shape of any command with an idempotent replay and an external
    // refusal. Two clients each seed one record and then read `Records`. A correct target holds at
    // most two records; a read listing three is one request applied twice, and every such history
    // must be a violation. The reference, recorded the same way, stays linearizable.
    let model = model_rewritten(RETRY, with_external);
    let workload = || {
        let client = |document: &str| {
            vec![
                Act::Call(Call::new(
                    "retry.core.Seed",
                    BTreeMap::from([("document".to_owned(), Node::Text(document.to_owned()))]),
                    Subject::Creates,
                )),
                Act::Read("retry.core.Records".to_owned()),
            ]
        };
        sessions::Workload {
            prefix: Vec::new(),
            clients: vec![client("first"), client("second")],
        }
    };
    let mut missed = Vec::new();
    let mut applied_twice = 0;
    for seed in 0..24 {
        let reference = Retained::new();
        let control =
            sessions::record_injected(&model, &Atomic(&reference), &reference, &workload(), seed)
                .expect("recorded");
        assert_eq!(
            linearize::check(&model, &control.history, linearize::DEFAULT_BUDGET)
                .expect("checked")
                .verdict,
            Verdict::Linearizable,
            "seed {seed}: the reference with the same injections"
        );
        let target = faulty::retry(Fault::RetryCreatesSecondEntity);
        let recorded = sessions::record_injected(&model, &target, &target, &workload(), seed)
            .expect("recorded");
        let shown: BTreeSet<&str> = recorded
            .history
            .operations
            .iter()
            .filter(|operation| operation.completion == Completion::Returned)
            .filter_map(|operation| operation.rows.as_ref())
            .flatten()
            .map(String::as_str)
            .collect();
        if shown.len() < 3 {
            continue;
        }
        applied_twice += 1;
        let checked = linearize::check(&model, &recorded.history, linearize::DEFAULT_BUDGET)
            .expect("checked");
        if checked.verdict == Verdict::Linearizable {
            missed.push((seed, recorded.injected.delayed.clone(), shown.len()));
        }
    }
    println!("histories showing a request applied twice: {applied_twice}; judged linearizable: {missed:?}");
    assert!(applied_twice > 0, "no seed applied a request twice");
    assert!(
        missed.is_empty(),
        "a read shows three records from two requests, and the history was judged linearizable: \
         {missed:?}"
    );
}
