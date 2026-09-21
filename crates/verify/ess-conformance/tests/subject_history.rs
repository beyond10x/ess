//! A successful answer and a backend bridge establish different session history.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
const MODEL: &str = include_str!("fixtures/subject-history.yaml");
#[test]
fn repeated_answer_is_arranged_from_observed_history_and_preserves_the_subject() {
    let raw = RawSpecFile::parse(MODEL).unwrap();
    let spec = Specification::assemble([(Source::new("history.yaml"), raw)]).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let synthesized = ess_conformance::synthesize::synthesize(&ir);
    assert!(
        synthesized.refusals.is_empty(),
        "{:?}",
        synthesized.refusals
    );
    let (_, scenario) = synthesized
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "calls.core.Answer/outcome/already-answered")
        .unwrap();
    let bytes = serde_json::to_string(&scenario.steps).unwrap();
    assert!(bytes.contains("answer_history"), "{bytes}");
    assert!(bytes.contains("expect_no_error"), "{bytes}");
    assert!(
        !bytes.contains("configure_external_outcome"),
        "history is an observed subject fact"
    );
}

fn synth(model: &str) -> ess_conformance::synthesize::Synthesis {
    let raw = RawSpecFile::parse(model).unwrap();
    let spec = Specification::assemble([(Source::new("history.yaml"), raw)]).unwrap();
    ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap())
}

#[test]
fn missing_history_or_incomplete_preservation_is_refused() {
    for model in [
        MODEL.replace("sets: {answer_history: Unanswered, note: input.note}", "sets: {note: input.note}"),
        MODEL.replace("      - {name: answer_history, type: calls.core.AnswerHistory}\n      - {name: note, type: String}\n", "      - {name: note, type: String}\n"),
        MODEL.replace("      - {name: note, type: String}\n", ""),
    ] {
        // Remove only observation columns; preserve entity declarations.
        let view_start = MODEL.find("views:").unwrap();
        let changed_view = &model[model.find("views:").unwrap()..];
        let test_model = if model.contains("sets: {note: input.note}") { model.clone() } else { format!("{}{}", &MODEL[..view_start], changed_view) };
        let result = synth(&test_model);
        assert!(result.refusals.iter().any(|refusal| format!("{refusal:?}").contains("already-answered")), "{:?}", result.refusals);
    }
}

#[test]
fn history_semantics_refuse_legacy_sources_and_invalid_facts() {
    for model in (1..=5)
        .map(|version| MODEL.replace("ess/6", &format!("ess/{version}")))
        .chain([
            MODEL.replace(
                "field: answer_history, equals: Answered",
                "field: absent, equals: Answered",
            ),
            MODEL.replace("equals: Answered", "equals: Unknown"),
            MODEL.replace(
                "preserves: calls.core.Call",
                "preserves: calls.core.Call\n        sets: {note: changed}",
            ),
        ])
    {
        let parsed = RawSpecFile::parse(&model);
        assert!(
            parsed.is_err()
                || Specification::assemble([(Source::new("bad.yaml"), parsed.unwrap())]).is_err()
        );
    }
}

fn runtime_suite() -> ess_conformance::ConformanceSuite {
    use ess_conformance::scenario::{CommandRef, OutcomeRef, ScenarioId, ScenarioStep};
    let mut suite = synth(MODEL).suite;
    // Keep two complementary arrangements: a successful answer establishes
    // history; a backend report reaches the same lifecycle without doing so.
    let mut reported = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "calls.core.Answer/outcome/answered")
        .unwrap()
        .1
        .clone();
    let at = reported.steps.iter().position(|step| matches!(step, ScenarioStep::ExecuteCommand { command, .. } if command.to_string() == "calls.core.Answer")).unwrap();
    let ScenarioStep::ExecuteCommand { input, .. } = &reported.steps[at] else {
        unreachable!()
    };
    let command = CommandRef::new("calls.core.Report".parse().unwrap());
    reported.steps.splice(
        at..at,
        [
            ScenarioStep::ExecuteCommand {
                command: command.clone(),
                actor: None,
                input: input.clone(),
            },
            ScenarioStep::ExpectOutcome {
                outcome: OutcomeRef::new(command, "observed".parse().unwrap()),
            },
        ],
    );
    suite
        .insert(
            ScenarioId::parse("calls.core/authored/backend-bridged-but-unanswered").unwrap(),
            reported,
        )
        .unwrap();
    suite
}

#[test]
fn emitted_runtimes_observe_history_no_error_and_real_row_preservation() {
    let suite = runtime_suite();
    let root = std::env::temp_dir().join(format!("ess-subject-history-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    for artifact in ess_conformance::go::emit(&suite).unwrap() {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/history\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        root.join("essconform/history_test.go"),
        include_str!("fixtures/subject-history-runtime.go"),
    )
    .unwrap();
    for artifact in ess_conformance::ts::emit(&suite).unwrap() {
        let path = root.join("typescript").join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let ts = root.join("typescript/essconform");
    std::fs::write(
        ts.join("history.mjs"),
        include_str!("fixtures/subject-history-runtime.mjs"),
    )
    .unwrap();
    std::fs::write(
        ts.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = std::process::Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&ts)
        .output()
        .unwrap();
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stdout)
    );
    for mutant in [
        "",
        "error",
        "rewrite",
        "lost_history",
        "event",
        "state_only",
        "duplicate",
        "missing",
    ] {
        for (tool, args, directory) in [
            ("go", vec!["test", "./essconform", "-count=1", "-v"], &root),
            ("node", vec!["--test", "history.mjs"], &ts),
        ] {
            let output = std::process::Command::new(tool)
                .args(args)
                .env("ESS_HISTORY_MUTANT", mutant)
                .env("ESS_REPORT_FORMAT", "2")
                .env("GOWORK", "off")
                .current_dir(directory)
                .output()
                .unwrap();
            let log = format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert_eq!(
                output.status.success(),
                mutant.is_empty(),
                "{tool} mutant={mutant}: {log}"
            );
            if !mutant.is_empty() {
                assert!(
                    log.contains("already-answered")
                        || log.contains("backend-bridged-but-unanswered"),
                    "{log}"
                );
            }
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_same_state_update_can_establish_new_history() {
    let model = MODEL
        .replace(
            "        - {name: bridge, from: [Init, Bridged], to: Bridged}\n",
            "",
        )
        .replace("moves: calls.core.Call.bridge", "updates: calls.core.Call")
        .replacen(
            "      - name: gone\n        wrong_state: true\n        error: calls.core.Gone\n",
            "",
            1,
        );
    let result = synth(&model);
    assert!(result.refusals.is_empty(), "{:?}", result.refusals);
    let repeat = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "calls.core.Answer/outcome/already-answered")
        .unwrap()
        .1;
    let answers = repeat.steps.iter().filter(|step| matches!(step, ess_conformance::ScenarioStep::ExecuteCommand {command,..} if command.to_string() == "calls.core.Answer")).count();
    assert_eq!(
        answers, 2,
        "the unchanged lifecycle must not collapse distinct answer histories"
    );
}

#[test]
fn older_suite_formats_refuse_snapshot_and_no_error_steps() {
    let suite = runtime_suite();
    for version in 1..=9 {
        let mut value = serde_json::to_value(&suite).unwrap();
        value["provenance"]["suite_version"] =
            serde_json::json!(format!("ess-conformance/{version}"));
        assert!(ess_conformance::AdmittedSuite::from_json(&value.to_string()).is_err());
    }
}
