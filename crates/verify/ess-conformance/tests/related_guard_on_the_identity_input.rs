//! A `when:` guard on the input a `when_related:` guard reads its row through is decided on the
//! identity of the row the scenario sends (<https://github.com/beyond10x/ess/issues/693>).
//!
//! `UseSlot(kind)` refuses `kind == b`, then refuses a missing or a disabled slot of that kind, and
//! otherwise uses it. The slot's identity is its kind, so the row a scenario sends fixes the value
//! the guard reads. Synthesis decided the guard on a candidate literal for `kind` and then sent a
//! captured row created under another kind, so `refused-b` was answered `disabled` on the
//! interpreted target. Each scenario now sends a row whose identity meets the guard it witnesses,
//! or the branch is refused.
//!
//! Every model is inline, so it is no repository model the suite-bytes tables walk.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

/// The guard of `refused-b`, as [`MODEL`] writes it.
const GUARD: &str = "        when: kind == b\n";

/// The issue's model.
const MODEL: &str = r"format: ess/32
system: demo
version: v1
domain: demo.slots
summary: A slot is opened by its kind; using a slot of kind b is refused.
types:
  - name: demo.slots.Kind
    kind: enum
    variants: [a, b, c]
entities:
  - name: demo.slots.Slot
    identity: {name: kind, type: demo.slots.Kind}
    fields:
      - {name: enabled, type: Boolean}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
views:
  - name: demo.slots.Slots
    source: demo.slots.Slot
    consistency: read_your_writes
    fields:
      - {name: kind, type: demo.slots.Kind}
      - {name: state, type: demo.slots.Slot.State}
      - {name: enabled, type: Boolean}
actors:
  - name: demo.slots.Caller
    may: [demo.slots.OpenSlot, demo.slots.UseSlot]
errors:
  - name: demo.slots.SlotMissing
    fields:
      - {name: kind, type: demo.slots.Kind}
  - name: demo.slots.SlotDisabled
    fields:
      - {name: kind, type: demo.slots.Kind}
  - name: demo.slots.KindRefused
    fields:
      - {name: kind, type: demo.slots.Kind}
commands:
  - name: demo.slots.OpenSlot
    input:
      - {name: kind, type: demo.slots.Kind}
      - {name: enabled, type: Boolean}
    outcomes:
      - name: opened
        creates: demo.slots.Slot
        instance: kind
        sets: {enabled: input.enabled}
        emits: [demo.slots.SlotOpened]
        payload:
          demo.slots.SlotOpened: {kind: input.kind}
  - name: demo.slots.UseSlot
    input:
      - {name: kind, type: demo.slots.Kind}
    outcomes:
      - name: refused-b
        when: kind == b
        error: demo.slots.KindRefused
        payload:
          demo.slots.KindRefused: {kind: input.kind}
      - name: missing
        when_related: {via: input.kind, exists: false}
        error: demo.slots.SlotMissing
        payload:
          demo.slots.SlotMissing: {kind: input.kind}
      - name: disabled
        when_related:
          via: input.kind
          predicate: enabled == false
        error: demo.slots.SlotDisabled
        payload:
          demo.slots.SlotDisabled: {kind: input.kind}
      - name: used
        emits: [demo.slots.SlotUsed]
        payload:
          demo.slots.SlotUsed: {kind: input.kind}
events:
  - name: demo.slots.SlotOpened
    fields:
      - {name: kind, type: demo.slots.Kind}
  - name: demo.slots.SlotUsed
    fields:
      - {name: kind, type: demo.slots.Kind}
";

const REFUSED_B: &str = "demo.slots.UseSlot/outcome/refused-b";

