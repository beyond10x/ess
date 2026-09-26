//! A generated seam can answer an unknown instance (`docs/design/unknown-instance-seams.md`).
//!
//! The unknown-instance rule answers a command whose `instance:` names no record with its
//! `wrong_state` outcome, and a seam whose only spelling of that outcome demands the error's fields
//! cannot say it. Where [`unknown_instance_answer`] says so, the Rust and Go seams gain a second
//! spelling carrying nothing; everywhere else they emit what they emitted before.
//!
//! The class is every command of every model, so the first case walks every example and checks
//! both targets against the one predicate, rather than naming the commands it knows about.

use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_compiler::{resolve::compile, source::SourceMap as Sources};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::unknown_instance::unknown_instance_answer;
use ess_synth::{synthesize_for, Target};

/// Every example directory holding a system.
fn examples() -> Vec<(String, EssIr)> {
    let root: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .canonicalize()
        .expect("the examples exist");
    let mut found = Vec::new();
    let mut names: Vec<_> = std::fs::read_dir(&root)
        .expect("the examples are readable")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.join("system.yaml").is_file())
        .collect();
    names.sort();
    for base in names {
        let mut labels = Vec::new();
        let mut pending = vec![base.clone()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(&directory).expect("the example is readable") {
                let path = entry.expect("an entry").path();
                if path.is_dir() {
                    pending.push(path);
                } else if path.extension().is_some_and(|it| it == "yaml") {
                    labels.push(
                        path.strip_prefix(&base)
                            .expect("inside the example")
                            .display()
                            .to_string(),
                    );
                }
            }
        }
        labels.sort();
        let mut sources = SourceMap::new();
        let mut parsed = Vec::new();
        for label in &labels {
            let text = std::fs::read_to_string(base.join(label)).expect("readable");
            let raw = RawSpecFile::parse(&text).expect("well formed");
            sources.insert(label.clone(), text);
            parsed.push((Source::new(label.clone()), raw));
        }
        let specification = Specification::assemble(parsed).expect("the example validates");
        let ir = compile_locating(&specification, &sources, &labels).expect("it resolves");
        let name = base
            .file_name()
            .expect("a directory name")
            .to_string_lossy()
            .into_owned();
        found.push((name, ir));
    }
    found
}

/// Every artifact of one target, joined.
fn emitted(ir: &EssIr, target: Target) -> String {
    synthesize_for(ir, target)
        .unwrap_or_else(|failure| panic!("the fixture synthesizes: {failure:?}"))
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

/// The Rust outcome enum of one command, from its declaration to its closing brace.
fn rust_outcome_enum<'a>(code: &'a str, command: &str) -> &'a str {
    let head = format!("pub enum {command}Outcome {{");
    let start = code
        .find(&head)
        .unwrap_or_else(|| panic!("the Rust types declare `{head}`"));
    let body = &code[start..];
    &body[..body.find("\n}\n").expect("the enum closes")]
}

#[test]
fn every_example_command_gains_the_second_spelling_exactly_where_the_answer_says_so() {
    let mut some = 0;
    let mut none = 0;
    for (example, ir) in examples() {
        let rust = emitted(&ir, Target::Rust);
        let go = emitted(&ir, Target::Go);
        for command in ir.commands().values() {
            let ty = command
                .name
                .segments()
                .last()
                .expect("a qualified name has a last segment")
                .clone();
            let answer = unknown_instance_answer(&ir, command);
            let spelled = answer.map(|declared| {
                let words: String = declared
                    .name
                    .as_str()
                    .split('-')
                    .map(|word| {
                        let mut chars = word.chars();
                        chars.next().map_or_else(String::new, |first| {
                            first.to_uppercase().chain(chars).collect()
                        })
                    })
                    .collect();
                format!("{words}UnknownInstance")
            });
            let in_rust = rust_outcome_enum(&rust, &ty).contains("UnknownInstance,");
            let in_go = go.contains(&format!("type {ty}Outcome"))
                && go.contains(&format!(
                    "{ty}Outcome{} struct{{}}",
                    spelled.as_deref().unwrap_or("WrongStateUnknownInstance")
                ));
            assert_eq!(
                (in_rust, in_go),
                (answer.is_some(), answer.is_some()),
                "`{}` in `{example}`: the unknown-instance spelling must exist in both targets \
                 exactly when the answer is `Some`",
                command.name
            );
            if answer.is_some() {
                some += 1;
            } else {
                none += 1;
            }
        }
    }
    // Both halves of the class are exercised by the committed examples, or this case proves half.
    assert!(
        some >= 5,
        "billing and gatepass hold five such commands, found {some}"
    );
    assert!(
        none >= 1,
        "some example command must keep its bytes, found {none}"
    );
}

#[test]
fn a_model_where_the_case_cannot_arise_emits_nothing_of_it() {
    // `oracle-fixture` declares `wrong_state` on every moving command, and its error carries no
    // field — the declared spelling already says the answer.
    let (_, ir) = examples()
        .into_iter()
        .find(|(name, _)| name == "oracle-fixture")
        .expect("the oracle fixture is an example");
    assert!(ir
        .commands()
        .values()
        .all(|command| unknown_instance_answer(&ir, command).is_none()));
    for target in [Target::Rust, Target::Go, Target::Web] {
        let code = emitted(&ir, target);
        assert!(
            !code.contains("UnknownInstance") && !code.contains("no record carries"),
            "{target:?} emitted an unknown-instance spelling for a model that needs none"
        );
    }
}

