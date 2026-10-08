//! Both emitted concurrent explorers use Optional/fresh draws and the actual native checker.

#[path = "../../../verify/ess-conformance/tests/support_explore_optional_unknown/subjects.rs"]
mod subjects;

use ess_cli::TemporaryDirectory;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn run(command: &mut Command, root: &Path) {
    let binary = Path::new(env!("CARGO_BIN_EXE_ess"));
    let inherited = std::env::var_os("PATH").unwrap();
    let paths = std::iter::once(binary.parent().unwrap().to_path_buf())
        .chain(std::env::split_paths(&inherited));
    let output = command
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("ESS_OPTIONAL_SPEC", root.join("model.yaml"))
        .env("ESS_OPTIONAL_HISTORY", root.join("histories"))
        .output()
        .expect("required runtime and native checker");
    assert!(
        output.status.success(),
        "{command:?}\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn suite(ir: &ess_compiler::ir::EssIr, case: &str) -> ess_conformance::scenario::ConformanceSuite {
    let synthesis = ess_conformance::synthesize(ir);
    if case == "demo.items.Bounded" {
        // The fixed-suite producer cannot observe this invariant through an Optional view field.
        // Preserve that separate refusal; concurrent histories are judged by the native checker.
        assert_eq!(synthesis.refusals.len(), 1, "{:?}", synthesis.refusals);
        assert!(matches!(
            &synthesis.refusals[0].cause,
            ess_conformance::synthesize::RefusalCause::ValueInvariantUnwitnessed { at: None, .. }
        ));
    } else {
        assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    }
    let mut suite = synthesis.suite;
    suite.select_fresh_format_for(ir);
    suite
}

fn lane(label: &str, source: &str, case: &str) -> (Value, Vec<Value>) {
    let root =
        TemporaryDirectory::create(&format!("ess-optional-concurrent-{label}-{case}")).unwrap();
    write(&root.join("model.yaml"), source);
    let raw = RawSpecFile::parse(source).unwrap();
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)]).unwrap();
    let mut sources = SourceMap::new();
    sources.insert("model.yaml", source);
    let ir = compile(&spec, &sources).unwrap();
    let suite = suite(&ir, case);
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
            "module example.invalid/optionalconcurrent\n\ngo 1.24\n",
        );
        let driver = format!(
            "{}{}",
            subjects::GO.split("func TestProbe").next().unwrap(),
            GO
        )
        .replace("UPSERT", "true");
        write(&runtime.join("probe_test.go"), &driver);
        run(
            Command::new("go")
                .args([
                    "test",
                    "./essconform",
                    "-run",
                    "TestProbe",
                    "-count=1",
                    "-v",
                ])
                .env("GOWORK", "off")
                .current_dir(&package),
            &root,
        );
    } else {
        let driver = format!(
            "{}{}",
            subjects::TS.split("const calls=[];").next().unwrap(),
            TS
        )
        .replace("UPSERT", "true")
        .replace("import {explore}", "import {exploreConcurrent}");
        write(&runtime.join("probe.mjs"), &driver);
        write(
            &runtime.join("probe.tsconfig.json"),
            r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
        );
        run(
            Command::new("tsc")
                .args(["--project", "probe.tsconfig.json"])
                .current_dir(&runtime),
            &root,
        );
        run(
            Command::new("node").arg("probe.mjs").current_dir(&runtime),
            &root,
        );
    }
    let result =
        serde_json::from_slice(&fs::read(root.join(format!("{label}-result.json"))).unwrap())
            .unwrap();
    let histories = (1..=4)
        .map(|seed| {
            serde_json::from_slice(
                &fs::read(root.join(format!("histories/history-{seed}.json"))).unwrap(),
            )
            .unwrap()
        })
        .collect();
    (result, histories)
}

const GO: &str = r#"func TestProbe(t *testing.T){
 calls:=[]map[string]any{}
 result,e:=ExploreConcurrent(func()Target{return &subjectTarget{calls:&calls}},ConcurrentOptions{Path:os.Getenv("ESS_OPTIONAL_SPEC"),Out:os.Getenv("ESS_OPTIONAL_HISTORY"),Seeds:4,Clients:2,Calls:4});if e!=nil{t.Fatal(e)}
 raw,_:=json.Marshal(map[string]any{"calls":calls,"result":result});if e:=os.WriteFile("../../go-result.json",raw,0600);e!=nil{t.Fatal(e)}
}"#;

const TS: &str = r"const calls=[];
const result=await exploreConcurrent(()=>target('',calls),{path:process.env.ESS_OPTIONAL_SPEC,out:process.env.ESS_OPTIONAL_HISTORY,seeds:4,clients:2,calls:4});
writeFileSync('../../ts-result.json',JSON.stringify({calls,result}));
";

#[test]
fn actual_concurrent_ports_record_optional_upserts_and_call_the_native_checker() {
    for kind in ["String", "Decimal", "Timestamp", "demo.items.Bounded"] {
        let mut source =
            subjects::source(true).replace("Optional<String>", &format!("Optional<{kind}>"));
        if kind == "demo.items.Bounded" {
            source = source.replace("types:\n", "types:\n  - {name: demo.items.Bounded, kind: newtype, of: Integer, invariants: ['value >= -10', 'value <= 10000']}\n");
        }
        let (go, go_histories) = lane("go", &source, kind);
        let (ts, ts_histories) = lane("ts", &source, kind);
        assert_eq!(go["calls"], ts["calls"]);
        assert_eq!(go_histories, ts_histories);
        for lane in [&go, &ts] {
            assert_eq!(lane["result"]["histories"], 4, "{lane}");
            assert_eq!(lane["result"]["linearizable"], 4, "{lane}");
            assert_eq!(lane["result"]["excluded"], serde_json::json!([]));
            let calls = lane["calls"].as_array().unwrap();
            assert!(calls.iter().any(|call| call["input"]["note"].is_null()));
            assert!(calls.iter().any(|call| !call["input"]["note"].is_null()));
            assert!(calls.iter().any(|call| call["command"] == "demo.items.Put"));
        }
    }
}

#[test]
fn prefix_freshness_does_not_reject_a_concurrent_creation_update_race() {
    let source = subjects::source(true).replace("of: Uuid", "of: String");
    let (go, histories) = lane("go", &source, "string");
    let (ts, ts_histories) = lane("ts", &source, "string");
    assert_eq!(go["calls"], ts["calls"]);
    assert_eq!(histories, ts_histories);
    for result in [&go, &ts] {
        assert_eq!(result["result"]["linearizable"], 4, "{result}");
    }
    // The first two operations are the serial prefix. A created key absent there followed by
    // an update in the concurrent workload demonstrates snapshot-fresh is not execution-fresh.
    assert!(
        histories.iter().any(|history| {
            let operations = history["operations"].as_array().unwrap();
            operations.iter().skip(2).any(|created| {
                created["command"] == "demo.items.Put"
                    && created["outcome"] == "created"
                    && !operations
                        .iter()
                        .take(2)
                        .any(|prefix| prefix["subject_key"] == created["subject_key"])
                    && operations.iter().skip(2).any(|updated| {
                        updated["command"] == "demo.items.Put"
                            && updated["outcome"] == "updated"
                            && updated["subject_key"] == created["subject_key"]
                    })
            })
        }),
        "no actual fresh-key race in {histories:?}"
    );
}
