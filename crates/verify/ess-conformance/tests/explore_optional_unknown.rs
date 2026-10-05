//! Optional inputs and missing subjects must be explored by both actual emitted runtimes.

mod support_explore_optional_unknown;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;
use std::{fmt::Write as _, fs, path::Path, process::Command, sync::OnceLock};

fn compiled(source: &str) -> EssIr {
    let raw = RawSpecFile::parse(source).expect("fixture parses");
    let spec = Specification::assemble([(Source::new("optional.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("source admission: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("optional.yaml", source);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compile: {errors}"))
}

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn run(command: &mut Command) {
    let output = command.output().expect("required runtime tool");
    assert!(
        output.status.success(),
        "{command:?}\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn lane(label: &str) -> Value {
    run_lane(
        label,
        "baseline",
        include_str!("support_explore_optional_unknown/baseline.yaml"),
        support_explore_optional_unknown::GO,
        support_explore_optional_unknown::TS,
    )
}

fn run_lane(label: &str, case: &str, source: &str, go: &str, ts: &str) -> Value {
    let root = std::env::temp_dir().join(format!(
        "ess-optional-unknown-{}-{label}-{case}",
        std::process::id()
    ));
    fs::create_dir_all(&root).unwrap();
    let ir = compiled(source);
    let synthesis = ess_conformance::synthesize(&ir);
    if case == "finite" {
        assert_eq!(synthesis.refusals.len(), 1, "{:?}", synthesis.refusals);
        assert!(synthesis.refusals[0]
            .to_string()
            .contains("too few values to name an identity"));
    } else if matches!(case, "recursive" | "depth-32") {
        // This fixture crosses the finite-witness resource boundary. Since finite recursive typed
        // input fixtures are admitted (beyond10x/ess#416) the fixed suite may refuse none of it;
        // whatever it still refuses names the recursion.
        assert!(synthesis.refusals.len() <= 2, "{:?}", synthesis.refusals);
        assert!(synthesis
            .refusals
            .iter()
            .all(|refusal| refusal.to_string().contains("refers to itself")));
    } else {
        assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    }
    let mut suite = synthesis.suite;
    suite.select_fresh_format_for(&ir);
    let artifacts: Vec<_> = if label == "go" {
        ess_conformance::go::emit_with_model(&suite, &ir)
            .unwrap()
            .into_iter()
            .map(|a| (a.path, a.contents))
            .collect()
    } else {
        ess_conformance::ts::emit_with_model(&suite, &ir)
            .unwrap()
            .into_iter()
            .map(|a| (a.path, a.contents))
            .collect()
    };
    let package = root.join(label);
    for (path, contents) in artifacts {
        write(&package.join(path), &contents);
    }
    let runtime = package.join("essconform");
    if label == "go" {
        write(
            &package.join("go.mod"),
            "module example.invalid/optional\n\ngo 1.24\n",
        );
        write(&runtime.join("probe_test.go"), go);
        run(Command::new("go")
            .args([
                "test",
                "./essconform",
                "-run",
                "TestProbe",
                "-count=1",
                "-v",
            ])
            .env("GOWORK", "off")
            .current_dir(&package));
    } else {
        write(&runtime.join("probe.mjs"), ts);
        write(
            &runtime.join("probe.tsconfig.json"),
            r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
        );
        run(Command::new("tsc")
            .args(["--project", "probe.tsconfig.json"])
            .current_dir(&runtime));
        run(Command::new("node").arg("probe.mjs").current_dir(&runtime));
    }
    serde_json::from_slice(&fs::read(root.join(format!("{label}-result.json"))).unwrap()).unwrap()
}

fn verify(label: &str) {
    let result = result(label);
    assert!(result["healthy_failure"].is_null(), "{result}");
    assert_eq!(
        result["control_detected"], true,
        "plain mutant must be detected"
    );
    for command in ["Create", "PlainNote", "OptionalNote", "Close"] {
        assert!(
            result["calls"][format!("demo.items.{command}")]
                .as_u64()
                .unwrap_or(0)
                > 0,
            "{label}: {command} was never called: {result}"
        );
    }
    assert_eq!(result["result"]["excluded"], serde_json::json!([]));
    for name in [
        "absent",
        "present",
        "missing-outcome",
        "missing-error",
        "missing-write",
        "input-precedence",
    ] {
        assert!(
            result["mutants"][name]["failure"].is_object(),
            "{label}: {name}: {result}"
        );
    }
}

fn result(label: &str) -> &'static Value {
    static GO: OnceLock<Value> = OnceLock::new();
    static TS: OnceLock<Value> = OnceLock::new();
    (if label == "go" { &GO } else { &TS }).get_or_init(|| lane(label))
}

#[test]
fn emitted_lanes_share_exact_draws_outcomes_and_mutant_replays() {
    let go = result("go");
    let ts = result("ts");
    for field in ["draws", "result", "mutants", "control"] {
        assert_eq!(go[field], ts[field], "lane disagreement for {field}");
    }
}

#[test]
fn precondition_known_absence_does_not_depend_on_random_draw_capability() {
    use support_explore_optional_unknown::precondition;
    let go = run_lane(
        "go",
        "precondition",
        precondition::SOURCE,
        precondition::GO,
        precondition::TS,
    );
    let ts = run_lane(
        "ts",
        "precondition",
        precondition::SOURCE,
        precondition::GO,
        precondition::TS,
    );
    assert_eq!(go, ts);
    assert!(go["healthy"]["failure"].is_null(), "{go}");
    assert_eq!(go["healthyCalls"], 1);
    assert_eq!(go["brokenCalls"], 1);
    assert_eq!(
        go["refused"], true,
        "fabricated absent precondition payload must fail setup"
    );
}

#[test]
fn optional_kinds_nested_presence_and_defined_have_actual_paired_witnesses() {
    use support_explore_optional_unknown::matrix;
    let source = matrix::source();
    let go = run_lane("go", "matrix", &source, matrix::GO, matrix::TS);
    let ts = run_lane("ts", "matrix", &source, matrix::GO, matrix::TS);
    assert_eq!(go, ts, "exact draws, judgments and shrunk replay parity");
    for lane in [&go, &ts] {
        assert!(lane["result"]["failure"].is_null(), "{lane}");
        assert_eq!(lane["result"]["excluded"], serde_json::json!([]));
        assert_eq!(lane["result"]["unreached"], serde_json::json!([]));
        for name in ["branch", "fabricate", "policy", "extra"] {
            assert!(
                lane["mutants"][name]["failure"].is_object(),
                "{name}: {lane}"
            );
        }
        let draws = lane["draws"].as_array().unwrap();
        for field in [
            "gate",
            "number",
            "text",
            "uuid",
            "choice",
            "label",
            "nested",
            "maybe_box",
        ] {
            assert!(
                draws.iter().any(|draw| draw["input"][field].is_null()),
                "absent {field}"
            );
            assert!(
                draws.iter().any(|draw| !draw["input"][field].is_null()),
                "present {field}"
            );
        }
        assert!(
            draws.iter().any(|draw| draw["input"]["gate"] == false),
            "false is present"
        );
    }
}

#[test]
fn defined_distinguishes_absence_from_zero_empty_text_and_empty_struct() {
    use support_explore_optional_unknown::matrix;
    for (kind, witness) in [
        ("Integer", serde_json::json!(0)),
        ("String", serde_json::json!("")),
        ("demo.values.Empty", serde_json::json!({})),
    ] {
        let source = matrix::source()
            .replace("format: ess/15", "format: ess/16")
            .replace(
                "name: gate, type: Optional<Boolean>",
                &format!("name: gate, type: Optional<{kind}>"),
            )
            .replace(
                "types:\n",
                "types:\n  - {name: demo.values.Empty, kind: struct, fields: [{name: member, type: Optional<Boolean>, presence: omitted_when_absent}]}\n",
            );
        let go = run_lane("go", kind, &source, matrix::GO, matrix::TS);
        let ts = run_lane("ts", kind, &source, matrix::GO, matrix::TS);
        assert_eq!(go, ts);
        assert!(go["result"]["failure"].is_null(), "{go}");
        assert_eq!(go["result"]["unreached"], serde_json::json!([]));
        assert!(
            go["draws"]
                .as_array()
                .unwrap()
                .iter()
                .any(|draw| draw["input"]["gate"] == witness),
            "{kind} did not draw {witness}"
        );
        assert!(go["mutants"]["branch"]["failure"].is_object());
    }
}

#[test]
fn external_redirects_keep_arrangement_and_upserts_keep_creation_effects() {
    use support_explore_optional_unknown::subjects;
    for upsert in [false, true] {
        let source = subjects::source(upsert);
        let go_driver = subjects::GO.replace("UPSERT", &upsert.to_string());
        let ts_driver = subjects::TS.replace("UPSERT", &upsert.to_string());
        let case = if upsert { "upsert" } else { "external" };
        let go = run_lane("go", case, &source, &go_driver, &ts_driver);
        let ts = run_lane("ts", case, &source, &go_driver, &ts_driver);
        assert_eq!(go["calls"], ts["calls"], "{case} callback parity");
        assert_eq!(go["result"], ts["result"], "{case} result parity");
        for lane in [&go, &ts] {
            assert!(lane["result"]["failure"].is_null(), "{case}: {lane}");
            assert_eq!(lane["result"]["excluded"], serde_json::json!([]));
            assert_eq!(lane["result"]["unreached"], serde_json::json!([]));
            for mutant in ["view-absence", "view-present"] {
                assert!(
                    lane["mutants"][mutant]["failure"].is_object(),
                    "{mutant}: {lane}"
                );
            }
            if upsert {
                for mutant in ["create", "update"] {
                    assert!(
                        lane["mutants"][mutant]["failure"].is_object(),
                        "{mutant}: {lane}"
                    );
                    assert_eq!(go["mutants"][mutant], ts["mutants"][mutant]);
                }
            } else {
                assert!(lane["mutants"]["ignore"]["failure"].is_object(), "{lane}");
                assert!(
                    lane["mutants"]["unarrangeable"]["failure"].is_null(),
                    "{lane}"
                );
                assert_eq!(go["mutants"]["ignore"], ts["mutants"]["ignore"]);
            }
        }
    }
}

#[test]
fn fresh_draw_collisions_preserve_command_reachability_and_runtime_parity() {
    use support_explore_optional_unknown::subjects;
    // The String draw pool has three values. Once all are rows, fresh draws must retry/skip;
    // an existing draw and the independent creator must remain usable.
    let source = subjects::source(true).replace("of: Uuid", "of: String");
    let go_driver = subjects::GO.replace("UPSERT", "true");
    let ts_driver = subjects::TS.replace("UPSERT", "true");
    let go = run_lane("go", "collisions", &source, &go_driver, &ts_driver);
    let ts = run_lane("ts", "collisions", &source, &go_driver, &ts_driver);
    assert_eq!(go["calls"], ts["calls"]);
    assert_eq!(go["result"], ts["result"]);
    assert!(go["result"]["failure"].is_null(), "{go}");
    assert_eq!(go["result"]["executed"], 256);
    assert_eq!(go["result"]["unreached"], serde_json::json!([]));
    let calls = go["calls"].as_array().unwrap();
    for id in ["", "a", "b"] {
        assert!(calls.iter().any(|call| call["input"]["id"] == id));
    }
}

#[test]
fn optional_preserves_precise_unsupported_inner_and_existing_recursion_limit() {
    use support_explore_optional_unknown::matrix;
    let original = matrix::source();
    let list = original.replace("Optional<Integer>", "Optional<List<Integer>>");
    let recursive = original.replace("types:\n", "types:\n  - name: demo.values.Loop\n    kind: struct\n    fields: [{name: next, type: Optional<demo.values.Loop>}]\n")
        .replace("Optional<Integer>", "Optional<demo.values.Loop>");
    for (case, source, reason) in [
        ("unsupported-list", list, "a `list`"),
        ("recursive", recursive, "nested too deeply"),
    ] {
        let go = run_lane("go", case, &source, matrix::GO, matrix::TS);
        let ts = run_lane("ts", case, &source, matrix::GO, matrix::TS);
        assert_eq!(go, ts);
        let exclusions = go["result"]["excluded"].as_array().unwrap();
        assert_eq!(exclusions.len(), 1);
        assert_eq!(
            exclusions[0]["reason"],
            format!("input `number` is {reason}")
        );
    }
}

#[test]
fn missing_optional_predicate_uses_the_same_known_absence_in_both_ports() {
    use support_explore_optional_unknown::matrix;
    let source = matrix::source().replace("defined(gate)", "missing(gate)");
    let go_driver = matrix::GO.replace(
        "outcome:=\"present\"; if r.Input[\"gate\"]==nil{outcome=\"absent\"}",
        "outcome:=\"absent\"; if r.Input[\"gate\"]==nil{outcome=\"present\"}",
    );
    let ts_driver = matrix::TS.replace(
        "input.gate==null?'absent':'present'",
        "input.gate==null?'present':'absent'",
    );
    assert_ne!(go_driver, matrix::GO);
    assert_ne!(ts_driver, matrix::TS);
    let go = run_lane("go", "missing", &source, &go_driver, &ts_driver);
    let ts = run_lane("ts", "missing", &source, &go_driver, &ts_driver);
    assert_eq!(go, ts);
    assert!(go["result"]["failure"].is_null(), "{go}");
    assert_eq!(go["result"]["unreached"], serde_json::json!([]));
    assert!(go["mutants"]["branch"]["failure"].is_object(), "{go}");
}

#[test]
fn optional_uses_the_existing_depth_boundary_without_a_shallower_limit() {
    use support_explore_optional_unknown::matrix;
    for count in [31, 32] {
        let aliases = (0..count).fold(String::new(), |mut aliases, index| {
            let inner = if index + 1 == count {
                "Integer".to_owned()
            } else {
                format!("demo.values.Layer{}", index + 1)
            };
            writeln!(
                aliases,
                "  - {{name: demo.values.Layer{index}, kind: newtype, of: {inner}}}"
            )
            .unwrap();
            aliases
        });
        let source = matrix::source()
            .replace("types:\n", &format!("types:\n{aliases}"))
            .replace("Optional<Integer>", "Optional<demo.values.Layer0>");
        let case = format!("depth-{count}");
        let go = run_lane("go", &case, &source, matrix::GO, matrix::TS);
        let ts = run_lane("ts", &case, &source, matrix::GO, matrix::TS);
        assert_eq!(go, ts);
        if count == 31 {
            assert!(go["result"]["failure"].is_null(), "{go}");
            assert_eq!(go["result"]["executed"], 128);
            assert_eq!(go["result"]["excluded"], serde_json::json!([]));
        } else {
            assert_eq!(
                go["result"]["excluded"][0]["reason"],
                "input `number` is nested too deeply"
            );
        }
    }
}

#[test]
fn nested_optional_identity_references_survive_actual_fresh_target_shrinking() {
    use support_explore_optional_unknown::subjects;
    let source = subjects::source(true)
        .replace("Optional<String>", "Optional<demo.items.Link>")
        .replace("types:\n", "types:\n  - name: demo.items.Link\n    kind: struct\n    fields: [{name: item, type: Optional<demo.items.ItemId>}]\n");
    let go_driver = subjects::GO.replace("UPSERT", "true");
    let ts_driver = subjects::TS.replace("UPSERT", "true");
    let go = run_lane("go", "references", &source, &go_driver, &ts_driver);
    let ts = run_lane("ts", "references", &source, &go_driver, &ts_driver);
    assert_eq!(go["calls"], ts["calls"]);
    assert_eq!(go["result"], ts["result"]);
    assert!(go["result"]["failure"].is_null(), "{go}");
    assert_eq!(go["result"]["unreached"], serde_json::json!([]));
    for mutant in ["create", "update", "view-absence", "view-present"] {
        assert!(
            go["mutants"][mutant]["failure"].is_object(),
            "{mutant}: {go}"
        );
        assert_eq!(go["mutants"][mutant], ts["mutants"][mutant]);
        assert_eq!(go["mutants"][mutant]["failure"]["shrinkComplete"], true);
    }
    assert!(go["calls"]
        .as_array()
        .unwrap()
        .iter()
        .any(|call| call["input"]["note"]["item"].is_string()));
}

#[test]
fn exhausted_finite_identity_domain_keeps_existing_upserts_executable() {
    use support_explore_optional_unknown::subjects;
    let mut source = subjects::source(true).replace(
        "kind: newtype, of: Uuid",
        "kind: enum, variants: [First, Second]",
    );
    let start = source.find("  - name: demo.items.Create\n").unwrap();
    let end = source.find("  - name: demo.items.Put\n").unwrap();
    source.replace_range(start..end, "");
    source = source.replace("[demo.items.Create, demo.items.Put]", "[demo.items.Put]");
    let go_driver = subjects::GO.replace("UPSERT", "true");
    let ts_driver = subjects::TS.replace("UPSERT", "true");
    let go = run_lane("go", "finite", &source, &go_driver, &ts_driver);
    let ts = run_lane("ts", "finite", &source, &go_driver, &ts_driver);
    assert_eq!(go["calls"], ts["calls"]);
    assert_eq!(go["result"], ts["result"]);
    assert!(go["result"]["failure"].is_null(), "{go}");
    assert_eq!(go["result"]["executed"], 256);
    assert_eq!(go["result"]["unreached"], serde_json::json!([]));
    for id in ["First", "Second"] {
        assert!(go["calls"]
            .as_array()
            .unwrap()
            .iter()
            .any(|call| call["input"]["id"] == id));
    }
}

#[test]
fn go_draws_optional_inputs_and_missing_subjects() {
    verify("go");
}

#[test]
fn typescript_draws_optional_inputs_and_missing_subjects() {
    verify("ts");
}

#[test]
fn redirected_external_and_unknown_creation_fixtures_are_source_admitted() {
    let source = include_str!("support_explore_optional_unknown/baseline.yaml")
        .replace("format: ess/15", "format: ess/16")
        .replace("states: [Open, Closed]", "states: [Open]")
        .replace("terminal: [Closed]", "terminal: [Open]")
        .replace(
            "transitions: [{name: close, from: [Open], to: Closed}]",
            "transitions: []",
        );
    let start = source.find("  - name: demo.items.Close\n").unwrap();
    let end = source.find("  - name: demo.items.PlainNote\n").unwrap();
    let external = r"  - name: demo.items.Close
    input: [{name: id, type: demo.items.ItemId}]
    outcomes:
      - name: ordinary
        error: demo.items.Conflict
      - name: first-provider
        external: the first provider accepts
        updates: demo.items.Item
        instance: id
        emits: [demo.items.Closed]
        payload: {demo.items.Closed: {id: input.id}}
      - name: second-provider
        external: the second provider accepts
        updates: demo.items.Item
        instance: id
        emits: [demo.items.Closed]
        payload: {demo.items.Closed: {id: input.id}}
      - {name: not-found, unknown_instance: true, error: demo.items.NotFound}
";
    compiled(&format!("{}{external}{}", &source[..start], &source[end..]));
    let upsert = r"  - name: demo.items.Close
    input: [{name: id, type: demo.items.ItemId}]
    outcomes:
      - name: updated
        updates: demo.items.Item
        instance: id
        emits: [demo.items.Closed]
        payload: {demo.items.Closed: {id: input.id}}
      - name: created
        unknown_instance: true
        creates: demo.items.Item
        instance: id
        emits: [demo.items.Created]
        payload: {demo.items.Created: {id: input.id}}
";
    compiled(&format!("{}{upsert}{}", &source[..start], &source[end..]));
}