const DOORS: &str = r"
format: ess/4
system: doors
version: v1
domain: doors.core
entities:
  - name: doors.core.Door
    identity: {name: door_id, type: Uuid}
    fields: []
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
events:
  - name: doors.core.Installed
    fields: [{name: door_id, type: Uuid}]
  - name: doors.core.Closed
    fields: [{name: door_id, type: Uuid}]
errors:
  - name: doors.core.DoorStateConflict
    fields:
      - {name: state, type: doors.core.Door.State}
commands:
  - name: doors.core.Install
    input:
      - {name: door_id, type: Uuid}
    outcomes:
      - name: installed
        creates: doors.core.Door
        instance: door_id
        emits: [doors.core.Installed]
        payload:
          doors.core.Installed: {door_id: input.door_id}
  - name: doors.core.Close
    input:
      - {name: door_id, type: Uuid}
    outcomes:
      - name: closed
        moves: doors.core.Door.close
        instance: door_id
        emits: [doors.core.Closed]
        payload:
          doors.core.Closed: {door_id: input.door_id}
      - name: wrong-state
        wrong_state: true
        error: doors.core.DoorStateConflict
components:
  - component: door-service
    owns: {domains: [doors.core]}
    accepts: {commands: [doors.core.Install, doors.core.Close]}
    publishes: {events: [doors.core.Installed, doors.core.Closed]}
    reached_by: network
";

fn doors(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("doors.yaml"),
        RawSpecFile::parse(source).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = Sources::new();
    sources.insert("doors.yaml", source);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("{errors}"))
}

#[test]
fn every_rust_match_over_the_outcome_names_the_second_spelling() {
    let code = emitted(&doors(DOORS), Target::Rust);
    assert!(rust_outcome_enum(&code, "Close").contains("    WrongStateUnknownInstance,"));
    // The component port publishes nothing for it, the system record and the served answer both
    // carry the branch and the error and no payload.
    assert!(code.contains("CloseOutcome::WrongStateUnknownInstance => {}"));
    let served = code
        .split("CloseOutcome::WrongStateUnknownInstance => {\n")
        .skip(1)
        .map(|arm| &arm[..arm.find("        }\n").expect("the arm closes")])
        .collect::<Vec<_>>();
    assert_eq!(
        served.len(),
        2,
        "the wire record and the served answer: {served:#?}"
    );
    for arm in &served {
        assert!(
            arm.contains("\"wrong-state\"") && arm.contains("\"doors.core.DoorStateConflict\"")
        );
        assert!(
            !arm.contains("payload"),
            "no payload for a door that does not exist: {arm}"
        );
    }
    assert!(
        served.iter().any(|arm| arm.contains("409")),
        "the branch's own status"
    );
}

#[test]
fn every_go_switch_over_the_outcome_names_the_second_spelling() {
    let code = emitted(&doors(DOORS), Target::Go);
    assert!(code.contains("type CloseOutcomeWrongStateUnknownInstance struct{}"));
    assert!(code.contains("func (CloseOutcomeWrongStateUnknownInstance) isCloseOutcome() {}"));
    let answered = code
        .split("case core.CloseOutcomeWrongStateUnknownInstance:\n")
        .skip(1)
        .collect::<Vec<_>>();
    assert_eq!(
        answered.len(),
        2,
        "the component port and the served answer"
    );
    let served = answered
        .iter()
        .find(|arm| arm.starts_with("\t\tbody["))
        .expect("the served answer");
    let served = &served[..served.find("return").expect("the arm returns")];
    assert!(
        served.contains("\"wrong-state\"") && served.contains("\"doors.core.DoorStateConflict\"")
    );
    assert!(
        !served.contains("payload"),
        "no payload for a door that does not exist"
    );
}

#[test]
fn an_outcome_spelling_the_same_rust_identifier_is_a_collision_not_a_shadow() {
    let source = DOORS
        .replace(
            "  - name: doors.core.Close\n    input:\n      - {name: door_id, type: Uuid}\n",
            "  - name: doors.core.Close\n    input:\n      - {name: door_id, type: Uuid}\n      - \
             {name: force, type: Integer}\n",
        )
        .replace(
            "      - name: wrong-state\n",
            "      - name: wrong-state-unknown-instance\n        when: force > 10\n        error: \
             doors.core.DoorStateConflict\n      - name: wrong-state\n",
        );
    let ir = doors(&source);
    let failure = synthesize_for(&ir, Target::Rust)
        .err()
        .expect("two outcome variants spelled alike refuse");
    let json: serde_json::Value = serde_json::from_str(&failure.to_canonical_json()).unwrap();
    assert!(
        json["causes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cause| cause["code"] == "symbol-collision"
                && cause.to_string().contains("WrongStateUnknownInstance")),
        "{json:#}"
    );
    // Go allocates names rather than refusing: the declared outcome keeps its name and the
    // generated one yields.
    let go = emitted(&ir, Target::Go);
    assert!(go.contains("type CloseOutcomeWrongStateUnknownInstance struct {"));
    assert!(go.contains("type CloseOutcomeWrongStateUnknownInstance_ struct{}"));
}
