//! The generated Go and TypeScript runners admit a suite whose shapes hold `json`, and hold a `Json`
//! value as the Rust reference runner does: structurally, so a wrong object fails its scenario.
//!
//! A specification with a `Json` field synthesized a suite writing `"kind": "json"`, and both
//! generated runners refused that suite at admission (`unknown primitive`): their primitive lists
//! stopped at `bytes` (story:go-and-ts-runners-admit-json-shapes).

mod support_go;

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use ess_compiler::refs::OutcomeRef;
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::ConformanceSuite;
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const SENT: &str = "demo.msgs.Send/outcome/sent";

const MODEL: &str = "format: ess/15
system: demo
version: v1
domain: demo.msgs
types:
  - {name: demo.msgs.Body, kind: newtype, of: Json}
events:
  - name: demo.msgs.Sent
    fields:
      - {name: body, type: demo.msgs.Body}
      - {name: headers, type: Json}
actors:
  - {name: demo.msgs.Sender, may: [demo.msgs.Send]}
commands:
  - name: demo.msgs.Send
    input:
      - {name: body, type: demo.msgs.Body}
      - {name: headers, type: Json}
    outcomes:
      - name: sent
        emits: [demo.msgs.Sent]
        payload:
          demo.msgs.Sent: {body: input.body, headers: input.headers}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("msgs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(MODEL));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let written = serde_json::to_string(&synthesis.suite).unwrap();
    assert!(
        written.contains(r#""kind":"json""#),
        "the suite holds a json shape: {written}"
    );
    synthesis.suite
}

/// What the target publishes for the `Sent` event.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Mode {
    /// Both objects exactly as sent.
    Correct,
    /// `headers` keeps its key and carries another value under it.
    WrongValue,
    /// `headers` gains a key the input did not carry.
    ExtraKey,
}

impl Mode {
    const ALL: [Self; 3] = [Self::Correct, Self::WrongValue, Self::ExtraKey];

    fn env(self) -> &'static str {
        match self {
            Self::Correct => "correct",
            Self::WrongValue => "wrong-value",
            Self::ExtraKey => "extra-key",
        }
    }
}

struct Messages(Mode);

impl ConformanceTarget for Messages {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("json-fixture", "1"))
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
        let body = request.input.get("body").cloned().unwrap_or(Node::Null);
        let mut headers = request.input.get("headers").cloned().unwrap_or(Node::Null);
        match (self.0, &mut headers) {
            (Mode::Correct, _) => {}
            (Mode::WrongValue, Node::Map(entries)) => {
                for value in entries.values_mut() {
                    *value = Node::Text("not-what-was-sent".to_owned());
                }
            }
            (Mode::ExtraKey, Node::Map(entries)) => {
                entries.insert("extra".to_owned(), Node::Bool(true));
            }
            (mode, other) => {
                panic!("{mode:?}: the witness of a Json input is an object: {other:?}")
            }
        }
        let mut event = ObservedEvent::new("demo.msgs.Sent".parse().unwrap());
        event.payload =
            BTreeMap::from([("body".to_owned(), body), ("headers".to_owned(), headers)]);
        Ok(SemanticCommandResult::took(OutcomeRef::new(
            request.command.clone(),
            OutcomeName::new("sent").unwrap(),
        ))
        .emitting(event))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("views", "the model declares none"))
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

/// The Rust reference verdicts: the correct target passes, and each wrong object fails `sent`.
fn reference(suite: &ConformanceSuite) -> BTreeMap<Mode, BTreeMap<String, String>> {
    Mode::ALL
        .into_iter()
        .map(|mode| {
            let verdicts = support_go::rust_outcomes(suite, &Messages(mode));
            assert!(verdicts.contains_key(SENT), "{verdicts:?}");
            assert_eq!(
                support_go::not_passed(&verdicts).is_empty(),
                mode == Mode::Correct,
                "Rust, {mode:?}: {verdicts:?}"
            );
            (mode, verdicts)
        })
        .collect()
}

