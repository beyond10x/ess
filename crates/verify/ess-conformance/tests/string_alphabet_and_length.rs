//! The witness under `alphabet:`, `example:` and `.count` over text and lists (beyond10x/ess#103,
//! beyond10x/ess#104, `story:count-guards-above-one-are-synthesized`).
//!
//! `docs/design/string-alphabet-and-length.md`, section 4 and the mutant table of section 7.

use std::cell::Cell;

use ess_compiler::{
    ir::EssIr, refs::CommandRef, refs::OutcomeRef, resolve::compile, source::SourceMap,
};
use ess_conformance::{
    report::Status, synthesize::Synthesis, target::*, AdmittedSuite, ConformanceScenario,
    ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const KEYPAD: &str = include_str!("fixtures/keypad.yaml");
const ALPHABET: &str = "0123456789*#ABCD";
const TOO_LONG: &str = "keypad.dial.SendKeys/outcome/too-long";
const SENT: &str = "keypad.dial.SendKeys/outcome/sent";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("keypad.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn with(model: &str, from: &str, to: &str) -> String {
    assert!(model.contains(from), "the model carries {from:?}");
    model.replacen(from, to, 1)
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> Option<&'a ConformanceScenario> {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

fn refusals_about(result: &Synthesis, id: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(ToString::to_string)
        .collect()
}

/// The text every `ExecuteCommand` of `command` in the scenario sent at `field`.
fn sent(scenario: &ConformanceScenario, command: &str, field: &str) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: executed,
                input,
                ..
            } if executed.to_string() == command => match input.get(field) {
                Some(ScenarioValue::Literal {
                    value: Node::Text(text),
                }) => Some(text.clone()),
                other => panic!("{field} is {other:?}"),
            },
            _ => None,
        })
        .collect()
}

fn keys(result: &Synthesis, id: &str) -> String {
    let scenario = scenario(&result.suite, id)
        .unwrap_or_else(|| panic!("no scenario {id}: {:?}", refusals_about(result, id)));
    let sent = sent(scenario, "keypad.dial.SendKeys", "keys");
    assert_eq!(sent.len(), 1, "{id} sends once: {sent:?}");
    sent[0].clone()
}

// ---- #103 and #104 together: the keypad ---------------------------------------------------------

#[test]
fn the_keypad_suite_witnesses_both_branches_with_sendable_keys() {
    let result = synthesis(KEYPAD);
    for id in [TOO_LONG, SENT] {
        assert!(
            refusals_about(&result, id).is_empty(),
            "{id}: {:?}",
            refusals_about(&result, id)
        );
    }
    let long = keys(&result, TOO_LONG);
    let short = keys(&result, SENT);
    assert_eq!(long.chars().count(), 65, "{long}");
    assert!(short.chars().count() <= 64, "{short}");
    for text in [&long, &short] {
        assert!(
            text.chars().all(|character| ALPHABET.contains(character)),
            "{text} holds a key the keypad cannot send"
        );
    }
    // The worked example: `keys` mapped into the alphabet, then cycled from its own start.
    assert_eq!(short, "#593");
    assert_eq!(long, format!("{}#", "#593".repeat(16)));
}

