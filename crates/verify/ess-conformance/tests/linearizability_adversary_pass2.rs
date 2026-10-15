//! Adversarial cases, second pass, for `story:linearizability-checker-over-the-interpreter`.
//!
//! Aimed at the pass-1 corrections — `Generated::Recorded` in the interpreter and `relabel` after
//! `shrink` — and at what pass 1 did not reach: identities a history reuses or never created, the
//! exact budget edge, and shrinking races recorded beside other subjects and other calls.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::{self, Completion, History, ReturnBound, Verdict};
use ess_conformance::interpret::Interpreted;
use ess_conformance::linearize::{self, DEFAULT_BUDGET};
use ess_conformance::record::{self, Atomic, Call, Subject, Workload};
use ess_conformance::reference::Billing;
use ess_conformance::scenario::SuiteProvenance;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const REGISTER: &str = include_str!("fixtures/register/register.yaml");

fn register_model() -> EssIr {
    let raw = RawSpecFile::parse(REGISTER).expect("the model parses");
    let specification = Specification::assemble([(Source::new("register.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the model resolves:\n{diagnostics}"))
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn billing_model() -> EssIr {
    let base = root()
        .join("examples/billing")
        .canonicalize()
        .expect("the billing example exists");
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("billing validates");
    compile(&specification, &sources).expect("billing resolves")
}

fn through_the_reader(ir: &EssIr, recorded: &History) -> History {
    let bytes = serde_json::to_vec(recorded).expect("a history serializes");
    history::read(&bytes, &SuiteProvenance::of(ir).spec_digest)
        .unwrap_or_else(|refusal| panic!("the history is admitted: {refusal}"))
}

fn verdict(ir: &EssIr, history: &History, budget: u64) -> Verdict {
    linearize::check(ir, history, budget)
        .unwrap_or_else(|refusal| panic!("the history is checked: {refusal:?}"))
        .verdict
}

/// Two registers created one after the other by the model's own interpreter, then written.
fn two_registers() -> (EssIr, History) {
    let model = register_model();
    let target = Interpreted::for_model(model.clone());
    let workload = Workload {
        prefix: vec![
            Call::new(
                "register.cell.CreateRegister",
                BTreeMap::new(),
                Subject::Creates,
            ),
            Call::new(
                "register.cell.CreateRegister",
                BTreeMap::new(),
                Subject::Creates,
            ),
            Call::new(
                "register.cell.WriteOne",
                BTreeMap::new(),
                Subject::Created(1),
            ),
        ],
        clients: Vec::new(),
    };
    let recorded = record::record(&model, &Atomic(&target), &workload, 0).expect("recorded");
    let history = through_the_reader(&model, &recorded);
    assert_ne!(
        history.operations[0].subject_key, history.operations[1].subject_key,
        "the interpreter mints two identities: {history:#?}"
    );
    assert_eq!(
        verdict(&model, &history, DEFAULT_BUDGET),
        Verdict::Linearizable,
        "the history as recorded is the model's own"
    );
    (model, history)
}

// ---- `Generated::Recorded`: identities a history reuses or never created ------------------------

#[test]
fn two_creations_that_both_published_one_identity_are_a_violation() {
    // `Recorded` uses a given slot as `Given` does, so an identity the store holds is not created
    // again. An implementation that hands out one identity twice is told apart from one that does
    // not.
    let (model, mut history) = two_registers();
    let first = history.operations[0].subject_key.clone();
    history.operations[1].subject_key.clone_from(&first);
    history.operations[2].subject_key = first;
    let history = through_the_reader(&model, &history);
    assert_eq!(
        verdict(&model, &history, DEFAULT_BUDGET),
        Verdict::Violation
    );
}

#[test]
fn an_unanswered_creation_of_a_held_identity_may_never_have_happened() {
    let (model, mut history) = two_registers();
    let first = history.operations[0].subject_key.clone();
    history.operations[1].subject_key.clone_from(&first);
    history.operations[1].completion = Completion::Indeterminate;
    history.operations[1].returned_at = None;
    history.operations[1].outcome = None;
    history.operations[2].subject_key = first;
    let history = through_the_reader(&model, &history);
    assert_eq!(
        verdict(&model, &history, DEFAULT_BUDGET),
        Verdict::Linearizable
    );
}

#[test]
fn a_write_to_an_identity_no_creation_published_is_a_violation() {
    // An absent slot is minted under `Recorded`; a write naming an identity nothing created must not
    // be explained by an instance the checker minted for it.
    let (model, mut history) = two_registers();
    history.operations[2].subject_key = "00000000-0000-4000-8000-0000000fffff".to_owned();
    let history = through_the_reader(&model, &history);
    assert_eq!(
        verdict(&model, &history, DEFAULT_BUDGET),
        Verdict::Violation
    );
}

// ---- the budget edge -----------------------------------------------------------------------------

#[test]
fn the_budget_a_search_spent_is_exactly_enough_and_one_less_is_unknown() {
    let model = billing_model();
    for seed in 0..6 {
        let workload = faulty::lost_update_workload();
        let faulted = record::record(&model, &faulty::billing(Fault::LostUpdate), &workload, seed)
            .expect("recorded");
        let reference = Billing::new();
        let unfaulted =
            record::record(&model, &Atomic(&reference), &workload, seed).expect("recorded");
        for history in [faulted, unfaulted] {
            let history = through_the_reader(&model, &history);
            let full = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
            assert_ne!(full.verdict, Verdict::Unknown, "seed {seed}");
            let exact = linearize::check(&model, &history, full.steps).expect("checked");
            assert_eq!(
                (exact.verdict, exact.steps),
                (full.verdict, full.steps),
                "seed {seed}: the steps a search reports spending are enough for it"
            );
            let short = linearize::check(&model, &history, full.steps - 1).expect("checked");
            assert_eq!(short.verdict, Verdict::Unknown, "seed {seed}");
        }
    }
}

// ---- shrinking races recorded beside other subjects and other calls ------------------------------

/// Client 0's prefix creates and issues `invoices` invoices; each list is one client's payments,
/// naming invoices by their index.
fn payments(invoices: usize, clients: &[&[usize]]) -> Workload {
    let template = faulty::lost_update_workload();
    let pay = template.clients[0][0].clone();
    let mut prefix = Vec::new();
    for invoice in 0..invoices {
        prefix.push(template.prefix[0].clone());
        let mut issue = template.prefix[1].clone();
        issue.subject = Subject::Created(invoice * 2);
        prefix.push(issue);
    }
    Workload {
        prefix,
        clients: clients
            .iter()
            .map(|calls| {
                calls
                    .iter()
                    .map(|&invoice| {
                        let mut call = pay.clone();
                        call.subject = Subject::Created(invoice * 2);
                        call
                    })
                    .collect()
            })
            .collect(),
    }
}

/// `true` when no client of `history` has two calls in flight at once.
fn every_client_is_sequential(history: &History) -> bool {
    let mut by_client: BTreeMap<u64, Vec<(u64, ReturnBound)>> = BTreeMap::new();
    for operation in &history.operations {
        by_client
            .entry(operation.client)
            .or_default()
            .push((operation.invoked_at, operation.return_bound()));
    }
    by_client.values_mut().all(|calls| {
        calls.sort();
        calls
            .windows(2)
            .all(|pair| pair[0].1 < ReturnBound::At(pair[1].0))
    })
}

#[test]
fn every_lost_update_race_shrinks_to_two_clients_and_four_operations_whatever_else_was_recorded() {
    let model = billing_model();
    let workloads: [(&str, Workload); 4] = [
        ("two clients paying twice", payments(1, &[&[0, 0], &[0, 0]])),
        ("two invoices, both raced", payments(2, &[&[0, 1], &[0, 1]])),
        (
            "two invoices, three clients",
            payments(2, &[&[0], &[1], &[0, 1]]),
        ),
        (
            "three clients on one invoice",
            payments(1, &[&[0], &[0], &[0]]),
        ),
    ];
    let mut failures: Vec<String> = Vec::new();
    for (name, workload) in &workloads {
        let mut violations = 0;
        for seed in 0..24 {
            let history = through_the_reader(
                &model,
                &record::record(&model, &faulty::billing(Fault::LostUpdate), workload, seed)
                    .expect("recorded"),
            );
            if verdict(&model, &history, DEFAULT_BUDGET) != Verdict::Violation {
                continue;
            }
            violations += 1;
            let shrunk = linearize::shrink(&model, &history, DEFAULT_BUDGET).expect("shrunk");
            let clients: BTreeSet<u64> = shrunk.operations.iter().map(|it| it.client).collect();
            let rechecked = verdict(&model, &through_the_reader(&model, &shrunk), DEFAULT_BUDGET);
            if clients.len() > 2
                || shrunk.clients > 2
                || shrunk.operations.len() > 4
                || !every_client_is_sequential(&shrunk)
                || rechecked != Verdict::Violation
            {
                failures.push(format!(
                    "{name}, seed {seed}: {} client(s) used, `clients` {}, {} operation(s), \
                     sequential clients {}, re-checked {rechecked:?}\n{}",
                    clients.len(),
                    shrunk.clients,
                    shrunk.operations.len(),
                    every_client_is_sequential(&shrunk),
                    shrunk
                        .operations
                        .iter()
                        .map(|it| format!(
                            "  client {} [{}, {:?}] {} -> {:?}",
                            it.client,
                            it.invoked_at,
                            it.returned_at,
                            it.command.as_str(),
                            it.outcome.as_ref().map(history::QualifiedName::as_str)
                        ))
                        .collect::<Vec<_>>()
                        .join("\n")
                ));
            }
        }
        assert!(violations > 0, "{name}: some seed overlaps two payments");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// ---- `ess-history/3`: the input and the events an operation records (beyond10x/ess#576) ---------

/// Adversarial cases for `ess-history/3`, aimed at the reader's spelling rules for the two new
/// fields and at what `check-history` does with a recorded input and recorded events.
mod ess_history_3_adversary {
    use super::*;
    use ess_conformance::history::{HistoryFormat, Node};
    use serde_json::{json, Value};

    /// One command over no entity whose `big` branch publishes two events in a declared order, and
    /// one whose branch is decided by whether an `Optional` input is defined.
    const PROBE: &str = "format: ess/16
system: probe
version: v1
domain: probe.core
actors:
  - {name: probe.core.Caller, may: [probe.core.Check, probe.core.Tag]}
commands:
  - name: probe.core.Check
    input:
      - {name: n, type: Integer}
    outcomes:
      - name: big
        when: n >= 10
        emits: [probe.core.Big, probe.core.Noted]
        payload:
          probe.core.Big: {n: input.n}
          probe.core.Noted: {n: input.n}
      - name: small
        emits: [probe.core.Small]
        payload:
          probe.core.Small: {n: input.n}
  - name: probe.core.Tag
    input:
      - {name: tag, type: Optional<String>}
    outcomes:
      - name: tagged
        when: {tag: {defined: true}}
        emits: [probe.core.Tagged]
        payload:
          probe.core.Tagged: {n: 1}
      - name: plain
        emits: [probe.core.Plain]
        payload:
          probe.core.Plain: {n: 1}
events:
  - name: probe.core.Big
    fields:
      - {name: n, type: Integer}
  - name: probe.core.Noted
    fields:
      - {name: n, type: Integer}
  - name: probe.core.Small
    fields:
      - {name: n, type: Integer}
  - name: probe.core.Tagged
    fields:
      - {name: n, type: Integer}
  - name: probe.core.Plain
    fields:
      - {name: n, type: Integer}
";

    fn probe() -> EssIr {
        let raw = RawSpecFile::parse(PROBE).expect("the model parses");
        let specification = Specification::assemble([(Source::new("probe.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
        compile(&specification, &SourceMap::new())
            .unwrap_or_else(|diagnostics| panic!("the model resolves:\n{diagnostics}"))
    }

    fn digest(ir: &EssIr) -> history::SpecDigest {
        SuiteProvenance::of(ir).spec_digest
    }

    /// One `Returned` operation of client 0 at `[2i+1, 2i+2]`, with `input` and `events` written
    /// only where given.
    fn returned(
        index: u64,
        command: &str,
        outcome: &str,
        input: Option<Value>,
        events: Option<&[&str]>,
    ) -> Value {
        let mut operation = json!({
            "operation_id": format!("00000000-0000-4000-8000-{:012}", index + 1),
            "client": 0,
            "command": command,
            "subject_key": "",
            "invoked_at": 2 * index + 1,
            "returned_at": 2 * index + 2,
            "completion": "Returned",
            "outcome": outcome,
        });
        // `ess-history/3` records each event with its payload: the one `PROBE` maps, `{n: 1}` for
        // `Tagged` and `Plain` and `{n: input.n}` for the rest, where the input has an `n`.
        let payload = |event: &str| match (event, input.as_ref().and_then(|input| input.get("n"))) {
            ("probe.core.Tagged" | "probe.core.Plain", _) => json!({"n": 1}),
            (_, Some(n)) => json!({ "n": n }),
            (_, None) => json!({}),
        };
        if let Some(events) = events {
            operation["events"] = events
                .iter()
                .map(|event| json!({"event": event, "payload": payload(event)}))
                .collect();
        }
        if let Some(input) = input {
            operation["input"] = input;
        }
        operation
    }

    /// The document in the writer's own spelling: compact, keys in declaration order (`input`'s
    /// own keys sorted, as every writer sorts them).
    fn document(ir: &EssIr, operations: &[Value]) -> Vec<u8> {
        const KEYS: [&str; 13] = [
            "operation_id",
            "client",
            "command",
            "subject_key",
            "invoked_at",
            "returned_at",
            "completion",
            "outcome",
            "rows",
            "retry_of",
            "decision_time",
            "input",
            "events",
        ];
        let operations = operations
            .iter()
            .map(|operation| {
                let members = KEYS
                    .iter()
                    .filter_map(|key| {
                        operation
                            .get(*key)
                            .map(|value| format!("{}:{value}", Value::from(*key)))
                    })
                    .collect::<Vec<_>>();
                format!("{{{}}}", members.join(","))
            })
            .collect::<Vec<_>>();
        format!(
            r#"{{"format":"ess-history/3","history_id":"00000000-0000-4000-8000-0000000000aa","spec_digest":"{}","seed":1,"clients":1,"operations":[{}]}}"#,
            digest(ir).as_str(),
            operations.join(",")
        )
        .into_bytes()
    }

    fn judged(ir: &EssIr, operations: &[Value]) -> Result<Verdict, String> {
        let bytes = document(ir, operations);
        let history = history::read(&bytes, &digest(ir)).map_err(|refusal| refusal.to_string())?;
        linearize::check(ir, &history, DEFAULT_BUDGET)
            .map(|checked| checked.verdict)
            .map_err(|refusal| format!("{refusal:?}"))
    }

    const CHECK: &str = "probe.core.Check";
    const TAG: &str = "probe.core.Tag";
    const BIG: &str = "probe.core.Big";
    const NOTED: &str = "probe.core.Noted";
    const SMALL: &str = "probe.core.Small";

    // ---- the reader -------------------------------------------------------------------------

    /// A key written twice inside `input` is two readings of one document: serde keeps the last,
    /// another reader keeps the first. The reader refuses a repeated operation field
    /// (`history_format2.rs`, `a_repeated_or_unknown_field_is_refused`); a repeated input field
    /// decides which branch the checker steps, so it is refused too.
    #[test]
    fn an_input_field_written_twice_is_refused() {
        let ir = probe();
        let once = String::from_utf8(document(
            &ir,
            &[returned(0, CHECK, "big", Some(json!({"n": 100})), None)],
        ))
        .unwrap();
        let twice = once.replace(r#""input":{"n":100}"#, r#""input":{"n":5,"n":100}"#);
        assert_ne!(once, twice);
        let read = history::read(twice.as_bytes(), &digest(&ir));
        assert!(
            read.is_err(),
            "admitted, input read as {:?}",
            read.unwrap().operations[0].input
        );
    }

    /// Whatever spelling of an input number the reader admits writes back as itself: the reader's
    /// promise is one spelling, and a history read and written back keeps its bytes.
    #[test]
    fn an_admitted_input_number_writes_back_in_the_spelling_it_was_read_in() {
        let ir = probe();
        let canonical = String::from_utf8(document(
            &ir,
            &[returned(0, CHECK, "big", Some(json!({"n": 100})), None)],
        ))
        .unwrap();
        let mut admitted = Vec::new();
        for spelling in [
            "100.0",
            "1e2",
            "100.00",
            "1.0e2",
            // How Go's encoding/json and JavaScript's JSON.stringify spell 10^21 and 2^-30.
            "1e+21",
            "9.313225746154785e-10",
        ] {
            let text = canonical.replace(r#""n":100"#, &format!(r#""n":{spelling}"#));
            assert_ne!(text, canonical);
            if let Ok(history) = history::read(text.as_bytes(), &digest(&ir)) {
                let written = String::from_utf8(serde_json::to_vec(&history).unwrap()).unwrap();
                if written != text {
                    admitted.push(format!("`{spelling}` read and written back as {written}"));
                }
            }
        }
        assert!(admitted.is_empty(), "{}", admitted.join("\n"));
    }

    /// An empty input (a command with no field) and an empty `events` (a refused branch) are each a
    /// recorded value, not an absent one: both are written back, and the history stays format 3.
    #[test]
    fn an_empty_input_and_empty_events_are_recorded_values_and_keep_format3() {
        let ir = probe();
        let bytes = document(
            &ir,
            &[returned(0, TAG, "plain", Some(json!({})), Some(&[]))],
        );
        let history = history::read(&bytes, &digest(&ir)).expect("admitted");
        assert_eq!(history.format, HistoryFormat::EssHistory3);
        assert_eq!(
            HistoryFormat::for_operations(&history.operations),
            HistoryFormat::EssHistory3
        );
        assert_eq!(history.operations[0].input, Some(BTreeMap::new()));
        assert_eq!(history.operations[0].events, Some(Vec::new()));
        assert_eq!(serde_json::to_vec(&history).unwrap(), bytes);
    }

    /// An `Optional` input sent as `null` and one left out are different documents, and each reads
    /// back as itself.
    #[test]
    fn an_optional_input_sent_null_and_one_left_out_read_and_write_back_apart() {
        let ir = probe();
        for input in [json!({"tag": null}), json!({})] {
            let bytes = document(&ir, &[returned(0, TAG, "plain", Some(input.clone()), None)]);
            let history = history::read(&bytes, &digest(&ir)).expect("admitted");
            assert_eq!(serde_json::to_vec(&history).unwrap(), bytes, "{input}");
        }
        let null = history::read(
            &document(
                &ir,
                &[returned(0, TAG, "plain", Some(json!({"tag": null})), None)],
            ),
            &digest(&ir),
        )
        .unwrap();
        assert_eq!(
            null.operations[0].input.as_ref().unwrap().get("tag"),
            Some(&Node::Null)
        );
    }

    // ---- check-history holds the answer to the input it records -----------------------------

    #[test]
    fn a_recorded_input_that_takes_small_does_not_explain_big() {
        let ir = probe();
        assert_eq!(
            judged(
                &ir,
                &[returned(0, CHECK, "big", Some(json!({"n": 5})), None)]
            ),
            Ok(Verdict::Violation)
        );
        assert_eq!(
            judged(
                &ir,
                &[returned(0, CHECK, "small", Some(json!({"n": 100})), None)]
            ),
            Ok(Verdict::Violation)
        );
        // The boundary itself.
        assert_eq!(
            judged(
                &ir,
                &[returned(0, CHECK, "big", Some(json!({"n": 10})), None)]
            ),
            Ok(Verdict::Linearizable)
        );
        assert_eq!(
            judged(
                &ir,
                &[returned(0, CHECK, "small", Some(json!({"n": 9})), None)]
            ),
            Ok(Verdict::Linearizable)
        );
    }

    #[test]
    fn the_recorded_events_are_held_to_the_branch_in_order_and_in_number() {
        let ir = probe();
        let big = || Some(json!({"n": 100}));
        assert_eq!(
            judged(
                &ir,
                &[returned(0, CHECK, "big", big(), Some(&[BIG, NOTED]))]
            ),
            Ok(Verdict::Linearizable),
            "the branch's own events, in its order"
        );
        for (events, why) in [
            (&[NOTED, BIG][..], "the same events in another order"),
            (&[BIG][..], "an event missing"),
            (&[BIG, NOTED, NOTED][..], "an event published twice"),
            (&[][..], "no event"),
            (&[SMALL][..], "the other branch's event"),
            (&[BIG, NOTED, SMALL][..], "the other branch's event besides"),
            (&["probe.core.Nowhere"][..], "an event no model declares"),
        ] {
            assert_eq!(
                judged(&ir, &[returned(0, CHECK, "big", big(), Some(events))]),
                Ok(Verdict::Violation),
                "{why}: {events:?}"
            );
            // Without the input too: no candidate input makes `big` publish these.
            assert_eq!(
                judged(&ir, &[returned(0, CHECK, "big", None, Some(events))]),
                Ok(Verdict::Violation),
                "{why}, input not recorded: {events:?}"
            );
        }
    }

    #[test]
    fn an_input_without_events_and_events_without_an_input_are_each_held_to_what_they_record() {
        let ir = probe();
        // Input recorded, events not: held to the input only.
        assert_eq!(
            judged(
                &ir,
                &[returned(0, CHECK, "big", Some(json!({"n": 100})), None)]
            ),
            Ok(Verdict::Linearizable)
        );
        // Events recorded, input not: some candidate input takes `big`.
        assert_eq!(
            judged(&ir, &[returned(0, CHECK, "big", None, Some(&[BIG, NOTED]))]),
            Ok(Verdict::Linearizable)
        );
        // Neither: as `ess-history/2` reads it.
        assert_eq!(
            judged(&ir, &[returned(0, CHECK, "small", None, None)]),
            Ok(Verdict::Linearizable)
        );
    }

    #[test]
    fn an_optional_input_is_held_to_whether_it_was_sent() {
        let ir = probe();
        for (input, outcome, events, expected) in [
            (
                json!({"tag": "x"}),
                "tagged",
                "probe.core.Tagged",
                Verdict::Linearizable,
            ),
            (
                json!({"tag": "x"}),
                "plain",
                "probe.core.Plain",
                Verdict::Violation,
            ),
            (
                json!({"tag": null}),
                "tagged",
                "probe.core.Tagged",
                Verdict::Violation,
            ),
            (
                json!({"tag": null}),
                "plain",
                "probe.core.Plain",
                Verdict::Linearizable,
            ),
            (json!({}), "tagged", "probe.core.Tagged", Verdict::Violation),
            (
                json!({}),
                "plain",
                "probe.core.Plain",
                Verdict::Linearizable,
            ),
        ] {
            assert_eq!(
                judged(
                    &ir,
                    &[returned(
                        0,
                        TAG,
                        outcome,
                        Some(input.clone()),
                        Some(&[events])
                    )]
                ),
                Ok(expected),
                "{input} answered {outcome}"
            );
        }
    }

    /// An input the command could not have been sent — a required field missing, a value of another
    /// type, a fraction for an `Integer` or a field the command does not declare — is no input a
    /// target answered `small` for.
    #[test]
    fn an_input_the_command_does_not_declare_explains_no_answer() {
        let ir = probe();
        let mut wrong = Vec::new();
        for input in [
            json!({}),
            json!({"n": "five"}),
            json!({"n": null}),
            json!({"n": 5.5}),
            json!({"n": 5, "undeclared": 1}),
        ] {
            let verdict = judged(
                &ir,
                &[returned(
                    0,
                    CHECK,
                    "small",
                    Some(input.clone()),
                    Some(&[SMALL]),
                )],
            );
            if verdict == Ok(Verdict::Linearizable) {
                wrong.push(format!("{input} explains `small`"));
            }
        }
        assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    }

    /// The shrunk history of a format 3 violation keeps what made it one: its format and every
    /// recorded input and event, and it is still a violation when read back.
    #[test]
    fn a_format3_violation_shrinks_to_a_format3_history_that_keeps_its_inputs_and_events() {
        let ir = probe();
        let operations = [
            returned(0, CHECK, "small", Some(json!({"n": 3})), Some(&[SMALL])),
            returned(1, CHECK, "big", Some(json!({"n": 50})), Some(&[BIG, NOTED])),
            returned(2, CHECK, "big", Some(json!({"n": 5})), Some(&[BIG, NOTED])),
        ];
        let bytes = document(&ir, &operations);
        let history = history::read(&bytes, &digest(&ir)).expect("admitted");
        let report = linearize::report(&ir, &history, DEFAULT_BUDGET).expect("checked");
        assert_eq!(report.verdict, Verdict::Violation);
        let shrunk = report.shrunk.expect("a violation is shrunk");
        assert_eq!(shrunk.format, HistoryFormat::EssHistory3);
        assert!(
            shrunk
                .operations
                .iter()
                .all(|operation| operation.input.is_some() && operation.events.is_some()),
            "{shrunk:#?}"
        );
        assert!(shrunk
            .operations
            .iter()
            .any(|operation| operation.input.as_ref().unwrap()["n"]
                == serde_json::from_str::<Node>("5").unwrap()));
        let again = through_the_reader(&ir, &shrunk);
        assert_eq!(verdict(&ir, &again, DEFAULT_BUDGET), Verdict::Violation);
    }

    // ---- an entity model: the recorded input and the recorded subject -----------------------

    /// `two_registers` written as format 3: each write records the input it sent.
    fn recorded_registers() -> (EssIr, History) {
        let (model, mut history) = two_registers();
        history.format = HistoryFormat::EssHistory3;
        for operation in &mut history.operations {
            let mut input = BTreeMap::new();
            if operation.command.as_str() != "register.cell.CreateRegister" {
                input.insert(
                    "register_id".to_owned(),
                    Node::Text(operation.subject_key.clone()),
                );
            }
            operation.input = Some(input);
        }
        let history = through_the_reader(&model, &history);
        assert_eq!(history.format, HistoryFormat::EssHistory3);
        (model, history)
    }

    #[test]
    fn a_format3_history_of_the_models_own_answers_is_linearizable() {
        let (model, history) = recorded_registers();
        assert_eq!(
            verdict(&model, &history, DEFAULT_BUDGET),
            Verdict::Linearizable
        );
    }

    /// The operation says it addressed one register and records an input naming the other. No
    /// client sent both: the document contradicts itself, and the checker may not explain the
    /// answer with whichever of the two suits it.
    #[test]
    fn an_input_naming_another_subject_than_subject_key_is_not_explained() {
        let (model, mut history) = recorded_registers();
        let other = history.operations[0].subject_key.clone();
        let write = &mut history.operations[2];
        assert_ne!(
            write.subject_key, other,
            "the write addresses the second register"
        );
        write
            .input
            .as_mut()
            .unwrap()
            .insert("register_id".to_owned(), Node::Text(other));
        let bytes = serde_json::to_vec(&history).unwrap();
        let outcome = history::read(&bytes, &SuiteProvenance::of(&model).spec_digest)
            .map_err(|refusal| refusal.to_string())
            .and_then(|history| {
                linearize::check(&model, &history, DEFAULT_BUDGET)
                    .map(|checked| checked.verdict)
                    .map_err(|refusal| format!("{refusal:?}"))
            });
        assert_ne!(outcome, Ok(Verdict::Linearizable));
    }

    /// A write that never answered may never have happened, and where it did it wrote the register
    /// its recorded input names.
    #[test]
    fn an_unanswered_write_with_its_input_may_or_may_not_have_happened() {
        let (model, mut history) = recorded_registers();
        let write = &mut history.operations[2];
        write.completion = Completion::Indeterminate;
        write.returned_at = None;
        write.outcome = None;
        let history = through_the_reader(&model, &history);
        assert_eq!(
            verdict(&model, &history, DEFAULT_BUDGET),
            Verdict::Linearizable
        );
    }
}

// ---- `ess-history/3`: what a call arranged, and what its events carried (beyond10x/ess#576) ----

/// The checker's side of the adversary's third and fourth findings, and the reader's one spelling
/// of a number: an `external:` answer is admitted only where an arrangement the history records
/// could have produced it, and a published payload is held to the value the model computes from
/// the recorded input.
mod ess_history_3_arrangements_and_payloads {
    use super::*;
    use ess_conformance::history::HistoryRefusal;
    use serde_json::{json, Value};

    /// One command over no entity: an external decline eligible from 1000 up, and an ordinary
    /// branch whose event carries the input's amount, its optional note and a literal version.
    const PAY: &str = "format: ess/16
system: arrange
version: v1
domain: arrange.core
actors:
  - {name: arrange.core.Caller, may: [arrange.core.Pay]}
commands:
  - name: arrange.core.Pay
    input:
      - {name: amount, type: Integer}
      - {name: note, type: Optional<String>}
    outcomes:
      - name: declined
        when: amount >= 1000
        external: the card network declines
        error: arrange.core.Declined
      - name: paid
        emits: [arrange.core.Paid]
        payload:
          arrange.core.Paid: {amount: input.amount, note: input.note, version: 2}
events:
  - name: arrange.core.Paid
    fields:
      - {name: amount, type: Integer}
      - {name: note, type: Optional<String>}
      - {name: version, type: Integer}
errors:
  - name: arrange.core.Declined
    summary: The card network declined.
";

    fn pay() -> EssIr {
        let raw = RawSpecFile::parse(PAY).expect("the model parses");
        let specification = Specification::assemble([(Source::new("pay.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
        compile(&specification, &SourceMap::new())
            .unwrap_or_else(|diagnostics| panic!("the model resolves:\n{diagnostics}"))
    }

    /// One call of `Pay`: `index` names it, `client` made it, it ran from `invoked` to `returned`
    /// and answered `outcome`, with its input, events and arrangements written only where given.
    struct Call {
        index: u64,
        client: u64,
        invoked: u64,
        returned: u64,
        outcome: &'static str,
        input: Option<Value>,
        events: Option<Value>,
        arranged: Option<Value>,
    }

    impl Call {
        fn new(index: u64, outcome: &'static str) -> Self {
            Self {
                index,
                client: 0,
                invoked: 2 * index + 1,
                returned: 2 * index + 2,
                outcome,
                input: None,
                events: None,
                arranged: None,
            }
        }

        fn json(&self) -> Value {
            let mut operation = json!({
                "operation_id": format!("00000000-0000-4000-8000-{:012}", self.index + 1),
                "client": self.client,
                "command": "arrange.core.Pay",
                "subject_key": "",
                "invoked_at": self.invoked,
                "returned_at": self.returned,
                "completion": "Returned",
                "outcome": self.outcome,
            });
            for (key, value) in [
                ("input", &self.input),
                ("events", &self.events),
                ("arranged", &self.arranged),
            ] {
                if let Some(value) = value {
                    operation[key] = value.clone();
                }
            }
            operation
        }
    }

    fn bytes(ir: &EssIr, calls: &[Call]) -> Vec<u8> {
        serde_json::to_vec(&json!({
            "format": "ess-history/3",
            "history_id": "00000000-0000-4000-8000-0000000000aa",
            "spec_digest": SuiteProvenance::of(ir).spec_digest.as_str(),
            "seed": 1,
            "clients": 2,
            "operations": calls.iter().map(Call::json).collect::<Vec<_>>(),
        }))
        .unwrap()
    }

    fn judged(ir: &EssIr, calls: &[Call]) -> Verdict {
        let history = history::read(&bytes(ir, calls), &SuiteProvenance::of(ir).spec_digest)
            .unwrap_or_else(|refusal| panic!("{refusal}"));
        linearize::check(ir, &history, DEFAULT_BUDGET)
            .unwrap_or_else(|refusal| panic!("{refusal:?}"))
            .verdict
    }

    /// A call of `amount` that answered `declined`, recording what it arranged.
    fn declined(index: u64, amount: i64, arranged: &[&str]) -> Call {
        Call {
            input: Some(json!({ "amount": amount })),
            arranged: Some(json!(arranged)),
            events: Some(json!([])),
            ..Call::new(index, "declined")
        }
    }

    #[test]
    fn an_external_answer_no_arrangement_explains_is_a_violation() {
        let ir = pay();
        assert_eq!(judged(&ir, &[declined(0, 5000, &[])]), Verdict::Violation);
        assert_eq!(
            judged(&ir, &[declined(0, 5000, &["declined"])]),
            Verdict::Linearizable,
            "its own arrangement explains it"
        );
        // An arrangement does not lift the branch's eligibility guard.
        assert_eq!(
            judged(&ir, &[declined(0, 5, &["declined"])]),
            Verdict::Violation
        );
    }

    /// A target keeps an arrangement until the scenario ends, and clients run at once: another
    /// call's arrangement explains an answer when that call was invoked before this one returned,
    /// and only then.
    #[test]
    fn another_calls_arrangement_explains_an_answer_only_when_it_came_before_this_one_returned() {
        let ir = pay();
        let arranging = |invoked, returned| Call {
            client: 0,
            invoked,
            returned,
            ..declined(0, 5000, &["declined"])
        };
        let unarranged = |invoked, returned| Call {
            client: 1,
            invoked,
            returned,
            ..declined(1, 5000, &[])
        };
        // Overlapping, the arrangement invoked first.
        assert_eq!(
            judged(&ir, &[arranging(1, 4), unarranged(2, 3)]),
            Verdict::Linearizable
        );
        // Long after it: the arrangement still holds.
        assert_eq!(
            judged(&ir, &[arranging(1, 2), unarranged(5, 6)]),
            Verdict::Linearizable
        );
        // The arrangement was made after the unarranged call had returned.
        assert_eq!(
            judged(&ir, &[unarranged(1, 2), arranging(3, 4)]),
            Verdict::Violation
        );
    }

    /// An arrangement may already have been used, so the ordinary branch stays open beside it;
    /// a history that records no arrangement at all keeps every eligible external branch open.
    #[test]
    fn the_ordinary_branch_stays_open_and_a_history_without_arrangements_is_read_as_before() {
        let ir = pay();
        let paid = Call {
            input: Some(json!({"amount": 5000})),
            arranged: Some(json!(["declined"])),
            ..Call::new(0, "paid")
        };
        assert_eq!(judged(&ir, &[paid]), Verdict::Linearizable);
        let unrecorded = Call {
            input: Some(json!({"amount": 5000})),
            ..Call::new(0, "declined")
        };
        assert_eq!(judged(&ir, &[unrecorded]), Verdict::Linearizable);
    }

    fn paid(index: u64, input: Option<Value>, payload: Value) -> Call {
        Call {
            input,
            events: Some(json!([{"event": "arrange.core.Paid", "payload": payload}])),
            arranged: Some(json!([])),
            ..Call::new(index, "paid")
        }
    }

    #[test]
    fn a_payload_is_held_to_what_the_recorded_input_and_the_literals_give() {
        let ir = pay();
        let input = || Some(json!({"amount": 5}));
        for (payload, expected, why) in [
            (
                json!({"amount": 5, "version": 2}),
                Verdict::Linearizable,
                "as mapped",
            ),
            (
                json!({"amount": 6, "version": 2}),
                Verdict::Violation,
                "another amount",
            ),
            (
                json!({"version": 2}),
                Verdict::Violation,
                "the amount left out",
            ),
            (
                json!({"amount": 5, "version": 3}),
                Verdict::Violation,
                "another literal",
            ),
            (
                json!({"amount": 5, "note": null, "version": 2}),
                Verdict::Linearizable,
                "an absent note written null",
            ),
            (
                json!({"amount": 5, "note": "x", "version": 2}),
                Verdict::Violation,
                "a note nobody sent",
            ),
            (
                json!({"amount": 5, "version": 2, "unmapped": true}),
                Verdict::Linearizable,
                "a field no mapping names is not compared",
            ),
        ] {
            assert_eq!(judged(&ir, &[paid(0, input(), payload)]), expected, "{why}");
        }
        assert_eq!(
            judged(
                &ir,
                &[paid(
                    0,
                    Some(json!({"amount": 5, "note": "x"})),
                    json!({"amount": 5, "note": "x", "version": 2})
                )]
            ),
            Verdict::Linearizable
        );
        // Without its input, an operation is held to the names of its events only.
        assert_eq!(
            judged(&ir, &[paid(0, None, json!({"amount": 999_999}))]),
            Verdict::Linearizable
        );
    }

    /// The bytes of a one-call history in the writer's own spelling, with `amount` written as
    /// `spelling` in the input and in the payload.
    fn spelled(ir: &EssIr, spelling: &str) -> String {
        let written = history::read(
            &bytes(
                ir,
                &[paid(0, Some(json!({"amount": 1})), json!({"amount": 1}))],
            ),
            &SuiteProvenance::of(ir).spec_digest,
        )
        .unwrap();
        let canonical = String::from_utf8(serde_json::to_vec(&written).unwrap()).unwrap();
        assert_eq!(canonical.matches(r#""amount":1"#).count(), 2, "{canonical}");
        canonical.replace(r#""amount":1"#, &format!(r#""amount":{spelling}"#))
    }

    #[test]
    fn a_number_in_an_input_or_a_payload_has_one_spelling_and_writes_back_as_itself() {
        let ir = pay();
        let digest = SuiteProvenance::of(&ir).spec_digest;
        for spelling in [
            "0",
            "-3",
            "0.5",
            "123456789012",
            "9007199254740992",
            "1e+21",
            "1.5e-7",
            "0.000001",
            "-1.25e+30",
        ] {
            let text = spelled(&ir, spelling);
            let history = history::read(text.as_bytes(), &digest).unwrap_or_else(|refusal| {
                panic!("`{spelling}` is its one spelling and is refused: {refusal}")
            });
            assert_eq!(
                String::from_utf8(serde_json::to_vec(&history).unwrap()).unwrap(),
                text,
                "`{spelling}`"
            );
        }
        for (spelling, one) in [
            ("-0", "0"),
            ("1.0", "1"),
            ("1E2", "100"),
            ("1e21", "1e+21"),
            ("0.0000001", "1e-7"),
            ("9007199254740993", "9007199254740992"),
        ] {
            let refused = history::read(spelled(&ir, spelling).as_bytes(), &digest).unwrap_err();
            assert_eq!(refused.code(), "history.malformed", "`{spelling}`");
            assert!(
                refused.to_string().contains(&format!(
                    "`operations[0].input.amount` is written `{spelling}`; its one spelling is \
                     `{one}`"
                )),
                "`{spelling}`: {refused}"
            );
        }
        // The payload is held to the same rule, and so is every depth of both.
        let payload = spelled(&ir, "1").replacen(r#""amount":1"#, "@", 1);
        let payload = payload
            .replace(r#""amount":1"#, r#""amount":1.0"#)
            .replace('@', r#""amount":1"#);
        let refused = history::read(payload.as_bytes(), &digest).unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("`operations[0].events[0].payload.amount` is written `1.0`"),
            "{refused}"
        );
        let nested = spelled(&ir, r#"1,"tags":[{"at":[2.50]}]"#);
        let refused = history::read(nested.as_bytes(), &digest).unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("`operations[0].input.tags[0].at[0]` is written `2.50`"),
            "{refused}"
        );
        assert!(matches!(
            history::read(spelled(&ir, "1e400").as_bytes(), &digest),
            Err(HistoryRefusal::Malformed { .. })
        ));
    }

    #[test]
    fn a_key_written_twice_in_a_payload_or_deep_in_an_input_is_refused_by_name() {
        let ir = pay();
        let digest = SuiteProvenance::of(&ir).spec_digest;
        let canonical = spelled(&ir, "1");
        let twice_in_payload = {
            let at = canonical.rfind(r#""amount":1"#).unwrap();
            format!("{}\"amount\":7,{}", &canonical[..at], &canonical[at..])
        };
        let refused = history::read(twice_in_payload.as_bytes(), &digest).unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("`operations[0].events[0].payload` writes the key `amount` twice"),
            "{refused}"
        );
        let deep = spelled(&ir, r#"1,"meta":{"a":1,"a":2}"#);
        let refused = history::read(deep.as_bytes(), &digest).unwrap_err();
        assert!(
            refused
                .to_string()
                .contains("`operations[0].input.meta` writes the key `a` twice"),
            "{refused}"
        );
    }
}