/// `text` with `part`, which it holds once, replaced by `with`.
fn replaced(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

/// [`MODEL`] with `refused-b` guarded by `guard`, a `when:` line.
fn guarded(guard: &str) -> String {
    replaced(MODEL, GUARD, guard)
}

/// [`MODEL`] with `Kind` a newtype of `String` and `refused-b` guarded by `kind == "x"`.
fn string_identity() -> String {
    let text = replaced(
        MODEL,
        "    kind: enum\n    variants: [a, b, c]\n",
        "    kind: newtype\n    of: String\n",
    );
    replaced(&text, GUARD, "        when: 'kind == \"x\"'\n")
}

/// [`MODEL`] with `Kind` two-valued and `refused-b` guarded on either kind: no slot leaves
/// `disabled` or `used` to answer.
fn no_row_left() -> String {
    let text = replaced(MODEL, "    variants: [a, b, c]\n", "    variants: [a, b]\n");
    replaced(
        &text,
        GUARD,
        "        when:\n          any: [kind == a, kind == b]\n",
    )
}

/// Two slots paired by their kinds; pairing with a slot of kind `x` is refused.
fn pair_slots() -> String {
    let text = replaced(
        MODEL,
        "    kind: enum\n    variants: [a, b, c]\n",
        "    kind: newtype\n    of: String\n",
    );
    let text = replaced(
        &text,
        "    may: [demo.slots.OpenSlot, demo.slots.UseSlot]\n",
        "    may: [demo.slots.OpenSlot, demo.slots.PairSlots]\n",
    );
    let start = text
        .find("  - name: demo.slots.UseSlot\n")
        .expect("UseSlot");
    let end = text.find("events:\n").expect("events");
    let mut text = text;
    text.replace_range(
        start..end,
        r#"  - name: demo.slots.PairSlots
    input:
      - {name: kind, type: demo.slots.Kind}
      - {name: other, type: demo.slots.Kind}
    outcomes:
      - name: refused-x
        when: 'other == "x"'
        error: demo.slots.KindRefused
        payload:
          demo.slots.KindRefused: {kind: input.other}
      - name: missing
        when_related: {via: input.kind, exists: false}
        error: demo.slots.SlotMissing
        payload:
          demo.slots.SlotMissing: {kind: input.kind}
      - name: other-missing
        when_related: {via: input.other, exists: false}
        error: demo.slots.SlotMissing
        payload:
          demo.slots.SlotMissing: {kind: input.other}
      - name: disabled
        when_related:
          via: input.kind
          predicate: enabled == false
        error: demo.slots.SlotDisabled
        payload:
          demo.slots.SlotDisabled: {kind: input.kind}
      - name: other-disabled
        when_related:
          via: input.other
          predicate: enabled == false
        error: demo.slots.SlotDisabled
        payload:
          demo.slots.SlotDisabled: {kind: input.other}
      - name: paired
        emits: [demo.slots.SlotsPaired]
        payload:
          demo.slots.SlotsPaired: {kind: input.kind}
"#,
    );
    replaced(
        &text,
        "  - name: demo.slots.SlotUsed\n",
        "  - name: demo.slots.SlotsPaired\n",
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("slots.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

struct Run {
    synthesis: Synthesis,
    results: BTreeMap<String, ScenarioResult>,
}

fn run(text: &str) -> Run {
    let model = ir(text);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let results = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect();
    Run { synthesis, results }
}

/// Every refusal of `wanted`, as `[code] text`.
fn refused(run: &Run, wanted: &str) -> Vec<String> {
    run.synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(wanted)
        })
        .map(|refusal| format!("[{}] {}", refusal.cause.code(), refusal.cause))
        .collect()
}

/// The scenario `wanted` as the suite serializes it, where one is written.
fn written(run: &Run, wanted: &str) -> Option<Value> {
    run.synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == wanted)
        .map(|(_, scenario)| serde_json::to_value(scenario).unwrap())
}

fn scenario(run: &Run, wanted: &str) -> Value {
    written(run, wanted)
        .unwrap_or_else(|| panic!("{wanted} is synthesized: {:#?}", refused(run, wanted)))
}

/// Every scenario passes on the interpreted target.
fn assert_all_pass(run: &Run) {
    let failed: Vec<String> = run
        .results
        .iter()
        .filter(|(_, result)| result.status != Status::Passed)
        .map(|(id, result)| {
            format!(
                "{id}: {:?} {:#?}\n{}",
                result.status,
                result.diagnostics().collect::<Vec<_>>(),
                serde_json::to_string_pretty(&scenario(run, id)).unwrap()
            )
        })
        .collect();
    assert!(
        failed.is_empty(),
        "every scenario passes on the interpreted target:\n{}",
        failed.join("\n")
    );
}

/// Every branch of `command` named in `branches` has a scenario or a refusal.
fn assert_each_written_or_refused(run: &Run, command: &str, branches: &[&str]) {
    for branch in branches {
        let id = format!("{command}/outcome/{branch}");
        assert!(
            written(run, &id).is_some() || !refused(run, &id).is_empty(),
            "{id} has a scenario or a refusal"
        );
    }
}

/// One send of `command` in a scenario: the outcome it is expected to answer, and for each input
/// field sent as a captured slot, the kind that slot was opened with.
#[derive(Debug)]
struct Send {
    expected: String,
    kinds: BTreeMap<String, Value>,
}

/// Every send of `command` in `scenario`.
fn sends(scenario: &Value, command: &str) -> Vec<Send> {
    let steps = scenario["steps"].as_array().expect("steps");
    let mut opened: BTreeMap<String, Value> = BTreeMap::new();
    let mut last_kind: Option<Value> = None;
    let mut found = Vec::new();
    for (at, step) in steps.iter().enumerate() {
        match step["step"].as_str() {
            Some("execute_command") if step["command"] == "demo.slots.OpenSlot" => {
                last_kind = Some(step["input"]["kind"]["value"].clone());
            }
            Some("capture_instance") if step["entity"] == "demo.slots.Slot" => {
                let name = step["instance"].as_str().expect("an instance").to_owned();
                opened.insert(name, last_kind.take().expect("a slot opened before it"));
            }
            Some("execute_command") if step["command"] == command => {
                let expected = steps[at + 1..]
                    .iter()
                    .find(|next| next["step"] == "expect_outcome")
                    .map(|next| next["outcome"]["outcome"].as_str().unwrap_or("").to_owned())
                    .unwrap_or_default();
                let kinds = step["input"]
                    .as_object()
                    .expect("an input")
                    .iter()
                    .filter(|(_, value)| value["kind"] == "instance")
                    .map(|(field, value)| {
                        let name = value["instance"].as_str().expect("a name");
                        let kind = opened
                            .get(name)
                            .unwrap_or_else(|| panic!("{name} was opened: {scenario:#}"))
                            .clone();
                        (field.clone(), kind)
                    })
                    .collect();
                found.push(Send { expected, kinds });
            }
            _ => {}
        }
    }
    found
}

/// The kind of the slot `field` names in every send of `UseSlot` in `wanted` expecting `wanted`'s
/// own outcome.
fn sent_kinds(run: &Run, wanted: &str, field: &str) -> Vec<Value> {
    let scenario = scenario(run, wanted);
    let outcome = wanted.rsplit('/').next().unwrap();
    let kinds: Vec<Value> = sends(&scenario, "demo.slots.UseSlot")
        .into_iter()
        .filter(|send| send.expected == outcome)
        .map(|send| {
            send.kinds
                .get(field)
                .cloned()
                .unwrap_or_else(|| panic!("{field} is sent as a captured slot: {scenario:#}"))
        })
        .collect();
    assert!(!kinds.is_empty(), "{wanted} sends UseSlot: {scenario:#}");
    kinds
}

#[test]
fn refused_b_is_sent_the_row_of_kind_b() {
    let run = run(MODEL);
    for kind in sent_kinds(&run, REFUSED_B, "kind") {
        assert_eq!(kind, "b", "refused-b is sent the slot opened as b");
    }
}

#[test]
fn siblings_are_sent_a_row_the_guard_refutes() {
    let run = run(MODEL);
    for branch in ["disabled", "used"] {
        let id = format!("demo.slots.UseSlot/outcome/{branch}");
        for kind in sent_kinds(&run, &id, "kind") {
            assert_ne!(kind, "b", "{branch} is sent a slot not opened as b");
        }
    }
}

#[test]
fn interpreted_target_passes_every_guard_probe() {
    for guard in [
        "        when: kind == b\n",
        "        when: kind == a\n",
        "        when: kind == c\n",
        "        when: {kind: [b]}\n",
    ] {
        let run = run(&guarded(guard));
        assert_each_written_or_refused(
            &run,
            "demo.slots.UseSlot",
            &["refused-b", "missing", "disabled", "used"],
        );
        assert_all_pass(&run);
    }
}

#[test]
fn a_string_identity_guard_is_decided_on_the_value_sent() {
    let run = run(&string_identity());
    for kind in sent_kinds(&run, REFUSED_B, "kind") {
        assert_eq!(kind, "x", "refused-b is sent the slot opened as x");
    }
    assert_all_pass(&run);
}

#[test]
fn no_row_meeting_the_guard_refuses_by_name() {
    let run = run(&no_row_left());
    for branch in ["disabled", "used"] {
        let id = format!("demo.slots.UseSlot/outcome/{branch}");
        assert!(
            written(&run, &id).is_none(),
            "{id} has no scenario: no slot leaves it to answer"
        );
        let refusals = refused(&run, &id);
        assert!(
            refusals
                .iter()
                .any(|text| text.starts_with("[ESS-SYNTH-003]")
                    && text.contains("decided on the rows sent under `(kind == a or kind == b)`")),
            "{id} is refused naming the guard: {refusals:#?}"
        );
    }
    assert_all_pass(&run);
}

#[test]
fn a_guard_on_a_beside_via_is_never_contradicted() {
    let run = run(&pair_slots());
    let refused_x = "demo.slots.PairSlots/outcome/refused-x";
    if let Some(scenario) = written(&run, refused_x) {
        for send in sends(&scenario, "demo.slots.PairSlots") {
            if send.expected == "refused-x" {
                assert_eq!(
                    send.kinds.get("other"),
                    Some(&Value::from("x")),
                    "refused-x names a slot opened as x: {scenario:#}"
                );
            }
        }
    } else {
        // A refusal names the guard as a predicate renders it: a text literal unquoted.
        let refusals = refused(&run, refused_x);
        assert!(
            refusals
                .iter()
                .any(|text| text.starts_with("[ESS-SYNTH-003]")
                    && text.contains("decided on the rows sent under `other == x`")),
            "refused-x is refused naming its guard: {refusals:#?}"
        );
    }
    assert!(
        written(&run, "demo.slots.PairSlots/outcome/missing").is_some(),
        "missing keeps its scenario: {:#?}",
        refused(&run, "demo.slots.PairSlots/outcome/missing")
    );
    for scenario in run.synthesis.suite.scenarios.values() {
        let scenario = serde_json::to_value(scenario).unwrap();
        for send in sends(&scenario, "demo.slots.PairSlots") {
            if let Some(other) = send.kinds.get("other") {
                assert_eq!(
                    send.expected == "refused-x",
                    other == "x",
                    "a send expects refused-x exactly where its other slot is x: {scenario:#}"
                );
            }
        }
    }
    assert_all_pass(&run);
}