/// A keypad that sends only `0-9 * # A-D` and refuses a sequence longer than 64 keys — the
/// correct implementation #103 describes, which the generated suite used to fail.
struct Keypad {
    minted: Cell<u64>,
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Keypad {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("keypad", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        let Some(Node::Text(keys)) = request.input.get("keys") else {
            return Ok(SemanticCommandResult::undeclared().with_consistency(token));
        };
        let refused = keys.chars().count() > 64
            || keys.chars().any(|character| !ALPHABET.contains(character));
        let result = if refused {
            SemanticCommandResult::took(outcome(&command, "too-long")).with_error(
                DeclaredErrorValue::new("keypad.dial.UnsupportedKey".parse().unwrap()),
            )
        } else {
            SemanticCommandResult::took(outcome(&command, "sent")).emitting(
                ObservedEvent::new("keypad.dial.KeysSent".parse().unwrap())
                    .with("session_id", request.input["session_id"].clone())
                    .with("keys", Node::Text(keys.clone())),
            )
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(Vec::new()))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

fn failing(model: &str) -> Vec<String> {
    let suite = synthesis(model).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(
            &admitted,
            &Keypad {
                minted: Cell::new(0),
            },
        )
        .into_report();
    assert!(!report.scenarios.is_empty(), "the suite ran nothing");
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{}: {:?}", scenario.scenario, scenario.status))
        .collect()
}

#[test]
fn a_keypad_that_refuses_unsendable_keys_passes_its_generated_suite() {
    assert_eq!(failing(KEYPAD), Vec::<String>::new());
}

#[test]
fn the_example_is_the_base_and_the_length_ladder_resizes_it() {
    let model = with(
        KEYPAD,
        "      - {name: keys, type: keypad.dial.KeySequence}\n    outcomes:",
        "      - {name: keys, type: keypad.dial.KeySequence, example: \"12#\"}\n    outcomes:",
    );
    let result = synthesis(&model);
    assert_eq!(keys(&result, SENT), "12#");
    assert_eq!(keys(&result, TOO_LONG), format!("{}12", "12#".repeat(21)));
    assert_eq!(failing(&model), Vec::<String>::new());
}

#[test]
fn a_count_just_under_the_cap_is_witnessed_and_one_past_it_is_refused_by_name() {
    let model = with(KEYPAD, "when: keys.count > 64", "when: keys.count > 1023");
    let result = synthesis(&model);
    assert_eq!(keys(&result, TOO_LONG).chars().count(), 1024);

    let model = with(KEYPAD, "when: keys.count > 64", "when: keys.count > 5000");
    let result = synthesis(&model);
    assert!(scenario(&result.suite, TOO_LONG).is_none());
    let about = refusals_about(&result, TOO_LONG);
    assert!(
        about.iter().any(|refusal| refusal.contains("ESS-SYNTH-018")
            && refusal.contains("keys.count")
            && refusal.contains("1024")
            && refusal.contains("ess-scenario/1")),
        "{about:?}"
    );
    assert!(
        about
            .iter()
            .all(|refusal| !refusal.contains("drop it from the command's input")),
        "{about:?}"
    );
    let sent = keys(&result, SENT);
    assert_eq!(sent, "#593", "the default is still witnessed");
}

#[test]
fn a_type_that_reads_its_own_length_is_witnessed_by_a_resized_base() {
    let model = with(
        &with(
            KEYPAD,
            "    alphabet: \"0123456789*#ABCD\"\n",
            "    alphabet: \"0123456789*#ABCD\"\n    invariants: [value.count >= 8]\n",
        ),
        "      - name: too-long\n        when: keys.count > 64\n        error: keypad.dial.UnsupportedKey\n",
        "",
    );
    let result = synthesis(&model);
    assert_eq!(keys(&result, SENT), "#593#593");
}

#[test]
fn a_length_guard_and_a_prefix_guard_on_one_path_are_met_together() {
    let model = with(
        &with(KEYPAD, "    alphabet: \"0123456789*#ABCD\"\n", ""),
        "when: keys.count > 64",
        "when: {all: [keys.count > 3, keys: {starts_with: \"#\"}]}",
    );
    let result = synthesis(&model);
    let long = keys(&result, TOO_LONG);
    assert!(long.starts_with('#') && long.chars().count() > 3, "{long}");
}

// ---- lists ---------------------------------------------------------------------------------------

const TAGS: &str = r"format: ess/11
system: tagging
version: v1
domain: tagging.core
errors:
  - name: tagging.core.Refused
    fields: []
events:
  - name: tagging.core.Tagged
    fields: []
commands:
  - name: tagging.core.Tag
    input:
      - {name: tags, type: List<String>}
    outcomes:
      - name: refused
        when: GUARD
        error: tagging.core.Refused
      - name: tagged
        emits: [tagging.core.Tagged]
";

fn tags(result: &Synthesis, id: &str) -> Vec<Node> {
    let scenario = scenario(&result.suite, id)
        .unwrap_or_else(|| panic!("no scenario {id}: {:?}", refusals_about(result, id)));
    let sent: Vec<Vec<Node>> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => match input.get("tags") {
                Some(ScenarioValue::Literal {
                    value: Node::Seq(values),
                }) => Some(values.clone()),
                other => panic!("tags is {other:?}"),
            },
            _ => None,
        })
        .collect();
    assert_eq!(sent.len(), 1);
    sent[0].clone()
}

#[test]
fn a_list_count_above_one_is_witnessed_by_copies_of_the_first_element() {
    let refused = "tagging.core.Tag/outcome/refused";
    let result = synthesis(&TAGS.replace("GUARD", "tags.count > 2"));
    assert_eq!(tags(&result, refused).len(), 3);
    let result = synthesis(&TAGS.replace(
        "GUARD",
        "{all: [tags.count > 1, {forall: {in: tags, as: t, that: t == vip}}]}",
    ));
    assert_eq!(
        tags(&result, refused),
        vec![Node::Text("vip".into()), Node::Text("vip".into())]
    );
}

#[test]
fn the_list_ladders_unit_a1_built_keep_their_candidates() {
    // Characterization, written and green before the count ladder existed: the candidates for
    // the three guards unit A1 synthesizes are byte-equal after it.
    let expected = [
        ("tags.count > 0", r#"[{"tags":[]},{"tags":["tags.0"]}]"#),
        (
            "{exists: {in: tags, as: t, that: t == vip}}",
            r#"[{"tags":[]},{"tags":["tags.0"]},{"tags":["vip"]}]"#,
        ),
        ("tags.0 == vip", r#"[{"tags":["tags.0"]},{"tags":["vip"]}]"#),
    ];
    for (guard, candidates) in expected {
        let compiled = ir(&TAGS.replace("GUARD", guard));
        let command = &compiled.commands()[&"tagging.core.Tag".parse().unwrap()];
        let guards: Vec<_> = command
            .outcomes
            .iter()
            .filter_map(ess_conformance::when)
            .collect();
        let found = ess_conformance::witness::candidates(
            &compiled,
            command,
            &guards,
            ess_conformance::witness::Distinction::PLAIN,
        )
        .unwrap();
        assert_eq!(
            serde_json::to_string(&found).unwrap(),
            candidates,
            "{guard}"
        );
    }
}

// ---- an example at a further instance ------------------------------------------------------------

const SESSIONS: &str = r#"format: ess/11
system: keypad
version: v1
domain: keypad.dial
types:
  - name: keypad.dial.Label
    kind: newtype
    of: String
entities:
  - name: keypad.dial.Session
    identity: {name: session_id, type: Uuid}
    fields:
      - {name: label, type: keypad.dial.Label}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
events:
  - name: keypad.dial.Opened
    fields:
      - {name: session_id, type: Uuid}
commands:
  - name: keypad.dial.Open
    input:
      - {name: label, type: keypad.dial.Label, example: "Ann"}
    outcomes:
      - name: opened
        creates: keypad.dial.Session
        instance: session_id
        sets: {label: input.label}
        emits: [keypad.dial.Opened]
        payload:
          keypad.dial.Opened:
            session_id: {generated: true}
views:
  - name: keypad.dial.Sessions
    source: keypad.dial.Session
    consistency: read_your_writes
    order_by: [label asc]
    fields:
      - {name: session_id, type: Uuid}
      - {name: label, type: keypad.dial.Label}
"#;

#[test]
fn an_example_is_the_first_instance_and_a_further_instance_is_its_own() {
    let result = synthesis(SESSIONS);
    let scenario = scenario(&result.suite, "keypad.dial.Open/outcome/opened")
        .unwrap_or_else(|| panic!("{:?}", result.refusals));
    let labels = sent(scenario, "keypad.dial.Open", "label");
    assert!(
        labels.len() >= 2,
        "an ordered view arranges two instances: {labels:?}"
    );
    assert_eq!(labels[0], "Ann", "{labels:?}");
    assert!(labels[1..].iter().all(|label| label != "Ann"), "{labels:?}");
}

// ---- a `.count` no view row asserts ----------------------------------------------------------------

fn all_refusals(result: &Synthesis) -> Vec<String> {
    result.refusals.iter().map(ToString::to_string).collect()
}

#[test]
fn an_entity_invariant_over_a_published_field_s_count_is_unassertable_not_unpublished() {
    for (field, invariant) in [("label", "label.count >= 1"), ("tags", "tags.count >= 0")] {
        let model = SESSIONS
            .replace(
                "      - {name: label, type: keypad.dial.Label}\n    lifecycle:",
                &format!(
                    "      - {{name: label, type: keypad.dial.Label}}\n      - {{name: tags, type: \
                     List<String>}}\n    invariants: [{invariant}]\n    lifecycle:"
                ),
            )
            .replace(
                "      - {name: label, type: keypad.dial.Label, example: \"Ann\"}\n",
                "      - {name: label, type: keypad.dial.Label, example: \"Ann\"}\n      - {name: tags, type: List<String>}\n",
            )
            .replace(
                "        sets: {label: input.label}",
                "        sets: {label: input.label, tags: input.tags}",
            )
            + "      - {name: tags, type: List<String>}\n";
        let result = synthesis(&model);
        let about: Vec<String> = all_refusals(&result)
            .into_iter()
            .filter(|refusal| refusal.contains("ESS-SYNTH-011"))
            .collect();
        assert!(!about.is_empty(), "{field}: {:?}", all_refusals(&result));
        for refusal in &about {
            assert!(
                refusal.contains(&format!("`{field}.count` is a `.count`")),
                "{refusal}"
            );
            assert!(!refusal.contains("published by no view"), "{refusal}");
            assert!(
                refusal.contains("held to it where it is built, on command input and setup"),
                "{refusal}"
            );
        }
    }
}

#[test]
fn a_value_object_whose_invariant_reads_a_length_names_the_count() {
    let model = SESSIONS.replace(
        "  - name: keypad.dial.Label\n    kind: newtype\n    of: String\n",
        "  - name: keypad.dial.Label\n    kind: newtype\n    of: String\n    invariants: [value.count <= 64]\n",
    );
    let result = synthesis(&model);
    let about: Vec<String> = all_refusals(&result)
        .into_iter()
        .filter(|refusal| refusal.contains("ESS-SYNTH-013"))
        .collect();
    assert!(
        about
            .iter()
            .any(|refusal| refusal.contains("`value.count` is a `.count`")),
        "{:?}",
        all_refusals(&result)
    );
}
