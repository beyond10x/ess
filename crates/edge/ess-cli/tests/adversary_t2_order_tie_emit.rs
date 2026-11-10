//! Adversary, pass 1, on the synthesized tie of a ranked view's later keys (beyond10x/ess#708), by
//! the acceptance's emitted route: `--emit DIR`, every emitted suite answered by the interpreter
//! over the unchanged specification with its report beside it, then `--collect DIR`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate;
use ess_conformance::runner::Runner;
use ess_conformance::{AdmittedSuite, CountReport};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use serde_json::Value;

const REPRODUCTION: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/mutation-order-tie.yaml");
const IDENTITY_LAST: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/mutation-order-tie-identity-last.yaml"
);

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn shown(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn s(path: &Path) -> &str {
    path.to_str().expect("a UTF-8 path")
}

fn scratch(label: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-t2-order-tie")
        .join(format!("{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    directory
}

fn json(path: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn emit_answer_collect(label: &str, yaml: &str) -> (Output, Value, PathBuf) {
    let directory = scratch(label);
    let spec = directory.join("spec");
    std::fs::create_dir_all(&spec).unwrap();
    std::fs::write(spec.join("system.yaml"), yaml).unwrap();
    let mut texts = SourceMap::new();
    texts.insert("system.yaml".to_owned(), yaml.to_owned());
    let parsed = vec![(
        Source::new("system.yaml"),
        RawSpecFile::parse(yaml).unwrap(),
    )];
    let ir = mutate::compile(parsed, &texts).expect("the fixture compiles");

    let emitted = directory.join("emitted");
    let output = ess(&[
        "verify",
        "conform",
        "mutate",
        "--path",
        s(&spec),
        "--class",
        "order-flip",
        "--emit",
        s(&emitted),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", shown(&output));

    let manifest = json(&emitted.join("manifest.json"));
    let mut dirs = vec![manifest["baseline"]["dir"].as_str().unwrap().to_owned()];
    dirs.extend(
        manifest["mutants"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|it| it["dir"].as_str().map(str::to_owned)),
    );
    for dir in dirs {
        let suite = std::fs::read_to_string(emitted.join(&dir).join("suite.json")).unwrap();
        let admitted = AdmittedSuite::from_json(&suite).expect("an emitted suite is admitted");
        let executed = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(ir.clone()));
        let report = CountReport::from_run(&executed, &admitted)
            .expect("a complete run")
            .to_canonical_json()
            .expect("serializes");
        std::fs::write(emitted.join(&dir).join("report.json"), report).unwrap();
    }

    let out = directory.join("collected.json");
    let collected = ess(&[
        "verify",
        "conform",
        "mutate",
        "--collect",
        s(&emitted),
        "--format",
        "json",
        "--report-out",
        s(&out),
    ]);
    (collected, json(&out), emitted)
}

#[test]
fn the_reproduction_by_emit_and_collect_kills_both_flips_and_exits_zero() {
    let (collected, report, emitted) = emit_answer_collect("reproduction", REPRODUCTION);
    assert_eq!(report["counts"]["mutants"], 2, "{report:#}");
    assert_eq!(report["counts"]["killed"], 2, "{report:#}");
    assert_eq!(report["counts"]["survived"], 0, "{report:#}");
    assert_eq!(collected.status.code(), Some(0), "{}", shown(&collected));

    // The emitted baseline scenario sends two PlaceOrder with one region and two priorities
    // before its query_view, and still asserts the ranked order.
    let manifest = json(&emitted.join("manifest.json"));
    let baseline = json(
        &emitted
            .join(manifest["baseline"]["dir"].as_str().unwrap())
            .join("suite.json"),
    );
    let text = baseline.to_string();
    assert!(
        text.contains(r#""expect":"ranked","order_by":["region asc","priority asc"]"#),
        "{text}"
    );
}

#[test]
fn the_identity_last_variant_by_emit_and_collect_kills_all_three_flips() {
    let (collected, report, _) = emit_answer_collect("identity-last", IDENTITY_LAST);
    assert_eq!(report["counts"]["mutants"], 3, "{report:#}");
    assert_eq!(report["counts"]["killed"], 3, "{report:#}");
    assert_eq!(collected.status.code(), Some(0), "{}", shown(&collected));
}
