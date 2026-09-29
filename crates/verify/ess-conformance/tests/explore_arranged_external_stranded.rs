//! The Go lane of the adversary's pass-1 finding on beyond10x/ess#235, asserted on its own.
//!
//! `adversary_explore_order_pass1.rs` runs both languages in one loop and fails on the TypeScript
//! lane first, so the Go explorer's answer there is never read. This file runs only the Go lane over
//! the same fixture (`explore-adv-order.yaml`, `Close` only) and target
//! (`explore_adv_order_target.go`).
//!
//! Entity Runtime sorts an external branch ahead of the default and admits a branch that moves
//! nothing in any state, so an arranged `bounced` answers on a `Closed` ticket; `wrong-state`
//! answers only where the default is selected and cannot move. The explorer must offer `bounced`
//! there and, once it is arranged, expect it.
//!
//! A missing `go` panics rather than skipping.

use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::scenario::ConformanceSuite;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/explore-adv-order.yaml");

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-adv-order.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification = Specification::assemble(vec![(Source::new("explore-adv-order.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
}

fn suite(ir: &EssIr) -> ConformanceSuite {
    let mut suite = ess_conformance::synthesize(ir).suite;
    suite.select_fresh_format();
    suite
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn write(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
    std::fs::write(path, contents).expect("writable");
}

fn printed(output: &std::process::Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// Runs the Go explorer over `explore-adv-order.yaml` with `Rate` not exposed, returning the
/// driver's assertion line and its JSON result.
fn go_lane() -> (String, Value) {
    let root = std::env::temp_dir().join(format!(
        "ess-explore-arranged-stranded-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let module = root.join("go");
    let ir = ir();
    for artifact in ess_conformance::go::emit_with_model(&suite(&ir), &ir).unwrap() {
        write(&module.join(&artifact.path), &artifact.contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/exploreadv\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_adv_order_target.go"),
        module.join("essconform/explore_adv_order_target_test.go"),
    )
    .expect("the target copies");
    let driver = std::fs::read_to_string(fixture("explore_preconditions_driver_test.go"))
        .expect("the driver reads")
        .replace(
            "newExplorePreTarget(one.Mode)",
            "newExploreAdvTarget(one.Mode)",
        );
    write(
        &module.join("essconform/explore_adv_order_driver_test.go"),
        &driver,
    );
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let cases = json!([{"name": "close", "mode": "no-rate", "allowExcluded": true}]);
    let run = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestExplorePreconditions",
            "-count=1",
            "-v",
        ])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .env("GOWORK", "off")
        .current_dir(&module)
        .output()
        .expect("`go` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    let assert = std::fs::read_to_string(out.join("close.assert"))
        .unwrap_or_else(|_| panic!("the case wrote no assertion:\n{log}"));
    let text = std::fs::read_to_string(out.join("close.json"))
        .unwrap_or_else(|_| panic!("the case wrote no result ({assert}):\n{log}"));
    std::fs::remove_dir_all(&root).ok();
    (
        assert.trim_end().to_owned(),
        serde_json::from_str(&text).expect("JSON"),
    )
}

#[test]
fn go_an_arranged_external_that_moves_nothing_answers_on_a_closed_subject() {
    let (assert, found) = go_lane();
    assert_eq!(
        assert, "ok",
        "go: a target in Entity Runtime's order is reported as disagreeing\n{found}"
    );
    assert!(
        found["external"].as_array().unwrap().iter().any(|reach| {
            reach["outcome"] == "exploreadv.desk.Close/bounced" && reach["reach"] == "reached"
        }),
        "go: `bounced` was never arranged\n{found}"
    );
    let reached: Vec<&str> = found["reached"]
        .as_array()
        .unwrap_or_else(|| panic!("a reached list, not {found}"))
        .iter()
        .map(|item| item.as_str().expect("a string"))
        .collect();
    assert!(
        reached.contains(&"exploreadv.desk.Close/wrong-state"),
        "go: `wrong-state` is no longer reached once `bounced` is offered beside it\n{found}"
    );
}