#[test]
fn go_admits_a_json_shape_and_holds_the_value_as_the_reference_runner_does() {
    let suite = suite();
    let reference = reference(&suite);
    for mode in Mode::ALL {
        let verdicts =
            support_go::assert_parity(&format!("json-{}", mode.env()), &suite, Messages(mode));
        assert_eq!(verdicts, reference[&mode], "Go, {mode:?}");
    }
}

/// The TypeScript twin of [`Messages`], switched by `ESS_JSON_MODE`.
const TYPESCRIPT_TARGET: &str = r"
import test from 'node:test';
import { run } from './dist/runtime.js';
class Messages {
  identity() { return {name: 'json-fixture', version: '1'}; }
  beginScenario() {}
  endScenario() {}
  configureExternalOutcome() { throw Error('nothing here is externally decided'); }
  executeCommand({input}) {
    const body = input.body ?? null;
    const headers = JSON.parse(JSON.stringify(input.headers ?? null));
    switch (process.env.ESS_JSON_MODE) {
      case 'correct':
        break;
      case 'wrong-value':
        for (const key of Object.keys(headers)) {
          headers[key] = 'not-what-was-sent';
        }
        break;
      case 'extra-key':
        headers.extra = true;
        break;
      default:
        throw Error(`unknown mode ${process.env.ESS_JSON_MODE}`);
    }
    return {
      outcome: 'sent',
      directEvents: [{event: 'demo.msgs.Sent', payload: {body, headers}}],
    };
  }
  queryView() { throw Error('the model declares no views'); }
  observeEvents() { return []; }
  redeliverEvent() { throw Error('no bindings'); }
  observeInvocations() { throw Error('no bindings'); }
}
await test('json', t => run(t, () => new Messages()));
";

/// Writes and compiles the emitted TypeScript package for `suite` under `root`.
fn typescript_package(root: &Path, suite: &ConformanceSuite) -> std::path::PathBuf {
    for artifact in ess_conformance::ts::emit(suite).unwrap_or_else(|error| panic!("{error}")) {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let package = root.join(ess_conformance::ts::PACKAGE);
    std::fs::write(
        package.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&package)
        .output()
        .expect("required TypeScript compiler");
    assert!(
        compiled.status.success(),
        "{}{}",
        String::from_utf8_lossy(&compiled.stdout),
        String::from_utf8_lossy(&compiled.stderr)
    );
    package
}

#[test]
fn typescript_admits_a_json_shape_and_holds_the_value_as_the_reference_runner_does() {
    let suite = suite();
    let reference = reference(&suite);
    let root = support_go::Scratch::new(format!("ess-json-typescript-{}", std::process::id()));
    let package = typescript_package(&root, &suite);
    std::fs::write(package.join("json.mjs"), TYPESCRIPT_TARGET).unwrap();
    let report = package.join("report.json");
    for mode in Mode::ALL {
        let _ = std::fs::remove_file(&report);
        let output = Command::new("node")
            .args(["--test", "json.mjs"])
            .env("ESS_JSON_MODE", mode.env())
            .env("ESS_REPORT_FORMAT", "2")
            .env("ESS_REPORT_OUT", &report)
            .current_dir(&package)
            .output()
            .expect("required Node runtime");
        let log = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            !log.contains("suite admission"),
            "TypeScript, {mode:?}: the runner refused its own suite:\n{log}"
        );
        let document: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&report).unwrap_or_else(|error| {
                panic!("TypeScript, {mode:?}: no report ({error}):\n{log}")
            }))
            .unwrap();
        let mut verdicts = BTreeMap::new();
        for (status, ids) in document["outcomes"].as_object().unwrap() {
            for id in ids.as_array().unwrap() {
                verdicts.insert(id.as_str().unwrap().to_owned(), status.clone());
            }
        }
        assert_eq!(verdicts, reference[&mode], "TypeScript, {mode:?}:\n{log}");
        assert_eq!(
            output.status.success(),
            mode == Mode::Correct,
            "TypeScript, {mode:?}:\n{log}"
        );
    }
}
